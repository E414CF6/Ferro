# Serve API Specification v1 (Production Ready)

Serve is the high-performance backend API service for the **Ferro Social Platform**, providing both a **RESTful HTTP API** and a strongly typed **GraphQL API (HTTP & WebSocket Subscription)**.

---

## 1. Overview & Architecture

* **API Version**: `v1.0.0`
* **Base URL**: `http://localhost:8080` (Configurable via `HOST` and `PORT` environment variables)
* **GraphQL HTTP Endpoint**: `POST /graphql`
* **GraphQL WebSocket Endpoint**: `GET /graphql/ws` (Protocols: `graphql-transport-ws`, `graphql-ws`)
* **GraphiQL Web IDE**: `GET /graphiql` or `GET /` (Enabled in development; disabled in production by default)
* **Authentication Scheme**: Dual Token Architecture (JWT Access Token + Database-backed Rotating Refresh Token)
  * Header: `Authorization: Bearer <ACCESS_TOKEN>`
* **Security & Reliability Constraints**:
  * **GraphQL Maximum Query Depth**: `15`
  * **GraphQL Maximum Query Complexity**: `2500`
  * **Introspection Shield**: Automatically disabled in production (`ENABLE_INTROSPECTION=false`)
  * **Distributed Rate Limiting**: Redis Lua sliding window with in-memory fallback (`120 req / 60s` default)
  * **HTTP Request Body Limit**: Max `100 MB` (Multipart file uploads up to `50 MB` per file)
  * **Security Response Headers**: `Strict-Transport-Security` (HSTS), `X-Content-Type-Options: nosniff`, `X-Frame-Options: SAMEORIGIN`, `Referrer-Policy: strict-origin-when-cross-origin`, `Permissions-Policy`
  * **Distributed Tracing**: `x-request-id` header generated or propagated on every request
* **CLI Subcommands**:
  * `serve migrate` or `serve --migrate`: Standalone database migration execution for Kubernetes initContainers

---

## 2. Authentication & Dual Token Architecture

Serve implements an enterprise-grade **Dual Token Architecture** with **Refresh Token Rotation (RTR)** and **Token Reuse Detection**:

```
+----------------+      1. Login / Signup        +---------------------+
|                | ----------------------------> |                     |
|                | <---------------------------- |      Serve API      |
|                |   Access Token (15m, JWT)     |                     |
|                |   Refresh Token (7d, Opaque)  +---------------------+
|     Client     |                                          |
| (Vision / App) |      2. Access Token Expired             | Hashed & Saved
|                | ---------------------------->            v
|                |   refreshToken(token)         +---------------------+
|                | <---------------------------- | PostgreSQL / SQLite |
|                |   New Access + Refresh Pair   |  (refresh_tokens)   |
+----------------+                               +---------------------+
```

1. **Access Token**:
   - Short lifespan: **15 minutes** (configurable via `JWT_EXPIRATION_MINUTES`).
   - Self-contained JWT signed with HMAC-SHA256 containing `sub` (User ID), `username`, `jti` (unique token ID), and `token_type: "access"`.
2. **Refresh Token**:
   - Long lifespan: **7 days** (configurable via `REFRESH_TOKEN_EXPIRATION_DAYS`).
   - Stored in hashed form (`SHA-256`) in the database `refresh_tokens` table.
   - **Rotation (RTR)**: Each time `/graphql` executes `refreshToken(refreshToken: ...)`, the old refresh token is marked as revoked and replaced with a newly issued token pair.
   - **Reuse Detection**: If an already revoked refresh token is presented, the system triggers a **Security Alert** and immediately revokes all active refresh tokens for that user account across all devices.
3. **Session Revocation (Logout)**:
   - Calling `logout(refreshToken: ...)` revokes the specific device session. Omitting the token revokes all active sessions for that user.

---

## 3. REST Endpoints

### 3.1 Health & Diagnostics Probes

#### `GET /health` / `GET /health/live`
* **Description**: Kubernetes Liveness Probe. Returns 200 OK if the process is running.
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
* **Description**: Kubernetes Readiness Probe. Verifies live ping connectivity to PostgreSQL/SQLite and Redis.
* **Authentication**: Public
* **Success Response (200 OK)**:
```json
{
  "status": "ready",
  "database": "connected",
  "driver": "postgres",
  "redis": "connected",
  "pool": {
    "size": 10,
    "idle": 8,
    "active": 2
  }
}
```
* **Failure Response (503 Service Unavailable)**:
```json
{
  "status": "unhealthy",
  "database": "disconnected",
  "redis": "disconnected",
  "error": "connection refused"
}
```

---

### 3.2 SRE Prometheus Metrics

#### `GET /metrics`
* **Description**: Exposes real-time runtime diagnostics, HTTP traffic statistics, and connection pool states formatted for Prometheus scraping.
* **Authentication**: Public or Protected by `Authorization: Bearer <METRICS_AUTH_TOKEN>` if `METRICS_AUTH_TOKEN` is configured.
* **Response (200 OK, Content-Type: `text/plain; version=0.0.4; charset=utf-8`)**:
```prometheus
# HELP ferro_serve_info Service build information
# TYPE ferro_serve_info gauge
ferro_serve_info{version="0.1.0"} 1

# HELP ferro_http_requests_total Total HTTP requests processed
# TYPE ferro_http_requests_total counter
ferro_http_requests_total{status="2xx"} 1420
ferro_http_requests_total{status="3xx"} 12
ferro_http_requests_total{status="4xx"} 45
ferro_http_requests_total{status="5xx"} 1

# HELP ferro_http_request_duration_seconds_total Total duration of processed HTTP requests in seconds
# TYPE ferro_http_request_duration_seconds_total counter
ferro_http_request_duration_seconds_total 12.450123

# HELP ferro_db_pool_connections_total Total database pool connections
# TYPE ferro_db_pool_connections_total gauge
ferro_db_pool_connections_total 10

# HELP ferro_db_pool_connections_idle Idle database connections
# TYPE ferro_db_pool_connections_idle gauge
ferro_db_pool_connections_idle 8

# HELP ferro_db_pool_connections_active Active database connections in use
# TYPE ferro_db_pool_connections_active gauge
ferro_db_pool_connections_active 2

# HELP ferro_redis_connected Redis cluster connectivity state (1 = connected, 0 = standalone/disconnected)
# TYPE ferro_redis_connected gauge
ferro_redis_connected 1

# HELP ferro_rate_limit_max_requests Maximum configured requests per window
# TYPE ferro_rate_limit_max_requests gauge
ferro_rate_limit_max_requests 120
```

---

### 3.3 Media Upload API

#### `POST /api/upload`
* **Description**: Uploads media files to persistent storage (Local directory or AWS S3 / Cloudflare R2 / MinIO).
* **Authentication**: **Required** (`Authorization: Bearer <ACCESS_TOKEN>`)
* **Content-Type**: `multipart/form-data`
* **Form Field**: `file` or `image`
* **Limits**: Maximum file size **50 MB** (Total body limit: 100 MB).
* **Security & Privacy Hardening**:
  * **Binary Magic Byte Inspection**: Strict validation preventing extension spoofing.
  * **EXIF Metadata Stripping**: For JPEG uploads, APP1 metadata segments (containing GPS coordinates, camera serials, and timestamps) are stripped losslessly to protect user privacy.
  * **Cloud S3 AWS SigV4 Signing**: When configured for S3/R2, uploads are signed using AWS Signature Version 4.

| Format | Magic Byte Signature | Extension | MIME Type |
|---|---|---|---|
| **JPEG** | `FF D8 FF` | `.jpg` | `image/jpeg` |
| **PNG** | `89 50 4E 47 0D 0A 1A 0A` | `.png` | `image/png` |
| **GIF** | `GIF87a` / `GIF89a` | `.gif` | `image/gif` |
| **WEBP** | `RIFF....WEBP` | `.webp` | `image/webp` |
| **HEIC** | `....ftypheic` / `....ftypmif1` | `.heic` | `image/heic` |
| **HEIF** | `....ftypheif` | `.heif` | `image/heif` |
| **AVIF** | `....ftypavif` | `.avif` | `image/avif` |
| **SVG** | `<svg` / `<?xml ... <svg` | `.svg` | `image/svg+xml` |

* **Success Response (201 Created)**:
```json
{
  "url": "/uploads/a3b1c2d3-e4f5-4678-90ab-cdef12345678.jpg",
  "key": "a3b1c2d3-e4f5-4678-90ab-cdef12345678.jpg",
  "size": 1048576,
  "content_type": "image/jpeg"
}
```
* **Error Responses**:
  * `401 Unauthorized`: Missing or invalid Bearer token (`UNAUTHORIZED`)
  * `400 Bad Request`: `EMPTY_FILE`, `MISSING_FILE`, `INVALID_FILE_SIGNATURE`, `READ_ERROR`
  * `413 Payload Too Large`: File exceeds 50 MB limit (`FILE_TOO_LARGE`)
  * `500 Internal Server Error`: Storage failure (`STORAGE_ERROR`)

#### `GET /uploads/{filename}`
* **Description**: Serves static media assets stored in local disk storage.

---

## 4. GraphQL Schema & Common Types

### 4.1 Common Enums

| Enum Name | Allowed Values | Description |
|---|---|---|
| **`MediaTypeGql`** | `IMAGE`, `VIDEO`, `GIF` | Attachment media format |
| **`PostAudienceGql`** | `PUBLIC`, `FOLLOWERS_ONLY`, `CLOSE_FRIENDS` | Post visibility scope |
| **`ReportTargetTypeGql`** | `POST`, `COMMENT`, `USER` | Target resource type for moderation |
| **`ReportReasonGql`** | `SPAM`, `HARASSMENT`, `HATE_SPEECH`, `INAPPROPRIATE`, `COPYRIGHT`, `OTHER` | Content report reason |
| **`ReportStatusGql`** | `PENDING`, `RESOLVED`, `DISMISSED` | Report resolution status |
| **`FollowRequestStatusGql`** | `PENDING`, `ACCEPTED`, `REJECTED` | Inbound follow request state |
| **`NotificationTypeGql`** | `FOLLOW`, `FOLLOW_REQUEST`, `FOLLOW_ACCEPTED`, `LIKE_POST`, `LIKE_COMMENT`, `COMMENT_POST`, `REPLY_COMMENT`, `REPOST`, `QUOTE`, `MENTION`, `DIRECT_MESSAGE` | Real-time notification event type |
| **`RankChangeType`** | `UP`, `DOWN`, `SAME`, `NEW` | Real-time trending rank delta |

---

### 4.2 Core Object Types

#### `AuthPayload`
Returned upon authentication (`login`, `signup`, `refreshToken`):
```graphql
type AuthPayload {
    token: String!           # Short-lived JWT Access Token (15 mins)
    refreshToken: String!    # Rotating Refresh Token (7 days)
    expiresIn: Int!          # Access token TTL in seconds (e.g. 900)
    tokenType: String!       # "Bearer"
    user: User!              # Authenticated user profile
}
```

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
    postsCount: Int!           # DataLoader batched
    posts: [Post!]!
    activeStories: [Story!]!
    hasActiveStories: Boolean!  # DataLoader batched
    likedPosts: [Post!]!
    savedPosts(limit: Int, offset: Int): [Post!]!
    collections: [BookmarkCollection!]!
    lists: [UserList!]!
    comments: [Comment!]!
    followers: [User!]!
    following: [User!]!
    followersCount: Int!       # DataLoader batched
    followingCount: Int!       # DataLoader batched
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
    author: User!               # DataLoader batched
    media: [PostMedia!]!        # DataLoader batched
    poll: Poll                  # DataLoader batched
    analytics: PostAnalytics!
    quotePost: Post             # DataLoader batched
    pinnedComment: Comment      # DataLoader batched
    likesCount: Int!            # DataLoader batched
    repostsCount: Int!          # DataLoader batched
    isLikedBy(userId: ID): Boolean!
    isRepostedByMe: Boolean!
    isSavedByMe: Boolean!
    hashtags: [String!]!
    comments(topLevelOnly: Boolean): [Comment!]!
    commentsConnection(topLevelOnly: Boolean, first: Int, after: String): CommentConnection!
}
```

#### `Story` (24h Lifespan Ephemeral Content)
```graphql
type Story {
    id: ID!
    mediaUrl: String!
    caption: String
    createdAt: DateTime!
    expiresAt: DateTime!
    isExpired: Boolean!
    author: User!               # DataLoader batched
    viewsCount: Int!
    isViewedByMe: Boolean!
    viewers: [User!]!           # Restricted to story creator only
}
```

#### `Comment` (Hierarchical Nested Threads)
```graphql
type Comment {
    id: ID!
    parentId: ID
    content: String!
    isEdited: Boolean!
    createdAt: DateTime!
    updatedAt: DateTime!
    author: User!               # DataLoader batched
    post: Post!                 # DataLoader batched
    parent: Comment             # DataLoader batched
    depth: Int!                 # Depth (0 = root comment)
    rootComment: Comment
    replies: [Comment!]!
    repliesConnection(first: Int, after: String): CommentConnection!
    repliesCount: Int!          # DataLoader batched
    likesCount: Int!            # DataLoader batched
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

#### `WikiArticle` & `WikiRevision` (Geospatial wMap)
```graphql
type WikiArticle {
    id: ID!
    userId: ID
    title: String!
    slug: String!
    summary: String
    content: String!
    latitude: Float!
    longitude: Float!
    zoom: Float!
    category: String!
    tags: [String!]!
    geojson: String
    author: String!
    views: Int!
    createdAt: DateTime!
    updatedAt: DateTime!
    revisions(limit: Int): [WikiRevision!]!
}

type WikiRevision {
    id: ID!
    articleId: ID!
    userId: ID
    title: String!
    content: String!
    latitude: Float!
    longitude: Float!
    editSummary: String
    author: String!
    createdAt: DateTime!
}
```

---

## 5. GraphQL Queries

| Query | Arguments | Return Type | Auth Required | Description |
|---|---|---|---|---|
| **`me`** | `userId: ID` | `User` | Optional | Authenticated user profile |
| **`user`** | `id: ID!` | `User` | None | User profile by ID |
| **`profile`** | `username: String!` | `User` | None | User profile by handle (`@username`) |
| **`users`** | - | `[User!]!` | None | All registered users |
| **`searchUsers`** | `query: String!`, `limit: Int` | `[User!]!` | None | Search users by username or display name |
| **`story`** | `id: ID!` | `Story` | None | Story details |
| **`storiesForUser`** | `userId: ID!` | `[Story!]!` | None | Active stories by user ID |
| **`storiesFeed`** | `userId: ID` | `[Story!]!` | Optional | Story tray from followed users |
| **`post`** | `id: ID!` | `Post` | None | Single post by ID |
| **`postAnalytics`** | `postId: ID!` | `PostAnalytics!` | None | Engagement and impression metrics |
| **`poll`** | `id: ID!` | `Poll` | None | Poll details and votes |
| **`posts`** | `limit: Int`, `offset: Int` | `[Post!]!` | None | Offset-paginated public posts |
| **`postsConnection`** | `first: Int`, `after: String` | `PostConnection!` | None | Cursor-paginated public feed |
| **`feed`** | `userId: ID`, `limit: Int`, `offset: Int` | `[Post!]!` | Optional | Offset-paginated home timeline |
| **`feedConnection`** | `userId: ID`, `first: Int`, `after: String` | `PostConnection!` | Optional | Cursor-paginated home timeline |
| **`savedPosts`** | `userId: ID!` | `[Post!]!` | Optional | Flat list of bookmarked posts |
| **`savedPostsConnection`** | `userId: ID`, `first: Int`, `after: String` | `PostConnection!` | Optional | Cursor-paginated bookmarked posts |
| **`searchPostsConnection`** | `query: String!`, `first: Int`, `after: String` | `PostConnection!` | None | Full-text search on posts |
| **`postsByHashtag`** | `hashtag: String!`, `first: Int`, `after: String` | `PostConnection!` | None | Posts tagged with hashtag |
| **`trendingHashtags`** | `limit: Int` | `[HashtagTrend!]!` | None | Real-time trending hashtags |
| **`comment`** | `id: ID!` | `Comment` | None | Single comment by ID |
| **`notificationsConnection`** | `userId: ID`, `first: Int`, `after: String` | `NotificationConnection!` | Optional | Cursor-paginated notifications |
| **`unreadNotificationsCount`** | `userId: ID` | `Int!` | Optional | Count of unread notifications |
| **`notifications`** | `userId: ID`, `limit: Int`, `offset: Int` | `[Notification!]!` | Optional | Recent notifications list |
| **`conversations`** | - | `[Conversation!]!` | **Required** | All 1:1 conversation summaries |
| **`directMessages`** | `otherUserId: ID!`, `limit: Int`, `offset: Int` | `[DirectMessage!]!` | **Required** | Messages in a 1:1 thread |
| **`directMessagesConnection`** | `otherUserId: ID!`, `first: Int`, `after: String` | `DirectMessageConnection!` | **Required** | Cursor-paginated messages |
| **`groupConversations`** | - | `[GroupConversation!]!` | **Required** | Joined group chat channels |
| **`groupConversation`** | `id: ID!` | `GroupConversation` | **Required** | Details of a group chat |
| **`unreadDmCount`** | - | `Int!` | **Required** | Total unread message count |
| **`pendingFollowRequests`** | - | `[FollowRequest!]!` | **Required** | Inbound pending follow requests |
| **`pendingFollowRequestsCount`**| - | `Int!` | **Required** | Total pending follow requests count |
| **`userLists`** | `userId: ID` | `[UserList!]!` | Optional | User curated lists |
| **`userList`** | `id: ID!` | `UserList` | None | Curated list details |
| **`listFeedConnection`** | `listId: ID!`, `first: Int`, `after: String` | `PostConnection!` | None | Posts from list members |
| **`bookmarkCollections`** | `userId: ID` | `[BookmarkCollection!]!` | Optional | Bookmark collection folders |
| **`bookmarkCollection`** | `id: ID!` | `BookmarkCollection` | None | Collection details |
| **`reports`** | `status: ReportStatusGql`, `limit: Int` | `[Report!]!` | **Required** | Moderation reports (admin) |
| **`articles`** | `filter: ArticleFilterInput` | `[WikiArticle!]!` | None | Spatial wiki articles (bounding box, tags) |
| **`article`** | `slug: String!` | `WikiArticle` | None | Wiki article by URL slug |
| **`wikiTrends`** | `forceRefresh: Boolean` | `TrendsData!` | None | Real-time Top 10 articles & trends |

---

## 6. GraphQL Mutations

### 6.1 Authentication & Session Management

```graphql
# User registration (Issues initial TokenPair)
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

# User login (Issues initial TokenPair)
login(
  usernameOrEmail: String!
  password: String!
): AuthPayload!

# Refresh token rotation (RTR)
refreshToken(
  refreshToken: String!
): AuthPayload!

# Logout session (Revokes refresh token in database)
logout(
  refreshToken: String
): Boolean!

# User profile updates
updateUserProfile(
  userId: ID
  displayName: String
  bio: String
  avatarUrl: String
  headerImageUrl: String
  location: String
  website: String
): User!

# Toggle profile privacy (Public vs Private)
updateUserPrivacy(
  isPrivate: Boolean!
  userId: ID
): User!

# TOTP 2FA secret generation
setup2fa(userId: ID): TotpSetup!

# Enable TOTP 2FA with 6-digit confirmation code
enable2fa(code: String!, userId: ID): Boolean!

# Disable TOTP 2FA with verification code
disable2fa(code: String!, userId: ID): Boolean!
```

### 6.2 Social Graph & Moderation

```graphql
# Follow user (creates FollowRequest if account is private)
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

# Mute user
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

# Resolve incident report (Admin only)
resolveReport(reportId: ID!, status: ReportStatusGql!): Report!
```

### 6.3 Posts, Polls & Ephemeral Stories

```graphql
# Create post with optional media, audience, and poll
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
unrepostPost(postId: ID!, userId: ID): Post!

# Edit post content
updatePost(postId: ID!, content: String!, authorId: ID): Post!

# Delete post
deletePost(postId: ID!, authorId: ID): Boolean!

# Record view impression
recordPostView(postId: ID!, viewerId: ID): Boolean!

# Likes & Bookmarks
likePost(postId: ID!, userId: ID): Post!
unlikePost(postId: ID!, userId: ID): Post!
savePost(postId: ID!, userId: ID): Post!
unsavePost(postId: ID!, userId: ID): Post!

# Vote on poll
votePoll(pollId: ID!, optionId: ID!, userId: ID): PollOption!

# Create 24-hour ephemeral story
createStory(mediaUrl: String!, caption: String, authorId: ID): Story!

# Record story impression
viewStory(storyId: ID!, viewerId: ID): Boolean!

# Delete story
deleteStory(storyId: ID!, authorId: ID): Boolean!
```

### 6.4 Comments

```graphql
# Create comment or reply
createComment(postId: ID!, content: String!, authorId: ID, parentId: ID): Comment!

# Edit comment
editComment(commentId: ID!, content: String!, authorId: ID): Comment!

# Delete comment
deleteComment(commentId: ID!, authorId: ID): Boolean!

# Like comment
likeComment(commentId: ID!, userId: ID): Comment!
unlikeComment(commentId: ID!, userId: ID): Comment!

# Pin comment to post header (Author only)
pinComment(postId: ID!, commentId: ID!, authorId: ID): Post!
unpinComment(postId: ID!, authorId: ID): Post!
```

### 6.5 Direct Messages & Real-Time Chat

```graphql
# Send 1:1 direct message
sendDirectMessage(recipientId: ID!, content: String!, senderId: ID): DirectMessage!

# Create group chat
createGroupConversation(title: String, participantIds: [ID!]!, creatorId: ID): GroupConversation!

# Send message to group chat
sendGroupMessage(conversationId: ID!, content: String!, senderId: ID): DirectMessage!

# Send typing indicator status
sendTypingIndicator(conversationId: ID, recipientId: ID, isTyping: Boolean!, userId: ID): Boolean!

# Edit / Delete direct message
editDirectMessage(messageId: ID!, content: String!, senderId: ID): DirectMessage!
deleteDirectMessage(messageId: ID!, senderId: ID): Boolean!

# Mark messages as read
markMessagesAsRead(senderId: ID!, readerId: ID): Boolean!
```

### 6.6 Geospatial Wiki Map (wMap)

```graphql
# Create geospatial wiki article (slug generated automatically)
createArticle(input: CreateArticleInput!): WikiArticle!

# Update article and create revision history record
updateArticle(slug: String!, input: UpdateArticleInput!): WikiArticle!

# Delete article
deleteArticle(slug: String!): Boolean!

# Record article view impression for real-time trending calculation
recordArticleView(articleId: ID!): Boolean!
```

---

## 7. GraphQL Subscriptions (Distributed Pub/Sub)

WebSocket Endpoint: `ws://localhost:8080/graphql/ws`

Real-time subscriptions are backed by **Redis Pub/Sub** across distributed nodes (`ferro:pubsub:dm`, `ferro:pubsub:notification`, `ferro:pubsub:typing`), with automatic fallback to local broadcast if Redis is offline.

### 7.1 Direct Message Streaming (`directMessageReceived`)
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

### 7.2 Platform Notifications (`notificationReceived`)
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

### 7.3 Real-Time Typing Indicator (`typingStatus`)
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

## 8. Domain Error Format & Standard Registry

All GraphQL domain errors populate `extensions.code` and `extensions.detail`:

```json
{
  "errors": [
    {
      "message": "AUTH_INVALID_CREDENTIALS",
      "locations": [{ "line": 2, "column": 3 }],
      "path": ["login"],
      "extensions": {
        "code": "AUTH_INVALID_CREDENTIALS",
        "detail": "Invalid username or password"
      }
    }
  ],
  "data": null
}
```

### Error Code Registry

| Category | Error Code (`code`) | Description |
|---|---|---|
| **Authentication & IAM** | `AUTH_INVALID_CREDENTIALS` | Incorrect username or password |
| | `AUTH_TOKEN_REQUIRED` | Missing `Authorization: Bearer <token>` header |
| | `AUTH_TOKEN_INVALID` | Invalid, expired, or revoked token |
| | `AUTH_USER_ALREADY_EXISTS` | Username or email already registered |
| | `AUTH_PASSWORD_TOO_SHORT` | Password fails minimum length requirement (8 chars) |
| | `TOTP_INVALID_CODE` | 2FA verification code mismatch |
| | `TOTP_ALREADY_ENABLED` | 2FA is already active |
| **Social Graph** | `USER_NOT_FOUND` | Target user account does not exist |
| | `USER_CANNOT_FOLLOW_SELF` | Self-follow forbidden |
| | `USER_BLOCKED` | Action barred due to an active block relationship |
| | `CANNOT_BLOCK_SELF` | Self-blocking forbidden |
| **Ephemeral Stories** | `STORY_NOT_FOUND` | Story not found or past 24h expiration |
| | `STORY_VIEWERS_UNAUTHORIZED` | View history restricted to story author |
| **Feed & Comments** | `POST_NOT_FOUND` | Referenced post not found |
| | `POST_CONTENT_INVALID` | Post content violates length constraints (1-5000 chars) |
| | `POST_ALREADY_REPOSTED` | User has already reposted this post |
| | `COMMENT_NOT_FOUND` | Referenced comment not found |
| **Polls** | `POLL_ALREADY_VOTED` | Duplicate vote attempt |
| | `POLL_EXPIRED` | Poll voting deadline exceeded |
| **Direct Messaging** | `DM_CANNOT_SEND_TO_SELF` | Self messaging forbidden |
| | `CONVERSATION_NOT_FOUND` | Conversation thread not found |
| **Geospatial Wiki (wMap)** | `WIKI_ARTICLE_NOT_FOUND` | Article not found |
| | `WIKI_ARTICLE_TITLE_INVALID` | Title empty or exceeds 100 characters |
| | `WIKI_ARTICLE_CONTENT_INVALID` | Article content body is empty |
| | `WIKI_ARTICLE_COORDINATES_INVALID`| Latitude/longitude out of range (-90..90, -180..180) |
| | `WIKI_ARTICLE_TAGS_INVALID` | Exceeded tag limit (maximum 15 tags) |
| | `WIKI_ARTICLE_CREATE_FAILED` | Database insertion failure |
| | `WIKI_ARTICLE_UPDATE_FAILED` | Database update failure |
| | `WIKI_ARTICLE_DELETE_FAILED` | Database deletion failure |
| **System & Security** | `RATE_LIMIT_EXCEEDED` | Rate limit threshold breached (HTTP 429) |
| | `INVALID_CURSOR` | Malformed or expired pagination cursor |
