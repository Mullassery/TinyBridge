import AppKit
import SwiftUI

@main
struct TinyBridgeApp: App {
    @StateObject private var appState = AppState()

    var body: some Scene {
        MenuBarExtra("TinyBridge", systemImage: "shippingbox") {
            MenuBarContentView()
                .environmentObject(appState)
                .frame(minWidth: 300, minHeight: 200, maxHeight: 500)
        }
        .menuBarExtraStyle(.window)
        .commands {
            CommandGroup(replacing: CommandGroupPlacement.appInfo) {
                Button(action: { DaemonLauncher.ensureRegistered() }) {
                    Text("Register Daemon")
                }
            }
        }
    }
}
