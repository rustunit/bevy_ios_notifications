//! Direct `UserNotifications` access via [`objc2`].
//!
//! Everything the system answers asynchronously is pushed into the `bevy_channel_message` channel
//! (see [`crate::channel`]) because the completion blocks run on arbitrary queues.

use std::collections::HashMap;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};

use block2::{DynBlock, RcBlock};
use objc2::rc::Retained;
use objc2::runtime::{Bool, NSObjectProtocol, ProtocolObject};
use objc2::{AnyThread, MainThreadMarker, define_class, msg_send, sel};
use objc2_foundation::{
    NSArray, NSDictionary, NSError, NSInteger, NSNumber, NSObject, NSString, NSUUID,
};
use objc2_ui_kit::UIApplication;
use objc2_user_notifications::{
    UNAuthorizationOptions, UNMutableNotificationContent, UNNotification,
    UNNotificationPresentationOptions, UNNotificationRequest, UNNotificationResponse,
    UNTimeIntervalNotificationTrigger, UNUserNotificationCenter, UNUserNotificationCenterDelegate,
};

use crate::channel::send_event;
use crate::plugin::IosNotificationPermissions;
use crate::{IosNotificationEvents, IosNotificationRequest, IosNotificationResponse};

/// Whether a notification that fires while the app is in the foreground is presented by the system.
static SHOW_IN_FOREGROUND: AtomicBool = AtomicBool::new(false);

fn center() -> Retained<UNUserNotificationCenter> {
    UNUserNotificationCenter::currentNotificationCenter()
}

/// The UIKit entry points below are all main-thread only; a `None` marker means we were called
/// from a system that is not the main schedule and the request cannot be served.
fn main_thread(what: &str) -> Option<MainThreadMarker> {
    let mtm = MainThreadMarker::new();
    if mtm.is_none() {
        bevy_log::error!("`{what}` has to be called on the main thread");
    }
    mtm
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = AnyThread]
    #[name = "BevyIosNotificationsDelegate"]
    struct Delegate;

    unsafe impl NSObjectProtocol for Delegate {}

    unsafe impl UNUserNotificationCenterDelegate for Delegate {
        #[unsafe(method(userNotificationCenter:willPresentNotification:withCompletionHandler:))]
        fn will_present(
            &self,
            _center: &UNUserNotificationCenter,
            notification: &UNNotification,
            completion_handler: &DynBlock<dyn Fn(UNNotificationPresentationOptions)>,
        ) {
            send_event(IosNotificationEvents::NotificationTriggered(
                notification.request().identifier().to_string(),
            ));

            let options = if SHOW_IN_FOREGROUND.load(Ordering::Relaxed) {
                UNNotificationPresentationOptions::Banner
                    | UNNotificationPresentationOptions::List
                    | UNNotificationPresentationOptions::Badge
                    | UNNotificationPresentationOptions::Sound
            } else {
                UNNotificationPresentationOptions::empty()
            };

            completion_handler.call((options,));
        }

        #[unsafe(method(userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:))]
        fn did_receive_response(
            &self,
            _center: &UNUserNotificationCenter,
            response: &UNNotificationResponse,
            completion_handler: &DynBlock<dyn Fn()>,
        ) {
            send_event(IosNotificationEvents::NotificationResponse(
                IosNotificationResponse {
                    identifier: response.notification().request().identifier().to_string(),
                    action: response.actionIdentifier().to_string(),
                },
            ));

            completion_handler.call(());
        }
    }
);

pub fn init() {
    static INITIALIZED: AtomicBool = AtomicBool::new(false);

    if INITIALIZED.swap(true, Ordering::SeqCst) {
        return;
    }

    // SAFETY: standard alloc/init pair on a class without ivars.
    let delegate: Retained<Delegate> = unsafe { msg_send![Delegate::alloc(), init] };

    center().setDelegate(Some(ProtocolObject::from_ref(&*delegate)));

    // the center only holds the delegate weakly, so it has to outlive the app like the plugin does
    std::mem::forget(delegate);
}

pub fn request_permissions(permissions: &IosNotificationPermissions) {
    let mut options = UNAuthorizationOptions::empty();

    if permissions.alert {
        options |= UNAuthorizationOptions::Alert;
    }
    if permissions.sound {
        options |= UNAuthorizationOptions::Sound;
    }
    if permissions.badge {
        options |= UNAuthorizationOptions::Badge;
    }

    let handler = RcBlock::new(move |granted: Bool, error: *mut NSError| {
        // SAFETY: the system passes a valid pointer or null.
        if let Some(error) = unsafe { error.as_ref() } {
            bevy_log::error!(
                "notification permission request failed: {}",
                error.localizedDescription()
            );
        }

        send_event(IosNotificationEvents::PermissionResponse(granted.as_bool()));
    });

    center().requestAuthorizationWithOptions_completionHandler(options, &handler);
}

fn user_info(data: HashMap<String, String>) -> Retained<NSDictionary<NSString, NSString>> {
    let (keys, values): (Vec<_>, Vec<_>) = data
        .into_iter()
        .map(|(key, value)| (NSString::from_str(&key), NSString::from_str(&value)))
        .unzip();

    let keys: Vec<&NSString> = keys.iter().map(|key| &**key).collect();

    NSDictionary::from_retained_objects(&keys, &values)
}

fn content(request: IosNotificationRequest) -> Retained<UNMutableNotificationContent> {
    let content = UNMutableNotificationContent::new();

    content.setTitle(&NSString::from_str(&request.title));
    content.setBody(&NSString::from_str(&request.body));
    content.setCategoryIdentifier(&NSString::from_str(&request.category_identifier));
    content.setSubtitle(&NSString::from_str(&request.subtitle));
    content.setRelevanceScore(request.relevance_score);
    content.setThreadIdentifier(&NSString::from_str(&request.thread_identifier));

    if let Some(badge) = request.badge {
        content.setBadge(Some(&NSNumber::numberWithInt(badge)));
    }

    if let Some(criteria) = &request.filter_criteria {
        // `filterCriteria` is iOS 16+, older systems would raise on the unknown selector
        if content.respondsToSelector(sel!(setFilterCriteria:)) {
            content.setFilterCriteria(Some(&NSString::from_str(criteria)));
        } else {
            bevy_log::warn!("`filter_criteria` needs iOS 16 or newer, ignoring it");
        }
    }

    if let Some(data) = request.user_data {
        // SAFETY: the dictionary only holds property list types, as `userInfo` requires.
        unsafe { content.setUserInfo(user_info(data).cast_unchecked()) };
    }

    content
}

pub fn schedule(request: IosNotificationRequest) -> String {
    let identifier = request
        .identifier
        .clone()
        .unwrap_or_else(|| NSUUID::new().UUIDString().to_string());

    let trigger = request.trigger.as_ref().map(|trigger| {
        UNTimeIntervalNotificationTrigger::triggerWithTimeInterval_repeats(
            trigger.seconds,
            trigger.repeat,
        )
    });

    let request = UNNotificationRequest::requestWithIdentifier_content_trigger(
        &NSString::from_str(&identifier),
        &content(request),
        trigger.as_deref().map(|trigger| trigger.as_ref()),
    );

    let scheduled = identifier.clone();
    let handler = RcBlock::new(move |error: *mut NSError| {
        // SAFETY: the system passes a valid pointer or null.
        let event = match unsafe { error.as_ref() } {
            Some(error) => {
                bevy_log::error!(
                    "scheduling `{scheduled}` failed: {}",
                    error.localizedDescription()
                );
                IosNotificationEvents::NotificationSchedulingFailed(scheduled.clone())
            }
            None => IosNotificationEvents::NotificationSchedulingSucceeded(scheduled.clone()),
        };

        send_event(event);
    });

    center().addNotificationRequest_withCompletionHandler(&request, Some(&handler));

    identifier
}

/// answers asynchronously via [`IosNotificationEvents::PendingNotifications`]
pub fn request_pending() {
    let handler = RcBlock::new(move |requests: NonNull<NSArray<UNNotificationRequest>>| {
        // SAFETY: the system hands us a valid array.
        let requests = unsafe { requests.as_ref() };

        send_event(IosNotificationEvents::PendingNotifications(
            requests
                .iter()
                .map(|request| request.identifier().to_string())
                .collect(),
        ));
    });

    center().getPendingNotificationRequestsWithCompletionHandler(&handler);
}

fn identifiers(ids: &[String]) -> Retained<NSArray<NSString>> {
    NSArray::from_retained_slice(
        &ids.iter()
            .map(|id| NSString::from_str(id))
            .collect::<Vec<_>>(),
    )
}

pub fn remove_pending(ids: &[String]) {
    center().removePendingNotificationRequestsWithIdentifiers(&identifiers(ids));
}

pub fn remove_delivered(ids: &[String]) {
    center().removeDeliveredNotificationsWithIdentifiers(&identifiers(ids));
}

pub fn remove_all_pending() {
    center().removeAllPendingNotificationRequests();
}

pub fn remove_all_delivered() {
    center().removeAllDeliveredNotifications();
}

// `setBadgeCount:withCompletionHandler:` replaces the setter, but it never got a getter - so both
// stay on the old property to keep reading and writing the badge symmetric.
#[allow(deprecated)]
pub fn badge_get() -> i32 {
    let Some(mtm) = main_thread("get_badge") else {
        return 0;
    };

    UIApplication::sharedApplication(mtm).applicationIconBadgeNumber() as i32
}

#[allow(deprecated)]
pub fn badge_set(value: i32) {
    let Some(mtm) = main_thread("set_badge") else {
        return;
    };

    UIApplication::sharedApplication(mtm).setApplicationIconBadgeNumber(value as NSInteger);
}

pub fn show_foregrounded() -> bool {
    SHOW_IN_FOREGROUND.load(Ordering::Relaxed)
}

pub fn set_show_foregrounded(value: bool) {
    SHOW_IN_FOREGROUND.store(value, Ordering::Relaxed);
}

pub fn registered_for_push() -> bool {
    let Some(mtm) = main_thread("registered_for_push") else {
        return false;
    };

    UIApplication::sharedApplication(mtm).isRegisteredForRemoteNotifications()
}

pub fn register_for_push() {
    let Some(mtm) = main_thread("register_for_push") else {
        return;
    };

    crate::push::register(mtm);
}
