import SwiftData
import SwiftUI

@main
struct NDZEAIApp: App {
    var body: some Scene {
        WindowGroup {
            ConversationsView()
                .tint(.mint)
        }
        .modelContainer(for: [Connection.self, Conversation.self, ChatMessage.self])
    }
}
