# bevy_ios_notifications

[![Following released Bevy versions](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://bevyengine.org/learn/quick-start/plugin-development/#main-branch-tracking)
[![crates.io](https://img.shields.io/crates/v/bevy_ios_notifications.svg)](https://crates.io/crates/bevy_ios_notifications)
[![docs.rs](https://docs.rs/bevy_ios_notifications/badge.svg)](https://docs.rs/bevy_ios_notifications)
[![discord][sh_discord]][lk_discord]

[sh_discord]: https://img.shields.io/discord/1176858176897953872?label=discord&color=5561E6
[lk_discord]: https://discord.gg/rQNeEnMhus

Rust crate to easily integrate iOS's native Notification API into a Bevy application.

Talks to `UserNotifications` directly via [objc2](https://github.com/madsmtm/objc2) - no Swift package or XCode setup required.

https://github.com/rustunit/bevy_ios_notifications/assets/776816/78e9d708-1cdd-4e54-af2a-c6a5b787f98b

Demo from our game using this crate: [zoolitaire.com](https://zoolitaire.com)

## Features

* change/read badge
* get remote push deviceToken
* scheduling local notifications
* enable/disable presenting notifications while app is foregrounded
* and the notoriously hard (in unity) to do things like: what notification was clicked

## Instructions

1. Add Rust dependency
2. Setup Plugin

Use [Apple Push Notification Dashboard](https://icloud.developer.apple.com/dashboard/notifications) to simply send push notifications for testing.

### 1. Add Rust dependency

```
cargo add bevy_ios_notifications
```

or

```
bevy_ios_notifications = { version = "0.8" }
```

### 2. Setup Plugin

Initialize Bevy Plugin:

```rust
// requests permissions for alerts, sounds and badges
app.add_plugins(bevy_ios_notifications::IosNotificationsPlugin::request_permissions_on_start(true, true, true));
```

Trigger Alert in your application code:

```rust
fn system_triggering_notifications(ios_notifications: NonSend<IosNotificationsResource>) {

    // should be called after the permission response arrives
    ios_notifications.registered_for_push();

    // set app icon badge
    ios_notifications.set_badge(1);

    // schedule a local notification
    let id = IosNotificationsResource::schedule(
        IosNotificationRequest::new()
            .title("title")
            .body("body")
            .trigger(IosNotificationTrigger::one_shot(4))
            // if not defined it will be creating a UUID for you
            .identifier("custom id")
            .build(),
    );

}

// this will clear the badge, the notification center and all pending ones
fn process_occluded_events(
    mut e: MessageReader<WindowOccluded>,
    ios_notifications: NonSend<IosNotificationsResource>,
) {
    for ev in e.read() {
        if !ev.occluded {
            ios_notifications.remove_all_pending();
            ios_notifications.remove_all_delivered();
            ios_notifications.set_badge(0);
        }
    }
}

// process async events coming in from ios notification system
fn process_notifications(
    mut events: MessageReader<IosNotificationEvents>,
) {
    for e in events.read() {
        match e {
            IosNotificationEvents::PermissionResponse(_) => todo!(),
            IosNotificationEvents::NotificationSchedulingSucceeded(_) => todo!(),
            IosNotificationEvents::NotificationSchedulingFailed(_) => todo!(),
            IosNotificationEvents::NotificationTriggered(_) => todo!(),
            IosNotificationEvents::PendingNotifications(_) => todo!(),
            IosNotificationEvents::NotificationResponse(_) => todo!(),
            IosNotificationEvents::RemoteNotificationRegistration(_) => todo!(),
        }
    }
}

```

## Implementation Details

* `UNUserNotificationCenter` is called directly through [objc2](https://github.com/madsmtm/objc2), the delegate that receives triggered notifications and their responses is a Rust class ([see code](./src/native.rs))
* device tokens are only handed to the `UIApplicationDelegate`, and [winit](https://github.com/rust-windowing/winit) still does not let you hook into its `didRegisterForRemoteNotificationsWithDeviceToken` callback ([see winit PR (dont hold your breath)](https://github.com/rust-windowing/winit/pull/3650)). So we graft the two remote registration callbacks onto whatever app delegate exists and install our own if there is none ([see code](./src/push.rs))
* this composes with [bevy_ios_app_delegate](https://github.com/rustunit/bevy_ios_app_delegate): if it already installed a delegate we hook that one instead of setting up a second, and an implementation we displace is still called

## Our Other Crates

- [bevy_debug_log](https://github.com/rustunit/bevy_debug_log)
- [bevy_device_lang](https://github.com/rustunit/bevy_device_lang)
- [bevy_web_popups](https://github.com/rustunit/bevy_web_popups)
- [bevy_libgdx_atlas](https://github.com/rustunit/bevy_libgdx_atlas)
- [bevy_ios_review](https://github.com/rustunit/bevy_ios_review)
- [bevy_ios_gamecenter](https://github.com/rustunit/bevy_ios_gamecenter)
- [bevy_ios_alerts](https://github.com/rustunit/bevy_ios_alerts)
- [bevy_ios_iap](https://github.com/rustunit/bevy_ios_iap)
- [bevy_ios_impact](https://github.com/rustunit/bevy_ios_impact)
- [bevy_ios_safearea](https://github.com/rustunit/bevy_ios_safearea)

## Bevy version support

|bevy|bevy\_ios\_notifications|
|----|---|
|0.19|0.7,0.8,main|
|0.18|0.6|
|0.17|0.5|
|0.16|0.4|
|0.15|0.3|
|0.14|0.2|
|0.13|0.1|

# License

All code in this repository is dual-licensed under either:

- MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

at your option. This means you can select the license you prefer.

## Your contributions
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
