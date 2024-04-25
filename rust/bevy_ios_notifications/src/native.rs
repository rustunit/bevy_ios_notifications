#![allow(temporary_cstring_as_ptr)]

use std::ffi;

use std::sync::OnceLock;

use bevy::prelude::*;
use prost::{bytes::BytesMut, Message};

use crate::{
    plugin::IosRemoteNotificationRegistration, AsyncEvent, IosNotificationEvents,
    IosNotificationResponse, Request, Response,
};

use bevy_crossbeam_event::CrossbeamEventSender;
use block2::{Block, RcBlock};

extern "C" {
    fn swift_notifications_init(cb: &Block<dyn Fn(*const ffi::c_uchar, ffi::c_uint) -> ()>);
    fn swift_notifications_badge_set(number: i32);
    fn swift_notifications_badge_get() -> i32;
    fn swift_notifications_registered_for_push() -> bool;
    fn swift_notifications_register_for_push();
    fn swift_notifications_show_in_foreground_set(show: bool);
    fn swift_notifications_show_in_foreground_get() -> bool;
    fn ios_notifications_request(
        buffer: *mut ffi::c_uchar,
        buffer_size: ffi::c_int,
        buffer_used: *mut ffi::c_int,
    ) -> bool;
}

static SENDER: OnceLock<Option<CrossbeamEventSender<IosNotificationEvents>>> = OnceLock::new();

pub fn set_sender(sender: CrossbeamEventSender<IosNotificationEvents>) {
    while !SENDER.set(Some(sender.clone())).is_ok() {}
}

pub fn init() {
    let block = RcBlock::new(|data, length| {
        let buf = unsafe { std::slice::from_raw_parts(data as *const u8, length as usize) };
        let response = AsyncEvent::decode(buf).unwrap();

        if let Some(calls) = response.calls {
            let response = match calls {
                crate::async_event::Calls::Permission(perm) => {
                    Some(IosNotificationEvents::PermissionResponse(perm.granted))
                }
                crate::async_event::Calls::Scheduled(v) => Some(if v.success {
                    IosNotificationEvents::NotificationSchedulingSucceeded(v.identifier)
                } else {
                    IosNotificationEvents::NotificationSchedulingFailed(v.identifier)
                }),
                crate::async_event::Calls::Pending(v) => {
                    Some(IosNotificationEvents::PendingNotifications(
                        v.items.into_iter().map(|item| item.identifier).collect(),
                    ))
                }
                crate::async_event::Calls::TriggeredWhileRunning(v) => {
                    Some(IosNotificationEvents::NotificationTriggered(v.identifier))
                }
                crate::async_event::Calls::NotificationResponse(v) => Some(
                    IosNotificationEvents::NotificationResponse(IosNotificationResponse {
                        identifier: v.identifier,
                        action: v.action_identifier,
                    }),
                ),
                crate::async_event::Calls::RemoteNotificationRegistration(v) => {
                    convert(v).map(|v| IosNotificationEvents::RemoteNotificationRegistration(v))
                }
            };

            if let Some(e) = response {
                debug!("forward native event: {e:?}");
                SENDER.get().unwrap().as_ref().unwrap().send(e);
            }
        }
    });

    unsafe {
        swift_notifications_init(&block);
    }
}

fn convert(
    v: crate::async_event::RemoteNotificationRegistration,
) -> Option<IosRemoteNotificationRegistration> {
    match v.results {
        Some(crate::async_event::remote_notification_registration::Results::Failed(
            crate::async_event::remote_notification_registration::Failed {
                localized_description,
                code,
            },
        )) => Some(IosRemoteNotificationRegistration::Failed {
            code: code,
            localized_description: localized_description,
        }),
        Some(crate::async_event::remote_notification_registration::Results::Token(
            crate::async_event::remote_notification_registration::DeviceToken { token },
        )) => Some(IosRemoteNotificationRegistration::DeviceToken(token)),
        None => None,
    }
}

pub fn badge_set(v: i32) {
    unsafe {
        swift_notifications_badge_set(v);
    }
}

pub fn badge_get() -> i32 {
    unsafe { swift_notifications_badge_get() }
}

pub fn show_foregrounded() -> bool {
    unsafe { swift_notifications_show_in_foreground_get() }
}

pub fn set_show_foregrounded(v: bool) {
    unsafe {
        swift_notifications_show_in_foreground_set(v);
    }
}

pub fn registered_for_push() -> bool {
    unsafe { swift_notifications_registered_for_push() }
}

pub fn register_for_push() {
    unsafe { swift_notifications_register_for_push() }
}

pub fn request(request: Request) -> Response {
    // let block = RcBlock::new(|_data, len| {
    //     info!("schedule cb: {len}");
    // });
    let mut buffer = BytesMut::with_capacity(1024);
    let mut data_out: ffi::c_int = request.encoded_len() as ffi::c_int;

    unsafe {
        request.encode(&mut buffer).unwrap();

        let result = ios_notifications_request(
            buffer.as_mut_ptr(),
            buffer.capacity() as ffi::c_int,
            &mut data_out as &mut ffi::c_int,
        );

        if !result {
            error!("request failed");
        }

        buffer.set_len(data_out as usize);
    }

    return Response::decode(buffer).unwrap();
}
