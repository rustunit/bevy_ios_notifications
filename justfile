build:
	cd rust/bevy_ios_notifications/src && cargo b
	cd rust/bevy_ios_notifications/src && protoc --swift_out=../../../Sources/bevy_ios_notifications Data.proto

