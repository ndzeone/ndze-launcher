import Foundation
import Security

enum Keychain {
    private static let service = "one.ndze.aichat.api-keys"

    struct Failure: LocalizedError {
        let status: OSStatus
        var errorDescription: String? {
            let text = SecCopyErrorMessageString(status, nil) as String? ?? "код \(status)"
            return "Keychain: \(text)"
        }
    }

    private static func query(_ account: String) -> [String: Any] {
        [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
        ]
    }

    static func save(_ secret: String, account: String) throws {
        let data = Data(secret.utf8)
        let update: [String: Any] = [
            kSecValueData as String: data,
            kSecAttrAccessible as String: kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly,
        ]
        var status = SecItemUpdate(query(account) as CFDictionary, update as CFDictionary)
        if status == errSecItemNotFound {
            status = SecItemAdd(query(account).merging(update) { $1 } as CFDictionary, nil)
        }
        guard status == errSecSuccess else { throw Failure(status: status) }
    }

    static func read(account: String) -> String? {
        var q = query(account)
        q[kSecReturnData as String] = true
        q[kSecMatchLimit as String] = kSecMatchLimitOne
        var result: CFTypeRef?
        guard SecItemCopyMatching(q as CFDictionary, &result) == errSecSuccess,
              let data = result as? Data else { return nil }
        return String(data: data, encoding: .utf8)
    }

    static func delete(account: String) {
        SecItemDelete(query(account) as CFDictionary)
    }
}

enum KeyMask {
    /// `tc_live_abcdef…1b33` → `tc_live_****************1b33`
    static func mask(_ key: String) -> String {
        let key = key.trimmingCharacters(in: .whitespacesAndNewlines)
        guard key.count > 8 else { return String(repeating: "*", count: key.count) }
        let prefix: Substring
        if let underscore = key.lastIndex(of: "_"), key.distance(from: key.startIndex, to: underscore) < 12 {
            prefix = key[...underscore]
        } else {
            prefix = key.prefix(3)
        }
        return String(prefix) + String(repeating: "*", count: 16) + String(key.suffix(4))
    }
}
