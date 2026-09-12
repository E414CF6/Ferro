import {POST_FIELDS} from "./fragments";

export const MUTATIONS = {
    LOGIN: `
    mutation Login($usernameOrEmail: String!, $password: String!) {
      login(usernameOrEmail: $usernameOrEmail, password: $password) {
        token
        user {
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
        }
      }
    }
  `,

    SIGNUP: `
    mutation Signup(
      $username: String!
      $email: String!
      $password: String!
      $displayName: String!
      $bio: String
      $avatarUrl: String
      $headerImageUrl: String
      $location: String
      $website: String
    ) {
      signup(
        username: $username
        email: $email
        password: $password
        displayName: $displayName
        bio: $bio
        avatarUrl: $avatarUrl
        headerImageUrl: $headerImageUrl
        location: $location
        website: $website
      ) {
        token
        user {
          id
          username
          email
          displayName
          bio
          avatarUrl
          headerImageUrl
          location
          website
          createdAt
        }
      }
    }
  `,

    UPDATE_PROFILE: `
    mutation UpdateProfile(
      $displayName: String
      $bio: String
      $avatarUrl: String
      $headerImageUrl: String
      $location: String
      $website: String
      $isPrivate: Boolean
    ) {
      updateUserProfile(
        displayName: $displayName
        bio: $bio
        avatarUrl: $avatarUrl
        headerImageUrl: $headerImageUrl
        location: $location
        website: $website
        isPrivate: $isPrivate
      ) {
        id
        username
        displayName
        bio
        avatarUrl
        headerImageUrl
        location
        website
        isPrivate
      }
    }
  `,

    CREATE_STORY: `
    mutation CreateStory($mediaUrl: String!, $caption: String) {
      createStory(mediaUrl: $mediaUrl, caption: $caption) {
        id
        mediaUrl
        caption
        createdAt
        expiresAt
        isExpired
        viewsCount
        author {
          id
          username
          displayName
          avatarUrl
        }
      }
    }
  `,

    VIEW_STORY: `
    mutation ViewStory($storyId: ID!) {
      viewStory(storyId: $storyId)
    }
  `,

    DELETE_STORY: `
    mutation DeleteStory($storyId: ID!) {
      deleteStory(storyId: $storyId)
    }
  `,

    CREATE_POST: `
    mutation CreatePost(
      $content: String!
      $media: [MediaInput!]
      $poll: CreatePollInput
      $quotePostId: ID
      $audience: PostAudienceGql
    ) {
      createPost(
        content: $content
        media: $media
        poll: $poll
        quotePostId: $quotePostId
        audience: $audience
      ) {
        ${POST_FIELDS}
      }
    }
  `,

    DELETE_POST: `
    mutation DeletePost($postId: ID!) {
      deletePost(postId: $postId)
    }
  `,

    REPOST_POST: `
    mutation RepostPost($postId: ID!) {
      repostPost(postId: $postId) {
        id
        repostsCount
        isRepostedByMe
      }
    }
  `,

    UNREPOST_POST: `
    mutation UnrepostPost($postId: ID!) {
      unrepostPost(postId: $postId) {
        id
        repostsCount
        isRepostedByMe
      }
    }
  `,

    QUOTE_POST: `
    mutation QuotePost($postId: ID!, $content: String!) {
      quotePost(postId: $postId, content: $content) {
        ${POST_FIELDS}
      }
    }
  `,

    VOTE_POLL: `
    mutation VotePoll($pollId: ID!, $optionId: ID!) {
      votePoll(pollId: $pollId, optionId: $optionId) {
        id
        optionText
        votesCount
        percentage
        isVotedByMe
      }
    }
  `,

    RECORD_POST_VIEW: `
    mutation RecordPostView($postId: ID!) {
      recordPostView(postId: $postId)
    }
  `,

    BOOKMARK_POST: `
    mutation BookmarkPost($postId: ID!) {
      bookmarkPost(postId: $postId)
    }
  `,

    UNBOOKMARK_POST: `
    mutation UnbookmarkPost($postId: ID!) {
      unbookmarkPost(postId: $postId)
    }
  `,

    CREATE_BOOKMARK_COLLECTION: `
    mutation CreateBookmarkCollection($name: String!, $description: String, $isPrivate: Boolean) {
      createBookmarkCollection(name: $name, description: $description, isPrivate: $isPrivate) {
        id
        name
        description
        isPrivate
      }
    }
  `,

    ADD_POST_TO_COLLECTION: `
    mutation AddPostToCollection($collectionId: ID!, $postId: ID!) {
      addPostToCollection(collectionId: $collectionId, postId: $postId)
    }
  `,

    CREATE_USER_LIST: `
    mutation CreateUserList($name: String!, $description: String, $isPrivate: Boolean, $memberIds: [ID!]) {
      createUserList(name: $name, description: $description, isPrivate: $isPrivate, memberIds: $memberIds) {
        id
        name
        description
        isPrivate
        membersCount
      }
    }
  `,

    ADD_USER_TO_LIST: `
    mutation AddUserToList($listId: ID!, $userId: ID!) {
      addUserToList(listId: $listId, userId: $userId)
    }
  `,

    CREATE_COMMENT: `
    mutation CreateComment($postId: ID!, $content: String!, $parentId: ID) {
      createComment(postId: $postId, content: $content, parentId: $parentId) {
        id
        content
        parentId
        depth
        isEdited
        likesCount
        isLikedByMe
        createdAt
        author {
          id
          username
          displayName
          avatarUrl
        }
      }
    }
  `,

    LIKE_COMMENT: `
    mutation LikeComment($commentId: ID!) {
      likeComment(commentId: $commentId) {
        id
        likesCount
        isLikedByMe
      }
    }
  `,

    UNLIKE_COMMENT: `
    mutation UnlikeComment($commentId: ID!) {
      unlikeComment(commentId: $commentId) {
        id
        likesCount
        isLikedByMe
      }
    }
  `,

    PIN_COMMENT: `
    mutation PinComment($postId: ID!, $commentId: ID!) {
      pinComment(postId: $postId, commentId: $commentId) {
        id
        pinnedComment {
          id
          content
          author {
            displayName
            username
          }
        }
      }
    }
  `,

    UNPIN_COMMENT: `
    mutation UnpinComment($postId: ID!) {
      unpinComment(postId: $postId) {
        id
        pinnedComment {
          id
        }
      }
    }
  `,

    REPORT_CONTENT: `
    mutation ReportContent($targetType: ReportTargetTypeGql!, $targetId: ID!, $reason: ReportReasonGql!, $details: String) {
      reportContent(targetType: $targetType, targetId: $targetId, reason: $reason, details: $details) {
        id
        status
        createdAt
      }
    }
  `,

    BLOCK_USER: `
    mutation BlockUser($userId: ID!) {
      blockUser(userId: $userId)
    }
  `,

    UNBLOCK_USER: `
    mutation UnblockUser($userId: ID!) {
      unblockUser(userId: $userId)
    }
  `,

    MUTE_USER: `
    mutation MuteUser($userId: ID!) {
      muteUser(userId: $userId)
    }
  `,

    UNMUTE_USER: `
    mutation UnmuteUser($userId: ID!) {
      unmuteUser(userId: $userId)
    }
  `,

    ACCEPT_FOLLOW_REQUEST: `
    mutation AcceptFollowRequest($requestId: ID!) {
      acceptFollowRequest(requestId: $requestId)
    }
  `,

    REJECT_FOLLOW_REQUEST: `
    mutation RejectFollowRequest($requestId: ID!) {
      rejectFollowRequest(requestId: $requestId)
    }
  `,

    SETUP_2FA: `
    mutation Setup2fa {
      setup2fa {
        secret
        otpauthUri
      }
    }
  `,

    ENABLE_2FA: `
    mutation Enable2fa($code: String!) {
      enable2fa(code: $code)
    }
  `,

    DISABLE_2FA: `
    mutation Disable2fa($code: String!) {
      disable2fa(code: $code)
    }
  `,

    LIKE_POST: `
    mutation LikePost($postId: ID!) {
      likePost(postId: $postId) {
        id
        likesCount
        isLikedBy
        isLikedByMe
      }
    }
  `,

    UNLIKE_POST: `
    mutation UnlikePost($postId: ID!) {
      unlikePost(postId: $postId) {
        id
        likesCount
        isLikedBy
        isLikedByMe
      }
    }
  `,

    FOLLOW_USER: `
    mutation FollowUser($followeeId: ID!) {
      followUser(followeeId: $followeeId) {
        id
        followersCount
        isFollowedByMe
        hasPendingFollowRequest
      }
    }
  `,

    UNFOLLOW_USER: `
    mutation UnfollowUser($followeeId: ID!) {
      unfollowUser(followeeId: $followeeId) {
        id
        followersCount
        isFollowedByMe
      }
    }
  `,

    SEND_DIRECT_MESSAGE: `
    mutation SendDirectMessage($recipientId: ID!, $content: String!) {
      sendDirectMessage(recipientId: $recipientId, content: $content) {
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

    MARK_MESSAGES_AS_READ: `
    mutation MarkMessagesAsRead($senderId: ID!) {
      markMessagesAsRead(senderId: $senderId)
    }
  `,

    MARK_NOTIFICATIONS_AS_READ: `
    mutation MarkNotificationsAsRead {
      markNotificationsAsRead
    }
  `,
};
