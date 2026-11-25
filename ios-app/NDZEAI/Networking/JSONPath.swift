import Foundation

/// Resolves dot paths like `choices.0.message.content` in a decoded JSON object.
enum JSONPath {
    static func value(at path: String, in object: Any) -> Any? {
        path.split(separator: ".").reduce(Optional(object)) { current, key in
            guard let current else { return nil }
            if let index = Int(key), let array = current as? [Any] {
                return array.indices.contains(index) ? array[index] : nil
            }
            return (current as? [String: Any])?[String(key)]
        }
    }

    static func string(at path: String, in data: Data) -> String? {
        guard let object = try? JSONSerialization.jsonObject(with: data, options: [.fragmentsAllowed]) else {
            return path.isEmpty ? String(data: data, encoding: .utf8) : nil
        }
        return value(at: path, in: object) as? String
    }
}
