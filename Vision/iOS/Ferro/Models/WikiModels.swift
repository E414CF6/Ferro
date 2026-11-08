import Foundation

public struct WikiRevision: Identifiable, Codable, Sendable, Hashable {
    public let id: String
    public let articleId: String
    public let title: String
    public let content: String
    public let latitude: Double
    public let longitude: Double
    public let editSummary: String?
    public let author: String
    public let createdAt: String

    public init(
        id: String,
        articleId: String,
        title: String,
        content: String,
        latitude: Double,
        longitude: Double,
        editSummary: String? = nil,
        author: String,
        createdAt: String = ""
    ) {
        self.id = id
        self.articleId = articleId
        self.title = title
        self.content = content
        self.latitude = latitude
        self.longitude = longitude
        self.editSummary = editSummary
        self.author = author
        self.createdAt = createdAt
    }
}

public struct WikiArticle: Identifiable, Codable, Sendable, Hashable {
    public let id: String
    public var userId: String?
    public var title: String
    public var slug: String
    public var summary: String?
    public var content: String
    public var latitude: Double
    public var longitude: Double
    public var zoom: Double
    public var category: String
    public var tags: [String]
    public var geojson: String?
    public var author: String
    public var views: Int
    public var createdAt: String
    public var updatedAt: String
    public var revisions: [WikiRevision]

    public init(
        id: String,
        userId: String? = nil,
        title: String,
        slug: String,
        summary: String? = nil,
        content: String,
        latitude: Double,
        longitude: Double,
        zoom: Double = 12.0,
        category: String = "General",
        tags: [String] = [],
        geojson: String? = nil,
        author: String,
        views: Int = 0,
        createdAt: String = "",
        updatedAt: String = "",
        revisions: [WikiRevision] = []
    ) {
        self.id = id
        self.userId = userId
        self.title = title
        self.slug = slug
        self.summary = summary
        self.content = content
        self.latitude = latitude
        self.longitude = longitude
        self.zoom = zoom
        self.category = category
        self.tags = tags
        self.geojson = geojson
        self.author = author
        self.views = views
        self.createdAt = createdAt
        self.updatedAt = updatedAt
        self.revisions = revisions
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.id = try container.decode(String.self, forKey: .id)
        self.userId = try container.decodeIfPresent(String.self, forKey: .userId)
        self.title = try container.decode(String.self, forKey: .title)
        self.slug = try container.decode(String.self, forKey: .slug)
        self.summary = try container.decodeIfPresent(String.self, forKey: .summary)
        self.content = try container.decode(String.self, forKey: .content)
        self.latitude = try container.decode(Double.self, forKey: .latitude)
        self.longitude = try container.decode(Double.self, forKey: .longitude)
        self.zoom = try container.decodeIfPresent(Double.self, forKey: .zoom) ?? 12.0
        self.category = try container.decodeIfPresent(String.self, forKey: .category) ?? "General"
        self.tags = try container.decodeIfPresent([String].self, forKey: .tags) ?? []
        self.geojson = try container.decodeIfPresent(String.self, forKey: .geojson)
        self.author = try container.decodeIfPresent(String.self, forKey: .author) ?? "Anonymous"
        self.views = try container.decodeIfPresent(Int.self, forKey: .views) ?? 0
        self.createdAt = try container.decodeIfPresent(String.self, forKey: .createdAt) ?? ""
        self.updatedAt = try container.decodeIfPresent(String.self, forKey: .updatedAt) ?? ""
        self.revisions = try container.decodeIfPresent([WikiRevision].self, forKey: .revisions) ?? []
    }
}

public struct HashtagTrend: Identifiable, Codable, Sendable, Hashable {
    public var id: String { hashtag }
    public let hashtag: String
    public let postsCount: Int
    public let rankChange: String

    public init(hashtag: String, postsCount: Int, rankChange: String = "SAME") {
        self.hashtag = hashtag
        self.postsCount = postsCount
        self.rankChange = rankChange
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.hashtag = try container.decode(String.self, forKey: .hashtag)
        self.postsCount = try container.decode(Int.self, forKey: .postsCount)
        self.rankChange = try container.decodeIfPresent(String.self, forKey: .rankChange) ?? "SAME"
    }
}
