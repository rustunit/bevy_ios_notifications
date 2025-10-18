use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

use crate::IosNotificationsResource;

#[derive(Clone, Debug, Default)]
pub struct IosNotificationPermissions {
    pub alert: bool,
    pub sound: bool,
    pub badge: bool,
}

#[derive(Clone, Debug, Default)]
pub struct IosNotificationResponse {
    pub identifier: String,
    pub action: String,
}

#[derive(Clone, Debug)]
pub enum IosRemoteNotificationRegistration {
    Failed {
        code: i32,
        localized_description: String,
    },
    DeviceToken(String),
}

#[derive(Message, Clone, Debug)]
pub enum IosNotificationEvents {
    PermissionResponse(bool),
    NotificationSchedulingSucceeded(String),
    NotificationSchedulingFailed(String),
    NotificationTriggered(String),
    PendingNotifications(Vec<String>),
    NotificationResponse(IosNotificationResponse),
    RemoteNotificationRegistration(IosRemoteNotificationRegistration),
}

#[allow(dead_code)]
#[derive(Default)]
pub struct IosNotificationsPlugin {
    permissions: Option<IosNotificationPermissions>,
}

impl IosNotificationsPlugin {
    pub fn request_permissions_on_start(alert: bool, sound: bool, badge: bool) -> Self {
        Self {
            permissions: Some(IosNotificationPermissions {
                alert,
                sound,
                badge,
            }),
        }
    }
}

impl Plugin for IosNotificationsPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<IosNotificationsResource>();

        #[cfg(not(target_os = "ios"))]
        {
            app.add_message::<IosNotificationEvents>();
        }

        #[cfg(target_os = "ios")]
        {
            crate::native::init();

            if let Some(permissions) = &self.permissions {
                crate::native::request(crate::Request {
                    calls: Some(crate::request::Calls::Permissions(
                        crate::request::Permissions {
                            alert: permissions.alert,
                            sound: permissions.sound,
                            badge: permissions.badge,
                        },
                    )),
                });
            }

            use bevy_channel_message::{ChannelMessageApp, ChannelMessageSender};

            app.add_channel_message::<IosNotificationEvents>();

            let sender = app
                .world()
                .get_resource::<ChannelMessageSender<IosNotificationEvents>>()
                .unwrap()
                .clone();

            crate::channel::set_sender(sender);
        }
    }
}
