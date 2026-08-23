#[cfg(target_os = "ios")]
mod channel;
#[cfg(target_os = "ios")]
mod native;
mod plugin;
#[cfg(target_os = "ios")]
mod push;
mod resource;

pub use plugin::{
    IosNotificationEvents, IosNotificationPermissions, IosNotificationResponse,
    IosNotificationsPlugin, IosRemoteNotificationRegistration,
};
pub use resource::{IosNotificationRequest, IosNotificationTrigger, IosNotifications};
