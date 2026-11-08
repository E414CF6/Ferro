import SwiftUI

public struct ExploreView: View {
    @State private var searchText = ""
    @State private var trendingHashtags: [HashtagTrend] = []
    @State private var wikiArticles: [WikiArticle] = []
    @State private var searchUsers: [User] = []
    @State private var selectedArticle: WikiArticle? = nil
    @State private var isSearching = false
    @State private var isLoading = false
    @State private var selectedCategory: String = "All"

    private let categories = ["All", "Metaverse Architecture", "Tech History", "Community", "Design"]

    public init() {}

    public var filteredArticles: [WikiArticle] {
        if selectedCategory == "All" {
            return wikiArticles
        }
        return wikiArticles.filter { $0.category == selectedCategory }
    }

    public var body: some View {
        NavigationStack {
            List {
                // If user is typing in search, show user search results
                if !searchText.isEmpty {
                    Section("Users") {
                        if isSearching {
                            HStack {
                                Spacer()
                                ProgressView("Searching users...")
                                Spacer()
                            }
                            .padding(.vertical, 8)
                        } else if searchUsers.isEmpty {
                            VStack(spacing: 8) {
                                Image(systemName: "person.slash")
                                    .font(.title2)
                                    .foregroundStyle(.secondary)
                                Text("No users found matching \"\(searchText)\"")
                                    .font(.subheadline)
                                    .foregroundStyle(.secondary)
                            }
                            .frame(maxWidth: .infinity)
                            .padding(.vertical, 16)
                        } else {
                            ForEach(searchUsers) { user in
                                userSearchResultRow(user: user)
                            }
                        }
                    }
                }

                // Section 1: Real-Time Trending Hashtags
                Section {
                    if isLoading && trendingHashtags.isEmpty {
                        HStack {
                            Spacer()
                            ProgressView()
                            Spacer()
                        }
                        .padding(.vertical, 8)
                    } else if trendingHashtags.isEmpty {
                        HStack {
                            Spacer()
                            VStack(spacing: 6) {
                                Image(systemName: "chart.line.uptrend.xyaxis")
                                    .font(.title2)
                                    .foregroundStyle(.tertiary)
                                Text("No trending topics right now")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                            .padding(.vertical, 8)
                            Spacer()
                        }
                    } else {
                        ForEach(trendingHashtags) { trend in
                            HStack {
                                VStack(alignment: .leading, spacing: 4) {
                                    Text("#\(trend.hashtag)")
                                        .font(.headline)
                                        .foregroundStyle(.primary)

                                    Text("\(trend.postsCount) posts")
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                }

                                Spacer()

                                // Rank badge
                                rankBadge(for: trend.rankChange)
                            }
                            .padding(.vertical, 2)
                        }
                    }
                } header: {
                    Label("Trending on Ferro", systemImage: "chart.line.uptrend.xyaxis")
                }

                // Section 2: Geospatial Wiki Map (wMap) Articles
                Section {
                    // Category filter chips in header or top row
                    ScrollView(.horizontal, showsIndicators: false) {
                        HStack(spacing: 8) {
                            ForEach(categories, id: \.self) { cat in
                                Button {
                                    selectedCategory = cat
                                } label: {
                                    Text(cat)
                                        .font(.caption.weight(.semibold))
                                        .padding(.horizontal, 12)
                                        .padding(.vertical, 6)
                                        .background(selectedCategory == cat ? Color.indigo : Color(.secondarySystemFill))
                                        .foregroundStyle(selectedCategory == cat ? Color.white : Color.primary)
                                        .clipShape(Capsule())
                                }
                                .buttonStyle(.plain)
                            }
                        }
                        .padding(.vertical, 4)
                    }
                    .listRowInsets(EdgeInsets(top: 8, leading: 16, bottom: 8, trailing: 16))

                    if isLoading && wikiArticles.isEmpty {
                        HStack {
                            Spacer()
                            ProgressView()
                            Spacer()
                        }
                        .padding(.vertical, 12)
                    } else if filteredArticles.isEmpty {
                        VStack(spacing: 8) {
                            Image(systemName: "map")
                                .font(.title)
                                .foregroundStyle(.tertiary)
                            Text("No articles found in this category")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 20)
                    } else {
                        ForEach(filteredArticles) { article in
                            Button {
                                selectedArticle = article
                            } label: {
                                VStack(alignment: .leading, spacing: 6) {
                                    HStack {
                                        Text(article.title)
                                            .font(.headline)
                                            .foregroundStyle(.primary)

                                        Spacer()

                                        Text(article.category)
                                            .font(.caption2)
                                            .fontWeight(.semibold)
                                            .padding(.horizontal, 8)
                                            .padding(.vertical, 3)
                                            .background(Color.indigo.opacity(0.12))
                                            .foregroundStyle(.indigo)
                                            .clipShape(Capsule())
                                    }

                                    if let summary = article.summary {
                                        Text(summary)
                                            .font(.subheadline)
                                            .foregroundStyle(.secondary)
                                            .lineLimit(2)
                                    }

                                    HStack(spacing: 12) {
                                        HStack(spacing: 4) {
                                            Image(systemName: "mappin.and.ellipse")
                                            Text(String(format: "%.4f, %.4f", article.latitude, article.longitude))
                                        }
                                        .font(.caption2)
                                        .foregroundStyle(.secondary)

                                        HStack(spacing: 4) {
                                            Image(systemName: "eye")
                                            Text("\(article.views)")
                                        }
                                        .font(.caption2)
                                        .foregroundStyle(.secondary)
                                    }
                                }
                                .padding(.vertical, 4)
                            }
                        }
                    }
                } header: {
                    Label("Geospatial wMap Articles", systemImage: "map")
                }
            }
            .searchable(text: $searchText, prompt: "Search hashtags, users, or places...")
            .onChange(of: searchText) { _, newValue in
                performSearch(query: newValue)
            }
            .refreshable {
                await loadExploreData()
            }
            .navigationTitle("Explore")
            .sheet(item: $selectedArticle) { article in
                articleDetailSheet(article: article)
            }
            .task {
                await loadExploreData()
            }
        }
    }

    private func userSearchResultRow(user: User) -> some View {
        HStack(spacing: 12) {
            AsyncImage(url: URL(string: user.avatarUrl ?? "")) { phase in
                if let image = phase.image {
                    image.resizable().scaledToFill()
                } else {
                    Circle().fill(Color(.tertiarySystemFill))
                }
            }
            .frame(width: 40, height: 40)
            .clipShape(Circle())

            VStack(alignment: .leading, spacing: 2) {
                Text(user.displayName)
                    .font(.subheadline)
                    .fontWeight(.semibold)
                Text("@\(user.username)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            Button {
                // Follow / Unfollow
            } label: {
                Text(user.isFollowedByMe ? "Following" : "Follow")
                    .font(.caption)
                    .fontWeight(.semibold)
                    .padding(.horizontal, 14)
                    .padding(.vertical, 6)
                    .background(user.isFollowedByMe ? Color(.secondarySystemFill) : Color.indigo)
                    .foregroundStyle(user.isFollowedByMe ? Color.primary : Color.white)
                    .clipShape(Capsule())
            }
        }
    }

    private func rankBadge(for rankChange: String) -> some View {
        HStack(spacing: 4) {
            switch rankChange {
            case "UP":
                Image(systemName: "arrow.up.right")
                    .foregroundStyle(.green)
            case "DOWN":
                Image(systemName: "arrow.down.right")
                    .foregroundStyle(.red)
            case "NEW":
                Text("NEW")
                    .font(.system(size: 10, weight: .bold))
                    .foregroundStyle(.purple)
            default:
                Image(systemName: "minus")
                    .foregroundStyle(.secondary)
            }
        }
        .font(.caption2)
        .padding(6)
        .background(Color(.secondarySystemBackground))
        .clipShape(Circle())
    }

    private func articleDetailSheet(article: WikiArticle) -> some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    Text(article.title)
                        .font(.title)
                        .fontWeight(.bold)

                    HStack(spacing: 12) {
                        Label("@\(article.author)", systemImage: "person.circle")
                        Label(article.category, systemImage: "folder")
                        Label("\(article.views) views", systemImage: "eye")
                    }
                    .font(.caption)
                    .foregroundStyle(.secondary)

                    // Coordinate Card
                    HStack {
                        VStack(alignment: .leading, spacing: 4) {
                            Text("Geospatial Coordinates")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                            Text("\(article.latitude, specifier: "%.4f"), \(article.longitude, specifier: "%.4f") (Zoom: \(Int(article.zoom)))")
                                .font(.system(.subheadline, design: .monospaced))
                                .fontWeight(.semibold)
                        }
                        Spacer()
                        Image(systemName: "location.north.circle.fill")
                            .font(.title)
                            .foregroundStyle(.indigo)
                    }
                    .padding()
                    .background(Color(.secondarySystemBackground))
                    .clipShape(RoundedRectangle(cornerRadius: 12))

                    // Content
                    Text(article.content)
                        .font(.body)
                        .lineSpacing(6)

                    // Tags
                    if !article.tags.isEmpty {
                        VStack(alignment: .leading, spacing: 8) {
                            Text("Tags")
                                .font(.headline)
                            HStack(spacing: 8) {
                                ForEach(article.tags, id: \.self) { tag in
                                    Text("#\(tag)")
                                        .font(.caption)
                                        .padding(.horizontal, 10)
                                        .padding(.vertical, 4)
                                        .background(Color(.secondarySystemFill))
                                        .clipShape(Capsule())
                                }
                            }
                        }
                    }
                }
                .padding()
            }
            .navigationTitle("Wiki Article")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Done") {
                        selectedArticle = nil
                    }
                }
            }
        }
    }

    private func performSearch(query: String) {
        guard !query.isEmpty else {
            searchUsers = []
            isSearching = false
            return
        }
        isSearching = true
        Task {
            do {
                let results = try await FerroService.shared.searchUsers(query: query)
                await MainActor.run {
                    self.searchUsers = results
                    self.isSearching = false
                }
            } catch {
                await MainActor.run {
                    self.isSearching = false
                }
            }
        }
    }

    private func loadExploreData() async {
        isLoading = true
        async let trendsTask = FerroService.shared.fetchTrendingHashtags()
        async let articlesTask = FerroService.shared.fetchWikiArticles()

        let trends = (try? await trendsTask) ?? []
        let articles = (try? await articlesTask) ?? []

        await MainActor.run {
            self.trendingHashtags = trends
            self.wikiArticles = articles
            self.isLoading = false
        }
    }
}
