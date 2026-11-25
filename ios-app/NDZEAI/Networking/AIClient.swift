import Foundation

enum AIClientError: LocalizedError {
    case invalidURL
    case http(status: Int, message: String)
    case unexpectedResponse(path: String)
    case server(String)

    var errorDescription: String? {
        switch self {
        case .invalidURL:
            "Некорректный адрес сервера. Пример: https://api.example.com"
        case let .http(status, message):
            message.isEmpty ? "HTTP \(status)" : "HTTP \(status): \(message)"
        case let .unexpectedResponse(path):
            "В ответе не найден текст по пути «\(path)». Проверьте настройку «Путь к ответу»."
        case let .server(message):
            message
        }
    }
}

struct AIClient: Sendable {
    var session: URLSession = .shared
    var timeout: TimeInterval = 120

    func stream(_ turns: [ChatTurn], config: EndpointConfig) -> AsyncThrowingStream<String, Error> {
        AsyncThrowingStream { continuation in
            let task = Task {
                do {
                    try await run(turns, config: config) { _ = continuation.yield($0) }
                    continuation.finish()
                } catch {
                    continuation.finish(throwing: error)
                }
            }
            continuation.onTermination = { _ in task.cancel() }
        }
    }

    func complete(_ turns: [ChatTurn], config: EndpointConfig) async throws -> String {
        var text = ""
        for try await chunk in stream(turns, config: config) {
            text += chunk
        }
        return text
    }

    func makeRequest(_ turns: [ChatTurn], config: EndpointConfig) throws -> URLRequest {
        guard let url = config.url else { throw AIClientError.invalidURL }
        var request = URLRequest(url: url, timeoutInterval: timeout)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue(config.usesStreaming ? "text/event-stream" : "application/json", forHTTPHeaderField: "Accept")

        let header = config.authHeader.trimmingCharacters(in: .whitespaces)
        if !header.isEmpty, !config.apiKey.isEmpty {
            request.setValue(config.authPrefix + config.apiKey, forHTTPHeaderField: header)
        }
        if config.format == .anthropic {
            request.setValue("2023-06-01", forHTTPHeaderField: "anthropic-version")
        }
        request.httpBody = try body(turns, config: config)
        return request
    }

    private func body(_ turns: [ChatTurn], config: EndpointConfig) throws -> Data {
        let system = config.systemPrompt.trimmingCharacters(in: .whitespacesAndNewlines)
        let messages = turns.map { ["role": $0.role, "content": $0.content] }

        switch config.format {
        case .openAI:
            var payload: [String: Any] = [
                "messages": (system.isEmpty ? [] : [["role": "system", "content": system]]) + messages,
                "stream": config.usesStreaming,
            ]
            if !config.model.isEmpty { payload["model"] = config.model }
            return try JSONSerialization.data(withJSONObject: payload)

        case .anthropic:
            var payload: [String: Any] = [
                "model": config.model,
                "max_tokens": 4096,
                "messages": messages,
                "stream": config.usesStreaming,
            ]
            if !system.isEmpty { payload["system"] = system }
            return try JSONSerialization.data(withJSONObject: payload)

        case .custom:
            return try BodyTemplate.render(config.bodyTemplate, turns: turns, model: config.model, system: system)
        }
    }

    private func run(_ turns: [ChatTurn], config: EndpointConfig, emit: @Sendable (String) -> Void) async throws {
        let request = try makeRequest(turns, config: config)
        let (bytes, response) = try await session.bytes(for: request)
        let status = (response as? HTTPURLResponse)?.statusCode ?? 0

        guard (200..<300).contains(status) else {
            var raw = Data()
            for try await byte in bytes {
                raw.append(byte)
                if raw.count > 8_192 { break }
            }
            throw AIClientError.http(status: status, message: Self.errorMessage(from: raw))
        }

        guard config.usesStreaming else {
            var raw = Data()
            for try await byte in bytes { raw.append(byte) }
            guard let text = JSONPath.string(at: config.responsePath, in: raw) else {
                throw AIClientError.unexpectedResponse(path: config.responsePath)
            }
            emit(text)
            return
        }

        for try await line in bytes.lines {
            try Task.checkCancellation()
            switch SSEParser.parse(line: line, format: config.format) {
            case let .delta(text): emit(text)
            case .done: return
            case let .failure(message): throw AIClientError.server(message)
            case nil: continue
            }
        }
    }

    static func errorMessage(from data: Data) -> String {
        for path in ["error.message", "message", "detail", "error"] {
            if let text = JSONPath.string(at: path, in: data) { return text }
        }
        return String(decoding: data.prefix(300), as: UTF8.self).trimmingCharacters(in: .whitespacesAndNewlines)
    }
}
