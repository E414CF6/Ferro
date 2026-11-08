import Foundation
@testable import Ferro

public struct MockData: Sendable {
    public static let currentUser = User(
        id: "usr_me_001",
        username: "kernelily",
        email: "kernelily@ferro.social",
        displayName: "Kernel Lily",
        bio: "Exploring VisionOS & iOS interfaces with Swift. Building high-performance decentralized systems.",
        avatarUrl: "https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=400",
        headerImageUrl: "https://images.unsplash.com/photo-1579546929518-9e396f3cc809?w=1200",
        location: "Seoul & Cupertino",
        website: "https://github.com/kernelily",
        isPrivate: false,
        is2faEnabled: true,
        createdAt: "2026-01-10T08:00:00Z",
        postsCount: 42,
        followersCount: 1280,
        followingCount: 350,
        isMe: true,
        isFollowedByMe: false,
        hasActiveStories: true
    )

    public static let alice = User(
        id: "usr_002",
        username: "alice_design",
        email: "alice@ferro.social",
        displayName: "Alice Vance",
        bio: "Product & Spatial Designer @ Ferro Labs ✦ Liquid Glass fanatic",
        avatarUrl: "https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=400",
        headerImageUrl: "https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=1200",
        location: "San Francisco, CA",
        website: "https://alicevance.design",
        isPrivate: false,
        is2faEnabled: false,
        createdAt: "2026-01-15T12:00:00Z",
        postsCount: 18,
        followersCount: 4520,
        followingCount: 210,
        isMe: false,
        isFollowedByMe: true,
        hasActiveStories: true
    )

    public static let bob = User(
        id: "usr_003",
        username: "bob_engineer",
        email: "bob@ferro.social",
        displayName: "Bob Chen",
        bio: "Distributed systems, Rust & Swift. Serve API contributor 🦀",
        avatarUrl: "https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=400",
        headerImageUrl: "https://images.unsplash.com/photo-1550745165-9bc0b252726f?w=1200",
        location: "Seattle, WA",
        website: "https://bobchen.dev",
        isPrivate: false,
        is2faEnabled: true,
        createdAt: "2026-02-01T09:30:00Z",
        postsCount: 89,
        followersCount: 2310,
        followingCount: 412,
        isMe: false,
        isFollowedByMe: true,
        hasActiveStories: true
    )

    public static let stories: [Story] = [
        Story(
            id: "story_01",
            mediaUrl: "https://images.unsplash.com/photo-1506744038136-46273834b3fb?w=800",
            caption: "Sunrise at Yosemite 🏔️ Beautiful morning to test new spatial features!",
            createdAt: "2 hours ago",
            expiresAt: "22 hours left",
            isExpired: false,
            author: alice,
            viewsCount: 312,
            isViewedByMe: false
        ),
        Story(
            id: "story_02",
            mediaUrl: "https://images.unsplash.com/photo-1518770660439-4636190af475?w=800",
            caption: "New server cluster deployed with Redis sliding window rate limits 🚀",
            createdAt: "4 hours ago",
            expiresAt: "20 hours left",
            isExpired: false,
            author: bob,
            viewsCount: 540,
            isViewedByMe: true
        ),
        Story(
            id: "story_03",
            mediaUrl: "https://images.unsplash.com/photo-1534447677768-be436bb09401?w=800",
            caption: "Working on Swift 6 strict concurrency migration ⚡️",
            createdAt: "7 hours ago",
            expiresAt: "17 hours left",
            isExpired: false,
            author: currentUser,
            viewsCount: 215,
            isViewedByMe: true
        )
    ]

    public static let sampleComments: [Comment] = [
        Comment(
            id: "cmt_01",
            parentId: nil,
            content: "The performance is unbelievable! Love the dual token architecture and RTR.",
            isEdited: false,
            createdAt: "15m ago",
            updatedAt: "15m ago",
            author: alice,
            depth: 0,
            replies: [
                Comment(
                    id: "cmt_02",
                    parentId: "cmt_01",
                    content: "Thanks Alice! We also added EXIF stripping for JPEG media uploads.",
                    isEdited: false,
                    createdAt: "10m ago",
                    updatedAt: "10m ago",
                    author: bob,
                    depth: 1,
                    replies: [],
                    repliesCount: 0,
                    likesCount: 5,
                    isLikedByMe: true
                )
            ],
            repliesCount: 1,
            likesCount: 14,
            isLikedByMe: true
        ),
        Comment(
            id: "cmt_03",
            parentId: nil,
            content: "Can't wait to test the Geospatial wMap integration on VisionOS!",
            isEdited: false,
            createdAt: "5m ago",
            updatedAt: "5m ago",
            author: currentUser,
            depth: 0,
            replies: [],
            repliesCount: 0,
            likesCount: 8,
            isLikedByMe: false
        )
    ]

    public static let posts: [Post] = [
        Post(
            id: "post_01",
            content: "Welcome to Ferro! We just shipped Serve API v1.0.0 with dual token JWT + RTR, GraphQL HTTP + WebSocket subscriptions, and SRE Prometheus metrics. Check out our high-throughput architecture. #Ferro #ServeAPI #SwiftUI",
            audience: .public,
            viewsCount: 1420,
            quotePostId: nil,
            createdAt: "25m ago",
            author: bob,
            media: [
                PostMedia(
                    id: "med_01",
                    url: "https://images.unsplash.com/photo-1558494949-ef010cbdcc31?w=1000",
                    mediaType: .image,
                    width: 1200,
                    height: 800,
                    aspectRatio: 1.5
                )
            ],
            poll: nil,
            analytics: PostAnalytics(impressions: 1420, likesCount: 132, repostsCount: 45, repliesCount: 12, bookmarksCount: 28),
            likesCount: 132,
            repostsCount: 45,
            commentsCount: 12,
            isLiked: true,
            isRepostedByMe: false,
            isSavedByMe: true,
            hashtags: ["Ferro", "ServeAPI", "SwiftUI"],
            comments: sampleComments
        ),
        Post(
            id: "post_02",
            content: "Quick poll for developers: Which backend protocol do you prefer for real-time mobile subscriptions?",
            audience: .public,
            viewsCount: 890,
            quotePostId: nil,
            createdAt: "1h ago",
            author: alice,
            media: [],
            poll: Poll(
                id: "poll_01",
                question: "Real-time subscription preference:",
                options: [
                    PollOption(id: "opt_1", text: "GraphQL WebSocket (graphql-transport-ws)", votesCount: 142, isVotedByMe: true),
                    PollOption(id: "opt_2", text: "gRPC Streaming", votesCount: 65, isVotedByMe: false),
                    PollOption(id: "opt_3", text: "Server-Sent Events (SSE)", votesCount: 48, isVotedByMe: false)
                ],
                totalVotes: 255,
                hasVoted: true,
                isClosed: false,
                expiresAt: "2 days left"
            ),
            analytics: PostAnalytics(impressions: 890, likesCount: 67, repostsCount: 18, repliesCount: 8, bookmarksCount: 15),
            likesCount: 67,
            repostsCount: 18,
            commentsCount: 8,
            isLiked: false,
            isRepostedByMe: true,
            isSavedByMe: false,
            hashtags: ["GraphQL", "WebSockets"],
            comments: []
        ),
        Post(
            id: "post_03",
            content: "Spatial UI designs on Vision are coming together! Glassmorphic cards with responsive depth look phenomenal. 🥽✨",
            audience: .closeFriends,
            viewsCount: 430,
            quotePostId: nil,
            createdAt: "3h ago",
            author: currentUser,
            media: [
                PostMedia(
                    id: "med_02",
                    url: "https://images.unsplash.com/photo-1593508512255-86ab42a8e620?w=1000",
                    mediaType: .image,
                    width: 1000,
                    height: 600,
                    aspectRatio: 1.66
                )
            ],
            poll: nil,
            analytics: PostAnalytics(impressions: 430, likesCount: 94, repostsCount: 12, repliesCount: 6, bookmarksCount: 19),
            likesCount: 94,
            repostsCount: 12,
            commentsCount: 6,
            isLiked: true,
            isRepostedByMe: false,
            isSavedByMe: false,
            hashtags: ["VisionOS", "LiquidGlass"],
            comments: []
        )
    ]

    public static let conversations: [Conversation] = [
        Conversation(
            otherUser: alice,
            lastMessage: DirectMessage(
                id: "dm_01",
                senderId: alice.id,
                recipientId: currentUser.id,
                content: "Hey Lily! Did you see the new wMap geospatial article draft?",
                isRead: false,
                createdAt: "12m ago"
            ),
            unreadCount: 1
        ),
        Conversation(
            otherUser: bob,
            lastMessage: DirectMessage(
                id: "dm_02",
                senderId: currentUser.id,
                recipientId: bob.id,
                content: "The multipart upload for images up to 50MB works seamlessly now.",
                isRead: true,
                createdAt: "2h ago"
            ),
            unreadCount: 0
        )
    ]

    public static let chatMessagesWithAlice: [DirectMessage] = [
        DirectMessage(
            id: "dm_a1",
            senderId: alice.id,
            recipientId: currentUser.id,
            content: "Hi Lily! How is the iOS client integration going?",
            isRead: true,
            createdAt: "10:30 AM"
        ),
        DirectMessage(
            id: "dm_a2",
            senderId: currentUser.id,
            recipientId: alice.id,
            content: "It's going great! Dual token JWT auth, GraphQL queries, and WebSocket subscriptions are set up.",
            isRead: true,
            createdAt: "10:32 AM"
        ),
        DirectMessage(
            id: "dm_a3",
            senderId: alice.id,
            recipientId: currentUser.id,
            content: "Hey Lily! Did you see the new wMap geospatial article draft?",
            isRead: false,
            createdAt: "10:45 AM"
        )
    ]

    public static let notifications: [NotificationItem] = [
        NotificationItem(
            id: "notif_01",
            notificationType: .likePost,
            isRead: false,
            createdAt: "10m ago",
            sender: alice,
            targetPost: posts[0]
        ),
        NotificationItem(
            id: "notif_02",
            notificationType: .commentPost,
            isRead: false,
            createdAt: "25m ago",
            sender: bob,
            targetPost: posts[0]
        ),
        NotificationItem(
            id: "notif_03",
            notificationType: .follow,
            isRead: true,
            createdAt: "1h ago",
            sender: alice
        ),
        NotificationItem(
            id: "notif_04",
            notificationType: .repost,
            isRead: true,
            createdAt: "3h ago",
            sender: bob,
            targetPost: posts[1]
        )
    ]

    public static let trendingHashtags: [HashtagTrend] = [
        HashtagTrend(hashtag: "Ferro", postsCount: 14820, rankChange: "UP"),
        HashtagTrend(hashtag: "SwiftUI", postsCount: 9230, rankChange: "UP"),
        HashtagTrend(hashtag: "VisionOS", postsCount: 6510, rankChange: "NEW"),
        HashtagTrend(hashtag: "GraphQL", postsCount: 4890, rankChange: "SAME"),
        HashtagTrend(hashtag: "OpenSource", postsCount: 3120, rankChange: "DOWN")
    ]

    public static let wikiArticles: [WikiArticle] = [
        WikiArticle(
            id: "wiki_01",
            title: "Ferro Spatial Hub",
            slug: "ferro-spatial-hub",
            summary: "A central landmark in the spatial metaverse connecting collaborative workspaces.",
            content: "Ferro Spatial Hub is the epicenter of next-generation collaborative computing. Built atop high-precision geospatial coordinates, visitors can interact with spatial widgets, shared whiteboards, and real-time audio rooms.",
            latitude: 37.7749,
            longitude: -122.4194,
            zoom: 14.0,
            category: "Metaverse Architecture",
            tags: ["spatial", "ferro", "hub", "wMap"],
            author: "alice_design",
            views: 3410,
            createdAt: "2026-02-10T10:00:00Z",
            updatedAt: "2026-03-01T15:20:00Z"
        ),
        WikiArticle(
            id: "wiki_02",
            title: "Cupertino Infinite Loop Archives",
            slug: "infinite-loop-archives",
            summary: "Historical landmark and interactive 3D model archive.",
            content: "An extensive digital archive documenting Apple's Infinite Loop campus history, hardware innovations, and design philosophies.",
            latitude: 37.3318,
            longitude: -122.0311,
            zoom: 15.0,
            category: "Tech History",
            tags: ["apple", "cupertino", "history"],
            author: "kernelily",
            views: 1890,
            createdAt: "2026-01-20T14:00:00Z",
            updatedAt: "2026-02-18T11:00:00Z"
        )
    ]
}
