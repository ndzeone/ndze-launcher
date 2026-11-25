import SwiftData
import SwiftUI

struct ConnectionDraft {
    var name = ""
    var baseURL = ""
    var format = APIFormat.openAI
    var model = ""
    var path = APIFormat.openAI.defaultPath
    var authHeader = APIFormat.openAI.defaultAuthHeader
    var authPrefix = APIFormat.openAI.defaultAuthPrefix
    var bodyTemplate = APIFormat.defaultTemplate
    var responsePath = APIFormat.openAI.defaultResponsePath
    var systemPrompt = ""
    var streaming = true

    init() {}

    init(_ connection: Connection) {
        name = connection.name
        baseURL = connection.baseURL
        format = connection.format
        model = connection.model
        path = connection.path
        authHeader = connection.authHeader
        authPrefix = connection.authPrefix
        bodyTemplate = connection.bodyTemplate
        responsePath = connection.responsePath
        systemPrompt = connection.systemPrompt
        streaming = connection.streaming
    }

    mutating func applyPreset(_ format: APIFormat) {
        path = format.defaultPath
        authHeader = format.defaultAuthHeader
        authPrefix = format.defaultAuthPrefix
        responsePath = format.defaultResponsePath
        streaming = format.supportsStreaming
        if format == .anthropic, baseURL.isEmpty { baseURL = "https://api.anthropic.com" }
    }

    func apply(to connection: Connection) {
        connection.name = name.trimmingCharacters(in: .whitespaces)
        connection.baseURL = baseURL.trimmingCharacters(in: .whitespaces)
        connection.format = format
        connection.model = model.trimmingCharacters(in: .whitespaces)
        connection.path = path
        connection.authHeader = authHeader
        connection.authPrefix = authPrefix
        connection.bodyTemplate = bodyTemplate
        connection.responsePath = responsePath
        connection.systemPrompt = systemPrompt
        connection.streaming = streaming
    }

    func endpoint(apiKey: String) -> EndpointConfig {
        EndpointConfig(
            baseURL: baseURL, path: path, format: format, model: model.trimmingCharacters(in: .whitespaces),
            apiKey: apiKey, authHeader: authHeader, authPrefix: authPrefix, bodyTemplate: bodyTemplate,
            responsePath: responsePath, systemPrompt: systemPrompt, streaming: streaming
        )
    }
}

struct ConnectionEditor: View {
    @Environment(\.modelContext) private var context
    @Environment(\.dismiss) private var dismiss

    let connection: Connection?
    @State private var draft: ConnectionDraft
    @State private var apiKey = ""
    @State private var revealKey = false
    @State private var test = TestState.idle
    @State private var saveError: String?

    enum TestState: Equatable {
        case idle
        case running
        case success(String)
        case failure(String)
    }

    init(connection: Connection?) {
        self.connection = connection
        _draft = State(initialValue: connection.map(ConnectionDraft.init) ?? ConnectionDraft())
    }

    private var trimmedKey: String { apiKey.trimmingCharacters(in: .whitespacesAndNewlines) }
    private var endpointURL: URL? { draft.endpoint(apiKey: "").url }
    private var canSave: Bool {
        !draft.name.trimmingCharacters(in: .whitespaces).isEmpty && endpointURL != nil
    }

    var body: some View {
        Form {
            Section("Подключение") {
                TextField("Название, например «mac os»", text: $draft.name)
                keyField
                TextField("https://api.example.com", text: $draft.baseURL)
                    .keyboardType(.URL)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
            }

            Section {
                Picker("Формат API", selection: $draft.format) {
                    ForEach(APIFormat.allCases) { Text($0.title).tag($0) }
                }
                .onChange(of: draft.format) { _, format in draft.applyPreset(format) }
                TextField(draft.format == .anthropic ? "Модель (обязательно)" : "Модель (необязательно)", text: $draft.model)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
            } footer: {
                Text(endpointURL.map { "POST \($0.absoluteString)" } ?? "Укажите адрес сервера с https://")
                    .font(.caption.monospaced())
            }

            Section("Системный промпт") {
                TextField("Необязательно: роль и стиль ответов", text: $draft.systemPrompt, axis: .vertical)
                    .lineLimit(2...6)
            }

            advancedSection

            Section {
                Button {
                    Task { await runTest() }
                } label: {
                    HStack {
                        Text("Проверить подключение")
                        Spacer()
                        if test == .running { ProgressView() }
                    }
                }
                .disabled(endpointURL == nil || test == .running)
            } footer: {
                switch test {
                case let .success(text): Label(text, systemImage: "checkmark.circle.fill").foregroundStyle(.green)
                case let .failure(text): Label(text, systemImage: "xmark.octagon.fill").foregroundStyle(.red)
                default: EmptyView()
                }
            }
        }
        .navigationTitle(connection == nil ? "Новое подключение" : "Подключение")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .confirmationAction) {
                Button("Сохранить", action: save).disabled(!canSave)
            }
        }
        .alert("Не удалось сохранить", isPresented: Binding(get: { saveError != nil }, set: { if !$0 { saveError = nil } })) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(saveError ?? "")
        }
    }

    private var keyField: some View {
        HStack {
            Group {
                if revealKey {
                    TextField(keyPlaceholder, text: $apiKey)
                } else {
                    SecureField(keyPlaceholder, text: $apiKey)
                }
            }
            .font(.body.monospaced())
            .textInputAutocapitalization(.never)
            .autocorrectionDisabled()
            .textContentType(.password)

            Button {
                revealKey.toggle()
            } label: {
                Image(systemName: revealKey ? "eye.slash" : "eye")
            }
            .buttonStyle(.borderless)
            .accessibilityLabel(revealKey ? "Скрыть ключ" : "Показать ключ")
        }
    }

    private var keyPlaceholder: String {
        if let masked = connection?.maskedKey, !masked.isEmpty { return masked }
        return "API-ключ, например tc_live_…"
    }

    private var advancedSection: some View {
        Section {
            DisclosureGroup("Дополнительно") {
                LabeledContent("Путь") {
                    TextField("/v1/chat/completions", text: $draft.path).multilineTextAlignment(.trailing)
                }
                LabeledContent("Заголовок ключа") {
                    TextField("Authorization", text: $draft.authHeader).multilineTextAlignment(.trailing)
                }
                LabeledContent("Префикс ключа") {
                    TextField("Bearer ", text: $draft.authPrefix).multilineTextAlignment(.trailing)
                }
                if draft.format.supportsStreaming {
                    Toggle("Потоковый ответ (SSE)", isOn: $draft.streaming)
                }
                if draft.format == .custom {
                    VStack(alignment: .leading, spacing: 6) {
                        Text("Тело запроса (JSON)").font(.subheadline)
                        TextEditor(text: $draft.bodyTemplate)
                            .font(.caption.monospaced())
                            .frame(minHeight: 110)
                    }
                    LabeledContent("Путь к ответу") {
                        TextField("choices.0.message.content", text: $draft.responsePath).multilineTextAlignment(.trailing)
                    }
                }
            }
            .textInputAutocapitalization(.never)
            .autocorrectionDisabled()
        } footer: {
            if draft.format == .custom {
                Text("Плейсхолдеры подставляются как JSON-значения (уже в кавычках): {{messages}} — массив [{role, content}], {{prompt}} — последнее сообщение, {{model}}, {{system}}. Путь к ответу — ключи через точку, индексы массивов числом; пустой путь = весь ответ как текст.")
            }
        }
    }

    private func currentKey() -> String {
        if !trimmedKey.isEmpty { return trimmedKey }
        return connection.flatMap { Keychain.read(account: $0.keychainAccount) } ?? ""
    }

    private func runTest() async {
        test = .running
        let clock = ContinuousClock()
        let started = clock.now
        do {
            let reply = try await AIClient(timeout: 30).complete(
                [ChatTurn(role: "user", content: "Ответь одним словом: ок")],
                config: draft.endpoint(apiKey: currentKey())
            )
            let elapsed = started.duration(to: clock.now).components
            let ms = elapsed.seconds * 1000 + elapsed.attoseconds / 1_000_000_000_000_000
            test = .success("\(ms) мс · \(reply.trimmingCharacters(in: .whitespacesAndNewlines).prefix(80))")
        } catch {
            test = .failure(error.localizedDescription)
        }
    }

    private func save() {
        let target = connection ?? Connection(name: draft.name)
        do {
            if !trimmedKey.isEmpty {
                try Keychain.save(trimmedKey, account: target.keychainAccount)
                target.maskedKey = KeyMask.mask(trimmedKey)
            }
            draft.apply(to: target)
            if connection == nil { context.insert(target) }
            dismiss()
        } catch {
            saveError = error.localizedDescription
        }
    }
}
