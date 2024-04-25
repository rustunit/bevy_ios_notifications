mod plugin;
mod resource;

#[cfg(target_os = "ios")]
mod native;
#[cfg(target_os = "ios")]
include!(concat!(env!("OUT_DIR"), "/bevy_ios.notifications.rs"));

pub use plugin::{IosNotificationEvents, IosNotificationResponse, IosNotificationsPlugin};
pub use resource::{IosNotificationRequest, IosNotificationTrigger, IosNotificationsResource};
