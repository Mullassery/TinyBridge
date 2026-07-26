import SwiftUI

struct MenuBarContentView: View {
    @EnvironmentObject var appState: AppState

    var body: some View {
        VStack(spacing: 12) {
            // Header with status
            HStack {
                Image(systemName: appState.daemonConnected ? "circle.fill" : "circle")
                    .foregroundColor(appState.daemonConnected ? .green : .red)
                Text(appState.daemonConnected ? "Connected" : "Daemon not running")
                    .font(.subheadline)
                Spacer()
                Button(action: {
                    Task {
                        await appState.refreshEnvironments()
                    }
                }) {
                    Image(systemName: "arrow.clockwise")
                        .font(.caption)
                }
                .buttonStyle(.plain)
                .disabled(appState.isLoading)
            }
            .padding(.horizontal, 12)
            .padding(.top, 12)

            Divider()

            // Environments list
            if appState.environments.isEmpty {
                if appState.daemonConnected {
                    Text("No environments running")
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .frame(maxWidth: .infinity, alignment: .center)
                        .padding(.vertical, 20)
                } else {
                    Text("Daemon not responding")
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .frame(maxWidth: .infinity, alignment: .center)
                        .padding(.vertical, 20)
                }
            } else {
                List(appState.environments) { env in
                    EnvironmentRowView(
                        environment: env,
                        onStart: { await appState.startEnvironment(env.name) },
                        onStop: { await appState.stopEnvironment(env.name) },
                        onOpenShell: { appState.openShell(env.name) }
                    )
                    .listRowInsets(EdgeInsets(top: 0, leading: 0, bottom: 0, trailing: 0))
                }
                .listStyle(.plain)
                .frame(maxHeight: .infinity)
            }

            Divider()

            // Footer
            HStack(spacing: 8) {
                Button(action: {
                    NSApplication.shared.terminate(nil)
                }) {
                    Image(systemName: "xmark.circle.fill")
                        .font(.caption)
                }
                .buttonStyle(.plain)
                .help("Quit")

                Spacer()

                if let error = appState.lastUpdateError {
                    Text(error)
                        .font(.caption2)
                        .foregroundColor(.orange)
                        .lineLimit(1)
                }
            }
            .padding(.horizontal, 12)
            .padding(.bottom, 12)
        }
        .frame(minHeight: 200)
    }
}
