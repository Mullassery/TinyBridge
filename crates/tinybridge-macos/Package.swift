// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "TinyBridgeApp",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .executable(name: "TinyBridgeApp", targets: ["TinyBridgeApp"])
    ],
    dependencies: [],
    targets: [
        .executableTarget(
            name: "TinyBridgeApp",
            dependencies: [],
            resources: [
                .process("Resources/Assets.xcassets")
            ]
        )
    ]
)
