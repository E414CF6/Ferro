// Reusable Post Fields Selection Fragment
export const POST_FIELDS = `
  id
  content
  createdAt
  likesCount
  repostsCount
  isLikedBy
  isLikedByMe
  isRepostedByMe
  isBookmarkedByMe
  viewsCount
  audience
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
  comments {
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
