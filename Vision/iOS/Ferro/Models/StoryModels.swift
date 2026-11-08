import Foundation

public struct Story: Identifiable, Codable, Sendable, Hashable {
    public let id: String
    public let mediaUrl: String
    public var caption: String?
    public let createdAt: String
    public let expiresAt: String
    public var isExpired: Bool
    public let author: User
    public var viewsCount: Int
    public var isViewedByMe: Bool
    public var viewers: [User]

    public init(
        id: String,
        mediaUrl: String,
        caption: String? = nil,
        createdAt: String = "",
        expiresAt: String = "",
        isExpired: Bool = false,
        author: User,
        viewsCount: Int = 0,
        isViewedByMe: Bool = false,
        viewers: [User] = []
    ) {
        self.id = id
        self.mediaUrl = mediaUrl
        self.caption = caption
        self.createdAt = createdAt
        self.expiresAt = expiresAt
        self.isExpired = isExpired
        self.author = author
        self.viewsCount = viewsCount
        self.isViewedByMe = isViewedByMe
        self.viewers = viewers
    }
}
