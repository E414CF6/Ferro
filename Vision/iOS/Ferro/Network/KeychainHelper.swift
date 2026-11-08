import Foundation
import Security

public final class KeychainHelper: @unchecked Sendable {
    public static let shared = KeychainHelper()

    private let accessTokenKey = "com.ferro.app.accessToken"
    private let refreshTokenKey = "com.ferro.app.refreshToken"
    private let serviceName = "com.ferro.app"

    // Fallback cache for sandbox/simulator or preview
    private var memoryStore: [String: String] = [:]

    private init() {}

    public func saveAccessToken(_ token: String) {
        save(key: accessTokenKey, value: token)
    }

    public func getAccessToken() -> String? {
        read(key: accessTokenKey)
    }

    public func saveRefreshToken(_ token: String) {
        save(key: refreshTokenKey, value: token)
    }

    public func getRefreshToken() -> String? {
        read(key: refreshTokenKey)
    }

    public func clearTokens() {
        delete(key: accessTokenKey)
        delete(key: refreshTokenKey)
        memoryStore.removeAll()
    }

    // MARK: - Private Keychain Ops

    private func save(key: String, value: String) {
        memoryStore[key] = value
        guard let data = value.data(using: .utf8) else { return }

        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: serviceName,
            kSecAttrAccount as String: key
        ]

        SecItemDelete(query as CFDictionary)

        var newQuery = query
        newQuery[kSecValueData as String] = data
        newQuery[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlock

        SecItemAdd(newQuery as CFDictionary, nil)
    }

    private func read(key: String) -> String? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: serviceName,
            kSecAttrAccount as String: key,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne
        ]

        var item: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &item)
        if status == errSecSuccess, let data = item as? Data, let string = String(data: data, encoding: .utf8) {
            memoryStore[key] = string
            return string
        }

        return memoryStore[key]
    }

    private func delete(key: String) {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: serviceName,
            kSecAttrAccount as String: key
        ]
        SecItemDelete(query as CFDictionary)
        memoryStore.removeValue(forKey: key)
    }
}
