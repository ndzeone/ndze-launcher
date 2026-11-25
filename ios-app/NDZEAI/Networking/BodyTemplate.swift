import Foundation

/// Renders a user-defined JSON body. Placeholders are replaced in a single pass
/// with JSON-encoded values, so message text can never break out of its string.
enum BodyTemplate {
    struct InvalidTemplate: LocalizedError {
        let reason: String
        var errorDescription: String? { "Шаблон тела запроса: \(reason)" }
    }

    static let placeholders = ["messages", "prompt", "model", "system"]

    static func render(_ template: String, turns: [ChatTurn], model: String, system: String) throws -> Data {
        let values: [String: String] = [
            "messages": try json(turns.map { ["role": $0.role, "content": $0.content] }),
            "prompt": try json(turns.last(where: { $0.role == "user" })?.content ?? ""),
            "model": try json(model),
            "system": try json(system),
        ]

        var output = ""
        var rest = Substring(template)
        while let open = rest.range(of: "{{") {
            output += rest[..<open.lowerBound]
            guard let close = rest[open.upperBound...].range(of: "}}") else {
                output += rest[open.lowerBound...]
                rest = ""
                break
            }
            let name = rest[open.upperBound..<close.lowerBound].trimmingCharacters(in: .whitespaces)
            if let value = values[name] {
                output += value
            } else {
                output += rest[open.lowerBound..<close.upperBound]
            }
            rest = rest[close.upperBound...]
        }
        output += rest

        let data = Data(output.utf8)
        do {
            _ = try JSONSerialization.jsonObject(with: data)
        } catch {
            throw InvalidTemplate(reason: "после подстановки получился невалидный JSON")
        }
        return data
    }

    private static func json(_ value: Any) throws -> String {
        let data = try JSONSerialization.data(withJSONObject: value, options: [.fragmentsAllowed, .withoutEscapingSlashes])
        return String(decoding: data, as: UTF8.self)
    }
}
