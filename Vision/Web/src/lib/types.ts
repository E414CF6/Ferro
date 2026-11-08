export interface PageInfo {
    hasNextPage: boolean;
    hasPreviousPage: boolean;
    startCursor?: string | null;
    endCursor?: string | null;
}

export interface Edge<T> {
    cursor: string;
    node: T;
}

export interface Connection<T> {
    edges: Edge<T>[];
    pageInfo: PageInfo;
    totalCount?: number | null;
}

export interface User {
    id: string;
    username: string;
    email?: string;
    displayName: string;
    bio?: string | null;
    avatarUrl?: string | null;
    headerImageUrl?: string | null;
    location?: string | null;
    website?: string | null;
    isPrivate?: boolean;
    is2faEnabled?: boolean;
    createdAt: string;
    postsCount?: number;
    hasActiveStories?: boolean;
    followersCount?: number;
    followingCount?: number;
    isMe?: boolean;
    isFollowedByMe?: boolean;
    isBlockedByMe?: boolean;
    isMutedByMe?: boolean;
    hasPendingFollowRequest?: boolean;
    posts?: Post[];
    likedPosts?: Post[];
    savedPosts?: Post[];
    comments?: Comment[];
    followers?: User[];
    following?: User[];
    activeStories?: Story[];
    stories?: Story[];
    collections?: BookmarkCollection[];
    lists?: UserList[];
}

export type MediaType = "IMAGE" | "VIDEO" | "GIF";

export interface PostMedia {
    id: string;
    postId?: string;
    mediaUrl: string;
    mediaType: MediaType;
    altText?: string | null;
    sortOrder: number;
    width?: number | null;
    height?: number | null;
}

export interface PollOption {
    id: string;
    optionText: string;
    sortOrder: number;
    votesCount: number;
    percentage: number;
    isVotedByMe?: boolean;
}

export interface Poll {
    id: string;
    postId?: string;
    question: string;
    expiresAt: string;
    isExpired: boolean;
    totalVotes: number;
    userVotedOptionId?: string | null;
    options: PollOption[];
}

export interface PostAnalytics {
    postId?: string;
    viewsCount: number;
    likesCount: number;
    repostsCount: number;
    commentsCount: number;
    engagementRate: number;
}

export interface Story {
    id: string;
    mediaUrl: string;
    caption?: string | null;
    createdAt: string;
    expiresAt: string;
    isExpired: boolean;
    author: {
        id?: string;
        username: string;
        displayName: string;
        avatarUrl?: string | null;
    };
    viewsCount: number;
    isViewedByMe?: boolean;
    viewers?: User[];
}

export interface Comment {
    id: string;
    postId?: string;
    parentId?: string | null;
    content: string;
    isEdited?: boolean;
    depth?: number;
    likesCount?: number;
    isLikedByMe?: boolean;
    repliesCount?: number;
    replies?: Comment[];
    createdAt: string;
    updatedAt?: string;
    author: {
        id: string;
        username: string;
        displayName: string;
        avatarUrl?: string | null;
    };
}

export type PostAudience = "PUBLIC" | "FOLLOWERS_ONLY" | "CLOSE_FRIENDS";

export interface Post {
    id: string;
    content: string;
    imageUrl?: string | null;
    media?: PostMedia[];
    poll?: Poll | null;
    quotePostId?: string | null;
    quotePost?: Post | null;
    pinnedCommentId?: string | null;
    pinnedComment?: Comment | null;
    audience?: PostAudience;
    viewsCount?: number;
    createdAt: string;
    author: {
        id: string;
        username: string;
        displayName: string;
        avatarUrl?: string | null;
        hasActiveStories?: boolean;
        isPrivate?: boolean;
    };
    likesCount: number;
    repostsCount?: number;
    isLikedBy?: boolean;
    isLikedByMe?: boolean;
    isRepostedByMe?: boolean;
    isSavedByMe?: boolean;
    isBookmarkedByMe?: boolean;
    hashtags?: string[];
    comments: Comment[];
}

export interface HashtagTrend {
    hashtag: string;
    postsCount: number;
}

export interface UserList {
    id: string;
    name: string;
    description?: string | null;
    isPrivate: boolean;
    owner: User;
    membersCount: number;
    members?: User[];
}

export interface BookmarkCollection {
    id: string;
    name: string;
    description?: string | null;
    isPrivate: boolean;
    user: User;
    postsCount?: number;
}

export type ReportTargetType = "POST" | "COMMENT" | "USER";
export type ReportReason = "SPAM" | "HARASSMENT" | "HATE_SPEECH" | "INAPPROPRIATE" | "COPYRIGHT" | "OTHER";
export type ReportStatus = "PENDING" | "RESOLVED" | "DISMISSED";

export interface Report {
    id: string;
    reporterId: string;
    targetType: ReportTargetType;
    targetId: string;
    reason: ReportReason;
    details?: string | null;
    status: ReportStatus;
    createdAt: string;
    reporter?: User;
}

export type NotificationType =
    | "LIKE_POST"
    | "COMMENT_POST"
    | "FOLLOW"
    | "REPLY_COMMENT"
    | "REPOST"
    | "QUOTE"
    | "MENTION"
    | "FOLLOW_REQUEST"
    | "FOLLOW_ACCEPTED"
    | "LIKE_COMMENT"
    | "POLL_ENDED"
    | "DIRECT_MESSAGE";

export interface Notification {
    id: string;
    recipientId?: string;
    senderId?: string;
    actorId?: string;
    actor?: User;
    sender?: User;
    notificationType: NotificationType;
    targetId?: string | null;
    entityId?: string | null;
    isRead: boolean;
    createdAt: string;
    post?: Post | null;
    comment?: Comment | null;
    targetPost?: Post | null;
    targetComment?: Comment | null;
}

export interface FollowRequest {
    id: string;
    requesterId: string;
    targetId: string;
    status: "PENDING" | "ACCEPTED" | "REJECTED";
    createdAt: string;
    requester: User;
}

export interface TotpSetup {
    secret: string;
    otpauthUri: string;
}

export interface AuthPayload {
    token: string;
    user: User;
}

export interface DirectMessage {
    id: string;
    conversationId?: string | null;
    senderId?: string;
    recipientId?: string;
    content: string;
    createdAt: string;
    isRead: boolean;
    isMine?: boolean;
    sender: User;
    recipient?: User;
}

export interface Conversation {
    otherUser: User;
    lastMessage: DirectMessage;
    unreadCount: number;
}

export interface GroupConversation {
    id: string;
    title?: string | null;
    creatorId: string;
    creator?: User;
    members: User[];
    messages?: DirectMessage[];
    createdAt: string;
}

export interface TypingEvent {
    userId: string;
    conversationId?: string | null;
    recipientId?: string | null;
    isTyping: boolean;
}

/* ========================================================================= */
/* Wiki Map (wMap) Types                                                      */
/* ========================================================================= */

export type RankChangeType = "UP" | "DOWN" | "SAME" | "NEW";

export interface WikiArticle {
    id: string;
    userId?: string | null;
    title: string;
    slug: string;
    summary?: string | null;
    content: string;
    latitude: number;
    longitude: number;
    zoom: number;
    category: string;
    tags: string[];
    geojson?: string | null;
    author: string;
    views: number;
    createdAt: string;
    updatedAt: string;
    revisions?: WikiRevision[];
}

export interface WikiRevision {
    id: string;
    articleId: string;
    userId?: string | null;
    title: string;
    content: string;
    latitude: number;
    longitude: number;
    editSummary?: string | null;
    author: string;
    createdAt: string;
}

export interface TrendingArticleItem {
    rank: number;
    prevRank?: number | null;
    change: RankChangeType;
    changeAmount?: number | null;
    id: string;
    title: string;
    slug: string;
    summary?: string | null;
    latitude: number;
    longitude: number;
    zoom: number;
    category: string;
    tags: string[];
    views: number;
    recentViews: number;
    score: number;
    updatedAt: string;
}

export interface TrendingTagItem {
    rank: number;
    prevRank?: number | null;
    change: RankChangeType;
    changeAmount?: number | null;
    tag: string;
    count: number;
    score: number;
}

export interface TrendsData {
    articles: TrendingArticleItem[];
    tags: TrendingTagItem[];
    updatedAt: string;
    totalArticles: number;
}

export interface CreateArticleInput {
    title: string;
    content: string;
    summary?: string;
    latitude: number;
    longitude: number;
    zoom?: number;
    tags?: string[];
    author?: string;
    geojson?: string;
}

export interface UpdateArticleInput {
    title?: string;
    content?: string;
    summary?: string;
    latitude?: number;
    longitude?: number;
    zoom?: number;
    tags?: string[];
    author?: string;
    geojson?: string;
    editSummary?: string;
}

export interface ArticleFilterInput {
    q?: string;
    tag?: string;
    minLat?: number;
    maxLat?: number;
    minLng?: number;
    maxLng?: number;
    limit?: number;
}
