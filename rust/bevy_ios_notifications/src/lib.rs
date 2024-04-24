#[cfg(target_os = "ios")]
mod native;
mod plugin;
mod resource;

include!(concat!(env!("OUT_DIR"), "/bevy_ios.notifications.rs"));

pub use plugin::{IosNotificationEvents, IosNotificationsPlugin};
pub use resource::{IosNotificationRequest, IosNotificationTrigger, IosNotificationsResource};
