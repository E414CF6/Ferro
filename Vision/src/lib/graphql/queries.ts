import {POST_FIELDS} from "./fragments";

export const QUERIES = {
    ME: `
    query GetMe {
      me {
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
    query GetStoriesFeed {
      storiesFeed {
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
    query GetActiveStories($userId: ID) {
      activeStories(userId: $userId) {
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
    query GetFeed($limit: Int, $offset: Int) {
      feed(limit: $limit, offset: $offset) {
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

    SEARCH_POSTS: `
    query SearchPosts($query: String!) {
      searchPosts(query: $query) {
        ${POST_FIELDS}
      }
    }
  `,

    SEARCH_HASHTAGS: `
    query SearchHashtags($query: String!) {
      searchHashtags(query: $query) {
        id
        name
        postsCount
      }
    }
  `,

    POST_ANALYTICS: `
    query GetPostAnalytics($postId: ID!) {
      postAnalytics(postId: $postId) {
        viewsCount
        likesCount
        repostsCount
        commentsCount
        engagementRate
      }
    }
  `,

    NOTIFICATIONS: `
    query GetNotifications($limit: Int, $offset: Int) {
      notifications(limit: $limit, offset: $offset) {
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
        post {
          id
          content
        }
        comment {
          id
          content
        }
      }
    }
  `,

    UNREAD_NOTIFICATIONS_COUNT: `
    query GetUnreadNotificationsCount {
      unreadNotificationsCount
    }
  `,

    BOOKMARK_COLLECTIONS: `
    query GetBookmarkCollections {
      bookmarkCollections {
        id
        name
        description
        isPrivate
        postsCount
      }
    }
  `,

    COLLECTION_POSTS: `
    query GetCollectionPosts($collectionId: ID!, $limit: Int, $offset: Int) {
      collectionPosts(collectionId: $collectionId, limit: $limit, offset: $offset) {
        ${POST_FIELDS}
      }
    }
  `,

    USER_LISTS: `
    query GetUserLists {
      userLists {
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
    query GetListFeed($listId: ID!, $limit: Int, $offset: Int) {
      listFeed(listId: $listId, limit: $limit, offset: $offset) {
        ${POST_FIELDS}
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

    UNREAD_DM_COUNT: `
    query GetUnreadDmCount {
      unreadDmCount
    }
  `,
};
