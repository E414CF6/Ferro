import Foundation

public struct DirectMessage: Identifiable, Codable, Sendable, Hashable {
    public let id: String
    public var conversationId: String?
    public let senderId: String
    public let recipientId: String
    public var content: String
    public var isRead: Bool
    public let createdAt: String
    public var sender: User?
    public var recipient: User?

    enum CodingKeys: String, CodingKey {
        case id
        case conversationId
        case senderId
        case recipientId
        case content
        case isRead
        case createdAt
        case sender
        case recipient
    }

    public init(
        id: String,
        conversationId: String? = nil,
        senderId: String,
        recipientId: String,
        content: String,
        isRead: Bool = false,
        createdAt: String = "",
        sender: User? = nil,
        recipient: User? = nil
    ) {
        self.id = id
        self.conversationId = conversationId
        self.senderId = senderId
        self.recipientId = recipientId
        self.content = content
        self.isRead = isRead
        self.createdAt = createdAt
        self.sender = sender
        self.recipient = recipient
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.id = try container.decode(String.self, forKey: .id)
        self.conversationId = try container.decodeIfPresent(String.self, forKey: .conversationId)
        self.content = try container.decode(String.self, forKey: .content)
        self.isRead = try container.decodeIfPresent(Bool.self, forKey: .isRead) ?? false
        self.createdAt = try container.decodeIfPresent(String.self, forKey: .createdAt) ?? ""

        let decodedSender = try container.decodeIfPresent(User.self, forKey: .sender)
        let decodedRecipient = try container.decodeIfPresent(User.self, forKey: .recipient)
        self.sender = decodedSender
        self.recipient = decodedRecipient

        if let sId = try container.decodeIfPresent(String.self, forKey: .senderId) {
            self.senderId = sId
        } else {
            self.senderId = decodedSender?.id ?? ""
        }

        if let rId = try container.decodeIfPresent(String.self, forKey: .recipientId) {
            self.recipientId = rId
        } else {
            self.recipientId = decodedRecipient?.id ?? ""
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(id, forKey: .id)
        try container.encodeIfPresent(conversationId, forKey: .conversationId)
        try container.encode(senderId, forKey: .senderId)
        try container.encode(recipientId, forKey: .recipientId)
        try container.encode(content, forKey: .content)
        try container.encode(isRead, forKey: .isRead)
        try container.encode(createdAt, forKey: .createdAt)
        try container.encodeIfPresent(sender, forKey: .sender)
        try container.encodeIfPresent(recipient, forKey: .recipient)
    }
}

public struct Conversation: Identifiable, Codable, Sendable, Hashable {
    public var id: String { otherUser.id }
    public var otherUser: User
    public var lastMessage: DirectMessage
    public var unreadCount: Int

    public init(otherUser: User, lastMessage: DirectMessage, unreadCount: Int = 0) {
        self.otherUser = otherUser
        self.lastMessage = lastMessage
        self.unreadCount = unreadCount
    }
}

public struct GroupConversation: Identifiable, Codable, Sendable, Hashable {
    public let id: String
    public var title: String?
    public let creatorId: String
    public var creator: User
    public var members: [User]
    public var messages: [DirectMessage]
    public let createdAt: String

    public init(
        id: String,
        title: String? = nil,
        creatorId: String,
        creator: User,
        members: [User] = [],
        messages: [DirectMessage] = [],
        createdAt: String = ""
    ) {
        self.id = id
        self.title = title
        self.creatorId = creatorId
        self.creator = creator
        self.members = members
        self.messages = messages
        self.createdAt = createdAt
    }
}

public struct TypingIndicator: Codable, Sendable {
    public let userId: String
    public let conversationId: String?
    public let recipientId: String?
    public let isTyping: Bool

    public init(userId: String, conversationId: String? = nil, recipientId: String? = nil, isTyping: Bool) {
        self.userId = userId
        self.conversationId = conversationId
        self.recipientId = recipientId
        self.isTyping = isTyping
    }
}
