# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed
* `IosNotificationsResource` is now `IosNotifications`, a `SystemParam` - take it directly instead
  of through `NonSend`:

  ```rust
  fn system(notifications: IosNotifications) { .. }
  ```

  It was never a `Resource` in practice: the plugin only ever registered it with `init_non_send`,
  but the derive let it be inserted as one and reached through `Res<_>` from a worker thread,
  which is the one thing the `NonSend` access exists to prevent. The `Clone` impl was a second
  way past the same guarantee and is gone too
* `register_for_push` and `schedule` take `&self` like every other method. `register_for_push`
  reaches UIKit and only works on the main thread; as associated functions they were callable from
  any system, so it silently did nothing when the executor happened to pick a worker thread. As
  methods on `IosNotifications` the main thread is guaranteed at compile time

## [0.8.0] - 2026-08-09

### Changed
* call `UserNotifications` directly via `objc2`, the Swift package dependency is gone
* the XCode setup step (adding the SPM package) is no longer needed
* repo flattened: the crate now lives at the repository root
* the protobuf bridge between Rust and Swift is gone, and with it the `prost`/`prost-build`
  dependencies and the `protoc` build requirement
* `filter_criteria` is now actually applied to the notification - the Swift side used to drop it
  (needs iOS 16 or newer, it is ignored with a warning below that)
* the message channel is now installed before the first call into the native side, so the response
  to `IosNotificationsPlugin::request_permissions_on_start` can no longer be dropped

### Added
* `IosNotificationPermissions` is exported - `IosNotificationsResource::request_permissions` takes
  it, but there was no way to name the type

### Fixed
* the foreground presentation completion handler is now always called, not only while
  `show_foregrounded` is on. Withholding it made the system log and stall on every notification
  that fired with presentation disabled
* building the plugin twice no longer hangs in an endless loop while installing the sender

### Removed
* a notification presented while the app is in the foreground no longer implicitly calls
  `registerForRemoteNotifications` - use `IosNotificationsResource::register_for_push` for that
* the generated protobuf types. They were the wire format of the Swift bridge, but `include!`ed at
  the crate root, so on iOS builds `Request`, `Response`, `AsyncEvent`, `NotificationId`, `UserData`
  and the `request`/`response`/`async_event` modules were public. Nothing else in the API changed

## [0.7.0] - 2026-08-09

### Changed
* support bevy `0.19`

## [0.6.0] - 2026-01-17

### Changed
* support bevy `0.18`

## [0.5.0] - 2025-10-18

### Changed
* support bevy `0.17`

## [0.4.0] - 2025-04-26

### Changed
* support bevy `0.16`

## [0.3.0] - 2024-12-02

### Changed
* support bevy `0.15`

## [0.2.3] - 2024-11-26

### Changed
* add `remove_pending` and `remove_delivered` methods to remove specific IDs

## [0.2.1] - 2024-11-02

### Fixed
* fixes `request failed` errors because of mismatch in signature

## [0.2.0] - 2024-07-12

### Changed
* update to bevy `0.14`
