import SwiftUI

public struct ChatView: View {
    let otherUser: User
    @Environment(AppState.self) private var appState

    @State private var messages: [DirectMessage] = []
    @State private var messageText = ""
    @State private var isOtherUserTyping = false
    @State private var isLoading = false

    public init(otherUser: User) {
        self.otherUser = otherUser
    }

    public var body: some View {
        VStack(spacing: 0) {
            ScrollViewReader { proxy in
                ScrollView {
                    if isLoading && messages.isEmpty {
                        VStack(spacing: 8) {
                            ProgressView()
                            Text("Loading chat history...")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                        .frame(maxWidth: .infinity)
                        .padding(.top, 40)
                    } else if messages.isEmpty {
                        VStack(spacing: 12) {
                            AsyncImage(url: URL(string: otherUser.avatarUrl ?? "")) { phase in
                                if let image = phase.image {
                                    image.resizable().scaledToFill()
                                } else {
                                    Circle().fill(Color(.tertiarySystemFill))
                                }
                            }
                            .frame(width: 64, height: 64)
                            .clipShape(Circle())
                            .padding(.top, 40)

                            Text(otherUser.displayName)
                                .font(.headline)

                            Text("@\(otherUser.username)")
                                .font(.caption)
                                .foregroundStyle(.secondary)

                            Text("No messages here yet. Say hello to start the conversation! 👋")
                                .font(.subheadline)
                                .foregroundStyle(.secondary)
                                .multilineTextAlignment(.center)
                                .padding(.horizontal, 32)
                                .padding(.top, 8)
                        }
                        .frame(maxWidth: .infinity)
                    } else {
                        LazyVStack(spacing: 12) {
                            ForEach(messages) { msg in
                                messageBubble(message: msg)
                            }

                            if isOtherUserTyping {
                                HStack {
                                    HStack(spacing: 4) {
                                        Circle().fill(Color.secondary).frame(width: 6, height: 6)
                                        Circle().fill(Color.secondary).frame(width: 6, height: 6)
                                        Circle().fill(Color.secondary).frame(width: 6, height: 6)
                                    }
                                    .padding(.horizontal, 14)
                                    .padding(.vertical, 10)
                                    .background(Color(.secondarySystemBackground))
                                    .clipShape(Capsule())

                                    Spacer()
                                }
                                .padding(.leading, 16)
                            }
                        }
                        .padding()
                    }
                }
                .onChange(of: messages.count) { _, _ in
                    if let last = messages.last {
                        withAnimation {
                            proxy.scrollTo(last.id, anchor: .bottom)
                        }
                    }
                }
            }

            Divider()

            // Input Bar
            HStack(spacing: 10) {
                TextField("Message \(otherUser.displayName)...", text: $messageText)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 10)
                    .background(Color(.secondarySystemBackground))
                    .clipShape(Capsule())

                Button {
                    sendMessage()
                } label: {
                    Image(systemName: "arrow.up.circle.fill")
                        .font(.system(size: 34))
                        .foregroundStyle(messageText.trimmingCharacters(in: .whitespaces).isEmpty ? Color.gray : Color.indigo)
                }
                .disabled(messageText.trimmingCharacters(in: .whitespaces).isEmpty)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
            .background(Color(.systemBackground))
        }
        .navigationTitle(otherUser.displayName)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                AsyncImage(url: URL(string: otherUser.avatarUrl ?? "")) { phase in
                    if let image = phase.image {
                        image.resizable().scaledToFill()
                    } else {
                        Circle().fill(Color(.tertiarySystemFill))
                    }
                }
                .frame(width: 32, height: 32)
                .clipShape(Circle())
            }
        }
        .task {
            await loadMessages()
        }
    }

    private func messageBubble(message: DirectMessage) -> some View {
        let isMe = message.senderId == (appState.currentUser?.id ?? "me")

        return HStack {
            if isMe { Spacer(minLength: 50) }

            VStack(alignment: isMe ? .trailing : .leading, spacing: 3) {
                Text(message.content)
                    .font(.body)
                    .foregroundStyle(isMe ? Color.white : Color.primary)
                    .padding(.horizontal, 14)
                    .padding(.vertical, 10)
                    .background(
                        isMe ?
                        LinearGradient(colors: [.indigo, .purple], startPoint: .topLeading, endPoint: .bottomTrailing) :
                        LinearGradient(colors: [Color(.secondarySystemBackground)], startPoint: .top, endPoint: .bottom)
                    )
                    .clipShape(RoundedRectangle(cornerRadius: 16))

                Text(message.createdAt)
                    .font(.system(size: 10))
                    .foregroundStyle(.tertiary)
                    .padding(.horizontal, 4)
            }

            if !isMe { Spacer(minLength: 50) }
        }
        .id(message.id)
    }

    private func sendMessage() {
        let text = messageText.trimmingCharacters(in: .whitespaces)
        guard !text.isEmpty else { return }

        let currentUserId = appState.currentUser?.id ?? "me"
        let optimisticMsg = DirectMessage(
            id: "dm_\(UUID().uuidString)",
            senderId: currentUserId,
            recipientId: otherUser.id,
            content: text,
            isRead: false,
            createdAt: "Just now"
        )

        messages.append(optimisticMsg)
        messageText = ""

        Task {
            _ = try? await FerroService.shared.sendDirectMessage(recipientId: otherUser.id, content: text)
        }
    }

    private func loadMessages() async {
        isLoading = true
        if let fetched = try? await FerroService.shared.fetchMessages(otherUserId: otherUser.id) {
            await MainActor.run {
                self.messages = fetched
                self.isLoading = false
            }
        } else {
            await MainActor.run {
                self.isLoading = false
            }
        }
    }
}
