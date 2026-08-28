# Serve API Specification

Serve is a high-performance backend service for the **Ferro Social Platform**, providing both a **RESTful HTTP API** and
a strongly typed **GraphQL API (HTTP & WebSocket Subscription)**.

---

## 1. Overview & Configuration

* **Base URL**: `http://localhost:8080` (Default port: `8080`, configurable via `HOST` and `PORT` environment variables)
* **GraphQL Endpoint**: `POST /graphql`
* **GraphQL WebSocket Subscription**: `GET /graphql/ws`
* **GraphiQL Web IDE**: `GET /graphiql` or `GET /` (Active in development mode)
* **Authentication**: JWT Bearer Token via `Authorization: Bearer <JWT_TOKEN>` header
* **Security Constraints**:
* **GraphQL Maximum Query Depth**: `10`
* **GraphQL Maximum Query Complexity**: `250`
* **Sliding Window Rate Limiting (IP-based)**: `120 req / 60s` (default)
* **HTTP Request Body Limit**: Max `10 MB` (File uploads limited to `5 MB` per file)
* **Security Headers**: `X-Content-Type-Options: nosniff`, `X-Frame-Options: SAMEORIGIN`,
  `Referrer-Policy: strict-origin-when-cross-origin`, `X-Request-Id` propagation and tracing

---

## 2. REST Endpoints

### 2.1 Health & Metrics

#### `GET /health` / `GET /health/live`

* **Description**: Verifies service liveness.
* **Authentication**: Public
* **Response (200 OK)**:

```json
{
  "status": "healthy",
  "service": "serve",
  "version": "0.1.0"
}

```

#### `GET /health/ready`

* **Description**: Evaluates readiness and connection pool status for the underlying database (PostgreSQL/SQLite).
* **Authentication**: Public
* **Success Response (200 OK)**:

```json
{
  "status": "ready",
  "database": "connected",
  "driver": "postgres",
  "pool": {
    "size": 20,
    "idle": 18,
    "active": 2
  }
}

```

* **Failure Response (503 Service Unavailable)**:

```json
{
  "status": "unhealthy",
  "database": "disconnected",
  "error": "connection refused"
}

```

#### `GET /metrics`

* **Description**: Returns real-time service runtime and database connection pool metrics in Prometheus format.
* **Authentication**: Public
* **Response (200 OK, Content-Type: `text/plain; version=0.0.4`)**:

```text
# HELP ferro_serve_info Service build information
# TYPE ferro_serve_info gauge
ferro_serve_info{version="0.1.0"} 1

# HELP ferro_db_pool_connections_total Total database pool connections
# TYPE ferro_db_pool_connections_total gauge
ferro_db_pool_connections_total 20

# HELP ferro_db_pool_connections_idle Idle database connections
# TYPE ferro_db_pool_connections_idle gauge
ferro_db_pool_connections_idle 18

# HELP ferro_db_pool_connections_active Active database connections in use
# TYPE ferro_db_pool_connections_active gauge
ferro_db_pool_connections_active 2

```

---

### 2.2 Media Upload API

#### `POST /api/upload`

* **Description**: Uploads image and media files to persistent storage (local or S3-compatible).
* **Authentication**: **Required** (`Authorization: Bearer <JWT_TOKEN>`)
* **Content-Type**: `multipart/form-data`
* **Form Field**: `file` or `image`
* **Validation Rules**:
* Maximum file size: **5 MB**
* **Magic Byte Signature Verification**: Direct binary byte inspection to prevent file extension spoofing:
* JPEG (`FF D8 FF` -> `.jpg`, `image/jpeg`)
* PNG (`89 50 4E 47 0D 0A 1A 0A` -> `.png`, `image/png`)
* GIF (`GIF87a`, `GIF89a` -> `.gif`, `image/gif`)
* WEBP (`RIFF....WEBP` -> `.webp`, `image/webp`)
* SVG (`<svg` or `<?xml ... <svg` -> `.svg`, `image/svg+xml`)


* **Success Response (201 Created)**:

```json
{
  "url": "/uploads/a3b1c2d3-e4f5-4678-90ab-cdef12345678.png",
  "key": "a3b1c2d3-e4f5-4678-90ab-cdef12345678.png",
  "size": 1048576,
  "content_type": "image/png"
}

```

* **Error Responses**:
* `401 Unauthorized`: Token missing or invalid (`UNAUTHORIZED`)
* `400 Bad Request`: `EMPTY_FILE`, `MISSING_FILE`, `INVALID_FILE_SIGNATURE`, `READ_ERROR`
* `413 Payload Too Large`: Payload exceeds 5 MB (`FILE_TOO_LARGE`)
* `500 Internal Server Error`: Storage write failure (`STORAGE_ERROR`)

#### `GET /uploads/{filename}`

* **Description**: Serves static media assets stored in local disk storage.

---

## 3. GraphQL Schema & Types

### 3.1 Common Enums

| Enum Name                    | Allowed Values                                                                                                                                              | Description                                    |
|------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------|
| **`MediaTypeGql`**           | `IMAGE`, `VIDEO`, `GIF`                                                                                                                                     | Attachment media format                        |
| **`PostAudienceGql`**        | `PUBLIC`, `FOLLOWERS_ONLY`, `CLOSE_FRIENDS`                                                                                                                 | Post visibility scope                          |
| **`ReportTargetTypeGql`**    | `POST`, `COMMENT`, `USER`                                                                                                                                   | Target resource type for moderation reports    |
| **`ReportReasonGql`**        | `SPAM`, `HARASSMENT`, `HATE_SPEECH`, `INAPPROPRIATE`, `COPYRIGHT`, `OTHER`                                                                                  | Justification code for content reports         |
| **`ReportStatusGql`**        | `PENDING`, `RESOLVED`, `DISMISSED`                                                                                                                          | Resolution status of an issue report           |
| **`FollowRequestStatusGql`** | `PENDING`, `ACCEPTED`, `REJECTED`                                                                                                                           | Inbound follow request state (private account) |
| **`NotificationTypeGql`**    | `FOLLOW`, `FOLLOW_REQUEST`, `FOLLOW_ACCEPTED`, `LIKE_POST`, `LIKE_COMMENT`, `COMMENT_POST`, `REPLY_COMMENT`, `REPOST`, `QUOTE`, `MENTION`, `DIRECT_MESSAGE` | Real-time event notification category          |

---

### 3.2 Input Objects

#### `MediaInput`

```graphql
input MediaInput {
    mediaUrl: String!
    mediaType: MediaTypeGql # Default: IMAGE
    altText: String
    sortOrder: Int
    width: Int
    height: Int
}

```

#### `CreatePollInput`

```graphql
input CreatePollInput {
    question: String!
    options: [String!]!
    durationSeconds: Int # Default: 86400 (24 hours)
}

```

---

### 3.3 Core Object Types

#### `User`

```graphql
type User {
    id: ID!
    username: String!
    email: String!
    displayName: String!
    bio: String
    avatarUrl: String
    headerImageUrl: String
    location: String
    website: String
    isPrivate: Boolean!
    is2faEnabled: Boolean!
    createdAt: DateTime!
    postsCount: Int!          # Optimized with DataLoader batching
    posts: [Post!]!
    activeStories: [Story!]!
    hasActiveStories: Boolean! # Optimized with DataLoader batching
    likedPosts: [Post!]!
    savedPosts(limit: Int, offset: Int): [Post!]!
    collections: [BookmarkCollection!]!
    lists: [UserList!]!
    comments: [Comment!]!
    followers: [User!]!
    following: [User!]!
    followersCount: Int!      # Optimized with DataLoader batching
    followingCount: Int!      # Optimized with DataLoader batching
    isMe: Boolean!
    isFollowedByMe: Boolean!
    isBlockingMe: Boolean!
    isBlockedByMe: Boolean!
    isMutedByMe: Boolean!
    hasPendingFollowRequest: Boolean!
}

```

#### `Post`

```graphql
type Post {
    id: ID!
    content: String!
    audience: PostAudienceGql!
    viewsCount: Int!
    quotePostId: ID
    createdAt: DateTime!
    author: User!              # Optimized with DataLoader batching
    media: [PostMedia!]!       # Optimized with DataLoader batching
    poll: Poll                 # Optimized with DataLoader batching
    analytics: PostAnalytics!
    quotePost: Post            # Optimized with DataLoader batching
    pinnedComment: Comment     # Optimized with DataLoader batching
    likesCount: Int!           # Optimized with DataLoader batching
    repostsCount: Int!         # Optimized with DataLoader batching
    isLikedBy(userId: ID): Boolean!
    isRepostedByMe: Boolean!
    isSavedByMe: Boolean!
    hashtags: [String!]!
    comments(topLevelOnly: Boolean): [Comment!]!
    commentsConnection(topLevelOnly: Boolean, first: Int, after: String): CommentConnection!
}

```

#### `Story` (24-hour ephemeral content)

```graphql
type Story {
    id: ID!
    mediaUrl: String!
    caption: String
    createdAt: DateTime!
    expiresAt: DateTime!
    isExpired: Boolean!
    author: User!              # Optimized with DataLoader batching
    viewsCount: Int!
    isViewedByMe: Boolean!
    viewers: [User!]!          # Restricted to the story author only
}

```

#### `Comment` (Hierarchical comment threads)

```graphql
type Comment {
    id: ID!
    parentId: ID
    content: String!
    isEdited: Boolean!
    createdAt: DateTime!
    updatedAt: DateTime!
    author: User!              # Optimized with DataLoader batching
    post: Post!                # Optimized with DataLoader batching
    parent: Comment            # Optimized with DataLoader batching
    depth: Int!                # Comment tree depth (0: top-level root comment)
    rootComment: Comment       # Top-level ancestor of the thread
    replies: [Comment!]!
    repliesConnection(first: Int, after: String): CommentConnection!
    repliesCount: Int!         # Optimized with DataLoader batching
    likesCount: Int!           # Optimized with DataLoader batching
    isLikedByMe: Boolean!
}

```

#### `DirectMessage` & `Conversation`

```graphql
type DirectMessage {
    id: ID!
    conversationId: ID
    senderId: ID!
    recipientId: ID!
    content: String!
    isRead: Boolean!
    createdAt: DateTime!
    sender: User!
    recipient: User!
}

type Conversation {
    otherUser: User!
    lastMessage: DirectMessage!
    unreadCount: Int!
}

type GroupConversation {
    id: ID!
    title: String
    creatorId: ID!
    creator: User!
    members: [User!]!
    messages(limit: Int, offset: Int): [DirectMessage!]!
    createdAt: DateTime!
}

```

#### `Poll` & `PollOption`

```graphql
type Poll {
    id: ID!
    postId: ID!
    question: String!
    expiresAt: DateTime!
    isExpired: Boolean!
    totalVotes: Int!
    userVotedOptionId: ID
    options: [PollOption!]!    # Optimized with DataLoader batching
}

type PollOption {
    id: ID!
    optionText: String!
    sortOrder: Int!
    votesCount: Int!           # Optimized with DataLoader batching
    percentage: Float!         # Calculated share percentage (1 decimal place)
    isVotedByMe: Boolean!
}

```

#### `Notification`

```graphql
type Notification {
    id: ID!
    recipientId: ID!
    senderId: ID!
    notificationType: NotificationTypeGql!
    targetId: ID
    isRead: Boolean!
    createdAt: DateTime!
    sender: User!
    targetPost: Post
    targetComment: Comment
}

```

#### `UserList`, `BookmarkCollection`, `Report`, `PostAnalytics`, `TotpSetup`

```graphql
type UserList {
    id: ID!
    name: String!
    description: String
    isPrivate: Boolean!
    owner: User!
    members: [User!]!
    membersCount: Int!
    feed(first: Int, after: String): PostConnection!
}

type BookmarkCollection {
    id: ID!
    name: String!
    description: String
    isPrivate: Boolean!
    user: User!
    postsConnection(first: Int, after: String): PostConnection!
}

type Report {
    id: ID!
    targetId: ID!
    targetType: ReportTargetTypeGql!
    reason: ReportReasonGql!
    details: String
    status: ReportStatusGql!
    createdAt: DateTime!
    reporter: User!
}

type PostAnalytics {
    postId: ID!
    viewsCount: Int!
    likesCount: Int!
    repostsCount: Int!
    commentsCount: Int!
    engagementRate: Float!
}

type TotpSetup {
    secret: String!
    otpauthUri: String!
}

type AuthPayload {
    token: String!
    user: User!
}

```

---

## 4. GraphQL Queries

| Query                            | Arguments                                         | Return Type                | Auth Required         | Description                                                     |
|----------------------------------|---------------------------------------------------|----------------------------|-----------------------|-----------------------------------------------------------------|
| **`me`**                         | `userId: ID`                                      | `User`                     | Optional (Header/Arg) | Fetches the authenticated user profile                          |
| **`user`**                       | `id: ID!`                                         | `User`                     | None                  | Fetches a profile by User ID                                    |
| **`profile`**                    | `username: String!`                               | `User`                     | None                  | Fetches a profile by handle (@username)                         |
| **`users`**                      | -                                                 | `[User!]!`                 | None                  | Retrieves all registered users                                  |
| **`searchUsers`**                | `query: String!`, `limit: Int`                    | `[User!]!`                 | None                  | Searches users by handle or display name                        |
| **`story`**                      | `id: ID!`                                         | `Story`                    | None                  | Fetches story details by ID                                     |
| **`storiesForUser`**             | `userId: ID!`                                     | `[Story!]!`                | None                  | Retrieves active stories for a specific user                    |
| **`storiesFeed`**                | `userId: ID`                                      | `[Story!]!`                | Optional              | Retrieves stories tray from followed accounts                   |
| **`post`**                       | `id: ID!`                                         | `Post`                     | None                  | Fetches a single post by ID                                     |
| **`postAnalytics`**              | `postId: ID!`                                     | `PostAnalytics!`           | None                  | Returns view counts, likes, comments, and engagement rates      |
| **`poll`**                       | `id: ID!`                                         | `Poll`                     | None                  | Retrieves poll configuration and results                        |
| **`posts`**                      | `limit: Int`, `offset: Int`                       | `[Post!]!`                 | None                  | Fetches posts using offset pagination                           |
| **`postsConnection`**            | `first: Int`, `after: String`                     | `PostConnection!`          | None                  | Cursor-paginated feed of all public posts                       |
| **`feed`**                       | `userId: ID`, `limit: Int`, `offset: Int`         | `[Post!]!`                 | Optional              | Fetches user timeline feed using offset pagination              |
| **`feedConnection`**             | `userId: ID`, `first: Int`, `after: String`       | `PostConnection!`          | Optional              | Cursor-paginated user timeline feed                             |
| **`savedPostsConnection`**       | `userId: ID`, `first: Int`, `after: String`       | `PostConnection!`          | Optional              | Cursor-paginated collection of bookmarked posts                 |
| **`searchPostsConnection`**      | `query: String!`, `first: Int`, `after: String`   | `PostConnection!`          | None                  | Full-text post content search with cursor pagination            |
| **`postsByHashtag`**             | `hashtag: String!`, `first: Int`, `after: String` | `PostConnection!`          | None                  | Cursor-paginated posts tagged with a specific hashtag           |
| **`trendingHashtags`**           | `limit: Int`                                      | `[HashtagTrend!]!`         | None                  | Retrieves trending hashtags in real time                        |
| **`comment`**                    | `id: ID!`                                         | `Comment`                  | None                  | Fetches a single comment by ID                                  |
| **`notificationsConnection`**    | `userId: ID`, `first: Int`, `after: String`       | `NotificationConnection!`  | Optional              | Cursor-paginated notification history with unread count         |
| **`unreadNotificationsCount`**   | `userId: ID`                                      | `Int!`                     | Optional              | Returns count of unread notifications                           |
| **`notifications`**              | `userId: ID`, `limit: Int`, `offset: Int`         | `[Notification!]!`         | Optional              | Fetches recent notifications using offset pagination            |
| **`conversations`**              | -                                                 | `[Conversation!]!`         | **Required**          | Summarizes all active 1:1 message threads                       |
| **`directMessages`**             | `otherUserId: ID!`, `limit: Int`, `offset: Int`   | `[DirectMessage!]!`        | **Required**          | Offset-based conversation history with a user                   |
| **`directMessagesConnection`**   | `otherUserId: ID!`, `first: Int`, `after: String` | `DirectMessageConnection!` | **Required**          | Cursor-paginated conversation history with a user               |
| **`groupConversations`**         | -                                                 | `[GroupConversation!]!`    | **Required**          | Lists joined group conversation channels                        |
| **`groupConversation`**          | `id: ID!`                                         | `GroupConversation`        | **Required**          | Fetches details of a specific group conversation                |
| **`unreadDmCount`**              | -                                                 | `Int!`                     | **Required**          | Total count of unread incoming direct messages                  |
| **`pendingFollowRequests`**      | -                                                 | `[FollowRequest!]!`        | **Required**          | Inbound follow requests pending user approval                   |
| **`pendingFollowRequestsCount`** | -                                                 | `Int!`                     | **Required**          | Total pending inbound follow requests                           |
| **`userLists`**                  | `userId: ID`                                      | `[UserList!]!`             | Optional              | Lists custom curated lists for a user                           |
| **`userList`**                   | `id: ID!`                                         | `UserList`                 | None                  | Details for a specified curated user list                       |
| **`listFeedConnection`**         | `listId: ID!`, `first: Int`, `after: String`      | `PostConnection!`          | None                  | Cursor-paginated timeline composed of list members              |
| **`bookmarkCollections`**        | `userId: ID`                                      | `[BookmarkCollection!]!`   | Optional              | Fetches user-defined bookmark collections                       |
| **`bookmarkCollection`**         | `id: ID!`                                         | `BookmarkCollection`       | None                  | Details for a specific bookmark collection folder               |
| **`reports`**                    | `status: ReportStatusGql`, `limit: Int`           | `[Report!]!`               | **Required**          | Administrative query for reviewable moderation incident reports |

---

## 5. GraphQL Mutations

### 5.1 Auth & Account

```graphql
# User registration
signup(
username: String!
email: String!
password: String!
displayName: String!
bio: String
avatarUrl: String
headerImageUrl: String
location: String
website: String
): AuthPayload!

# Authentication
login(
usernameOrEmail: String!
password: String!
): AuthPayload!

# Profile update
updateUserProfile(
userId: ID
displayName: String
bio: String
avatarUrl: String
headerImageUrl: String
location: String
website: String
): User!

# Toggle profile privacy
updateUserPrivacy(
isPrivate: Boolean!
userId: ID
): User!

# Generate TOTP 2FA configuration secret
setup2fa(userId: ID): TotpSetup!

# Confirm and enable TOTP 2FA
enable2fa(code: String!, userId: ID): Boolean!

# Disable TOTP 2FA
disable2fa(code: String!, userId: ID): Boolean!

```

### 5.2 Social Graph & Moderation

```graphql
# Follow user (creates FollowRequest if target account is private)
followUser(followeeId: ID!, followerId: ID): User!

# Unfollow user
unfollowUser(followeeId: ID!, followerId: ID): User!

# Accept incoming follow request
acceptFollowRequest(requesterId: ID!, targetId: ID): User!

# Reject incoming follow request
rejectFollowRequest(requesterId: ID!, targetId: ID): Boolean!

# Block user (revokes visibility and messaging)
blockUser(userId: ID!, blockerId: ID): Boolean!

# Unblock user
unblockUser(userId: ID!, blockerId: ID): Boolean!

# Mute user (hides target activity from feed)
muteUser(userId: ID!, muterId: ID): Boolean!

# Unmute user
unmuteUser(userId: ID!, muterId: ID): Boolean!

# Submit moderation incident report
reportContent(
targetType: ReportTargetTypeGql!
targetId: ID!
reason: ReportReasonGql!
details: String
reporterId: ID
): Report!

# Resolve moderation incident report (Admin only)
resolveReport(reportId: ID!, status: ReportStatusGql!): Report!

```

### 5.3 Posts & Stories

```graphql
# Publish post with optional media, visibility, and poll
createPost(
content: String!
authorId: ID
audience: PostAudienceGql
media: [MediaInput!]
poll: CreatePollInput
): Post!

# Quote an existing post
quotePost(postId: ID!, content: String!, authorId: ID): Post!

# Repost
repostPost(postId: ID!, userId: ID): Post!

# Delete existing repost
unrepostPost(postId: ID!, userId: ID): Post!

# Update post content (Author only)
updatePost(postId: ID!, content: String!, authorId: ID): Post!

# Delete post (Author only)
deletePost(postId: ID!, authorId: ID): Boolean!

# Register an impression view
recordPostView(postId: ID!, viewerId: ID): Boolean!

# Toggle like status
likePost(postId: ID!, userId: ID): Post!
unlikePost(postId: ID!, userId: ID): Post!

# Toggle bookmark status
savePost(postId: ID!, userId: ID): Post!
unsavePost(postId: ID!, userId: ID): Post!

# Cast vote on poll
votePoll(pollId: ID!, optionId: ID!, userId: ID): PollOption!

# Publish 24-hour ephemeral story
createStory(mediaUrl: String!, caption: String, authorId: ID): Story!

# Record story impression
viewStory(storyId: ID!, viewerId: ID): Boolean!

# Delete story
deleteStory(storyId: ID!, authorId: ID): Boolean!

```

### 5.4 Comments

```graphql
# Publish comment or nested thread reply
createComment(
postId: ID!
content: String!
authorId: ID
parentId: ID
): Comment!

# Update comment content (Author only)
editComment(commentId: ID!, content: String!, authorId: ID): Comment!

# Delete comment (Author only)
deleteComment(commentId: ID!, authorId: ID): Boolean!

# Toggle comment like status
likeComment(commentId: ID!, userId: ID): Comment!
unlikeComment(commentId: ID!, userId: ID): Comment!

# Pin comment to post header (Post author only)
pinComment(postId: ID!, commentId: ID!, authorId: ID): Post!

# Unpin comment
unpinComment(postId: ID!, authorId: ID): Post!

```

### 5.5 Direct Messages & Real-Time Chat

```graphql
# Dispatch 1:1 direct message
sendDirectMessage(recipientId: ID!, content: String!, senderId: ID): DirectMessage!

# Initialize multi-user group chat
createGroupConversation(
title: String
participantIds: [ID!]!
creatorId: ID
): GroupConversation!

# Send message to group chat
sendGroupMessage(conversationId: ID!, content: String!, senderId: ID): DirectMessage!

# Broadcast typing activity status
sendTypingIndicator(
conversationId: ID
recipientId: ID
isTyping: Boolean!
userId: ID
): Boolean!

# Modify message content (Sender only)
editDirectMessage(messageId: ID!, content: String!, senderId: ID): DirectMessage!

# Remove message (Sender only)
deleteDirectMessage(messageId: ID!, senderId: ID): Boolean!

# Mark incoming messages as read
markMessagesAsRead(senderId: ID!, readerId: ID): Boolean!

```

### 5.6 Notifications, Lists & Collections

```graphql
# Mark specific notification as read
markNotificationAsRead(notificationId: ID!, userId: ID): Boolean!

# Mark all received notifications as read
markAllNotificationsAsRead(userId: ID): Boolean!

# Create custom curated list
createUserList(
name: String!
description: String
isPrivate: Boolean
memberIds: [ID!]
ownerId: ID
): UserList!

# Mutate or delete custom curated list
updateUserList(listId: ID!, name: String, description: String, isPrivate: Boolean, ownerId: ID): UserList!
deleteUserList(listId: ID!, ownerId: ID): Boolean!

# Membership management for custom lists
addUserToList(listId: ID!, userId: ID!, ownerId: ID): Boolean!
removeUserFromList(listId: ID!, userId: ID!, ownerId: ID): Boolean!

# Bookmark collection management
createBookmarkCollection(name: String!, description: String, isPrivate: Boolean, userId: ID): BookmarkCollection!
updateBookmarkCollection(collectionId: ID!, name: String, description: String, isPrivate: Boolean, userId: ID): BookmarkCollection!
deleteBookmarkCollection(collectionId: ID!, userId: ID): Boolean!

# Organize saved posts within collections
addPostToCollection(collectionId: ID!, postId: ID!, userId: ID): Boolean!
removePostFromCollection(collectionId: ID!, postId: ID!, userId: ID): Boolean!

```

---

## 6. GraphQL Subscriptions

WebSocket Endpoint: `ws://localhost:8080/graphql/ws` (Protocols: `graphql-transport-ws`, `graphql-ws`)

### 6.1 Real-Time DM Delivery (`directMessageReceived`)

* **Description**: Streams incoming direct and group messages addressed to the authenticated user.

```graphql
subscription OnDirectMessage($userId: ID!) {
    directMessageReceived(userId: $userId) {
        id
        senderId
        recipientId
        content
        createdAt
        sender {
            id
            username
            displayName
            avatarUrl
        }
    }
}

```

### 6.2 Real-Time Notifications (`notificationReceived`)

* **Description**: Streams incoming platform notifications (follows, likes, comments, mentions, reposts).

```graphql
subscription OnNotification($userId: ID!) {
    notificationReceived(userId: $userId) {
        id
        notificationType
        isRead
        createdAt
        sender {
            id
            username
            displayName
            avatarUrl
        }
        targetPost {
            id
            content
        }
    }
}

```

### 6.3 Real-Time Typing Indicators (`typingStatus`)

* **Description**: Monitors real-time typing activity within direct and group message sessions.

```graphql
subscription OnTyping($conversationId: ID, $recipientId: ID) {
    typingStatus(conversationId: $conversationId, recipientId: $recipientId) {
        userId
        conversationId
        recipientId
        isTyping
    }
}

```

---

## 7. Domain Error Codes

When a request encounters a domain error, the GraphQL execution result populates the `extensions` field with a standard
`code` and an explanatory `detail` message:

```json
{
  "errors": [
    {
      "message": "AUTH_INVALID_CREDENTIALS",
      "locations": [
        {
          "line": 2,
          "column": 3
        }
      ],
      "path": [
        "login"
      ],
      "extensions": {
        "code": "AUTH_INVALID_CREDENTIALS",
        "detail": "Invalid username or password"
      }
    }
  ],
  "data": null
}

```

### Standard Error Registry

| Category                 | Error Code (`ErrorCode`)     | Description                                               |
|--------------------------|------------------------------|-----------------------------------------------------------|
| **Authentication & IAM** | `AUTH_INVALID_CREDENTIALS`   | Incorrect username or password                            |
|                          | `AUTH_TOKEN_REQUIRED`        | Bearer authentication token missing                       |
|                          | `AUTH_TOKEN_INVALID`         | Signature invalid or expired bearer token                 |
|                          | `AUTH_USER_ALREADY_EXISTS`   | Specified handle or email already taken                   |
|                          | `AUTH_PASSWORD_TOO_SHORT`    | Password does not satisfy length constraints (min 8 char) |
|                          | `TOTP_INVALID_CODE`          | Verification code mismatch for 2FA validation             |
|                          | `TOTP_ALREADY_ENABLED`       | 2FA is already active on this account                     |
| **Social Graph**         | `USER_NOT_FOUND`             | Targeted user account does not exist                      |
|                          | `USER_CANNOT_FOLLOW_SELF`    | Self-follow actions are forbidden                         |
|                          | `USER_BLOCKED`               | Action barred due to an active block status               |
|                          | `CANNOT_BLOCK_SELF`          | Self-blocking is forbidden                                |
| **Stories**              | `STORY_NOT_FOUND`            | Story does not exist or has exceeded the 24h lifespan     |
|                          | `STORY_VIEWERS_UNAUTHORIZED` | View history restricted to content author                 |
| **Feed & Comments**      | `POST_NOT_FOUND`             | Referenced post not found                                 |
|                          | `POST_CONTENT_INVALID`       | Post body length violates constraints (1-5,000 chars)     |
|                          | `POST_ALREADY_REPOSTED`      | Post has already been reposted by this user               |
|                          | `COMMENT_NOT_FOUND`          | Referenced comment not found                              |
| **Polls**                | `POLL_ALREADY_VOTED`         | Duplicate vote attempt on recorded poll                   |
|                          | `POLL_EXPIRED`               | Poll voting deadline reached                              |
| **Messaging**            | `DM_CANNOT_SEND_TO_SELF`     | Self-directed direct messages are forbidden               |
|                          | `CONVERSATION_NOT_FOUND`     | Conversation thread not found                             |
| **System & Security**    | `RATE_LIMIT_EXCEEDED`        | Sliding rate limit threshold breached (HTTP 429)          |
|                          | `INVALID_CURSOR`             | Invalid, unparseable, or expired pagination cursor        |
