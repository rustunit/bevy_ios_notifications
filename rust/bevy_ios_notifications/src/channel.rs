use crate::IosNotificationEvents;
use bevy_channel_message::ChannelMessageSender;
use std::sync::OnceLock;

static SENDER: OnceLock<Option<ChannelMessageSender<IosNotificationEvents>>> = OnceLock::new();

pub fn send_event(e: IosNotificationEvents) {
    let Some(sender) = SENDER.get().map(Option::as_ref).flatten() else {
        return bevy_log::error!(
            "`IosNotificationsPlugin` not installed correctly (no sender found)"
        );
    };
    sender.send(e);
}

pub fn set_sender(sender: ChannelMessageSender<IosNotificationEvents>) {
    while SENDER.set(Some(sender.clone())).is_err() {}
}
