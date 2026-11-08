import SwiftUI

public struct ConversationsView: View {
    @Environment(AppState.self) private var appState

    @State private var conversations: [Conversation] = []
    @State private var isLoading = false
    @State private var searchChatText = ""

    public init() {}

    public var filteredConversations: [Conversation] {
        if searchChatText.trimmingCharacters(in: .whitespaces).isEmpty {
            return conversations
        }
        return conversations.filter {
            $0.otherUser.displayName.localizedCaseInsensitiveContains(searchChatText) ||
            $0.otherUser.username.localizedCaseInsensitiveContains(searchChatText) ||
            $0.lastMessage.content.localizedCaseInsensitiveContains(searchChatText)
        }
    }

    public var body: some View {
        NavigationStack {
            Group {
                if isLoading && conversations.isEmpty {
                    VStack(spacing: 12) {
                        ProgressView()
                            .controlSize(.large)
                        Text("Loading conversations...")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else if conversations.isEmpty {
                    VStack(spacing: 16) {
                        Image(systemName: "bubble.left.and.text.bubble.right")
                            .font(.system(size: 52))
                            .foregroundStyle(
                                LinearGradient(
                                    colors: [.indigo, .cyan],
                                    startPoint: .topLeading,
                                    endPoint: .bottomTrailing
                                )
                            )
                        Text("No Messages Yet")
                            .font(.title3.weight(.bold))
                        Text("Direct messages are real-time, encrypted, and fast. Find friends on Explore to start chatting.")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                            .multilineTextAlignment(.center)
                            .padding(.horizontal, 36)

                        Button {
                            appState.selectedTab = .explore
                        } label: {
                            Label("Discover Users", systemImage: "magnifyingglass")
                                .font(.subheadline.weight(.semibold))
                                .padding(.horizontal, 18)
                                .padding(.vertical, 10)
                                .background(Color.indigo)
                                .foregroundStyle(.white)
                                .clipShape(Capsule())
                        }
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    List {
                        ForEach(filteredConversations) { conv in
                            NavigationLink {
                                ChatView(otherUser: conv.otherUser)
                            } label: {
                                HStack(spacing: 12) {
                                    // Avatar
                                    ZStack(alignment: .bottomTrailing) {
                                        AsyncImage(url: URL(string: conv.otherUser.avatarUrl ?? "")) { phase in
                                            if let image = phase.image {
                                                image.resizable().scaledToFill()
                                            } else {
                                                Circle().fill(Color(.tertiarySystemFill))
                                            }
                                        }
                                        .frame(width: 48, height: 48)
                                        .clipShape(Circle())

                                        if conv.unreadCount > 0 {
                                            Circle()
                                                .fill(Color.indigo)
                                                .frame(width: 14, height: 14)
                                                .overlay(Circle().stroke(Color(.systemBackground), lineWidth: 2))
                                        }
                                    }

                                    // Content Preview
                                    VStack(alignment: .leading, spacing: 4) {
                                        HStack {
                                            Text(conv.otherUser.displayName)
                                                .font(.subheadline)
                                                .fontWeight(.semibold)

                                            Spacer()

                                            Text(conv.lastMessage.createdAt)
                                                .font(.caption2)
                                                .foregroundStyle(.secondary)
                                        }

                                        Text(conv.lastMessage.content)
                                            .font(.subheadline)
                                            .foregroundStyle(conv.unreadCount > 0 ? .primary : .secondary)
                                            .fontWeight(conv.unreadCount > 0 ? .semibold : .regular)
                                            .lineLimit(1)
                                    }
                                }
                                .padding(.vertical, 4)
                            }
                        }
                    }
                    .searchable(text: $searchChatText, prompt: "Search messages or users")
                }
            }
            .navigationTitle("Messages")
            .refreshable {
                await loadConversations()
            }
            .task {
                await loadConversations()
            }
        }
    }

    private func loadConversations() async {
        isLoading = true
        do {
            let list = try await FerroService.shared.fetchConversations()
            await MainActor.run {
                self.conversations = list
                self.appState.unreadMessagesCount = list.reduce(0) { $0 + $1.unreadCount }
                self.isLoading = false
            }
        } catch {
            await MainActor.run {
                self.isLoading = false
            }
        }
    }
}
