import Foundation

public enum MediaTypeGql: String, Codable, Sendable, CaseIterable {
    case image = "IMAGE"
    case video = "VIDEO"
    case gif = "GIF"
}

public enum PostAudienceGql: String, Codable, Sendable, CaseIterable {
    case `public` = "PUBLIC"
    case followersOnly = "FOLLOWERS_ONLY"
    case closeFriends = "CLOSE_FRIENDS"

    public var title: String {
        switch self {
        case .public:
            return "Public"
        case .followersOnly:
            return "Followers Only"
        case .closeFriends:
            return "Close Friends"
        }
    }
}

public struct PostMedia: Identifiable, Codable, Sendable, Hashable {
    public let id: String
    public let url: String
    public let mediaType: MediaTypeGql
    public let width: Int?
    public let height: Int?
    public let aspectRatio: Double?

    enum CodingKeys: String, CodingKey {
        case id
        case url
        case mediaUrl
        case mediaType
        case width
        case height
        case aspectRatio
    }

    public init(
        id: String,
        url: String,
        mediaType: MediaTypeGql = .image,
        width: Int? = nil,
        height: Int? = nil,
        aspectRatio: Double? = 1.0
    ) {
        self.id = id
        self.url = url
        self.mediaType = mediaType
        self.width = width
        self.height = height
        self.aspectRatio = aspectRatio
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.id = try container.decode(String.self, forKey: .id)
        if let mediaUrlStr = try container.decodeIfPresent(String.self, forKey: .mediaUrl) {
            self.url = mediaUrlStr
        } else if let urlStr = try container.decodeIfPresent(String.self, forKey: .url) {
            self.url = urlStr
        } else {
            self.url = ""
        }
        self.mediaType = try container.decodeIfPresent(MediaTypeGql.self, forKey: .mediaType) ?? .image
        self.width = try container.decodeIfPresent(Int.self, forKey: .width)
        self.height = try container.decodeIfPresent(Int.self, forKey: .height)
        self.aspectRatio = try container.decodeIfPresent(Double.self, forKey: .aspectRatio)
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(id, forKey: .id)
        try container.encode(url, forKey: .url)
        try container.encode(mediaType, forKey: .mediaType)
        try container.encodeIfPresent(width, forKey: .width)
        try container.encodeIfPresent(height, forKey: .height)
        try container.encodeIfPresent(aspectRatio, forKey: .aspectRatio)
    }
}

public struct PollOption: Identifiable, Codable, Sendable, Hashable {
    public let id: String
    public var text: String
    public var votesCount: Int
    public var isVotedByMe: Bool

    enum CodingKeys: String, CodingKey {
        case id
        case text
        case optionText
        case votesCount
        case isVotedByMe
    }

    public init(id: String, text: String, votesCount: Int = 0, isVotedByMe: Bool = false) {
        self.id = id
        self.text = text
        self.votesCount = votesCount
        self.isVotedByMe = isVotedByMe
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.id = try container.decode(String.self, forKey: .id)
        if let optText = try container.decodeIfPresent(String.self, forKey: .optionText) {
            self.text = optText
        } else if let txt = try container.decodeIfPresent(String.self, forKey: .text) {
            self.text = txt
        } else {
            self.text = ""
        }
        self.votesCount = try container.decodeIfPresent(Int.self, forKey: .votesCount) ?? 0
        self.isVotedByMe = try container.decodeIfPresent(Bool.self, forKey: .isVotedByMe) ?? false
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(id, forKey: .id)
        try container.encode(text, forKey: .text)
        try container.encode(votesCount, forKey: .votesCount)
        try container.encode(isVotedByMe, forKey: .isVotedByMe)
    }
}

public struct Poll: Identifiable, Codable, Sendable, Hashable {
    public let id: String
    public var question: String
    public var options: [PollOption]
    public var totalVotes: Int
    public var hasVoted: Bool
    public var isClosed: Bool
    public var expiresAt: String?

    public init(
        id: String,
        question: String,
        options: [PollOption],
        totalVotes: Int = 0,
        hasVoted: Bool = false,
        isClosed: Bool = false,
        expiresAt: String? = nil
    ) {
        self.id = id
        self.question = question
        self.options = options
        self.totalVotes = totalVotes
        self.hasVoted = hasVoted
        self.isClosed = isClosed
        self.expiresAt = expiresAt
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.id = try container.decode(String.self, forKey: .id)
        self.question = try container.decode(String.self, forKey: .question)
        self.options = try container.decodeIfPresent([PollOption].self, forKey: .options) ?? []
        self.totalVotes = try container.decodeIfPresent(Int.self, forKey: .totalVotes) ?? 0
        self.hasVoted = try container.decodeIfPresent(Bool.self, forKey: .hasVoted) ?? false
        self.isClosed = try container.decodeIfPresent(Bool.self, forKey: .isClosed) ?? false
        self.expiresAt = try container.decodeIfPresent(String.self, forKey: .expiresAt)
    }
}

public struct PostAnalytics: Codable, Sendable, Hashable {
    public var impressions: Int
    public var likesCount: Int
    public var repostsCount: Int
    public var repliesCount: Int
    public var bookmarksCount: Int

    public init(
        impressions: Int = 0,
        likesCount: Int = 0,
        repostsCount: Int = 0,
        repliesCount: Int = 0,
        bookmarksCount: Int = 0
    ) {
        self.impressions = impressions
        self.likesCount = likesCount
        self.repostsCount = repostsCount
        self.repliesCount = repliesCount
        self.bookmarksCount = bookmarksCount
    }
}

public struct Comment: Identifiable, Codable, Sendable, Hashable {
    public let id: String
    public var parentId: String?
    public var content: String
    public var isEdited: Bool
    public var createdAt: String
    public var updatedAt: String
    public var author: User
    public var depth: Int
    public var replies: [Comment]
    public var repliesCount: Int
    public var likesCount: Int
    public var isLikedByMe: Bool

    public init(
        id: String,
        parentId: String? = nil,
        content: String,
        isEdited: Bool = false,
        createdAt: String = "",
        updatedAt: String = "",
        author: User,
        depth: Int = 0,
        replies: [Comment] = [],
        repliesCount: Int = 0,
        likesCount: Int = 0,
        isLikedByMe: Bool = false
    ) {
        self.id = id
        self.parentId = parentId
        self.content = content
        self.isEdited = isEdited
        self.createdAt = createdAt
        self.updatedAt = updatedAt
        self.author = author
        self.depth = depth
        self.replies = replies
        self.repliesCount = repliesCount
        self.likesCount = likesCount
        self.isLikedByMe = isLikedByMe
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.id = try container.decode(String.self, forKey: .id)
        self.parentId = try container.decodeIfPresent(String.self, forKey: .parentId)
        self.content = try container.decode(String.self, forKey: .content)
        self.isEdited = try container.decodeIfPresent(Bool.self, forKey: .isEdited) ?? false
        self.createdAt = try container.decodeIfPresent(String.self, forKey: .createdAt) ?? ""
        self.updatedAt = try container.decodeIfPresent(String.self, forKey: .updatedAt) ?? ""
        self.author = try container.decode(User.self, forKey: .author)
        self.depth = try container.decodeIfPresent(Int.self, forKey: .depth) ?? 0
        self.replies = try container.decodeIfPresent([Comment].self, forKey: .replies) ?? []
        self.repliesCount = try container.decodeIfPresent(Int.self, forKey: .repliesCount) ?? 0
        self.likesCount = try container.decodeIfPresent(Int.self, forKey: .likesCount) ?? 0
        self.isLikedByMe = try container.decodeIfPresent(Bool.self, forKey: .isLikedByMe) ?? false
    }
}

public struct Post: Identifiable, Codable, Sendable, Hashable {
    public let id: String
    public var content: String
    public var audience: PostAudienceGql
    public var viewsCount: Int
    public var quotePostId: String?
    public var createdAt: String
    public var author: User
    public var media: [PostMedia]
    public var poll: Poll?
    public var analytics: PostAnalytics?
    public var likesCount: Int
    public var repostsCount: Int
    public var commentsCount: Int
    public var isLiked: Bool
    public var isRepostedByMe: Bool
    public var isSavedByMe: Bool
    public var hashtags: [String]
    public var comments: [Comment]

    public init(
        id: String,
        content: String,
        audience: PostAudienceGql = .public,
        viewsCount: Int = 0,
        quotePostId: String? = nil,
        createdAt: String = "",
        author: User,
        media: [PostMedia] = [],
        poll: Poll? = nil,
        analytics: PostAnalytics? = nil,
        likesCount: Int = 0,
        repostsCount: Int = 0,
        commentsCount: Int = 0,
        isLiked: Bool = false,
        isRepostedByMe: Bool = false,
        isSavedByMe: Bool = false,
        hashtags: [String] = [],
        comments: [Comment] = []
    ) {
        self.id = id
        self.content = content
        self.audience = audience
        self.viewsCount = viewsCount
        self.quotePostId = quotePostId
        self.createdAt = createdAt
        self.author = author
        self.media = media
        self.poll = poll
        self.analytics = analytics
        self.likesCount = likesCount
        self.repostsCount = repostsCount
        self.commentsCount = commentsCount
        self.isLiked = isLiked
        self.isRepostedByMe = isRepostedByMe
        self.isSavedByMe = isSavedByMe
        self.hashtags = hashtags
        self.comments = comments
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.id = try container.decode(String.self, forKey: .id)
        self.content = try container.decode(String.self, forKey: .content)
        self.audience = try container.decodeIfPresent(PostAudienceGql.self, forKey: .audience) ?? .public
        self.viewsCount = try container.decodeIfPresent(Int.self, forKey: .viewsCount) ?? 0
        self.quotePostId = try container.decodeIfPresent(String.self, forKey: .quotePostId)
        self.createdAt = try container.decodeIfPresent(String.self, forKey: .createdAt) ?? ""
        self.author = try container.decode(User.self, forKey: .author)
        self.media = try container.decodeIfPresent([PostMedia].self, forKey: .media) ?? []
        self.poll = try container.decodeIfPresent(Poll.self, forKey: .poll)
        self.analytics = try container.decodeIfPresent(PostAnalytics.self, forKey: .analytics)
        self.likesCount = try container.decodeIfPresent(Int.self, forKey: .likesCount) ?? 0
        self.repostsCount = try container.decodeIfPresent(Int.self, forKey: .repostsCount) ?? 0
        self.commentsCount = try container.decodeIfPresent(Int.self, forKey: .commentsCount) ?? 0
        self.isLiked = try container.decodeIfPresent(Bool.self, forKey: .isLiked) ?? false
        self.isRepostedByMe = try container.decodeIfPresent(Bool.self, forKey: .isRepostedByMe) ?? false
        self.isSavedByMe = try container.decodeIfPresent(Bool.self, forKey: .isSavedByMe) ?? false
        self.hashtags = try container.decodeIfPresent([String].self, forKey: .hashtags) ?? []
        self.comments = try container.decodeIfPresent([Comment].self, forKey: .comments) ?? []
    }
}

public struct PageInfo: Codable, Sendable {
    public let hasNextPage: Bool
    public let endCursor: String?

    public init(hasNextPage: Bool, endCursor: String? = nil) {
        self.hasNextPage = hasNextPage
        self.endCursor = endCursor
    }
}

public struct PostConnection: Codable, Sendable {
    public let edges: [Post]
    public let pageInfo: PageInfo

    public init(edges: [Post], pageInfo: PageInfo) {
        self.edges = edges
        self.pageInfo = pageInfo
    }
}
