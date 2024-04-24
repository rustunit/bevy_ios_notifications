use bevy::prelude::*;

use crate::{request::Permissions, IosNotificationsResource};

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
    permissions: Permissions,
}

impl IosNotificationsPlugin {
    pub fn with_permissions(alert: bool, sound: bool, badge: bool) -> Self {
        Self {
            permissions: Permissions {
                alert,
                sound,
                badge,
            },
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
            crate::native::request(crate::Request {
                calls: Some(crate::request::Calls::Permissions(self.permissions.clone())),
            });

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
