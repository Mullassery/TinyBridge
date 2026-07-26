// swift-tools-version:5.10
import PackageDescription

let package = Package(
    name: "TinyBridgeSwift",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "TinyBridgeVZBridge",
            type: .dynamic,
            targets: ["TinyBridgeVZBridge"]
        ),
        .executable(
            name: "TinyBridgeApp",
            targets: ["TinyBridgeApp"]
        ),
    ],
    targets: [
        .target(
            name: "TinyBridgeVZBridge",
            path: "TinyBridgeVZBridge",
            exclude: ["include"],
            publicHeadersPath: "include",
            linkerSettings: [
                .linkedFramework("Virtualization"),
                .linkedFramework("Foundation"),
            ]
        ),
        .testTarget(
            name: "TinyBridgeVZBridgeTests",
            dependencies: ["TinyBridgeVZBridge"]
        ),
        .executableTarget(
            name: "TinyBridgeApp",
            path: "TinyBridgeApp",
            linkerSettings: [
                .linkedFramework("AppKit"),
                .linkedFramework("Foundation"),
            ]
        ),
    ]
)
