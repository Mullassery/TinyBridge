import SwiftUI
import AppKit

// MARK: - App Entry Point

@main
struct TinyBridgeApp: App {
    @StateObject private var daemonClient = DaemonClient()
    @State private var showOnboarding = false

    var body: some Scene {
        MenuBarExtra("TinyBridge", systemImage: "desktopcomputer") {
            MenuBarView(
                client: daemonClient,
                showOnboarding: $showOnboarding
            )
        }
        .menuBarExtraStyle(.window)
    }
}

// MARK: - Daemon Client (JSON-RPC Communication)

@MainActor
class DaemonClient: NSObject, ObservableObject {
    @Published var environments: [VMEnvironment] = []
    @Published var isConnected = false
    @Published var error: String?

    private var daemonTask: Process?
    private var monitoringTimer: Timer?
    private let daemonPort = 7890

    override init() {
        super.init()
        startDaemonIfNeeded()
    }

    func startDaemonIfNeeded() {
        // Check if daemon is already running
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/bin/bash")
        task.arguments = ["-c", "lsof -i :\(daemonPort) > /dev/null 2>&1 && echo running || echo stopped"]

        let pipe = Pipe()
        task.standardOutput = pipe

        do {
            try task.run()
            task.waitUntilExit()

            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            let status = String(data: data, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines) ?? "stopped"

            if status == "stopped" {
                launchDaemon()
            }

            isConnected = true
            startMonitoring()
        } catch {
            print("Error checking daemon: \(error)")
            launchDaemon()
        }
    }

    func launchDaemon() {
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/usr/local/bin/tinybridged")
        task.arguments = ["--socket", "/tmp/tinybridge.sock"]

        do {
            try task.run()
            self.daemonTask = task
            DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) {
                self.startMonitoring()
            }
        } catch {
            print("Error launching daemon: \(error)")
            self.error = "Failed to start TinyBridge daemon"
        }
    }

    func startMonitoring() {
        monitoringTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            self?.fetchEnvironmentStatus()
        }

        // Initial fetch
        fetchEnvironmentStatus()
    }

    func fetchEnvironmentStatus() {
        // Make HTTP request to daemon
        let url = URL(string: "http://127.0.0.1:\(daemonPort)/tinybridge")!
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        let jsonRpc = """
        {
            "jsonrpc": "2.0",
            "method": "environment.list",
            "params": {},
            "id": 1
        }
        """

        request.httpBody = jsonRpc.data(using: .utf8)

        URLSession.shared.dataTask(with: request) { [weak self] data, response, error in
            guard let data = data, error == nil else { return }

            do {
                let json = try JSONSerialization.jsonObject(with: data) as? [String: Any]
                if let result = json?["result"] as? [[String: Any]] {
                    DispatchQueue.main.async {
                        self?.environments = result.compactMap { env in
                            guard let name = env["name"] as? String else { return nil }
                            return VMEnvironment(
                                name: name,
                                running: env["status"] as? String == "running",
                                cpuCores: env["cpu_cores"] as? Int ?? 4,
                                memory: "\(env["memory_gb"] as? Int ?? 8)",
                                uptime: env["uptime"] as? Int ?? 0
                            )
                        }
                    }
                }
            } catch {
                print("Error parsing response: \(error)")
            }
        }.resume()
    }

    func startEnvironment(_ name: String) {
        sendCommand("environment.up", params: ["name": name])
    }

    func stopEnvironment(_ name: String) {
        sendCommand("environment.down", params: ["name": name])
    }

    func createEnvironment(
        name: String,
        template: String,
        cores: Int,
        memory: Int
    ) {
        sendCommand("environment.create", params: [
            "name": name,
            "template": template,
            "cpu_cores": cores,
            "memory_gb": memory
        ])
    }

    func openShell(_ name: String) {
        // Launch Terminal with tinybridge shell command
        let script = "open -a Terminal --args tinybridge shell \(name)"
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/bin/bash")
        task.arguments = ["-c", script]
        try? task.run()
    }

    private func sendCommand(_ method: String, params: [String: Any]) {
        let url = URL(string: "http://127.0.0.1:\(daemonPort)/tinybridge")!
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        do {
            let jsonRpc = [
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
                "id": 1
            ] as [String: Any]

            request.httpBody = try JSONSerialization.data(withJSONObject: jsonRpc)

            URLSession.shared.dataTask(with: request) { _, _, _ in
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                    self.fetchEnvironmentStatus()
                }
            }.resume()
        } catch {
            print("Error sending command: \(error)")
        }
    }

    deinit {
        monitoringTimer?.invalidate()
    }
}

// MARK: - Data Models

struct VMEnvironment: Identifiable {
    let id = UUID()
    let name: String
    let running: Bool
    let cpuCores: Int
    let memory: String
    let uptime: Int
}

// MARK: - Menu Bar View

struct MenuBarView: View {
    @ObservedObject var client: DaemonClient
    @Binding var showOnboarding: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            // Header
            HStack {
                Text("TinyBridge")
                    .font(.headline)
                    .fontWeight(.bold)

                Spacer()

                if client.isConnected {
                    HStack(spacing: 4) {
                        Circle()
                            .fill(Color.green)
                            .frame(width: 8, height: 8)
                        Text("Online")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                } else {
                    HStack(spacing: 4) {
                        Circle()
                            .fill(Color.red)
                            .frame(width: 8, height: 8)
                        Text("Offline")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
            }

            Divider()

            // Environments List
            if client.environments.isEmpty {
                VStack(alignment: .center, spacing: 8) {
                    Image(systemName: "folder.badge.questionmark")
                        .font(.system(size: 32))
                        .foregroundColor(.secondary)

                    Text("No Environments")
                        .font(.body)
                        .fontWeight(.medium)

                    Text("Create your first environment")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .frame(maxWidth: .infinity)
                .padding()
            } else {
                ScrollView {
                    VStack(alignment: .leading, spacing: 8) {
                        ForEach(client.environments) { env in
                            EnvironmentRow(
                                environment: env,
                                client: client
                            )
                        }
                    }
                }
                .frame(maxHeight: 300)
            }

            Divider()

            // Quick Actions
            VStack(alignment: .leading, spacing: 4) {
                Button(action: { showOnboarding = true }) {
                    HStack {
                        Image(systemName: "plus.circle.fill")
                            .foregroundColor(.blue)
                        Text("New Environment")
                    }
                }
                .buttonStyle(.plain)

                Button(action: { openPreferences() }) {
                    HStack {
                        Image(systemName: "gear")
                            .foregroundColor(.gray)
                        Text("Preferences")
                    }
                }
                .buttonStyle(.plain)

                Divider()

                Button(action: { NSApplication.shared.terminate(nil) }) {
                    HStack {
                        Image(systemName: "power")
                            .foregroundColor(.red)
                        Text("Quit TinyBridge")
                    }
                }
                .buttonStyle(.plain)
            }
        }
        .padding(12)
        .frame(minWidth: 300)
        .sheet(isPresented: $showOnboarding) {
            OnboardingSheet(client: client, isPresented: $showOnboarding)
        }
    }
}

// MARK: - Environment Row

struct EnvironmentRow: View {
    let environment: VMEnvironment
    @ObservedObject var client: DaemonClient
    @State private var showContextMenu = false

    var body: some View {
        HStack(spacing: 12) {
            // Status indicator
            VStack(spacing: 2) {
                Circle()
                    .fill(environment.running ? Color.green : Color.gray)
                    .frame(width: 10, height: 10)

                Spacer()
            }
            .frame(height: 40)

            // Environment info
            VStack(alignment: .leading, spacing: 2) {
                Text(environment.name)
                    .font(.body)
                    .fontWeight(.semibold)

                Text(environment.running ?
                    "Running • \(environment.cpuCores) cores • \(environment.memory)GB" :
                    "Stopped")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Spacer()

            // Action buttons
            HStack(spacing: 8) {
                if environment.running {
                    Button(action: { client.stopEnvironment(environment.name) }) {
                        Image(systemName: "stop.fill")
                            .font(.caption)
                            .foregroundColor(.red)
                    }
                    .buttonStyle(.plain)
                    .help("Stop environment")

                    Button(action: { client.openShell(environment.name) }) {
                        Image(systemName: "terminal.fill")
                            .font(.caption)
                            .foregroundColor(.blue)
                    }
                    .buttonStyle(.plain)
                    .help("Open shell")
                } else {
                    Button(action: { client.startEnvironment(environment.name) }) {
                        Image(systemName: "play.fill")
                            .font(.caption)
                            .foregroundColor(.green)
                    }
                    .buttonStyle(.plain)
                    .help("Start environment")
                }
            }
        }
        .padding(8)
        .background(Color(nsColor: .controlBackgroundColor))
        .cornerRadius(6)
        .contextMenu {
            Button("View Details") { }
            Button("Edit Configuration") { }
            Button("View Logs") { }
            Divider()
            Button("Delete", action: { })
                .foregroundColor(.red)
        }
    }
}

// MARK: - Onboarding Sheet

struct OnboardingSheet: View {
    @ObservedObject var client: DaemonClient
    @Binding var isPresented: Bool

    @State private var currentStep = 0
    @State private var selectedTemplate = "python"
    @State private var environmentName = ""
    @State private var cpuCores = 4
    @State private var memory = 8
    @State private var isCreating = false

    var body: some View {
        VStack(spacing: 0) {
            // Progress indicator
            HStack {
                ForEach(0..<5, id: \.self) { step in
                    VStack {
                        Circle()
                            .fill(step <= currentStep ? Color.blue : Color.gray.opacity(0.3))
                            .frame(width: 8, height: 8)
                    }

                    if step < 4 {
                        Spacer()
                    }
                }
            }
            .padding()

            Divider()

            // Step content
            Group {
                if currentStep == 0 {
                    WelcomeStepView()
                } else if currentStep == 1 {
                    TemplateStepView(selected: $selectedTemplate)
                } else if currentStep == 2 {
                    ResourcesStepView(cores: $cpuCores, memory: $memory)
                } else if currentStep == 3 {
                    NameStepView(name: $environmentName)
                } else {
                    ReviewStepView(
                        template: selectedTemplate,
                        name: environmentName,
                        cores: cpuCores,
                        memory: memory
                    )
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .padding()

            Divider()

            // Navigation buttons
            HStack {
                if currentStep > 0 {
                    Button("Back") {
                        currentStep -= 1
                    }
                }

                Spacer()

                if currentStep < 4 {
                    Button("Next") {
                        currentStep += 1
                    }
                    .keyboardShortcut(.return, modifiers: [])
                } else {
                    Button(action: {
                        isCreating = true
                        client.createEnvironment(
                            name: environmentName,
                            template: selectedTemplate,
                            cores: cpuCores,
                            memory: memory
                        )

                        DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) {
                            isCreating = false
                            isPresented = false

                            // Show notification
                            let notification = NSUserNotification()
                            notification.title = "Environment Created"
                            notification.subtitle = "✅ \(environmentName) is ready"
                            notification.informativeText = "Click to open shell"
                            notification.soundName = NSUserNotificationDefaultSoundName
                            NSUserNotificationCenter.default.deliver(notification)
                        }
                    }) {
                        if isCreating {
                            HStack(spacing: 6) {
                                ProgressView()
                                    .scaleEffect(0.8)
                                Text("Creating...")
                            }
                        } else {
                            Text("Create Environment")
                                .frame(minWidth: 140)
                        }
                    }
                    .disabled(isCreating || environmentName.isEmpty)
                    .keyboardShortcut(.return, modifiers: [])
                }
            }
            .padding()
        }
        .frame(minWidth: 500, minHeight: 450)
    }
}

// MARK: - Onboarding Steps

struct WelcomeStepView: View {
    var body: some View {
        VStack(alignment: .center, spacing: 20) {
            Image(systemName: "macwindow")
                .font(.system(size: 60))
                .foregroundColor(.blue)

            VStack(alignment: .center, spacing: 8) {
                Text("Welcome to TinyBridge")
                    .font(.title2)
                    .fontWeight(.bold)

                Text("Create your first Linux environment in just a few clicks")
                    .font(.body)
                    .foregroundColor(.secondary)
            }

            VStack(alignment: .leading, spacing: 8) {
                HStack(spacing: 8) {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(.green)
                    Text("Zero configuration required")
                }

                HStack(spacing: 8) {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(.green)
                    Text("Pre-configured templates")
                }

                HStack(spacing: 8) {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(.green)
                    Text("Instant Linux shell")
                }
            }
            .font(.body)

            Spacer()
        }
    }
}

struct TemplateStepView: View {
    @Binding var selected: String

    let templates: [(id: String, emoji: String, name: String, desc: String)] = [
        ("python", "🐍", "Python", "Python 3.11 + NumPy + Pandas"),
        ("rust", "🦀", "Rust", "Rust 1.70 + Cargo"),
        ("ros2", "🤖", "ROS 2", "ROS 2 Humble + DDS"),
        ("ml", "🧠", "ML", "PyTorch + CUDA ready"),
    ]

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Select Template")
                .font(.title3)
                .fontWeight(.bold)

            Text("Choose a pre-configured environment")
                .font(.caption)
                .foregroundColor(.secondary)

            VStack(spacing: 8) {
                ForEach(templates, id: \.id) { template in
                    TemplateOption(
                        template: template,
                        isSelected: selected == template.id,
                        action: { selected = template.id }
                    )
                }
            }

            Spacer()
        }
    }
}

struct TemplateOption: View {
    let template: (id: String, emoji: String, name: String, desc: String)
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 12) {
                Text(template.emoji)
                    .font(.title2)

                VStack(alignment: .leading, spacing: 2) {
                    Text(template.name)
                        .font(.body)
                        .fontWeight(.semibold)
                    Text(template.desc)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Spacer()

                if isSelected {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(.blue)
                }
            }
            .padding(12)
            .background(isSelected ? Color.blue.opacity(0.1) : Color.gray.opacity(0.05))
            .cornerRadius(8)
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(isSelected ? Color.blue : Color.clear, lineWidth: 2)
            )
        }
        .buttonStyle(.plain)
    }
}

struct ResourcesStepView: View {
    @Binding var cores: Int
    @Binding var memory: Int

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            Text("Configure Resources")
                .font(.title3)
                .fontWeight(.bold)

            VStack(alignment: .leading, spacing: 12) {
                HStack {
                    Text("CPU Cores: \(cores)")
                        .font(.body)
                        .fontWeight(.semibold)
                    Spacer()
                    Text("(max 8)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Slider(value: .init(get: { Double(cores) }, set: { cores = Int($0) }),
                       in: 1...8, step: 1)
            }

            VStack(alignment: .leading, spacing: 12) {
                HStack {
                    Text("Memory: \(memory)GB")
                        .font(.body)
                        .fontWeight(.semibold)
                    Spacer()
                    Text("(max 16GB)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Slider(value: .init(get: { Double(memory) }, set: { memory = Int($0) }),
                       in: 1...16, step: 1)
            }

            Spacer()
        }
    }
}

struct NameStepView: View {
    @Binding var name: String

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Environment Name")
                .font(.title3)
                .fontWeight(.bold)

            Text("Choose a unique name for your environment")
                .font(.caption)
                .foregroundColor(.secondary)

            TextField("e.g., my-project", text: $name)
                .textFieldStyle(.roundedBorder)
                .padding()
                .background(Color.gray.opacity(0.05))
                .cornerRadius(8)

            Spacer()
        }
    }
}

struct ReviewStepView: View {
    let template: String
    let name: String
    let cores: Int
    let memory: Int

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Review Configuration")
                .font(.title3)
                .fontWeight(.bold)

            VStack(alignment: .leading, spacing: 12) {
                ReviewRow(label: "Name", value: name)
                ReviewRow(label: "Template", value: template.capitalized)
                ReviewRow(label: "CPU", value: "\(cores) cores")
                ReviewRow(label: "Memory", value: "\(memory)GB")
            }
            .padding()
            .background(Color.blue.opacity(0.05))
            .cornerRadius(8)

            Text("✅ All set! Click 'Create Environment' to launch")
                .font(.caption)
                .foregroundColor(.green)

            Spacer()
        }
    }
}

struct ReviewRow: View {
    let label: String
    let value: String

    var body: some View {
        HStack {
            Text(label)
                .foregroundColor(.secondary)
            Spacer()
            Text(value)
                .fontWeight(.semibold)
        }
    }
}

// MARK: - Utilities

func openPreferences() {
    // Open System Preferences or preferences window
    NSApplication.shared.sendAction(
        #selector(NSApplication.orderFrontPreferencesWindow(_:)),
        to: NSApplication.shared.delegate,
        from: nil
    )
}
