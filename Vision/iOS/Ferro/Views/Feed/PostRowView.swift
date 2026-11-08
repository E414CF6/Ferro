import SwiftUI

public struct PostRowView: View {
    @Binding var post: Post
    var onCommentTapped: () -> Void

    @State private var isLiking = false
    @State private var isSaving = false

    public init(post: Binding<Post>, onCommentTapped: @escaping () -> Void) {
        self._post = post
        self.onCommentTapped = onCommentTapped
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Header: Author Info & Audience Badge
            HStack(alignment: .center, spacing: 10) {
                // Avatar with story ring if active
                ZStack {
                    if post.author.hasActiveStories {
                        Circle()
                            .stroke(
                                LinearGradient(colors: [.purple, .indigo, .pink], startPoint: .topLeading, endPoint: .bottomTrailing),
                                lineWidth: 2.5
                            )
                            .frame(width: 44, height: 44)
                    }

                    AsyncImage(url: URL(string: post.author.avatarUrl ?? "")) { phase in
                        if let image = phase.image {
                            image
                                .resizable()
                                .scaledToFill()
                        } else {
                            Circle()
                                .fill(Color(.tertiarySystemFill))
                                .overlay(
                                    Image(systemName: "person.fill")
                                        .foregroundStyle(.secondary)
                                )
                        }
                    }
                    .frame(width: 38, height: 38)
                    .clipShape(Circle())
                }

                VStack(alignment: .leading, spacing: 2) {
                    HStack(spacing: 6) {
                        Text(post.author.displayName)
                            .font(.subheadline)
                            .fontWeight(.bold)
                            .lineLimit(1)

                        if post.author.is2faEnabled {
                            Image(systemName: "checkmark.seal.fill")
                                .font(.caption2)
                                .foregroundStyle(.blue)
                        }
                    }

                    HStack(spacing: 6) {
                        Text("@\(post.author.username)")
                            .font(.caption)
                            .foregroundStyle(.secondary)

                        Text("•")
                            .font(.caption)
                            .foregroundStyle(.secondary)

                        Text(post.createdAt)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }

                Spacer()

                // Audience Badge
                audienceBadge(audience: post.audience)
            }

            // Post Content
            Text(post.content)
                .font(.body)
                .lineSpacing(3)
                .fixedSize(horizontal: false, vertical: true)

            // Hashtags (if any)
            if !post.hashtags.isEmpty {
                HStack(spacing: 8) {
                    ForEach(post.hashtags, id: \.self) { tag in
                        Text("#\(tag)")
                            .font(.footnote)
                            .fontWeight(.medium)
                            .foregroundStyle(.indigo)
                    }
                }
            }

            // Media Carousel / Grid
            if !post.media.isEmpty {
                ForEach(post.media) { mediaItem in
                    AsyncImage(url: URL(string: mediaItem.url)) { phase in
                        if let image = phase.image {
                            image
                                .resizable()
                                .scaledToFill()
                                .frame(maxWidth: .infinity)
                                .frame(maxHeight: 260)
                                .clipped()
                                .clipShape(RoundedRectangle(cornerRadius: 14))
                        } else if phase.error != nil {
                            RoundedRectangle(cornerRadius: 14)
                                .fill(Color(.secondarySystemFill))
                                .frame(height: 180)
                                .overlay(
                                    Image(systemName: "photo")
                                        .foregroundStyle(.secondary)
                                )
                        } else {
                            RoundedRectangle(cornerRadius: 14)
                                .fill(Color(.secondarySystemFill))
                                .frame(height: 180)
                                .overlay(ProgressView())
                        }
                    }
                }
            }

            // Interactive Poll (if attached)
            if let poll = post.poll {
                pollView(poll: poll)
            }

            // Bottom Actions Bar
            HStack(spacing: 24) {
                // Like Button
                Button {
                    toggleLike()
                } label: {
                    HStack(spacing: 6) {
                        Image(systemName: post.isLiked ? "heart.fill" : "heart")
                            .foregroundStyle(post.isLiked ? .red : .secondary)
                        Text("\(post.likesCount)")
                            .font(.footnote)
                            .foregroundStyle(post.isLiked ? .red : .secondary)
                    }
                }

                // Comment Button
                Button {
                    onCommentTapped()
                } label: {
                    HStack(spacing: 6) {
                        Image(systemName: "bubble.right")
                            .foregroundStyle(.secondary)
                        Text("\(post.commentsCount)")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                }

                // Repost Button
                Button {
                    post.isRepostedByMe.toggle()
                    post.repostsCount += post.isRepostedByMe ? 1 : -1
                } label: {
                    HStack(spacing: 6) {
                        Image(systemName: "arrow.rectanglepath")
                            .foregroundStyle(post.isRepostedByMe ? .green : .secondary)
                        Text("\(post.repostsCount)")
                            .font(.footnote)
                            .foregroundStyle(post.isRepostedByMe ? .green : .secondary)
                    }
                }

                Spacer()

                // Bookmark / Save
                Button {
                    toggleSave()
                } label: {
                    Image(systemName: post.isSavedByMe ? "bookmark.fill" : "bookmark")
                        .foregroundStyle(post.isSavedByMe ? .indigo : .secondary)
                }

                // Share
                ShareLink(item: post.content) {
                    Image(systemName: "square.and.arrow.up")
                        .foregroundStyle(.secondary)
                }
            }
            .font(.subheadline)
            .padding(.top, 4)
        }
        .padding(.vertical, 12)
        .padding(.horizontal, 16)
        .background(Color(.systemBackground))
    }

    private func audienceBadge(audience: PostAudienceGql) -> some View {
        HStack(spacing: 4) {
            Image(systemName: audience == .public ? "globe" : (audience == .closeFriends ? "star.fill" : "person.2.fill"))
                .font(.system(size: 10))
            Text(audience.title)
                .font(.system(size: 11, weight: .semibold))
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .background(
            audience == .closeFriends ? Color.green.opacity(0.15) : Color(.tertiarySystemFill)
        )
        .foregroundStyle(audience == .closeFriends ? Color.green : Color.secondary)
        .clipShape(Capsule())
    }

    private func pollView(poll: Poll) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(poll.question)
                .font(.subheadline)
                .fontWeight(.semibold)

            VStack(spacing: 6) {
                ForEach(poll.options) { option in
                    let percentage = poll.totalVotes > 0 ? Double(option.votesCount) / Double(poll.totalVotes) : 0.0

                    Button {
                        voteOption(optionId: option.id)
                    } label: {
                        ZStack(alignment: .leading) {
                            // Progress bar
                            GeometryReader { geo in
                                RoundedRectangle(cornerRadius: 8)
                                    .fill(option.isVotedByMe ? Color.indigo.opacity(0.25) : Color(.secondarySystemFill))
                                    .frame(width: geo.size.width * CGFloat(percentage))
                            }

                            HStack {
                                Text(option.text)
                                    .font(.subheadline)
                                    .fontWeight(option.isVotedByMe ? .semibold : .regular)
                                    .foregroundStyle(.primary)

                                Spacer()

                                if poll.hasVoted {
                                    Text("\(Int(percentage * 100))%")
                                        .font(.caption)
                                        .fontWeight(.bold)
                                        .foregroundStyle(.secondary)
                                }
                            }
                            .padding(.horizontal, 12)
                            .padding(.vertical, 10)
                        }
                        .frame(height: 40)
                        .clipShape(RoundedRectangle(cornerRadius: 8))
                    }
                    .disabled(poll.hasVoted || poll.isClosed)
                }
            }

            HStack {
                Text("\(poll.totalVotes) votes")
                if let exp = poll.expiresAt {
                    Text("• \(exp)")
                }
            }
            .font(.caption2)
            .foregroundStyle(.secondary)
        }
        .padding(12)
        .background(Color(.secondarySystemBackground))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }

    private func voteOption(optionId: String) {
        guard var poll = post.poll, !poll.hasVoted else { return }
        poll.hasVoted = true
        poll.totalVotes += 1
        for i in 0..<poll.options.count {
            if poll.options[i].id == optionId {
                poll.options[i].isVotedByMe = true
                poll.options[i].votesCount += 1
            }
        }
        post.poll = poll
    }

    private func toggleLike() {
        post.isLiked.toggle()
        post.likesCount += post.isLiked ? 1 : -1
        Task {
            _ = try? await FerroService.shared.toggleLikePost(postId: post.id, currentlyLiked: !post.isLiked)
        }
    }

    private func toggleSave() {
        post.isSavedByMe.toggle()
        Task {
            _ = try? await FerroService.shared.toggleSavePost(postId: post.id, currentlySaved: !post.isSavedByMe)
        }
    }
}
