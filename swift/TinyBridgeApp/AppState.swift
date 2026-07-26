import Foundation
import SwiftUI
import Combine

class AppState: NSObject, ObservableObject {
    @Published var daemonConnected = false
    @Published var environments: [EnvironmentSummary] = []
    @Published var isLoading = false
    @Published var lastUpdateError: String?

    private(set) var objectWillChange = PassthroughSubject<Void, Never>()
    private var pollTimer: Timer?
    private let pollInterval = 3.0

    override init() {
        super.init()
        startPolling()
    }

    func startPolling() {
        stopPolling()
        pollTimer = Timer.scheduledTimer(withTimeInterval: pollInterval, repeats: true) { [weak self] _ in
            Task {
                await self?.refreshEnvironments()
            }
        }
        Task {
            await refreshEnvironments()
        }
    }

    func stopPolling() {
        pollTimer?.invalidate()
        pollTimer = nil
    }

    @MainActor
    func refreshEnvironments() async {
        isLoading = true
        do {
            let (connected, envs) = try await CLIBridge.listEnvironments()
            self.daemonConnected = connected
            self.environments = envs
            self.lastUpdateError = nil
        } catch {
            self.daemonConnected = false
            self.environments = []
            self.lastUpdateError = error.localizedDescription
        }
        isLoading = false
    }

    @MainActor
    func startEnvironment(_ name: String) async {
        do {
            try await CLIBridge.upEnvironment(name)
            await refreshEnvironments()
        } catch {
            lastUpdateError = "Failed to start environment: \(error)"
        }
    }

    @MainActor
    func stopEnvironment(_ name: String) async {
        do {
            try await CLIBridge.downEnvironment(name)
            await refreshEnvironments()
        } catch {
            lastUpdateError = "Failed to stop environment: \(error)"
        }
    }

    @MainActor
    func openShell(_ name: String) {
        CLIBridge.openShellTerminal(environment: name)
    }

    deinit {
        stopPolling()
    }
}
