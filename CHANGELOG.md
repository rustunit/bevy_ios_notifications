# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.8.0] - 2026-08-09

### Changed
* the XCode setup step (adding the SPM package) is no longer needed
* call `UserNotifications` directly via `objc2`, the Swift package dependency is gone
* the protobuf bridge between Rust and Swift is gone
* `IosNotificationsResource` is now `IosNotifications`, a `SystemParam` - take it directly instead
  of through `NonSend`:

  ```rust
  fn system(notifications: IosNotifications) { .. }
  ```
* `register_for_push` and `schedule` take `&self` like every other method.
* `filter_criteria` is now actually applied to the notification
* the message channel is now installed before the first call into the native side, so the response
  to `IosNotificationsPlugin::request_permissions_on_start` can no longer be dropped

### Added
* support for `IosNotificationRequest::sound`
* `IosNotificationPermissions` is exported now

### Fixed
* scheduling with a non-positive interval, or a repeating one below 60 seconds, no longer aborts.
* the foreground presentation completion handler is now always called, not only while `show_foregrounded` is on.
* building the plugin twice no longer hangs in an endless loop while installing the sender

### Removed
* a notification presented while the app is in the foreground no longer implicitly calls `registerForRemoteNotifications`

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
