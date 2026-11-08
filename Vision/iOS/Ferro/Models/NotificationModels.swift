import Foundation

public enum NotificationTypeGql: String, Codable, Sendable, CaseIterable {
    case follow = "FOLLOW"
    case followRequest = "FOLLOW_REQUEST"
    case followAccepted = "FOLLOW_ACCEPTED"
    case likePost = "LIKE_POST"
    case likeComment = "LIKE_COMMENT"
    case commentPost = "COMMENT_POST"
    case replyComment = "REPLY_COMMENT"
    case repost = "REPOST"
    case quote = "QUOTE"
    case mention = "MENTION"
    case directMessage = "DIRECT_MESSAGE"

    public var systemImage: String {
        switch self {
        case .follow, .followAccepted:
            return "person.badge.plus"
        case .followRequest:
            return "person.crop.circle.badge.questionmark"
        case .likePost, .likeComment:
            return "heart.fill"
        case .commentPost, .replyComment:
            return "bubble.right.fill"
        case .repost:
            return "arrow.rectanglepath"
        case .quote:
            return "quote.bubble.fill"
        case .mention:
            return "at"
        case .directMessage:
            return "envelope.fill"
        }
    }

    public var actionDescription: String {
        switch self {
        case .follow:
            return "started following you"
        case .followRequest:
            return "requested to follow you"
        case .followAccepted:
            return "accepted your follow request"
        case .likePost:
            return "liked your post"
        case .likeComment:
            return "liked your comment"
        case .commentPost:
            return "commented on your post"
        case .replyComment:
            return "replied to your comment"
        case .repost:
            return "reposted your post"
        case .quote:
            return "quoted your post"
        case .mention:
            return "mentioned you in a post"
        case .directMessage:
            return "sent you a message"
        }
    }
}

public struct NotificationItem: Identifiable, Codable, Sendable, Hashable {
    public let id: String
    public let notificationType: NotificationTypeGql
    public var isRead: Bool
    public let createdAt: String
    public let sender: User
    public var targetPost: Post?
    public var targetComment: Comment?

    enum CodingKeys: String, CodingKey {
        case id
        case notificationType
        case type
        case isRead
        case createdAt
        case actor
        case sender
        case post
        case targetPost
        case comment
        case targetComment
    }

    public init(
        id: String,
        notificationType: NotificationTypeGql,
        isRead: Bool = false,
        createdAt: String = "",
        sender: User,
        targetPost: Post? = nil,
        targetComment: Comment? = nil
    ) {
        self.id = id
        self.notificationType = notificationType
        self.isRead = isRead
        self.createdAt = createdAt
        self.sender = sender
        self.targetPost = targetPost
        self.targetComment = targetComment
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.id = try container.decode(String.self, forKey: .id)

        if let nType = try container.decodeIfPresent(NotificationTypeGql.self, forKey: .notificationType) {
            self.notificationType = nType
        } else if let t = try container.decodeIfPresent(NotificationTypeGql.self, forKey: .type) {
            self.notificationType = t
        } else {
            self.notificationType = .likePost
        }

        self.isRead = try container.decodeIfPresent(Bool.self, forKey: .isRead) ?? false
        self.createdAt = try container.decodeIfPresent(String.self, forKey: .createdAt) ?? ""

        if let act = try container.decodeIfPresent(User.self, forKey: .actor) {
            self.sender = act
        } else if let snd = try container.decodeIfPresent(User.self, forKey: .sender) {
            self.sender = snd
        } else {
            self.sender = User(id: "unknown", username: "unknown", displayName: "Unknown")
        }

        if let p = try container.decodeIfPresent(Post.self, forKey: .post) {
            self.targetPost = p
        } else {
            self.targetPost = try container.decodeIfPresent(Post.self, forKey: .targetPost)
        }

        if let c = try container.decodeIfPresent(Comment.self, forKey: .comment) {
            self.targetComment = c
        } else {
            self.targetComment = try container.decodeIfPresent(Comment.self, forKey: .targetComment)
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(id, forKey: .id)
        try container.encode(notificationType, forKey: .notificationType)
        try container.encode(isRead, forKey: .isRead)
        try container.encode(createdAt, forKey: .createdAt)
        try container.encode(sender, forKey: .sender)
        try container.encodeIfPresent(targetPost, forKey: .targetPost)
        try container.encodeIfPresent(targetComment, forKey: .targetComment)
    }
}
