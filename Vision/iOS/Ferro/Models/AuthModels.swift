import Foundation

public struct AuthPayload: Codable, Sendable {
    public let token: String
    public let refreshToken: String
    public let expiresIn: Int
    public let tokenType: String
    public let user: User

    public init(
        token: String,
        refreshToken: String = "",
        expiresIn: Int = 900,
        tokenType: String = "Bearer",
        user: User
    ) {
        self.token = token
        self.refreshToken = refreshToken
        self.expiresIn = expiresIn
        self.tokenType = tokenType
        self.user = user
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.token = try container.decode(String.self, forKey: .token)
        self.refreshToken = try container.decodeIfPresent(String.self, forKey: .refreshToken) ?? ""
        self.expiresIn = try container.decodeIfPresent(Int.self, forKey: .expiresIn) ?? 900
        self.tokenType = try container.decodeIfPresent(String.self, forKey: .tokenType) ?? "Bearer"
        self.user = try container.decode(User.self, forKey: .user)
    }
}

public struct User: Identifiable, Codable, Sendable, Hashable {
    public let id: String
    public var username: String
    public var email: String
    public var displayName: String
    public var bio: String?
    public var avatarUrl: String?
    public var headerImageUrl: String?
    public var location: String?
    public var website: String?
    public var isPrivate: Bool
    public var is2faEnabled: Bool
    public var createdAt: String
    public var postsCount: Int
    public var followersCount: Int
    public var followingCount: Int
    public var isMe: Bool
    public var isFollowedByMe: Bool
    public var isBlockingMe: Bool
    public var isBlockedByMe: Bool
    public var isMutedByMe: Bool
    public var hasPendingFollowRequest: Bool
    public var hasActiveStories: Bool

    public init(
        id: String,
        username: String,
        email: String = "",
        displayName: String? = nil,
        bio: String? = nil,
        avatarUrl: String? = nil,
        headerImageUrl: String? = nil,
        location: String? = nil,
        website: String? = nil,
        isPrivate: Bool = false,
        is2faEnabled: Bool = false,
        createdAt: String = "",
        postsCount: Int = 0,
        followersCount: Int = 0,
        followingCount: Int = 0,
        isMe: Bool = false,
        isFollowedByMe: Bool = false,
        isBlockingMe: Bool = false,
        isBlockedByMe: Bool = false,
        isMutedByMe: Bool = false,
        hasPendingFollowRequest: Bool = false,
        hasActiveStories: Bool = false
    ) {
        self.id = id
        self.username = username
        self.email = email
        self.displayName = displayName ?? username
        self.bio = bio
        self.avatarUrl = avatarUrl
        self.headerImageUrl = headerImageUrl
        self.location = location
        self.website = website
        self.isPrivate = isPrivate
        self.is2faEnabled = is2faEnabled
        self.createdAt = createdAt
        self.postsCount = postsCount
        self.followersCount = followersCount
        self.followingCount = followingCount
        self.isMe = isMe
        self.isFollowedByMe = isFollowedByMe
        self.isBlockingMe = isBlockingMe
        self.isBlockedByMe = isBlockedByMe
        self.isMutedByMe = isMutedByMe
        self.hasPendingFollowRequest = hasPendingFollowRequest
        self.hasActiveStories = hasActiveStories
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let idVal = try container.decode(String.self, forKey: .id)
        let usernameVal = try container.decode(String.self, forKey: .username)

        self.id = idVal
        self.username = usernameVal
        self.email = try container.decodeIfPresent(String.self, forKey: .email) ?? ""
        self.displayName = try container.decodeIfPresent(String.self, forKey: .displayName) ?? usernameVal
        self.bio = try container.decodeIfPresent(String.self, forKey: .bio)
        self.avatarUrl = try container.decodeIfPresent(String.self, forKey: .avatarUrl)
        self.headerImageUrl = try container.decodeIfPresent(String.self, forKey: .headerImageUrl)
        self.location = try container.decodeIfPresent(String.self, forKey: .location)
        self.website = try container.decodeIfPresent(String.self, forKey: .website)
        self.isPrivate = try container.decodeIfPresent(Bool.self, forKey: .isPrivate) ?? false
        self.is2faEnabled = try container.decodeIfPresent(Bool.self, forKey: .is2faEnabled) ?? false
        self.createdAt = try container.decodeIfPresent(String.self, forKey: .createdAt) ?? ""
        self.postsCount = try container.decodeIfPresent(Int.self, forKey: .postsCount) ?? 0
        self.followersCount = try container.decodeIfPresent(Int.self, forKey: .followersCount) ?? 0
        self.followingCount = try container.decodeIfPresent(Int.self, forKey: .followingCount) ?? 0
        self.isMe = try container.decodeIfPresent(Bool.self, forKey: .isMe) ?? false
        self.isFollowedByMe = try container.decodeIfPresent(Bool.self, forKey: .isFollowedByMe) ?? false
        self.isBlockingMe = try container.decodeIfPresent(Bool.self, forKey: .isBlockingMe) ?? false
        self.isBlockedByMe = try container.decodeIfPresent(Bool.self, forKey: .isBlockedByMe) ?? false
        self.isMutedByMe = try container.decodeIfPresent(Bool.self, forKey: .isMutedByMe) ?? false
        self.hasPendingFollowRequest = try container.decodeIfPresent(Bool.self, forKey: .hasPendingFollowRequest) ?? false
        self.hasActiveStories = try container.decodeIfPresent(Bool.self, forKey: .hasActiveStories) ?? false
    }
}
