use bevy_ecs::system::{NonSend, SystemParam};
use builder_pattern::Builder;
use std::collections::HashMap;

use crate::plugin::IosNotificationPermissions;

#[derive(Default)]
pub struct IosNotificationTrigger {
    pub repeat: bool,
    pub seconds: f64,
}

impl IosNotificationTrigger {
    pub fn one_shot(seconds: u32) -> Self {
        Self {
            repeat: false,
            seconds: seconds.into(),
        }
    }

    pub fn new(seconds: u32, repeat: bool) -> Self {
        Self {
            repeat,
            seconds: seconds.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum IosNotificationSound {
    Default,
    /// name of a sound file bundled with the app, e.g. `"chime.caf"`
    Named(String),
}

/// ```rust
/// use bevy_ios_notifications::{IosNotificationRequest, IosNotificationSound};
///
/// let request = IosNotificationRequest::new()
///     .title("title")
///     .body("body")
///     .sound(IosNotificationSound::Default)
///     .build();
/// ```
#[derive(Builder)]
pub struct IosNotificationRequest {
    #[into]
    pub title: String,
    #[into]
    #[default(String::new())]
    pub body: String,
    #[into]
    #[default(String::new())]
    pub category_identifier: String,
    #[into]
    #[default(String::new())]
    pub subtitle: String,
    #[default(0.)]
    pub relevance_score: f64,
    #[into]
    #[default(String::new())]
    pub thread_identifier: String,
    #[into]
    #[default(None)]
    pub filter_criteria: Option<String>,
    #[into]
    #[default(None)]
    pub user_data: Option<HashMap<String, String>>,
    #[default(None)]
    #[into]
    pub identifier: Option<String>,
    #[default(None)]
    #[into]
    pub badge: Option<i32>,
    /// `None` delivers the notification silently
    #[default(None)]
    #[into]
    pub sound: Option<IosNotificationSound>,
    #[default(None)]
    #[into]
    pub trigger: Option<IosNotificationTrigger>,
}

#[derive(Default)]
pub(crate) struct IosNotificationsNonSendResource;

/// access to the iOS notification system, pinned to the main thread by its `NonSend`
///
/// ```rust
/// use bevy_ios_notifications::IosNotifications;
///
/// fn bevy_system(notifications: IosNotifications) {
///     notifications.set_badge(0);
/// }
/// ```
#[derive(SystemParam)]
pub struct IosNotifications<'w> {
    _non_send: NonSend<'w, IosNotificationsNonSendResource>,
}

impl IosNotifications<'_> {
    pub fn get_badge(&self) -> i32 {
        #[cfg(target_os = "ios")]
        return crate::native::badge_get();
        #[cfg(not(target_os = "ios"))]
        return 0;
    }

    pub fn set_badge(&self, _value: i32) {
        #[cfg(target_os = "ios")]
        crate::native::badge_set(_value);
    }

    /// whether notifications that fire while the app is in the foreground are presented
    pub fn show_foregrounded(&self) -> bool {
        #[cfg(target_os = "ios")]
        return crate::native::show_foregrounded();
        #[cfg(not(target_os = "ios"))]
        return true;
    }

    pub fn set_show_foregrounded(&self, _value: bool) {
        #[cfg(target_os = "ios")]
        crate::native::set_show_foregrounded(_value);
    }

    pub fn registered_for_push(&self) -> bool {
        #[cfg(target_os = "ios")]
        return crate::native::registered_for_push();
        #[cfg(not(target_os = "ios"))]
        return false;
    }

    /// will respond async via [`crate::IosNotificationEvents::RemoteNotificationRegistration`]
    pub fn register_for_push(&self) {
        #[cfg(target_os = "ios")]
        crate::native::register_for_push();
    }

    /// will respond async via [`crate::IosNotificationEvents::NotificationSchedulingSucceeded`] or
    /// [`crate::IosNotificationEvents::NotificationSchedulingFailed`], carrying the identifier this
    /// returns
    pub fn schedule(&self, _request: IosNotificationRequest) -> String {
        #[cfg(target_os = "ios")]
        return crate::native::schedule(_request);
        #[cfg(not(target_os = "ios"))]
        return String::new();
    }

    /// will respond async via [`crate::IosNotificationEvents::PermissionResponse`]
    pub fn request_permissions(&self, _permissions: IosNotificationPermissions) {
        #[cfg(target_os = "ios")]
        crate::native::request_permissions(&_permissions);
    }

    /// will respond async via [`crate::IosNotificationEvents::PendingNotifications`]
    pub fn request_pending(&self) {
        #[cfg(target_os = "ios")]
        crate::native::request_pending();
    }

    pub fn remove_pending(&self, _ids: &[String]) {
        #[cfg(target_os = "ios")]
        crate::native::remove_pending(_ids);
    }

    pub fn remove_delivered(&self, _ids: &[String]) {
        #[cfg(target_os = "ios")]
        crate::native::remove_delivered(_ids);
    }

    /// remove all pending notifications
    pub fn remove_all_pending(&self) {
        #[cfg(target_os = "ios")]
        crate::native::remove_all_pending();
    }

    /// remove all delivered notifications
    pub fn remove_all_delivered(&self) {
        #[cfg(target_os = "ios")]
        crate::native::remove_all_delivered();
    }
}
