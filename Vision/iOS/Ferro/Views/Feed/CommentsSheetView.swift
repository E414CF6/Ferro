import SwiftUI

public struct CommentsSheetView: View {
    @Binding var post: Post
    @Environment(\.dismiss) private var dismiss
    @Environment(AppState.self) private var appState

    @State private var commentText = ""
    @State private var replyingTo: Comment? = nil
    @State private var isSubmitting = false
    @State private var isLoading = false

    public init(post: Binding<Post>) {
        self._post = post
    }

    public var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                // Comments list
                if isLoading && post.comments.isEmpty {
                    VStack {
                        Spacer()
                        ProgressView()
                            .controlSize(.large)
                        Text("Loading comments...")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .padding(.top, 8)
                        Spacer()
                    }
                } else if post.comments.isEmpty {
                    VStack(spacing: 12) {
                        Spacer()
                        Image(systemName: "bubble.left.and.bubble.right")
                            .font(.system(size: 48))
                            .foregroundStyle(.tertiary)
                        Text("No comments yet")
                            .font(.headline)
                            .foregroundStyle(.secondary)
                        Text("Be the first to join the conversation!")
                            .font(.subheadline)
                            .foregroundStyle(.tertiary)
                        Spacer()
                    }
                } else {
                    ScrollView {
                        LazyVStack(alignment: .leading, spacing: 16) {
                            ForEach(post.comments) { comment in
                                commentRow(comment: comment, isReply: false)

                                // Render nested replies
                                ForEach(comment.replies) { reply in
                                    commentRow(comment: reply, isReply: true)
                                }
                            }
                        }
                        .padding()
                    }
                }

                Divider()

                // Replying Indicator Banner
                if let target = replyingTo {
                    HStack {
                        Text("Replying to @\(target.author.username)")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        Spacer()
                        Button {
                            replyingTo = nil
                        } label: {
                            Image(systemName: "xmark.circle.fill")
                                .foregroundStyle(.secondary)
                        }
                    }
                    .padding(.horizontal)
                    .padding(.top, 8)
                }

                // Input Bar
                HStack(spacing: 12) {
                    TextField("Add a comment...", text: $commentText)
                        .padding(.horizontal, 14)
                        .padding(.vertical, 10)
                        .background(Color(.secondarySystemBackground))
                        .clipShape(Capsule())

                    Button {
                        submitComment()
                    } label: {
                        if isSubmitting {
                            ProgressView()
                                .frame(width: 32, height: 32)
                        } else {
                            Image(systemName: "arrow.up.circle.fill")
                                .font(.system(size: 32))
                                .foregroundStyle(commentText.trimmingCharacters(in: .whitespaces).isEmpty ? Color.gray : Color.indigo)
                        }
                    }
                    .disabled(commentText.trimmingCharacters(in: .whitespaces).isEmpty || isSubmitting)
                }
                .padding()
                .background(Color(.systemBackground))
            }
            .navigationTitle("Comments")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
            .task {
                await loadComments()
            }
        }
    }

    private func loadComments() async {
        isLoading = true
        do {
            let fetched = try await FerroService.shared.fetchPostComments(postId: post.id)
            if !fetched.isEmpty {
                self.post.comments = fetched
            }
        } catch {
            // Keep existing comments on failure
        }
        isLoading = false
    }

    private func commentRow(comment: Comment, isReply: Bool) -> some View {
        HStack(alignment: .top, spacing: 10) {
            AsyncImage(url: URL(string: comment.author.avatarUrl ?? "")) { phase in
                if let image = phase.image {
                    image.resizable().scaledToFill()
                } else {
                    Circle().fill(Color(.tertiarySystemFill))
                }
            }
            .frame(width: isReply ? 28 : 34, height: isReply ? 28 : 34)
            .clipShape(Circle())

            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 6) {
                    Text(comment.author.displayName)
                        .font(.footnote)
                        .fontWeight(.bold)
                    Text("@\(comment.author.username)")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                    Text("•")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                    Text(comment.createdAt)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }

                Text(comment.content)
                    .font(.subheadline)

                HStack(spacing: 16) {
                    Button {
                        replyingTo = comment
                    } label: {
                        Text("Reply")
                            .font(.caption2)
                            .fontWeight(.semibold)
                            .foregroundStyle(.secondary)
                    }

                    HStack(spacing: 4) {
                        Image(systemName: comment.isLikedByMe ? "heart.fill" : "heart")
                            .font(.caption2)
                            .foregroundStyle(comment.isLikedByMe ? .red : .secondary)
                        if comment.likesCount > 0 {
                            Text("\(comment.likesCount)")
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                    }
                }
                .padding(.top, 2)
            }
        }
        .padding(.leading, isReply ? 36 : 0)
    }

    private func submitComment() {
        let content = commentText.trimmingCharacters(in: .whitespaces)
        guard !content.isEmpty else { return }

        let parent = replyingTo
        isSubmitting = true

        Task {
            do {
                let created = try await FerroService.shared.createComment(
                    postId: post.id,
                    content: content,
                    parentId: parent?.id
                )

                await MainActor.run {
                    if let p = parent {
                        for i in 0..<post.comments.count {
                            if post.comments[i].id == p.id {
                                post.comments[i].replies.append(created)
                                post.comments[i].repliesCount += 1
                                break
                            }
                        }
                    } else {
                        post.comments.append(created)
                    }
                    post.commentsCount += 1
                    commentText = ""
                    replyingTo = nil
                    isSubmitting = false
                }
            } catch {
                await MainActor.run {
                    // Fallback local optimistic append if offline
                    let fallbackUser = appState.currentUser ?? User(
                        id: "anon",
                        username: "me",
                        email: "",
                        displayName: "Me"
                    )
                    let fallbackComment = Comment(
                        id: "cmt_\(UUID().uuidString)",
                        parentId: parent?.id,
                        content: content,
                        createdAt: "Just now",
                        updatedAt: "Just now",
                        author: fallbackUser,
                        depth: parent != nil ? 1 : 0
                    )
                    if let p = parent {
                        for i in 0..<post.comments.count {
                            if post.comments[i].id == p.id {
                                post.comments[i].replies.append(fallbackComment)
                                post.comments[i].repliesCount += 1
                                break
                            }
                        }
                    } else {
                        post.comments.append(fallbackComment)
                    }
                    post.commentsCount += 1
                    commentText = ""
                    replyingTo = nil
                    isSubmitting = false
                }
            }
        }
    }
}
