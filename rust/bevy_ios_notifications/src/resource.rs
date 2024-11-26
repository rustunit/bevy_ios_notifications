use std::collections::HashMap;

use bevy::prelude::*;
use builder_pattern::Builder;

use crate::plugin::IosNotificationPermissions;
#[cfg(target_os = "ios")]
use crate::{request, response, NotificationId, Request};

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
    #[default(None)]
    #[into]
    pub trigger: Option<IosNotificationTrigger>,
}

#[cfg(target_os = "ios")]
impl From<IosNotificationRequest> for request::Schedule {
    fn from(v: IosNotificationRequest) -> Self {
        Self {
            title: v.title,
            body: v.body,
            category_identifier: v.category_identifier,
            subtitle: v.subtitle,
            relevance_score: v.relevance_score,
            thread_identifier: v.thread_identifier,
            filter_criteria: v.filter_criteria,
            user_data: v.user_data.map(|data| crate::UserData { data }),
            identifier: v.identifier,
            badge: v.badge,
            trigger: v.trigger.map(|trigger| request::schedule::Trigger {
                repeat: trigger.repeat,
                seconds: trigger.seconds,
            }),
        }
    }
}

#[cfg(target_os = "ios")]
impl From<&String> for NotificationId {
    fn from(v: &String) -> Self {
        Self {
            identifier: v.clone(),
        }
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct IosNotificationsResource;

impl IosNotificationsResource {
    ///
    #[cfg(target_os = "ios")]
    pub fn get_badge(&self) -> i32 {
        crate::native::badge_get()
    }

    ///
    #[cfg(target_os = "ios")]
    pub fn show_foregrounded(&self) -> bool {
        crate::native::show_foregrounded()
    }

    ///
    #[cfg(not(target_os = "ios"))]
    pub fn show_foregrounded(&self) -> bool {
        true
    }

    ///
    #[cfg(target_os = "ios")]
    pub fn set_show_foregrounded(&self, v: bool) {
        crate::native::set_show_foregrounded(v)
    }

    ///
    #[cfg(not(target_os = "ios"))]
    pub fn set_show_foregrounded(&self, _v: bool) {}

    ///
    #[cfg(target_os = "ios")]
    pub fn registered_for_push(&self) -> bool {
        crate::native::registered_for_push()
    }

    ///
    #[cfg(not(target_os = "ios"))]
    pub fn registered_for_push(&self) -> bool {
        false
    }

    #[cfg(not(target_os = "ios"))]
    pub fn get_badge(&self) -> i32 {
        0
    }

    ///
    #[cfg(target_os = "ios")]
    pub fn set_badge(&self, v: i32) {
        crate::native::badge_set(v);
    }

    ///
    #[cfg(not(target_os = "ios"))]
    pub fn set_badge(&self, _v: i32) {}

    ///
    #[cfg(target_os = "ios")]
    pub fn schedule(request: IosNotificationRequest) -> String {
        let res = crate::native::request(Request {
            calls: Some(request::Calls::Schedule(request.into())),
        });

        res.calls
            .map(|calls| {
                if let response::Calls::Schedule(response::Schedule { identifier }) = calls {
                    identifier
                } else {
                    String::new()
                }
            })
            .unwrap_or_default()
    }

    #[cfg(not(target_os = "ios"))]
    pub fn schedule(_request: IosNotificationRequest) -> String {
        String::new()
    }

    pub fn register_for_push() {
        #[cfg(target_os = "ios")]
        crate::native::register_for_push();
    }

    #[cfg(target_os = "ios")]
    pub fn request_permissions(&self, permissions: IosNotificationPermissions) {
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

    #[cfg(not(target_os = "ios"))]
    pub fn request_permissions(&self, _permissions: IosNotificationPermissions) {}

    /// will respond async via `IosNotificationEvents::PendingNotifications` event
    pub fn request_pending(&self) {
        #[cfg(target_os = "ios")]
        crate::native::request(Request {
            calls: Some(request::Calls::Pending(request::Pending::default())),
        });
    }

    ///
    #[cfg(target_os = "ios")]
    pub fn remove_pending(&self, ids: &[String]) {
        #[cfg(target_os = "ios")]
        crate::native::request(Request {
            calls: Some(request::Calls::RemovePending(request::RemovePending {
                items: ids
                    .into_iter()
                    .map(|id| Into::<NotificationId>::into(id))
                    .collect::<Vec<_>>(),
            })),
        });
    }

    #[cfg(not(target_os = "ios"))]
    pub fn remove_pending(&self, _ids: &[String]) {}

    ///
    #[cfg(target_os = "ios")]
    pub fn remove_delivered(&self, ids: &[String]) {
        crate::native::request(Request {
            calls: Some(request::Calls::RemoveDelivered(request::RemoveDelivered {
                items: ids
                    .into_iter()
                    .map(|id| Into::<NotificationId>::into(id))
                    .collect::<Vec<_>>(),
            })),
        });
    }

    ///
    #[cfg(not(target_os = "ios"))]
    pub fn remove_delivered(&self, _ids: &[String]) {}

    ///
    pub fn remove_all_pending(&self) {
        #[cfg(target_os = "ios")]
        crate::native::request(Request {
            calls: Some(request::Calls::RemoveAllPending(
                request::RemoveAllPending::default(),
            )),
        });
    }

    ///
    pub fn remove_all_delivered(&self) {
        #[cfg(target_os = "ios")]
        crate::native::request(Request {
            calls: Some(request::Calls::RemoveAllDelivered(
                request::RemoveAllDelivered::default(),
            )),
        });
    }
}
