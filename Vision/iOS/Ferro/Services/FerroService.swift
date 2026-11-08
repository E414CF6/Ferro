import Foundation

// MARK: - FerroService
public actor FerroService {
    public static let shared = FerroService()

    // MARK: - GraphQL DTOs
    public struct LoginResponse: Codable, Sendable {
        public let login: AuthPayload
    }

    public struct SignupResponse: Codable, Sendable {
        public let signup: AuthPayload
    }

    public struct LogoutResponse: Codable, Sendable {
        public let logout: Bool
    }

    public struct MeResponse: Codable, Sendable {
        public let me: User?
    }

    public struct FeedResponse: Codable, Sendable {
        public let feed: [Post]?
    }

    public struct PostsResponse: Codable, Sendable {
        public let posts: [Post]
    }

    public struct StoriesResponse: Codable, Sendable {
        public let storiesFeed: [Story]
    }

    public struct CreatePostResponse: Codable, Sendable {
        public let createPost: Post
    }

    public struct PostDetailCommentsResponse: Codable, Sendable {
        public struct PostDetail: Codable, Sendable {
            public let id: String
            public let comments: [Comment]
        }
        public let post: PostDetail?
    }

    public struct CreateCommentResponse: Codable, Sendable {
        public let createComment: Comment
    }

    public struct LikeResult: Codable, Sendable {
        public let id: String
        public let likesCount: Int?
    }

    public struct SaveResult: Codable, Sendable {
        public let id: String
    }

    public struct ConversationsResponse: Codable, Sendable {
        public let conversations: [Conversation]
    }

    public struct DMResponse: Codable, Sendable {
        public let directMessages: [DirectMessage]
    }

    public struct SendDMResponse: Codable, Sendable {
        public let sendDirectMessage: DirectMessage
    }

    public struct NotifResponse: Codable, Sendable {
        public let notifications: [NotificationItem]
    }

    public struct TrendsResponse: Codable, Sendable {
        public let trendingHashtags: [HashtagTrend]
    }

    public struct ArticlesResponse: Codable, Sendable {
        public let articles: [WikiArticle]
    }

    public struct SearchResponse: Codable, Sendable {
        public let searchUsers: [User]
    }

    private let gqlClient = GraphQLClient.shared
    private let restClient = RESTClient.shared
    private let wsClient = WebSocketClient.shared

    public init() {}

    // MARK: - Health & Diagnostic Probes
    public func checkBackendStatus() async -> (isLive: Bool, isReady: Bool, detail: String) {
        do {
            let liveness = try await restClient.checkLiveness()
            let readiness = try await restClient.checkReadiness()
            let ver = liveness.version ?? "1.0.0"
            let db = readiness.database ?? "connected"
            return (true, readiness.status == "ready", "Connected to Serve v\(ver) (\(db))")
        } catch {
            return (false, false, error.localizedDescription)
        }
    }

    // MARK: - Authentication
    public func login(usernameOrEmail: String, password: String) async throws -> AuthPayload {
        let mutation = """
        mutation Login($usernameOrEmail: String!, $password: String!) {
            login(usernameOrEmail: $usernameOrEmail, password: $password) {
                token
                refreshToken
                expiresIn
                tokenType
                user {
                    id
                    username
                    email
                    displayName
                    bio
                    avatarUrl
                    headerImageUrl
                    location
                    website
                    isPrivate
                    is2faEnabled
                    createdAt
                    postsCount
                    followersCount
                    followingCount
                    isMe
                    hasActiveStories
                }
            }
        }
        """

        let response: LoginResponse = try await gqlClient.execute(
            query: mutation,
            variables: ["usernameOrEmail": usernameOrEmail, "password": password],
            requiresAuth: false
        )

        KeychainHelper.shared.saveAccessToken(response.login.token)
        KeychainHelper.shared.saveRefreshToken(response.login.refreshToken)
        return response.login
    }

    public func signup(
        username: String,
        email: String,
        password: String,
        displayName: String
    ) async throws -> AuthPayload {
        let mutation = """
        mutation Signup($username: String!, $email: String!, $password: String!, $displayName: String!) {
            signup(username: $username, email: $email, password: $password, displayName: $displayName) {
                token
                refreshToken
                expiresIn
                tokenType
                user {
                    id
                    username
                    email
                    displayName
                    bio
                    avatarUrl
                    createdAt
                    postsCount
                    followersCount
                    followingCount
                    isMe
                }
            }
        }
        """

        let response: SignupResponse = try await gqlClient.execute(
            query: mutation,
            variables: [
                "username": username,
                "email": email,
                "password": password,
                "displayName": displayName
            ],
            requiresAuth: false
        )

        KeychainHelper.shared.saveAccessToken(response.signup.token)
        KeychainHelper.shared.saveRefreshToken(response.signup.refreshToken)
        return response.signup
    }

    public func logout() async {
        if let token = KeychainHelper.shared.getRefreshToken() {
            let mutation = """
            mutation Logout($refreshToken: String) {
                logout(refreshToken: $refreshToken)
            }
            """
            _ = try? await gqlClient.execute(
                query: mutation,
                variables: ["refreshToken": token],
                requiresAuth: false
            ) as LogoutResponse
        }
        KeychainHelper.shared.clearTokens()
    }

    public func fetchMe() async throws -> User {
        let query = """
        query Me {
            me {
                id
                username
                email
                displayName
                bio
                avatarUrl
                headerImageUrl
                location
                website
                isPrivate
                is2faEnabled
                createdAt
                postsCount
                followersCount
                followingCount
                isMe
                hasActiveStories
            }
        }
        """

        let response: MeResponse = try await gqlClient.execute(query: query, requiresAuth: true)
        guard let user = response.me else {
            throw FerroAPIError.unauthorized
        }
        return user
    }

    // MARK: - Feed & Posts
    public func fetchFeed(limit: Int = 20, offset: Int = 0) async throws -> [Post] {
        // Query the live posts endpoint which supports rich media, polls, hashtags and engagement
        let query = """
        query Posts($limit: Int, $offset: Int) {
            posts(limit: $limit, offset: $offset) {
                id
                content
                audience
                viewsCount
                quotePostId
                createdAt
                likesCount
                repostsCount
                isSavedByMe
                isRepostedByMe
                hashtags
                author {
                    id
                    username
                    displayName
                    avatarUrl
                    hasActiveStories
                    is2faEnabled
                }
                media {
                    id
                    mediaUrl
                    mediaType
                    width
                    height
                    sortOrder
                }
                poll {
                    id
                    question
                    totalVotes
                    expiresAt
                    options {
                        id
                        optionText
                        votesCount
                        isVotedByMe
                    }
                }
            }
        }
        """

        let response: PostsResponse = try await gqlClient.execute(
            query: query,
            variables: ["limit": limit, "offset": offset],
            requiresAuth: false
        )
        return response.posts
    }

    public func fetchStories() async throws -> [Story] {
        let query = """
        query StoriesFeed {
            storiesFeed {
                id
                mediaUrl
                caption
                createdAt
                expiresAt
                isExpired
                viewsCount
                isViewedByMe
                author {
                    id
                    username
                    displayName
                    avatarUrl
                }
            }
        }
        """

        let response: StoriesResponse = try await gqlClient.execute(query: query, requiresAuth: false)
        return response.storiesFeed
    }

    public func createPost(
        content: String,
        audience: PostAudienceGql = .public,
        pollQuestion: String? = nil,
        pollOptions: [String] = []
    ) async throws -> Post {
        let mutation = """
        mutation CreatePost($content: String!, $audience: PostAudienceGql, $poll: CreatePollInput) {
            createPost(content: $content, audience: $audience, poll: $poll) {
                id
                content
                audience
                createdAt
                likesCount
                repostsCount
                isSavedByMe
                isRepostedByMe
                hashtags
                author {
                    id
                    username
                    displayName
                    avatarUrl
                }
                poll {
                    id
                    question
                    options {
                        id
                        optionText
                        votesCount
                    }
                }
            }
        }
        """

        var variables: [String: Any] = [
            "content": content,
            "audience": audience.rawValue
        ]
        if let question = pollQuestion, !pollOptions.isEmpty {
            variables["poll"] = [
                "question": question,
                "options": pollOptions
            ]
        }

        let response: CreatePostResponse = try await gqlClient.execute(
            query: mutation,
            variables: variables,
            requiresAuth: true
        )
        return response.createPost
    }

    public func toggleLikePost(postId: String, currentlyLiked: Bool) async throws -> Bool {
        let mutation = currentlyLiked ? """
        mutation UnlikePost($postId: ID!) {
            unlikePost(postId: $postId) {
                id
                likesCount
            }
        }
        """ : """
        mutation LikePost($postId: ID!) {
            likePost(postId: $postId) {
                id
                likesCount
            }
        }
        """

        _ = try await gqlClient.execute(
            query: mutation,
            variables: ["postId": postId],
            requiresAuth: true
        ) as [String: LikeResult]

        return !currentlyLiked
    }

    public func toggleSavePost(postId: String, currentlySaved: Bool) async throws -> Bool {
        let mutation = currentlySaved ? """
        mutation UnsavePost($postId: ID!) {
            unsavePost(postId: $postId) {
                id
            }
        }
        """ : """
        mutation SavePost($postId: ID!) {
            savePost(postId: $postId) {
                id
            }
        }
        """

        _ = try await gqlClient.execute(
            query: mutation,
            variables: ["postId": postId],
            requiresAuth: true
        ) as [String: SaveResult]

        return !currentlySaved
    }

    public func fetchPostComments(postId: String, limit: Int = 30, offset: Int = 0) async throws -> [Comment] {
        let query = """
        query PostComments($postId: ID!) {
            post(id: $postId) {
                id
                comments {
                    id
                    parentId
                    content
                    isEdited
                    createdAt
                    updatedAt
                    depth
                    likesCount
                    isLikedByMe
                    repliesCount
                    author {
                        id
                        username
                        displayName
                        avatarUrl
                    }
                    replies {
                        id
                        parentId
                        content
                        createdAt
                        author {
                            id
                            username
                            displayName
                            avatarUrl
                        }
                    }
                }
            }
        }
        """

        let response: PostDetailCommentsResponse = try await gqlClient.execute(
            query: query,
            variables: ["postId": postId],
            requiresAuth: false
        )
        return response.post?.comments ?? []
    }

    public func createComment(postId: String, content: String, parentId: String? = nil) async throws -> Comment {
        let mutation = """
        mutation CreateComment($postId: ID!, $content: String!, $parentId: ID) {
            createComment(postId: $postId, content: $content, parentId: $parentId) {
                id
                parentId
                content
                createdAt
                depth
                author {
                    id
                    username
                    displayName
                    avatarUrl
                }
            }
        }
        """

        var vars: [String: Any] = ["postId": postId, "content": content]
        if let pid = parentId { vars["parentId"] = pid }

        let response: CreateCommentResponse = try await gqlClient.execute(
            query: mutation,
            variables: vars,
            requiresAuth: true
        )
        return response.createComment
    }

    // MARK: - Conversations & Direct Messages
    public func fetchConversations() async throws -> [Conversation] {
        let query = """
        query Conversations {
            conversations {
                unreadCount
                otherUser {
                    id
                    username
                    displayName
                    avatarUrl
                }
                lastMessage {
                    id
                    conversationId
                    content
                    isRead
                    createdAt
                    sender {
                        id
                        username
                        displayName
                    }
                    recipient {
                        id
                        username
                        displayName
                    }
                }
            }
        }
        """

        let response: ConversationsResponse = try await gqlClient.execute(query: query, requiresAuth: true)
        return response.conversations
    }

    public func fetchMessages(otherUserId: String, limit: Int = 50, offset: Int = 0) async throws -> [DirectMessage] {
        let query = """
        query DirectMessages($otherUserId: ID!, $limit: Int, $offset: Int) {
            directMessages(otherUserId: $otherUserId, limit: $limit, offset: $offset) {
                id
                conversationId
                content
                isRead
                createdAt
                sender {
                    id
                    username
                    displayName
                }
                recipient {
                    id
                    username
                    displayName
                }
            }
        }
        """

        let response: DMResponse = try await gqlClient.execute(
            query: query,
            variables: ["otherUserId": otherUserId, "limit": limit, "offset": offset],
            requiresAuth: true
        )
        return response.directMessages
    }

    public func sendDirectMessage(recipientId: String, content: String) async throws -> DirectMessage {
        let mutation = """
        mutation SendDirectMessage($recipientId: ID!, $content: String!) {
            sendDirectMessage(recipientId: $recipientId, content: $content) {
                id
                conversationId
                content
                isRead
                createdAt
                sender {
                    id
                    username
                    displayName
                }
                recipient {
                    id
                    username
                    displayName
                }
            }
        }
        """

        let response: SendDMResponse = try await gqlClient.execute(
            query: mutation,
            variables: ["recipientId": recipientId, "content": content],
            requiresAuth: true
        )
        return response.sendDirectMessage
    }

    // MARK: - Notifications
    public func fetchNotifications(limit: Int = 30) async throws -> [NotificationItem] {
        let query = """
        query Notifications($limit: Int) {
            notifications(limit: $limit) {
                id
                notificationType
                isRead
                createdAt
                actor {
                    id
                    username
                    displayName
                    avatarUrl
                }
                post {
                    id
                    content
                }
                comment {
                    id
                    content
                }
            }
        }
        """

        let response: NotifResponse = try await gqlClient.execute(
            query: query,
            variables: ["limit": limit],
            requiresAuth: true
        )
        return response.notifications
    }

    // MARK: - Explore & Geospatial Wiki
    public func fetchTrendingHashtags() async throws -> [HashtagTrend] {
        let query = """
        query TrendingHashtags {
            trendingHashtags {
                hashtag
                postsCount
            }
        }
        """

        let response: TrendsResponse = try await gqlClient.execute(query: query, requiresAuth: false)
        return response.trendingHashtags
    }

    public func fetchWikiArticles() async throws -> [WikiArticle] {
        let query = """
        query Articles {
            articles {
                id
                title
                slug
                summary
                content
                latitude
                longitude
                zoom
                category
                tags
                author
                views
                createdAt
                updatedAt
            }
        }
        """

        let response: ArticlesResponse = try await gqlClient.execute(query: query, requiresAuth: false)
        return response.articles
    }

    public func searchUsers(query: String) async throws -> [User] {
        let gql = """
        query SearchUsers($query: String!) {
            searchUsers(query: $query) {
                id
                username
                displayName
                avatarUrl
                bio
                followersCount
                isFollowedByMe
            }
        }
        """

        let response: SearchResponse = try await gqlClient.execute(
            query: gql,
            variables: ["query": query],
            requiresAuth: false
        )
        return response.searchUsers
    }

    // MARK: - Media Upload
    public func uploadMedia(fileData: Data, fileName: String, mimeType: String) async throws -> UploadResult {
        return try await restClient.uploadMedia(fileData: fileData, fileName: fileName, mimeType: mimeType)
    }
}
