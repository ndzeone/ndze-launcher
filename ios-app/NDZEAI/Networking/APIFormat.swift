import Foundation

enum APIFormat: String, CaseIterable, Identifiable, Codable, Sendable {
    case openAI
    case anthropic
    case custom

    var id: String { rawValue }

    var title: String {
        switch self {
        case .openAI: "OpenAI-совместимый"
        case .anthropic: "Anthropic Messages"
        case .custom: "Свой формат"
        }
    }

    var defaultPath: String {
        switch self {
        case .openAI, .custom: "/v1/chat/completions"
        case .anthropic: "/v1/messages"
        }
    }

    var defaultAuthHeader: String {
        self == .anthropic ? "x-api-key" : "Authorization"
    }

    var defaultAuthPrefix: String {
        self == .anthropic ? "" : "Bearer "
    }

    var defaultResponsePath: String {
        self == .anthropic ? "content.0.text" : "choices.0.message.content"
    }

    var supportsStreaming: Bool { self != .custom }

    static let defaultTemplate = #"{"model": {{model}}, "messages": {{messages}}}"#
}

struct ChatTurn: Sendable, Equatable {
    let role: String
    let content: String
}

struct EndpointConfig: Sendable {
    var baseURL: String
    var path: String
    var format: APIFormat
    var model: String
    var apiKey: String
    var authHeader: String
    var authPrefix: String
    var bodyTemplate: String
    var responsePath: String
    var systemPrompt: String
    var streaming: Bool

    var url: URL? {
        let base = baseURL.trimmingCharacters(in: .whitespaces).trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        let trimmedPath = path.trimmingCharacters(in: .whitespaces)
        let suffix = trimmedPath.isEmpty || trimmedPath.hasPrefix("/") ? trimmedPath : "/" + trimmedPath
        guard let url = URL(string: base + suffix), let scheme = url.scheme?.lowercased(),
              scheme == "https" || scheme == "http", url.host != nil else { return nil }
        return url
    }

    var usesStreaming: Bool { streaming && format.supportsStreaming }
}
