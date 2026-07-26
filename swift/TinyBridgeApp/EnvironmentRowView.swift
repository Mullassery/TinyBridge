import SwiftUI

struct EnvironmentRowView: View {
    let environment: EnvironmentSummary
    let onStart: () async -> Void
    let onStop: () async -> Void
    let onOpenShell: () -> Void

    @State private var isStarting = false
    @State private var isStopping = false

    var isRunning: Bool {
        environment.status.lowercased() == "running"
    }

    var body: some View {
        HStack(spacing: 12) {
            // Status indicator
            Image(systemName: isRunning ? "circle.fill" : "circle")
                .font(.caption)
                .foregroundColor(isRunning ? .green : .gray)

            // Environment name
            VStack(alignment: .leading, spacing: 2) {
                Text(environment.name)
                    .font(.body)
                    .fontWeight(.medium)

                if let ip = environment.ip_address {
                    Text(ip)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }

            Spacer()

            // Control buttons
            if isRunning {
                Button(action: {
                    isStopping = true
                    Task {
                        await onStop()
                        isStopping = false
                    }
                }) {
                    Image(systemName: "stop.fill")
                        .font(.caption)
                }
                .buttonStyle(.plain)
                .disabled(isStopping)
                .help("Stop environment")

                Button(action: onOpenShell) {
                    Image(systemName: "terminal.fill")
                        .font(.caption)
                }
                .buttonStyle(.plain)
                .help("Open shell")
            } else {
                Button(action: {
                    isStarting = true
                    Task {
                        await onStart()
                        isStarting = false
                    }
                }) {
                    Image(systemName: "play.fill")
                        .font(.caption)
                }
                .buttonStyle(.plain)
                .disabled(isStarting)
                .help("Start environment")
            }
        }
        .padding(.vertical, 6)
        .padding(.horizontal, 12)
    }
}
