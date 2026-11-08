import {POST_FIELDS} from "./fragments";

export const QUERIES = {
    ME: `
    query GetMe($userId: ID) {
      me(userId: $userId) {
        id
        username
        email
        displayName
        bio
        avatarUrl
        headerImageUrl
        location
        website
        isPrivate
        is2faEnabled
        createdAt
        postsCount
        followersCount
        followingCount
        hasActiveStories
      }
    }
  `,

    STORIES_FEED: `
    query GetStoriesFeed($userId: ID) {
      storiesFeed(userId: $userId) {
        id
        mediaUrl
        caption
        createdAt
        expiresAt
        isExpired
        viewsCount
        isViewedByMe
        author {
          id
          username
          displayName
          avatarUrl
        }
      }
    }
  `,

    ACTIVE_STORIES: `
    query GetActiveStories($userId: ID!) {
      storiesForUser(userId: $userId) {
        id
        mediaUrl
        caption
        createdAt
        expiresAt
        isExpired
        viewsCount
        isViewedByMe
        author {
          id
          username
          displayName
          avatarUrl
        }
      }
    }
  `,

    STORY_VIEWERS: `
    query GetStoryViewers($id: ID!) {
      story(id: $id) {
        id
        viewsCount
        viewers {
          id
          username
          displayName
          avatarUrl
        }
      }
    }
  `,

    FEED: `
    query GetFeed($userId: ID, $limit: Int, $offset: Int) {
      feed(userId: $userId, limit: $limit, offset: $offset) {
        ${POST_FIELDS}
      }
    }
  `,

    GLOBAL_POSTS: `
    query GetPosts($limit: Int, $offset: Int) {
      posts(limit: $limit, offset: $offset) {
        ${POST_FIELDS}
      }
    }
  `,

    POST: `
    query GetPost($id: ID!) {
      post(id: $id) {
        ${POST_FIELDS}
      }
    }
  `,

    SEARCH_POSTS: `
    query SearchPostsConnection($query: String!, $first: Int, $after: String) {
      searchPostsConnection(query: $query, first: $first, after: $after) {
        edges {
          cursor
          node {
            ${POST_FIELDS}
          }
        }
        pageInfo {
          hasNextPage
          hasPreviousPage
          startCursor
          endCursor
        }
      }
    }
  `,

    POSTS_BY_HASHTAG: `
    query GetPostsByHashtag($hashtag: String!, $first: Int, $after: String) {
      postsByHashtag(hashtag: $hashtag, first: $first, after: $after) {
        edges {
          cursor
          node {
            ${POST_FIELDS}
          }
        }
        pageInfo {
          hasNextPage
          hasPreviousPage
          startCursor
          endCursor
        }
      }
    }
  `,

    TRENDING_HASHTAGS: `
    query GetTrendingHashtags($limit: Int) {
      trendingHashtags(limit: $limit) {
        hashtag
        postsCount
      }
    }
  `,

    POST_ANALYTICS: `
    query GetPostAnalytics($postId: ID!) {
      postAnalytics(postId: $postId) {
        postId
        viewsCount
        likesCount
        repostsCount
        commentsCount
        engagementRate
      }
    }
  `,

    NOTIFICATIONS: `
    query GetNotifications($userId: ID, $limit: Int, $offset: Int) {
      notifications(userId: $userId, limit: $limit, offset: $offset) {
        id
        notificationType
        isRead
        createdAt
        actor {
          id
          username
          displayName
          avatarUrl
        }
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
        targetComment {
          id
          content
        }
      }
    }
  `,

    UNREAD_NOTIFICATIONS_COUNT: `
    query GetUnreadNotificationsCount($userId: ID) {
      unreadNotificationsCount(userId: $userId)
    }
  `,

    BOOKMARK_COLLECTIONS: `
    query GetBookmarkCollections($userId: ID) {
      bookmarkCollections(userId: $userId) {
        id
        name
        description
        isPrivate
      }
    }
  `,

    BOOKMARK_COLLECTION: `
    query GetBookmarkCollection($id: ID!, $first: Int, $after: String) {
      bookmarkCollection(id: $id) {
        id
        name
        description
        isPrivate
        postsConnection(first: $first, after: $after) {
          edges {
            cursor
            node {
              ${POST_FIELDS}
            }
          }
          pageInfo {
            hasNextPage
            startCursor
            endCursor
          }
        }
      }
    }
  `,

    SAVED_POSTS: `
    query GetSavedPostsConnection($userId: ID, $first: Int, $after: String) {
      savedPostsConnection(userId: $userId, first: $first, after: $after) {
        edges {
          cursor
          node {
            ${POST_FIELDS}
          }
        }
        pageInfo {
          hasNextPage
          startCursor
          endCursor
        }
      }
    }
  `,

    USER_LISTS: `
    query GetUserLists($userId: ID) {
      userLists(userId: $userId) {
        id
        name
        description
        isPrivate
        membersCount
        members {
          id
          username
          displayName
          avatarUrl
        }
      }
    }
  `,

    LIST_FEED: `
    query GetListFeedConnection($listId: ID!, $first: Int, $after: String) {
      listFeedConnection(listId: $listId, first: $first, after: $after) {
        edges {
          cursor
          node {
            ${POST_FIELDS}
          }
        }
        pageInfo {
          hasNextPage
          hasPreviousPage
          startCursor
          endCursor
        }
      }
    }
  `,

    USERS_LIST: `
    query GetUsers {
      users {
        id
        username
        displayName
        bio
        avatarUrl
        isPrivate
        hasActiveStories
        followersCount
        followingCount
        isFollowedByMe
        isBlockedByMe
        isMutedByMe
        hasPendingFollowRequest
        isMe
      }
    }
  `,

    SEARCH_USERS: `
    query SearchUsers($query: String!, $limit: Int) {
      searchUsers(query: $query, limit: $limit) {
        id
        username
        displayName
        bio
        avatarUrl
        isPrivate
        hasActiveStories
        followersCount
        followingCount
        isFollowedByMe
      }
    }
  `,

    USER_PROFILE: `
    query GetUserProfile($username: String!) {
      profile(username: $username) {
        id
        username
        email
        displayName
        bio
        avatarUrl
        headerImageUrl
        location
        website
        isPrivate
        is2faEnabled
        createdAt
        postsCount
        followersCount
        followingCount
        hasActiveStories
        isMe
        isFollowedByMe
        isBlockedByMe
        isMutedByMe
        hasPendingFollowRequest
        posts {
          ${POST_FIELDS}
        }
        likedPosts {
          ${POST_FIELDS}
        }
        savedPosts {
          ${POST_FIELDS}
        }
        followers {
          id
          username
          displayName
          avatarUrl
          bio
          isFollowedByMe
        }
        following {
          id
          username
          displayName
          avatarUrl
          bio
          isFollowedByMe
        }
        activeStories {
          id
          mediaUrl
          caption
          createdAt
          expiresAt
          isExpired
          viewsCount
          isViewedByMe
          author {
            id
            username
            displayName
            avatarUrl
          }
        }
      }
    }
  `,

    CONVERSATIONS: `
    query GetConversations {
      conversations {
        unreadCount
        otherUser {
          id
          username
          displayName
          avatarUrl
          hasActiveStories
        }
        lastMessage {
          id
          content
          createdAt
          isRead
          isMine
        }
      }
    }
  `,

    DIRECT_MESSAGES: `
    query GetDirectMessages($otherUserId: ID!, $limit: Int, $offset: Int) {
      directMessages(otherUserId: $otherUserId, limit: $limit, offset: $offset) {
        id
        conversationId
        content
        createdAt
        isRead
        isMine
        sender {
          id
          username
          displayName
          avatarUrl
        }
        recipient {
          id
          username
          displayName
          avatarUrl
        }
      }
    }
  `,

    GROUP_CONVERSATIONS: `
    query GetGroupConversations {
      groupConversations {
        id
        title
        creatorId
        createdAt
        members {
          id
          username
          displayName
          avatarUrl
        }
        messages(limit: 1) {
          id
          content
          createdAt
          sender {
            id
            username
            displayName
          }
        }
      }
    }
  `,

    GROUP_CONVERSATION: `
    query GetGroupConversation($id: ID!) {
      groupConversation(id: $id) {
        id
        title
        creatorId
        createdAt
        members {
          id
          username
          displayName
          avatarUrl
        }
        messages(limit: 60) {
          id
          content
          createdAt
          isMine
          sender {
            id
            username
            displayName
            avatarUrl
          }
        }
      }
    }
  `,

    UNREAD_DM_COUNT: `
    query GetUnreadDmCount {
      unreadDmCount
    }
  `,

    PENDING_FOLLOW_REQUESTS: `
    query GetPendingFollowRequests {
      pendingFollowRequests {
        id
        requesterId
        targetId
        status
        createdAt
        requester {
          id
          username
          displayName
          avatarUrl
          bio
        }
      }
    }
  `,

    PENDING_FOLLOW_REQUESTS_COUNT: `
    query GetPendingFollowRequestsCount {
      pendingFollowRequestsCount
    }
  `,

    REPORTS: `
    query GetReports($status: ReportStatusGql, $limit: Int) {
      reports(status: $status, limit: $limit) {
        id
        reporterId
        targetType
        targetId
        reason
        details
        status
        createdAt
        reporter {
          id
          username
          displayName
          avatarUrl
        }
      }
    }
  `,

    /* ========================================================================= */
    /* Wiki Map (wMap) Queries                                                   */
    /* ========================================================================= */

    WIKI_ARTICLES: `
    query GetWikiArticles($filter: ArticleFilterInput) {
      articles(filter: $filter) {
        id
        userId
        title
        slug
        summary
        content
        latitude
        longitude
        zoom
        category
        tags
        geojson
        author
        views
        createdAt
        updatedAt
      }
    }
  `,

    WIKI_ARTICLE: `
    query GetWikiArticle($slug: String!) {
      article(slug: $slug) {
        id
        userId
        title
        slug
        summary
        content
        latitude
        longitude
        zoom
        category
        tags
        geojson
        author
        views
        createdAt
        updatedAt
        revisions(limit: 25) {
          id
          articleId
          userId
          title
          content
          latitude
          longitude
          editSummary
          author
          createdAt
        }
      }
    }
  `,

    WIKI_TRENDS: `
    query GetWikiTrends($forceRefresh: Boolean) {
      wikiTrends(forceRefresh: $forceRefresh) {
        totalArticles
        updatedAt
        articles {
          rank
          prevRank
          change
          changeAmount
          id
          title
          slug
          summary
          latitude
          longitude
          zoom
          category
          tags
          views
          recentViews
          score
          updatedAt
        }
        tags {
          rank
          prevRank
          change
          changeAmount
          tag
          count
          score
        }
      }
    }
  `,
};
