import Foundation
import SwiftData

@Model
final class Connection {
    @Attribute(.unique) var id: UUID = UUID()
    var name: String = ""
    var baseURL: String = ""
    var formatRaw: String = APIFormat.openAI.rawValue
    var model: String = ""
    var path: String = APIFormat.openAI.defaultPath
    var authHeader: String = APIFormat.openAI.defaultAuthHeader
    var authPrefix: String = APIFormat.openAI.defaultAuthPrefix
    var bodyTemplate: String = APIFormat.defaultTemplate
    var responsePath: String = APIFormat.openAI.defaultResponsePath
    var systemPrompt: String = ""
    var streaming: Bool = true
    var maskedKey: String = ""
    var createdAt: Date = Date()

    @Relationship(deleteRule: .cascade, inverse: \Conversation.connection)
    var conversations: [Conversation] = []

    init(name: String) {
        self.name = name
    }

    var format: APIFormat {
        get { APIFormat(rawValue: formatRaw) ?? .openAI }
        set { formatRaw = newValue.rawValue }
    }

    var keychainAccount: String { id.uuidString }

    func endpoint(apiKey: String) -> EndpointConfig {
        EndpointConfig(
            baseURL: baseURL,
            path: path,
            format: format,
            model: model,
            apiKey: apiKey,
            authHeader: authHeader,
            authPrefix: authPrefix,
            bodyTemplate: bodyTemplate,
            responsePath: responsePath,
            systemPrompt: systemPrompt,
            streaming: streaming
        )
    }
}

@Model
final class Conversation {
    static let defaultTitle = "Новый чат"

    @Attribute(.unique) var id: UUID = UUID()
    var title: String = Conversation.defaultTitle
    var createdAt: Date = Date()
    var updatedAt: Date = Date()
    var connection: Connection?

    @Relationship(deleteRule: .cascade, inverse: \ChatMessage.conversation)
    var messages: [ChatMessage] = []

    init(connection: Connection) {
        self.connection = connection
    }

    var orderedMessages: [ChatMessage] {
        messages.sorted { $0.index < $1.index }
    }
}

@Model
final class ChatMessage {
    @Attribute(.unique) var id: UUID = UUID()
    var roleRaw: String = "user"
    var content: String = ""
    var index: Int = 0
    var isError: Bool = false
    var createdAt: Date = Date()
    var conversation: Conversation?

    init(role: Role, content: String, index: Int) {
        self.roleRaw = role.rawValue
        self.content = content
        self.index = index
    }

    enum Role: String {
        case user
        case assistant
    }

    var role: Role { Role(rawValue: roleRaw) ?? .user }
}
