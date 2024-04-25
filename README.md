# bevy_ios_notifications

[![crates.io](https://img.shields.io/crates/v/bevy_ios_notifications.svg)](https://crates.io/crates/bevy_ios_notifications)

Rust crate and Swift package to easily integrate iOS's native Notification API into a Bevy application.

https://github.com/rustunit/bevy_ios_notifications/assets/776816/78e9d708-1cdd-4e54-af2a-c6a5b787f98b

Demo from our game using this crate: [zoolitaire.com](https://zoolitaire.com)

## Features

* change/read badge
* get remote push deviceToken
* scheduling local notifications
* enable/disable presenting notifications while app is foregrounded
* and the notoriously hard (in unity) to do things like: what notification was clicked

## Instructions

1. Add to XCode: Add SPM (Swift Package Manager) dependency
2. Add Rust dependency
3. Setup Plugin

Use [Apple Push Notification Dashboard](https://icloud.developer.apple.com/dashboard/notifications) to simply send push notifications for testing.

### 1. Add to XCode

Go to `File` -> `Add Package Dependencies` and paste `https://github.com/rustunit/bevy_ios_notifications.git` into the search bar on the top right:
![xcode](./assets/xcode-spm.png)

### 2. Add Rust dependency

```
cargo add bevy_ios_notifications
``` 

or 

```
bevy_ios_notifications = { version = "0.1" }
```

### 3. Setup Plugin

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
    mut e: EventReader<WindowOccluded>,
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
    mut events: EventReader<IosNotificationEvents>,
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