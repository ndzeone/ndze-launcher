import Foundation

enum SSEEvent: Equatable {
    case delta(String)
    case done
    case failure(String)
}

/// Parses one `data:` line of an OpenAI or Anthropic streaming response.
enum SSEParser {
    static func parse(line: String, format: APIFormat) -> SSEEvent? {
        guard line.hasPrefix("data:") else { return nil }
        let payload = line.dropFirst(5).trimmingCharacters(in: .whitespaces)
        if payload == "[DONE]" { return .done }
        guard let data = payload.data(using: .utf8),
              let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        else { return nil }

        if let message = JSONPath.value(at: "error.message", in: object) as? String {
            return .failure(message)
        }

        switch format {
        case .anthropic:
            switch object["type"] as? String {
            case "content_block_delta":
                return (JSONPath.value(at: "delta.text", in: object) as? String).map(SSEEvent.delta)
            case "message_stop":
                return .done
            default:
                return nil
            }
        case .openAI, .custom:
            return (JSONPath.value(at: "choices.0.delta.content", in: object) as? String).map(SSEEvent.delta)
        }
    }
}
