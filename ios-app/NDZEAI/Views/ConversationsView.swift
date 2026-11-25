import SwiftData
import SwiftUI

struct ConversationsView: View {
    @Environment(\.modelContext) private var context
    @Query(sort: \Conversation.updatedAt, order: .reverse) private var conversations: [Conversation]
    @Query(sort: \Connection.createdAt) private var connections: [Connection]
    @State private var path: [Conversation] = []
    @State private var showingKeys = false

    var body: some View {
        NavigationStack(path: $path) {
            List {
                ForEach(conversations) { conversation in
                    NavigationLink(value: conversation) {
                        ConversationRow(conversation: conversation)
                    }
                }
                .onDelete { offsets in
                    for offset in offsets { context.delete(conversations[offset]) }
                }
            }
            .overlay { emptyState }
            .navigationTitle("Чаты")
            .navigationDestination(for: Conversation.self) { ChatView(conversation: $0) }
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button { showingKeys = true } label: {
                        Label("API-ключи", systemImage: "key.horizontal")
                    }
                }
                ToolbarItem(placement: .topBarTrailing) { newChatButton }
            }
            .sheet(isPresented: $showingKeys) {
                NavigationStack { ConnectionsView() }
            }
        }
    }

    @ViewBuilder
    private var emptyState: some View {
        if connections.isEmpty {
            ContentUnavailableView {
                Label("Нет подключений", systemImage: "key.horizontal")
            } description: {
                Text("Добавьте адрес сервера и API-ключ, чтобы общаться со своими ИИ.")
            } actions: {
                Button("Добавить ключ") { showingKeys = true }
                    .buttonStyle(.borderedProminent)
            }
        } else if conversations.isEmpty {
            ContentUnavailableView {
                Label("Пока пусто", systemImage: "bubble.left.and.bubble.right")
            } description: {
                Text("Начните новый чат кнопкой в правом верхнем углу.")
            }
        }
    }

    @ViewBuilder
    private var newChatButton: some View {
        if connections.count > 1 {
            Menu {
                ForEach(connections) { connection in
                    Button(connection.name) { start(with: connection) }
                }
            } label: {
                Label("Новый чат", systemImage: "square.and.pencil")
            }
        } else {
            Button {
                if let connection = connections.first {
                    start(with: connection)
                } else {
                    showingKeys = true
                }
            } label: {
                Label("Новый чат", systemImage: "square.and.pencil")
            }
        }
    }

    private func start(with connection: Connection) {
        let conversation = Conversation(connection: connection)
        context.insert(conversation)
        path.append(conversation)
    }
}

private struct ConversationRow: View {
    let conversation: Conversation

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(alignment: .firstTextBaseline) {
                Text(conversation.title)
                    .font(.headline)
                    .lineLimit(1)
                Spacer()
                Text(conversation.updatedAt.formatted(.relative(presentation: .named)))
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Text(preview)
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .lineLimit(2)
            if let name = conversation.connection?.name {
                Text(name)
                    .font(.caption2.weight(.semibold))
                    .foregroundStyle(.tint)
            }
        }
        .padding(.vertical, 2)
    }

    private var preview: String {
        conversation.orderedMessages.last?.content ?? "Нет сообщений"
    }
}
