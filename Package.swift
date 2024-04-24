// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "bevy_ios_notifications",
    platforms: [.iOS("15.0")],
    products: [
        // Products define the executables and libraries a package produces, making them visible to other packages.
        .library(
            name: "bevy_ios_notifications",
            targets: ["bevy_ios_notifications"]),
    ],
    
    dependencies: [
        .package(url: "https://github.com/apple/swift-protobuf.git", from: "1.6.0")
    ],
    targets: [
        // Targets are the basic building blocks of a package, defining a module or a test suite.
        // Targets can depend on other targets in this package and products from dependencies.
        .target(
            name: "bevy_ios_notifications",
            dependencies: [.product(name: "SwiftProtobuf", package: "swift-protobuf")]),
        .testTarget(
            name: "bevy_ios_notificationsTests",
            dependencies: ["bevy_ios_notifications"]),
    ]
)
