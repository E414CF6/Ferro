import Foundation
import SwiftUI

public enum MainTab: Int, CaseIterable, Identifiable {
    case feed = 0
    case explore = 1
    case messages = 2
    case notifications = 3
    case profile = 4

    public var id: Int { rawValue }

    public var title: String {
        switch self {
        case .feed: return "Feed"
        case .explore: return "Explore"
        case .messages: return "Messages"
        case .notifications: return "Alerts"
        case .profile: return "Profile"
        }
    }

    public var systemImage: String {
        switch self {
        case .feed: return "sparkles"
        case .explore: return "magnifyingglass"
        case .messages: return "bubble.left.and.bubble.right"
        case .notifications: return "bell"
        case .profile: return "person.crop.circle"
        }
    }
}

@Observable
@MainActor
public final class AppState {
    public static let shared = AppState()

    public var isAuthenticated: Bool
    public var currentUser: User?
    public var selectedTab: MainTab = .feed

    // Diagnostic & Probe info
    public var isServerLive: Bool = false
    public var isServerReady: Bool = false
    public var serverStatusMessage: String = "Connecting..."
    public var isConnecting: Bool = false

    // Real-time badge indicators
    public var unreadMessagesCount: Int = 0
    public var unreadNotificationsCount: Int = 0

    // Alerts & Notifications
    public var errorMessage: String? = nil
    public var isShowingError: Bool = false

    public init() {
        let hasToken = KeychainHelper.shared.getAccessToken() != nil
        self.isAuthenticated = hasToken

        Task {
            await probeServer()
            if hasToken {
                await refreshCurrentUser()
            }
        }
    }

    public func probeServer() async {
        self.isConnecting = true
        let (live, ready, msg) = await FerroService.shared.checkBackendStatus()
        self.isServerLive = live
        self.isServerReady = ready
        self.serverStatusMessage = msg
        self.isConnecting = false
    }

    public func refreshCurrentUser() async {
        do {
            let me = try await FerroService.shared.fetchMe()
            self.currentUser = me
            self.isAuthenticated = true
        } catch {
            // Token might be expired or invalid
            print("Failed to fetch user session: \(error)")
        }
    }

    public func login(usernameOrEmail: String, password: String) async {
        isConnecting = true
        defer { isConnecting = false }
        do {
            let payload = try await FerroService.shared.login(usernameOrEmail: usernameOrEmail, password: password)
            self.currentUser = payload.user
            self.isAuthenticated = true
        } catch {
            showError(error.localizedDescription)
        }
    }

    public func signup(username: String, email: String, password: String, displayName: String) async {
        isConnecting = true
        defer { isConnecting = false }
        do {
            let payload = try await FerroService.shared.signup(
                username: username,
                email: email,
                password: password,
                displayName: displayName
            )
            self.currentUser = payload.user
            self.isAuthenticated = true
        } catch {
            showError(error.localizedDescription)
        }
    }

    public func logout() async {
        await FerroService.shared.logout()
        self.currentUser = nil
        self.isAuthenticated = false
    }

    public func showError(_ message: String) {
        self.errorMessage = message
        self.isShowingError = true
    }
}
