use crate::IosNotificationEvents;
use bevy_channel_message::ChannelMessageSender;
use std::sync::OnceLock;

static SENDER: OnceLock<ChannelMessageSender<IosNotificationEvents>> = OnceLock::new();

/// A callback can arrive before the plugin was built; dropping the event is the only option, since
/// unwinding out of an Objective-C block would be undefined behavior.
pub fn send_event(e: IosNotificationEvents) {
    let Some(sender) = SENDER.get() else {
        return bevy_log::error!(
            "`IosNotificationsPlugin` not installed correctly (no sender found)"
        );
    };

    bevy_log::debug!("forward native event: {e:?}");

    sender.send(e);
}

pub fn set_sender(sender: ChannelMessageSender<IosNotificationEvents>) {
    let _ = SENDER.set(sender);
}
