import SwiftData
import SwiftUI

struct ChatView: View {
    @Bindable var conversation: Conversation
    @Environment(\.modelContext) private var context
    @Query(sort: \Connection.createdAt) private var connections: [Connection]
    @State private var draft = ""
    @State private var task: Task<Void, Never>?
    @FocusState private var inputFocused: Bool

    private var isStreaming: Bool { task != nil }
    private var trimmedDraft: String { draft.trimmingCharacters(in: .whitespacesAndNewlines) }

    var body: some View {
        let messages = conversation.orderedMessages
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(spacing: 10) {
                    ForEach(messages) { message in
                        MessageBubble(message: message).id(message.id)
                    }
                }
                .padding(.horizontal)
                .padding(.vertical, 12)
            }
            .defaultScrollAnchor(.bottom)
            .scrollDismissesKeyboard(.interactively)
            .onChange(of: messages.last?.content) {
                if let last = messages.last { proxy.scrollTo(last.id, anchor: .bottom) }
            }
        }
        .overlay {
            if messages.isEmpty {
                ContentUnavailableView(
                    "Спросите что-нибудь",
                    systemImage: "sparkles",
                    description: Text(conversation.connection.map { "Подключение: \($0.name)" } ?? "Подключение не выбрано")
                )
            }
        }
        .safeAreaInset(edge: .bottom) { inputBar }
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .principal) { connectionMenu }
        }
        .onDisappear {
            task?.cancel()
            if conversation.messages.isEmpty { context.delete(conversation) }
        }
    }

    private var connectionMenu: some View {
        Menu {
            Picker("Подключение", selection: Binding(
                get: { conversation.connection },
                set: { conversation.connection = $0 }
            )) {
                ForEach(connections) { connection in
                    Text(connection.name).tag(Optional(connection))
                }
            }
        } label: {
            VStack(spacing: 0) {
                Text(conversation.title)
                    .font(.headline)
                    .lineLimit(1)
                HStack(spacing: 2) {
                    Text(subtitle)
                    Image(systemName: "chevron.down")
                }
                .font(.caption2)
                .foregroundStyle(.secondary)
            }
            .foregroundStyle(.primary)
        }
        .disabled(isStreaming)
    }

    private var subtitle: String {
        guard let connection = conversation.connection else { return "Выберите подключение" }
        return connection.model.isEmpty ? connection.name : "\(connection.name) · \(connection.model)"
    }

    private var inputBar: some View {
        HStack(alignment: .bottom, spacing: 8) {
            TextField("Сообщение", text: $draft, axis: .vertical)
                .lineLimit(1...6)
                .focused($inputFocused)
                .padding(.horizontal, 14)
                .padding(.vertical, 10)
                .background(Color(.secondarySystemBackground), in: RoundedRectangle(cornerRadius: 20, style: .continuous))

            Button(action: isStreaming ? stop : send) {
                Image(systemName: isStreaming ? "stop.circle.fill" : "arrow.up.circle.fill")
                    .font(.system(size: 34))
                    .symbolRenderingMode(.hierarchical)
            }
            .disabled(!isStreaming && (trimmedDraft.isEmpty || conversation.connection == nil))
            .accessibilityLabel(isStreaming ? "Остановить" : "Отправить")
        }
        .padding(.horizontal)
        .padding(.vertical, 8)
        .background(.bar)
    }

    @discardableResult
    private func append(_ role: ChatMessage.Role, _ content: String) -> ChatMessage {
        let message = ChatMessage(role: role, content: content, index: (conversation.messages.map(\.index).max() ?? -1) + 1)
        context.insert(message)
        message.conversation = conversation
        conversation.updatedAt = .now
        return message
    }

    private func send() {
        let text = trimmedDraft
        guard !text.isEmpty, task == nil, let connection = conversation.connection else { return }

        let apiKey = Keychain.read(account: connection.keychainAccount) ?? ""
        draft = ""
        append(.user, text)
        if conversation.title == Conversation.defaultTitle {
            conversation.title = String(text.prefix(40))
        }

        let history = conversation.orderedMessages
            .filter { !$0.isError && !$0.content.isEmpty }
            .map { ChatTurn(role: $0.roleRaw, content: $0.content) }
        let reply = append(.assistant, "")
        let config = connection.endpoint(apiKey: apiKey)

        task = Task {
            do {
                for try await chunk in AIClient().stream(history, config: config) {
                    reply.content += chunk
                }
                if reply.content.isEmpty { reply.content = "(пустой ответ)" }
            } catch {
                if Task.isCancelled {
                    if reply.content.isEmpty { context.delete(reply) }
                } else {
                    reply.content = error.localizedDescription
                    reply.isError = true
                }
            }
            conversation.updatedAt = .now
            task = nil
        }
    }

    private func stop() {
        task?.cancel()
    }
}
