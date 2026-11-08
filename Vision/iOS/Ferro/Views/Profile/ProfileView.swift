import SwiftUI

public struct ProfileView: View {
    @Environment(AppState.self) private var appState

    @State private var selectedSubtab = 0
    @State private var showSettings = false
    @State private var allFeedPosts: [Post] = []
    @State private var isLoadingPosts = false
    @State private var commentTargetPostIndex: Int? = nil

    public init() {}

    private var displayedPosts: [Post] {
        switch selectedSubtab {
        case 0:
            // My posts
            let myId = appState.currentUser?.id
            return allFeedPosts.filter { $0.author.isMe || ($0.author.id == myId) }
        case 1:
            // Liked posts
            return allFeedPosts.filter { $0.isLiked }
        case 2:
            // Saved posts
            return allFeedPosts.filter { $0.isSavedByMe }
        default:
            return []
        }
    }

    public var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 0) {
                    let user = appState.currentUser ?? User(
                        id: "me",
                        username: "profile",
                        email: "",
                        displayName: "User"
                    )

                    // Header Banner
                    AsyncImage(url: URL(string: user.headerImageUrl ?? "")) { phase in
                        if let image = phase.image {
                            image
                                .resizable()
                                .scaledToFill()
                                .frame(height: 140)
                                .clipped()
                        } else {
                            LinearGradient(
                                colors: [.indigo.opacity(0.8), .purple.opacity(0.6), .cyan.opacity(0.7)],
                                startPoint: .topLeading,
                                endPoint: .bottomTrailing
                            )
                            .frame(height: 140)
                        }
                    }

                    // Avatar & Actions Row
                    HStack(alignment: .bottom) {
                        // Avatar
                        ZStack {
                            Circle()
                                .fill(Color(.systemBackground))
                                .frame(width: 84, height: 84)

                            AsyncImage(url: URL(string: user.avatarUrl ?? "")) { phase in
                                if let image = phase.image {
                                    image
                                        .resizable()
                                        .scaledToFill()
                                } else {
                                    Circle().fill(Color(.tertiarySystemFill))
                                }
                            }
                            .frame(width: 78, height: 78)
                            .clipShape(Circle())
                        }
                        .offset(y: -42)
                        .padding(.bottom, -42)

                        Spacer()

                        // Action Buttons
                        HStack(spacing: 8) {
                            Button {
                                showSettings = true
                            } label: {
                                Image(systemName: "gearshape")
                                    .font(.subheadline)
                                    .padding(8)
                                    .background(Color(.secondarySystemBackground))
                                    .clipShape(Circle())
                            }

                            Button {
                                // Edit Profile action
                            } label: {
                                Text("Edit Profile")
                                    .font(.subheadline)
                                    .fontWeight(.semibold)
                                    .padding(.horizontal, 16)
                                    .padding(.vertical, 8)
                                    .background(Color(.secondarySystemBackground))
                                    .clipShape(Capsule())
                            }
                        }
                    }
                    .padding(.horizontal, 16)
                    .padding(.top, 10)

                    // Profile Info
                    VStack(alignment: .leading, spacing: 10) {
                        VStack(alignment: .leading, spacing: 2) {
                            HStack(spacing: 6) {
                                Text(user.displayName)
                                    .font(.title2)
                                    .fontWeight(.bold)

                                if user.is2faEnabled {
                                    Image(systemName: "checkmark.seal.fill")
                                        .foregroundStyle(.blue)
                                        .font(.subheadline)
                                }
                            }

                            Text("@\(user.username)")
                                .font(.subheadline)
                                .foregroundStyle(.secondary)
                        }

                        if let bio = user.bio, !bio.isEmpty {
                            Text(bio)
                                .font(.subheadline)
                                .lineSpacing(3)
                        }

                        // Location & Website
                        HStack(spacing: 16) {
                            if let loc = user.location, !loc.isEmpty {
                                Label(loc, systemImage: "mappin.circle.fill")
                            }
                            if let web = user.website, !web.isEmpty {
                                Label(web.replacingOccurrences(of: "https://", with: ""), systemImage: "link")
                            }
                        }
                        .font(.caption)
                        .foregroundStyle(.secondary)

                        // Followers / Following Stats
                        HStack(spacing: 20) {
                            statItem(count: user.postsCount, label: "Posts")
                            statItem(count: user.followersCount, label: "Followers")
                            statItem(count: user.followingCount, label: "Following")
                        }
                        .padding(.top, 4)
                    }
                    .padding(.horizontal, 16)
                    .padding(.top, 8)

                    Divider()
                        .padding(.top, 16)

                    // Subtabs Picker
                    Picker("Profile Section", selection: $selectedSubtab) {
                        Text("Posts").tag(0)
                        Text("Liked").tag(1)
                        Text("Saved").tag(2)
                    }
                    .pickerStyle(.segmented)
                    .padding()

                    // Posts list
                    if isLoadingPosts && displayedPosts.isEmpty {
                        VStack(spacing: 8) {
                            ProgressView()
                            Text("Loading posts...")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 24)
                    } else if displayedPosts.isEmpty {
                        VStack(spacing: 12) {
                            Image(systemName: subtabEmptyIcon)
                                .font(.system(size: 40))
                                .foregroundStyle(.tertiary)
                                .padding(.top, 24)

                            Text(subtabEmptyTitle)
                                .font(.headline)

                            Text(subtabEmptySubtitle)
                                .font(.subheadline)
                                .foregroundStyle(.secondary)
                                .multilineTextAlignment(.center)
                                .padding(.horizontal, 32)
                        }
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 20)
                    } else {
                        LazyVStack(spacing: 0) {
                            ForEach(displayedPosts) { post in
                                if let idx = allFeedPosts.firstIndex(where: { $0.id == post.id }) {
                                    VStack(spacing: 0) {
                                        PostRowView(post: $allFeedPosts[idx]) {
                                            commentTargetPostIndex = idx
                                        }
                                        Divider()
                                    }
                                }
                            }
                        }
                    }
                }
            }
            .navigationTitle("Profile")
            .navigationBarTitleDisplayMode(.inline)
            .refreshable {
                await appState.refreshCurrentUser()
                await loadPosts()
            }
            .task {
                await loadPosts()
            }
            .sheet(isPresented: $showSettings) {
                SettingsView()
            }
            .sheet(isPresented: Binding(
                get: { commentTargetPostIndex != nil },
                set: { if !$0 { commentTargetPostIndex = nil } }
            )) {
                if let idx = commentTargetPostIndex, idx < allFeedPosts.count {
                    CommentsSheetView(post: $allFeedPosts[idx])
                }
            }
        }
    }

    private var subtabEmptyIcon: String {
        switch selectedSubtab {
        case 0: return "square.and.pencil"
        case 1: return "heart.slash"
        case 2: return "bookmark.slash"
        default: return "sparkles"
        }
    }

    private var subtabEmptyTitle: String {
        switch selectedSubtab {
        case 0: return "No Posts Yet"
        case 1: return "No Liked Posts"
        case 2: return "No Saved Posts"
        default: return "No Items"
        }
    }

    private var subtabEmptySubtitle: String {
        switch selectedSubtab {
        case 0: return "Posts and thoughts you publish will appear here."
        case 1: return "Posts you like will be collected in this tab."
        case 2: return "Save interesting posts to read them again later."
        default: return ""
        }
    }

    private func statItem(count: Int, label: String) -> some View {
        HStack(spacing: 4) {
            Text("\(count)")
                .fontWeight(.bold)
            Text(label)
                .foregroundStyle(.secondary)
        }
        .font(.subheadline)
    }

    private func loadPosts() async {
        isLoadingPosts = true
        do {
            let posts = try await FerroService.shared.fetchFeed()
            await MainActor.run {
                self.allFeedPosts = posts
                self.isLoadingPosts = false
            }
        } catch {
            await MainActor.run {
                self.isLoadingPosts = false
            }
        }
    }
}
