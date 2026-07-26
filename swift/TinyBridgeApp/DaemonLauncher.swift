import Foundation

struct DaemonLauncher {
    static let launchAgentLabel = "com.mullassery.tinybridge.daemon"
    static let launchAgentPlistName = "com.mullassery.tinybridge.daemon.plist"

    static func ensureRegistered() {
        let fm = FileManager.default
        guard let home = NSHomeDirectory() as String? else { return }

        let launchAgentDir = (home as NSString).appendingPathComponent("Library/LaunchAgents")
        let launchAgentPath = (launchAgentDir as NSString).appendingPathComponent(launchAgentPlistName)

        // Check if already registered
        if fm.fileExists(atPath: launchAgentPath) {
            debugPrint("LaunchAgent already registered at \(launchAgentPath)")
            return
        }

        // Copy template from app bundle
        guard let templatePath = Bundle.main.resourcePath
            .flatMap({ $0 as NSString? })
            .map({ $0.appendingPathComponent(launchAgentPlistName) }),
            fm.fileExists(atPath: templatePath) else {
            debugPrint("LaunchAgent template not found in bundle")
            return
        }

        // Create LaunchAgents directory if needed
        do {
            try fm.createDirectory(atPath: launchAgentDir, withIntermediateDirectories: true)

            // Read and substitute the template
            let templateContent = try String(contentsOfFile: templatePath, encoding: .utf8)
            let substitutedContent = templateContent.replacingOccurrences(
                of: "__HOME__",
                with: home
            )

            // Write to LaunchAgents
            try substitutedContent.write(toFile: launchAgentPath, atomically: true, encoding: .utf8)

            // Register with launchctl (modern APIs, no sudo needed)
            registerWithLaunchctl(path: launchAgentPath)

            debugPrint("LaunchAgent registered at \(launchAgentPath)")
        } catch {
            debugPrint("Failed to register LaunchAgent: \(error)")
        }
    }

    private static func registerWithLaunchctl(path: String) {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/bin/launchctl")

        guard let uid = getuid() as uid_t? else { return }
        let domain = "gui/\(uid)"

        process.arguments = ["bootstrap", domain, path]

        do {
            try process.run()
            process.waitUntilExit()

            if process.terminationStatus == 0 {
                debugPrint("launchctl bootstrap succeeded")
                enableLaunchAgent()
            } else {
                debugPrint("launchctl bootstrap failed with status \(process.terminationStatus)")
            }
        } catch {
            debugPrint("Failed to run launchctl: \(error)")
        }
    }

    private static func enableLaunchAgent() {
        guard let uid = getuid() as uid_t? else { return }
        let domain = "gui/\(uid)"

        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/bin/launchctl")
        process.arguments = ["enable", "\(domain)/\(launchAgentLabel)"]

        do {
            try process.run()
            process.waitUntilExit()
            if process.terminationStatus == 0 {
                debugPrint("launchctl enable succeeded")
            }
        } catch {
            debugPrint("Failed to enable LaunchAgent: \(error)")
        }
    }
}

// Stub for getuid if not available
func getuid() -> uid_t {
    #if os(macOS)
    return Darwin.getuid()
    #else
    return 0
    #endif
}
