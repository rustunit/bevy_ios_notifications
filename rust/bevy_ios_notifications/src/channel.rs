use crate::IosNotificationEvents;
use bevy_crossbeam_event::CrossbeamEventSender;
use std::sync::OnceLock;

static SENDER: OnceLock<Option<CrossbeamEventSender<IosNotificationEvents>>> = OnceLock::new();

pub fn send_event(e: IosNotificationEvents) {
    let Some(sender) = SENDER.get().map(Option::as_ref).flatten() else {
        return bevy::log::error!(
            "`IosNotificationsPlugin` not installed correctly (no sender found)"
        );
    };
    sender.send(e);
}

pub fn set_sender(sender: CrossbeamEventSender<IosNotificationEvents>) {
    while SENDER.set(Some(sender.clone())).is_err() {}
}
