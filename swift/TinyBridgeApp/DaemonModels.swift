import Foundation

struct EnvironmentSummary: Codable, Identifiable {
    let id: String
    let name: String
    let status: String
    let ip_address: String?
    let uptime_secs: Int?

    enum CodingKeys: String, CodingKey {
        case id
        case name
        case status
        case ip_address
        case uptime_secs
    }
}

struct ListResponse: Codable {
    let environments: [EnvironmentSummary]
}

struct StatusResponse: Codable {
    let status: String
    let connected: Bool?
}
