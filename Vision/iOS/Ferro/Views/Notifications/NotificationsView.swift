import SwiftUI

public struct NotificationsView: View {
    @Environment(AppState.self) private var appState

    @State private var notifications: [NotificationItem] = []
    @State private var isLoading = false
    @State private var selectedFilter: NotificationFilter = .all

    enum NotificationFilter: String, CaseIterable, Identifiable {
        case all = "All"
        case interactions = "Likes & Comments"
        case follows = "Follows"

        var id: String { rawValue }
    }

    public init() {}

    private var filteredNotifications: [NotificationItem] {
        switch selectedFilter {
        case .all:
            return notifications
        case .interactions:
            return notifications.filter {
                $0.notificationType == .likePost ||
                $0.notificationType == .commentPost ||
                $0.notificationType == .replyComment ||
                $0.notificationType == .likeComment ||
                $0.notificationType == .repost
            }
        case .follows:
            return notifications.filter {
                $0.notificationType == .follow ||
                $0.notificationType == .followRequest ||
                $0.notificationType == .followAccepted
            }
        }
    }

    public var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                // Filter picker
                Picker("Filter", selection: $selectedFilter) {
                    ForEach(NotificationFilter.allCases) { filter in
                        Text(filter.rawValue).tag(filter)
                    }
                }
                .pickerStyle(.segmented)
                .padding(.horizontal, 16)
                .padding(.vertical, 8)

                if isLoading && notifications.isEmpty {
                    VStack(spacing: 12) {
                        ProgressView()
                            .controlSize(.large)
                        Text("Loading notifications...")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else if filteredNotifications.isEmpty {
                    VStack(spacing: 16) {
                        Image(systemName: "bell.slash")
                            .font(.system(size: 48))
                            .foregroundStyle(.tertiary)
                            .padding(.top, 40)
                        Text("No Notifications")
                            .font(.headline)
                        Text("When people follow you, like your posts, or reply to your thoughts, you'll see them here.")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                            .multilineTextAlignment(.center)
                            .padding(.horizontal, 32)
                        Spacer()
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    List {
                        ForEach(filteredNotifications) { item in
                            HStack(alignment: .top, spacing: 14) {
                                // Type Icon badge over avatar
                                ZStack(alignment: .bottomTrailing) {
                                    AsyncImage(url: URL(string: item.sender.avatarUrl ?? "")) { phase in
                                        if let image = phase.image {
                                            image.resizable().scaledToFill()
                                        } else {
                                            Circle().fill(Color(.tertiarySystemFill))
                                        }
                                    }
                                    .frame(width: 44, height: 44)
                                    .clipShape(Circle())

                                    Circle()
                                        .fill(badgeColor(for: item.notificationType))
                                        .frame(width: 18, height: 18)
                                        .overlay(
                                            Image(systemName: item.notificationType.systemImage)
                                                .font(.system(size: 9, weight: .bold))
                                                .foregroundStyle(.white)
                                        )
                                }

                                // Text content
                                VStack(alignment: .leading, spacing: 4) {
                                    Text(attributedNotificationText(for: item))
                                        .font(.subheadline)
                                        .lineLimit(3)

                                    Text(item.createdAt)
                                        .font(.caption2)
                                        .foregroundStyle(.secondary)
                                }

                                Spacer()

                                // Unread dot indicator
                                if !item.isRead {
                                    Circle()
                                        .fill(Color.indigo)
                                        .frame(width: 8, height: 8)
                                        .padding(.top, 6)
                                }
                            }
                            .padding(.vertical, 4)
                            .listRowBackground(item.isRead ? Color(.systemBackground) : Color.indigo.opacity(0.04))
                        }
                    }
                    .listStyle(.plain)
                }
            }
            .navigationTitle("Notifications")
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    if notifications.contains(where: { !$0.isRead }) {
                        Button("Mark All Read") {
                            markAllAsRead()
                        }
                        .font(.footnote)
                    }
                }
            }
            .refreshable {
                await loadNotifications()
            }
            .task {
                await loadNotifications()
            }
        }
    }

    private func attributedNotificationText(for item: NotificationItem) -> AttributedString {
        var str = AttributedString(item.sender.displayName)
        str.font = .subheadline.bold()

        let action = AttributedString(" " + item.notificationType.actionDescription)

        if let post = item.targetPost {
            var snippet = AttributedString(": \"\(post.content.prefix(40))...\"")
            snippet.foregroundColor = .secondary
            snippet.font = .subheadline
            str.append(action)
            str.append(snippet)
        } else {
            str.append(action)
        }

        return str
    }

    private func badgeColor(for type: NotificationTypeGql) -> Color {
        switch type {
        case .likePost, .likeComment:
            return .red
        case .commentPost, .replyComment:
            return .blue
        case .follow, .followAccepted:
            return .indigo
        case .repost, .quote:
            return .green
        default:
            return .purple
        }
    }

    private func markAllAsRead() {
        for i in 0..<notifications.count {
            notifications[i].isRead = true
        }
        appState.unreadNotificationsCount = 0
    }

    private func loadNotifications() async {
        isLoading = true
        if let list = try? await FerroService.shared.fetchNotifications() {
            await MainActor.run {
                self.notifications = list
                self.appState.unreadNotificationsCount = list.filter { !$0.isRead }.count
                self.isLoading = false
            }
        } else {
            await MainActor.run {
                self.isLoading = false
            }
        }
    }
}
