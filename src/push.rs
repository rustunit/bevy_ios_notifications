//! Remote notification registration.
//!
//! Device tokens are only delivered to the `UIApplicationDelegate`, so the two callbacks are
//! grafted onto whatever delegate the app already has. Since winit `0.30.10` there usually is
//! none, in which case we install our own.

use std::ffi::CStr;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, Imp, NSObjectProtocol, ProtocolObject, Sel};
use objc2::{MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_foundation::{NSData, NSError};
use objc2_ui_kit::{UIApplication, UIApplicationDelegate, UIResponder};

use crate::IosNotificationEvents;
use crate::channel::send_event;
use crate::plugin::IosRemoteNotificationRegistration;

/// `void (id self, SEL _cmd, id application, id argument)`
const ENCODING: &CStr = c"v@:@@";

fn on_device_token(token: &NSData) {
    let token = token.to_vec().iter().fold(String::new(), |mut acc, byte| {
        use std::fmt::Write;
        let _ = write!(acc, "{byte:02x}");
        acc
    });

    send_event(IosNotificationEvents::RemoteNotificationRegistration(
        IosRemoteNotificationRegistration::DeviceToken(token),
    ));
}

fn on_registration_error(error: &NSError) {
    send_event(IosNotificationEvents::RemoteNotificationRegistration(
        IosRemoteNotificationRegistration::Failed {
            code: error.code() as i32,
            localized_description: error.localizedDescription().to_string(),
        },
    ));
}

define_class!(
    #[unsafe(super(UIResponder))]
    #[thread_kind = MainThreadOnly]
    #[name = "BevyIosNotificationsAppDelegate"]
    struct AppDelegate;

    unsafe impl NSObjectProtocol for AppDelegate {}

    unsafe impl UIApplicationDelegate for AppDelegate {
        #[unsafe(method(application:didRegisterForRemoteNotificationsWithDeviceToken:))]
        fn did_register(&self, _application: &UIApplication, device_token: &NSData) {
            on_device_token(device_token);
        }

        #[unsafe(method(application:didFailToRegisterForRemoteNotificationsWithError:))]
        fn did_fail(&self, _application: &UIApplication, error: &NSError) {
            on_registration_error(error);
        }
    }
);

type DidRegister = extern "C-unwind" fn(*mut AnyObject, Sel, *mut UIApplication, *mut NSData);
type DidFail = extern "C-unwind" fn(*mut AnyObject, Sel, *mut UIApplication, *mut NSError);

static ORIGINAL_DID_REGISTER: OnceLock<DidRegister> = OnceLock::new();
static ORIGINAL_DID_FAIL: OnceLock<DidFail> = OnceLock::new();

extern "C-unwind" fn did_register(
    this: *mut AnyObject,
    sel: Sel,
    application: *mut UIApplication,
    device_token: *mut NSData,
) {
    // SAFETY: UIKit passes a valid pointer.
    if let Some(device_token) = unsafe { device_token.as_ref() } {
        on_device_token(device_token);
    }

    if let Some(original) = ORIGINAL_DID_REGISTER.get() {
        original(this, sel, application, device_token);
    }
}

extern "C-unwind" fn did_fail(
    this: *mut AnyObject,
    sel: Sel,
    application: *mut UIApplication,
    error: *mut NSError,
) {
    // SAFETY: UIKit passes a valid pointer.
    if let Some(error) = unsafe { error.as_ref() } {
        on_registration_error(error);
    }

    if let Some(original) = ORIGINAL_DID_FAIL.get() {
        original(this, sel, application, error);
    }
}

/// Puts `imp` in front of `sel` on `class` and returns the implementation it displaced, if the
/// delegate had one.
///
/// # Safety
/// `imp` has to match the selector's signature, which [`ENCODING`] describes.
unsafe fn hook(class: &AnyClass, sel: Sel, imp: Imp) -> Option<Imp> {
    // Only a method the class declares itself can be replaced in place - swizzling an inherited one
    // would reach into the superclass and change it for every sibling as well.
    if class.instance_methods().iter().any(|m| m.name() == sel) {
        let method = class.instance_method(sel)?;
        return Some(unsafe { method.set_implementation(imp) });
    }

    // An inherited implementation is still worth chaining to once ours overrides it.
    let inherited = class.instance_method(sel).map(|m| m.implementation());

    let added = unsafe {
        objc2::ffi::class_addMethod(
            (class as *const AnyClass).cast_mut(),
            sel,
            imp,
            ENCODING.as_ptr(),
        )
    };

    if !added.as_bool() {
        bevy_log::error!("could not add `{sel}` to the app delegate, no device token will arrive");
    }

    inherited
}

fn graft(class: &AnyClass) {
    // SAFETY: both shims take `(id, SEL, UIApplication*, id)` and return void, as `ENCODING` says.
    unsafe {
        if let Some(original) = hook(
            class,
            sel!(application:didRegisterForRemoteNotificationsWithDeviceToken:),
            std::mem::transmute::<DidRegister, Imp>(did_register),
        ) {
            let _ = ORIGINAL_DID_REGISTER.set(std::mem::transmute::<Imp, DidRegister>(original));
        }

        if let Some(original) = hook(
            class,
            sel!(application:didFailToRegisterForRemoteNotificationsWithError:),
            std::mem::transmute::<DidFail, Imp>(did_fail),
        ) {
            let _ = ORIGINAL_DID_FAIL.set(std::mem::transmute::<Imp, DidFail>(original));
        }
    }
}

fn install(mtm: MainThreadMarker, application: &UIApplication) {
    // SAFETY: reading the delegate on the main thread.
    if let Some(delegate) = unsafe { application.delegate() } {
        let delegate: &AnyObject = delegate.as_ref();
        graft(delegate.class());
        return;
    }

    let delegate: Retained<AppDelegate> =
        unsafe { msg_send![super(mtm.alloc().set_ivars(())), init] };

    // SAFETY: our delegate conforms to `UIApplicationDelegate`.
    unsafe { application.setDelegate(Some(ProtocolObject::from_ref(&*delegate))) };

    // the application does not retain its delegate, so it has to live as long as the app does
    std::mem::forget(delegate);
}

pub fn register(mtm: MainThreadMarker) {
    static INSTALLED: AtomicBool = AtomicBool::new(false);

    let application = UIApplication::sharedApplication(mtm);

    if !INSTALLED.swap(true, Ordering::SeqCst) {
        install(mtm, &application);
    }

    application.registerForRemoteNotifications();
}
