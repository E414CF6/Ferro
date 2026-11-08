import SwiftUI

public struct StoryTrayView: View {
    let stories: [Story]
    let currentUser: User?
    let onSelectStory: (Story) -> Void
    let onAddStory: () -> Void

    public init(
        stories: [Story],
        currentUser: User?,
        onSelectStory: @escaping (Story) -> Void,
        onAddStory: @escaping () -> Void
    ) {
        self.stories = stories
        self.currentUser = currentUser
        self.onSelectStory = onSelectStory
        self.onAddStory = onAddStory
    }

    public var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            LazyHStack(spacing: 16) {
                // Current User "Add Story" item
                Button {
                    onAddStory()
                } label: {
                    VStack(spacing: 6) {
                        ZStack(alignment: .bottomTrailing) {
                            AsyncImage(url: URL(string: currentUser?.avatarUrl ?? "")) { phase in
                                if let image = phase.image {
                                    image
                                        .resizable()
                                        .scaledToFill()
                                } else {
                                    Circle().fill(Color(.tertiarySystemFill))
                                }
                            }
                            .frame(width: 62, height: 62)
                            .clipShape(Circle())

                            Circle()
                                .fill(Color.indigo)
                                .frame(width: 22, height: 22)
                                .overlay(
                                    Image(systemName: "plus")
                                        .font(.caption2)
                                        .fontWeight(.bold)
                                        .foregroundStyle(.white)
                                )
                        }

                        Text("Your Story")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                }

                // Active Stories from Followed Users
                ForEach(stories) { story in
                    Button {
                        onSelectStory(story)
                    } label: {
                        VStack(spacing: 6) {
                            ZStack {
                                Circle()
                                    .stroke(
                                        story.isViewedByMe ?
                                        LinearGradient(colors: [Color(.systemGray4)], startPoint: .top, endPoint: .bottom) :
                                        LinearGradient(colors: [.indigo, .purple, .pink], startPoint: .topLeading, endPoint: .bottomTrailing),
                                        lineWidth: 2.5
                                    )
                                    .frame(width: 66, height: 66)

                                AsyncImage(url: URL(string: story.author.avatarUrl ?? "")) { phase in
                                    if let image = phase.image {
                                        image
                                            .resizable()
                                            .scaledToFill()
                                    } else {
                                        Circle().fill(Color(.tertiarySystemFill))
                                    }
                                }
                                .frame(width: 58, height: 58)
                                .clipShape(Circle())
                            }

                            Text(story.author.displayName)
                                .font(.caption2)
                                .fontWeight(.medium)
                                .foregroundStyle(.primary)
                                .lineLimit(1)
                                .frame(width: 70)
                        }
                    }
                }
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
        }
    }
}

public struct StoryViewerModal: View {
    let story: Story
    let onDismiss: () -> Void

    @State private var progress: Double = 0.0
    @State private var timer: Timer?

    public init(story: Story, onDismiss: @escaping () -> Void) {
        self.story = story
        self.onDismiss = onDismiss
    }

    public var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()

            // Story Media Image
            AsyncImage(url: URL(string: story.mediaUrl)) { phase in
                if let image = phase.image {
                    image
                        .resizable()
                        .scaledToFill()
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                        .clipped()
                } else {
                    ProgressView().tint(.white)
                }
            }

            // Top Overlay: Progress Bar & Author Header
            VStack {
                // Progress Bar
                GeometryReader { geo in
                    ZStack(alignment: .leading) {
                        Capsule()
                            .fill(Color.white.opacity(0.3))
                            .frame(height: 3)

                        Capsule()
                            .fill(Color.white)
                            .frame(width: geo.size.width * CGFloat(progress), height: 3)
                    }
                }
                .frame(height: 3)
                .padding(.horizontal, 12)
                .padding(.top, 16)

                // Header Info
                HStack(spacing: 10) {
                    AsyncImage(url: URL(string: story.author.avatarUrl ?? "")) { phase in
                        if let image = phase.image {
                            image.resizable().scaledToFill()
                        } else {
                            Circle().fill(Color.gray)
                        }
                    }
                    .frame(width: 36, height: 36)
                    .clipShape(Circle())

                    VStack(alignment: .leading, spacing: 2) {
                        Text(story.author.displayName)
                            .font(.subheadline)
                            .fontWeight(.bold)
                            .foregroundStyle(.white)

                        Text("\(story.createdAt) • \(story.expiresAt)")
                            .font(.caption2)
                            .foregroundStyle(.white.opacity(0.8))
                    }

                    Spacer()

                    Button {
                        onDismiss()
                    } label: {
                        Image(systemName: "xmark")
                            .font(.subheadline)
                            .fontWeight(.bold)
                            .foregroundStyle(.white)
                            .padding(8)
                            .background(Circle().fill(Color.black.opacity(0.4)))
                    }
                }
                .padding(.horizontal, 16)
                .padding(.top, 8)

                Spacer()

                // Bottom Caption & Impression views
                if let caption = story.caption {
                    VStack(alignment: .leading, spacing: 6) {
                        Text(caption)
                            .font(.body)
                            .foregroundStyle(.white)
                            .padding(.horizontal, 16)
                            .padding(.vertical, 12)
                            .background(
                                RoundedRectangle(cornerRadius: 14)
                                    .fill(Material.ultraThinMaterial)
                            )
                            .padding(.horizontal, 16)

                        HStack {
                            Image(systemName: "eye.fill")
                                .font(.caption2)
                            Text("\(story.viewsCount) views")
                                .font(.caption)
                        }
                        .foregroundStyle(.white.opacity(0.8))
                        .padding(.leading, 24)
                    }
                    .padding(.bottom, 24)
                }
            }
        }
        .onAppear {
            startProgress()
        }
        .onDisappear {
            timer?.invalidate()
        }
        .onTapGesture {
            onDismiss()
        }
    }

    private func startProgress() {
        progress = 0.0
        timer = Timer.scheduledTimer(withTimeInterval: 0.05, repeats: true) { _ in
            if progress < 1.0 {
                progress += 0.01
            } else {
                timer?.invalidate()
                onDismiss()
            }
        }
    }
}
