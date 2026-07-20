import Foundation

public enum TinyBridgeVZBridge {
    public static let version = "0.1.0"

    public static var isAvailable: Bool {
        tb_is_available()
    }
}
