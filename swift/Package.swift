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
    ],
    targets: [
        // C shim module exposing the tinybridge_vz.h C ABI (TBVMConfig, TBVMStatus, etc.)
        // so the Swift target below can reference those types directly. The actual
        // C-callable symbols (tb_vm_create, tb_vm_start, ...) are implemented in Swift
        // via @_cdecl in TinyBridgeVZBridge and exported from the same dylib.
        .systemLibrary(
            name: "CTinyBridgeVZ",
            path: "Sources/CTinyBridgeVZ"
        ),
        .target(
            name: "TinyBridgeVZBridge",
            dependencies: ["CTinyBridgeVZ"],
            path: "Sources/TinyBridgeVZBridge",
            linkerSettings: [
                .linkedFramework("Virtualization"),
                .linkedFramework("Foundation"),
                .linkedFramework("AppKit"),
            ]
        ),
        .testTarget(
            name: "TinyBridgeVZBridgeTests",
            dependencies: ["TinyBridgeVZBridge"]
        ),
    ]
)
