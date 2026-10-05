// Reusable Post Fields Selection Fragment compatible with Serve GraphQL schema
export const POST_FIELDS = `
  id
  content
  createdAt
  likesCount
  repostsCount
  isLikedBy
  isRepostedByMe
  isSavedByMe
  viewsCount
  audience
  hashtags
  author {
    id
    username
    displayName
    avatarUrl
    hasActiveStories
    isPrivate
  }
  media {
    id
    mediaUrl
    mediaType
    altText
    sortOrder
  }
  poll {
    id
    question
    isExpired
    totalVotes
    userVotedOptionId
    options {
      id
      optionText
      votesCount
      percentage
      isVotedByMe
    }
  }
  quotePost {
    id
    content
    createdAt
    author {
      id
      username
      displayName
      avatarUrl
    }
    media {
      id
      mediaUrl
      mediaType
    }
  }
  pinnedComment {
    id
    content
    createdAt
    author {
      id
      username
      displayName
      avatarUrl
    }
  }
  comments(topLevelOnly: false) {
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
`;
