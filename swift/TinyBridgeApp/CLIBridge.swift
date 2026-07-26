import Foundation
import AppKit

struct CLIBridge {
    static let binaryName = "tinybridge"

    static func tinyBridgeBinary() -> String {
        let bundle = Bundle.main
        let exe = bundle.executablePath ?? bundle.bundlePath
        let baseURL = URL(fileURLWithPath: exe).deletingLastPathComponent().deletingLastPathComponent()
        return baseURL.appendingPathComponent("MacOS/\(binaryName)").path
    }

    static func runCommand(_ args: [String]) async throws -> String {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: tinyBridgeBinary())
        process.arguments = args

        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = pipe

        try process.run()
        process.waitUntilExit()

        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        guard let output = String(data: data, encoding: .utf8) else {
            throw CLIError.decodingFailed
        }

        if process.terminationStatus != 0 {
            throw CLIError.commandFailed(output.trimmingCharacters(in: .whitespacesAndNewlines))
        }

        return output
    }

    static func listEnvironments() async throws -> (Bool, [EnvironmentSummary]) {
        do {
            let output = try await runCommand(["list", "--json"])
            let decoder = JSONDecoder()
            let response = try decoder.decode(ListResponse.self, from: output.data(using: .utf8)!)
            return (true, response.environments)
        } catch {
            return (false, [])
        }
    }

    static func upEnvironment(_ name: String) async throws {
        _ = try await runCommand(["up", name])
    }

    static func downEnvironment(_ name: String) async throws {
        _ = try await runCommand(["down", name])
    }

    static func openShellTerminal(environment: String) {
        let script = """
        tell application "Terminal"
            do script "\(tinyBridgeBinary()) shell \(environment)"
            activate
        end tell
        """

        var error: NSDictionary?
        if let script = NSAppleScript(source: script) {
            script.executeAndReturnError(&error)
            if error != nil {
                DispatchQueue.main.async {
                    let alert = NSAlert()
                    alert.messageText = "Failed to open terminal"
                    alert.informativeText = error?.description ?? "Unknown error"
                    alert.runModal()
                }
            }
        }
    }
}

enum CLIError: LocalizedError {
    case decodingFailed
    case commandFailed(String)
    case binaryNotFound

    var errorDescription: String? {
        switch self {
        case .decodingFailed:
            return "Failed to decode response"
        case .commandFailed(let msg):
            return "Command failed: \(msg)"
        case .binaryNotFound:
            return "TinyBridge binary not found"
        }
    }
}
