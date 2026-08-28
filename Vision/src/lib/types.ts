export interface User {
    id: string;
    username: string;
    email: string;
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
    comments?: Comment[];
    followers?: User[];
    following?: User[];
    activeStories?: Story[];
    stories?: Story[];
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
        id?: string; username: string; displayName: string; avatarUrl?: string | null;
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
    createdAt: string;
    updatedAt?: string;
    author: {
        id: string; username: string; displayName: string; avatarUrl?: string | null;
    };
}

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
    audience?: "PUBLIC" | "FOLLOWERS_ONLY" | "CLOSE_FRIENDS";
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
    isBookmarkedByMe?: boolean;
    comments: Comment[];
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
export type ReportReason = | "SPAM" | "HARASSMENT" | "HATE_SPEECH" | "INAPPROPRIATE" | "COPYRIGHT" | "OTHER";
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
    | "POLL_ENDED";

export interface Notification {
    id: string;
    recipientId: string;
    actorId: string;
    actor?: User;
    notificationType: NotificationType;
    entityId?: string | null;
    isRead: boolean;
    createdAt: string;
    post?: Post | null;
    comment?: Comment | null;
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
    content: string;
    createdAt: string;
    isRead: boolean;
    isMine?: boolean;
    sender: User;
    recipient: User;
}

export interface Conversation {
    otherUser: User;
    lastMessage: DirectMessage;
    unreadCount: number;
}
