import SwiftUI

public struct FeedView: View {
    @Environment(AppState.self) private var appState

    @State private var posts: [Post] = []
    @State private var stories: [Story] = []
    @State private var isLoading = false
    @State private var loadErrorMessage: String? = nil
    @State private var showCreatePost = false
    @State private var activeStory: Story? = nil
    @State private var commentTargetPostIndex: Int? = nil

    public init() {}

    public var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 0) {
                    // Ephemeral Stories Bar
                    if !stories.isEmpty || appState.currentUser != nil {
                        StoryTrayView(
                            stories: stories,
                            currentUser: appState.currentUser,
                            onSelectStory: { story in
                                activeStory = story
                            },
                            onAddStory: {
                                if let user = appState.currentUser {
                                    let newStory = Story(
                                        id: "story_\(UUID().uuidString)",
                                        mediaUrl: "https://images.unsplash.com/photo-1518770660439-4636190af475?w=800",
                                        caption: "Shared a moment to Ferro stories! ✨",
                                        createdAt: "Just now",
                                        expiresAt: "24 hours left",
                                        author: user
                                    )
                                    stories.insert(newStory, at: 0)
                                }
                            }
                        )

                        Divider()
                    }

                    // Error banner
                    if let error = loadErrorMessage {
                        HStack(spacing: 12) {
                            Image(systemName: "exclamationmark.triangle.fill")
                                .foregroundStyle(.orange)
                            Text(error)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                                .lineLimit(2)
                            Spacer()
                            Button("Retry") {
                                Task { await loadFeed() }
                            }
                            .font(.caption.weight(.semibold))
                            .buttonStyle(.bordered)
                            .tint(.indigo)
                        }
                        .padding(.horizontal, 16)
                        .padding(.vertical, 10)
                        .background(Color.orange.opacity(0.1))
                    }

                    // Content State
                    if isLoading && posts.isEmpty {
                        // Skeleton placeholder loading state
                        VStack(spacing: 16) {
                            ForEach(0..<3, id: \.self) { _ in
                                FeedSkeletonCard()
                            }
                        }
                        .padding(.top, 16)
                    } else if posts.isEmpty {
                        // Empty State
                        VStack(spacing: 16) {
                            Image(systemName: "sparkles.rectangle.stack")
                                .font(.system(size: 52))
                                .foregroundStyle(
                                    LinearGradient(
                                        colors: [.indigo, .purple, .pink],
                                        startPoint: .topLeading,
                                        endPoint: .bottomTrailing
                                    )
                                )
                                .padding(.top, 40)

                            Text("Your Feed is Ready")
                                .font(.title3.weight(.bold))

                            Text("Discover stories, read thoughts from creators, or share what's on your mind.")
                                .font(.subheadline)
                                .foregroundStyle(.secondary)
                                .multilineTextAlignment(.center)
                                .padding(.horizontal, 32)

                            HStack(spacing: 12) {
                                Button {
                                    appState.selectedTab = .explore
                                } label: {
                                    Label("Explore Trends", systemImage: "safari")
                                        .font(.subheadline.weight(.semibold))
                                        .padding(.horizontal, 16)
                                        .padding(.vertical, 10)
                                        .background(.ultraThinMaterial)
                                        .clipShape(Capsule())
                                }

                                Button {
                                    showCreatePost = true
                                } label: {
                                    Label("Write a Post", systemImage: "square.and.pencil")
                                        .font(.subheadline.weight(.semibold))
                                        .foregroundStyle(.white)
                                        .padding(.horizontal, 16)
                                        .padding(.vertical, 10)
                                        .background(Color.indigo)
                                        .clipShape(Capsule())
                                }
                            }
                            .padding(.top, 8)
                        }
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 40)
                    } else {
                        // Timeline Posts
                        LazyVStack(spacing: 0) {
                            ForEach($posts) { $post in
                                VStack(spacing: 0) {
                                    PostRowView(post: $post) {
                                        if let idx = posts.firstIndex(where: { $0.id == post.id }) {
                                            commentTargetPostIndex = idx
                                        }
                                    }
                                    Divider()
                                }
                            }
                        }
                    }
                }
            }
            .refreshable {
                await loadFeed()
            }
            .navigationTitle("Ferro")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    HStack(spacing: 6) {
                        Image(systemName: "circle.hexagongrid.fill")
                            .foregroundStyle(.indigo)
                        Text("Ferro")
                            .font(.headline)
                            .fontWeight(.bold)
                    }
                }

                ToolbarItem(placement: .topBarTrailing) {
                    Button {
                        showCreatePost = true
                    } label: {
                        Image(systemName: "square.and.pencil")
                            .font(.system(size: 18, weight: .semibold))
                            .foregroundStyle(.indigo)
                    }
                }
            }
            .sheet(isPresented: $showCreatePost) {
                CreatePostView { newPost in
                    posts.insert(newPost, at: 0)
                }
            }
            .sheet(isPresented: Binding(
                get: { commentTargetPostIndex != nil },
                set: { if !$0 { commentTargetPostIndex = nil } }
            )) {
                if let idx = commentTargetPostIndex, idx < posts.count {
                    CommentsSheetView(post: $posts[idx])
                }
            }
            .fullScreenCover(item: $activeStory) { story in
                StoryViewerModal(story: story) {
                    activeStory = nil
                }
            }
            .task {
                await loadFeed()
            }
        }
    }

    private func loadFeed() async {
        isLoading = true
        loadErrorMessage = nil
        do {
            self.posts = try await FerroService.shared.fetchFeed()
        } catch {
            self.loadErrorMessage = error.localizedDescription
        }

        do {
            self.stories = try await FerroService.shared.fetchStories()
        } catch {
            // Non-critical, ignore if stories fail
        }
        self.isLoading = false
    }
}

// MARK: - Feed Skeleton Card Placeholder
private struct FeedSkeletonCard: View {
    @State private var isAnimating = false

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(spacing: 12) {
                Circle()
                    .fill(Color.gray.opacity(0.2))
                    .frame(width: 44, height: 44)

                VStack(alignment: .leading, spacing: 6) {
                    RoundedRectangle(cornerRadius: 4)
                        .fill(Color.gray.opacity(0.25))
                        .frame(width: 120, height: 14)

                    RoundedRectangle(cornerRadius: 4)
                        .fill(Color.gray.opacity(0.15))
                        .frame(width: 70, height: 11)
                }
                Spacer()
            }

            RoundedRectangle(cornerRadius: 4)
                .fill(Color.gray.opacity(0.2))
                .frame(maxWidth: .infinity)
                .frame(height: 14)

            RoundedRectangle(cornerRadius: 4)
                .fill(Color.gray.opacity(0.2))
                .frame(width: 200, height: 14)

            RoundedRectangle(cornerRadius: 12)
                .fill(Color.gray.opacity(0.12))
                .frame(maxWidth: .infinity)
                .frame(height: 180)

            HStack {
                ForEach(0..<4, id: \.self) { _ in
                    RoundedRectangle(cornerRadius: 4)
                        .fill(Color.gray.opacity(0.15))
                        .frame(width: 40, height: 14)
                    Spacer()
                }
            }
            .padding(.top, 4)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
        .opacity(isAnimating ? 0.6 : 1.0)
        .animation(.easeInOut(duration: 0.9).repeatForever(autoreverses: true), value: isAnimating)
        .onAppear {
            isAnimating = true
        }
    }
}
