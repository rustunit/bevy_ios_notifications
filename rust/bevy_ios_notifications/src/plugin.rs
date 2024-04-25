use bevy::prelude::*;

use crate::IosNotificationsResource;

#[derive(Clone, Debug, Default)]
pub struct IosNotificationPermissions {
    pub alert: bool,
    pub sound: bool,
    pub badge: bool,
}

#[derive(Event, Clone, Debug)]
pub enum IosNotificationEvents {
    PermissionResponse(bool),
    NotificationSchedulingSucceeded(String),
    NotificationSchedulingFailed(String),
    NotificationTriggered(String),
    PendingNotifications(Vec<String>),
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
            app.add_event::<IosNotificationEvents>();
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

            use bevy_crossbeam_event::{CrossbeamEventApp, CrossbeamEventSender};

            app.add_crossbeam_event::<IosNotificationEvents>();

            let sender = app
                .world
                .get_resource::<CrossbeamEventSender<IosNotificationEvents>>()
                .unwrap()
                .clone();

            crate::native::set_sender(sender);
        }
    }
}
