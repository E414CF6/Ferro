"use client";

import React, { useEffect, useState } from "react";
import Link from "next/link";
import { Comment, Poll, PollOption, Post } from "@/lib/types";
import { useAuth } from "@/lib/auth-context";
import { fetchGraphQL, MUTATIONS } from "@/lib/graphql";
import { useToast } from "@/lib/toast-context";
import { formatErrorMessage } from "@/lib/i18n";
import QuoteModal from "./QuoteModal";
import ReportModal from "./ReportModal";
import PostAnalyticsModal from "./PostAnalyticsModal";
import {
    BarChart3,
    Bookmark,
    CheckCircle2,
    CornerDownRight,
    Heart,
    Link as LinkIcon,
    Lock,
    MessageCircle,
    MoreHorizontal,
    Pin,
    Quote,
    Repeat,
    Send,
    Share2,
    ShieldAlert,
    Trash2,
    Users,
    X,
} from "lucide-react";

interface PostCardProps {
    post: Post;
    onPostDeleted?: () => void;
    onPostUpdated?: () => void;
}

export default function PostCard({ post, onPostDeleted, onPostUpdated }: PostCardProps) {
    const { user } = useAuth();
    const { showToast } = useToast();

    // Likes
    const [isLiked, setIsLiked] = useState(post.isLikedByMe ?? post.isLikedBy ?? false);
    const [likesCount, setLikesCount] = useState(post.likesCount || 0);
    const [likeLoading, setLikeLoading] = useState(false);

    // Reposts
    const [isReposted, setIsReposted] = useState(post.isRepostedByMe || false);
    const [repostsCount, setRepostsCount] = useState(post.repostsCount || 0);
    const [repostLoading, setRepostLoading] = useState(false);

    // Bookmarks (Saved)
    const [isSaved, setIsSaved] = useState(post.isSavedByMe ?? post.isBookmarkedByMe ?? false);
    const [saveLoading, setSaveLoading] = useState(false);

    // Poll state
    const [poll, setPoll] = useState<Poll | null | undefined>(post.poll);
    const [votingOptionId, setVotingOptionId] = useState<string | null>(null);

    // Comments & Replies
    const [showComments, setShowComments] = useState(false);
    const [comments, setComments] = useState<Comment[]>(post.comments || []);
    const [newComment, setNewComment] = useState("");
    const [replyToComment, setReplyToComment] = useState<Comment | null>(null);
    const [commentLoading, setCommentLoading] = useState(false);
    const [pinnedComment, setPinnedComment] = useState<Comment | null | undefined>(post.pinnedComment);

    // Modals
    const [showQuoteModal, setShowQuoteModal] = useState(false);
    const [showReportModal, setShowReportModal] = useState(false);
    const [showAnalyticsModal, setShowAnalyticsModal] = useState(false);
    const [previewImage, setPreviewImage] = useState<string | null>(null);
    const [showMenu, setShowMenu] = useState(false);
    const [deleteLoading, setDeleteLoading] = useState(false);

    const isAuthor = user && user.username === post.author.username;
    const defaultAvatar = "https://api.dicebear.com/7.x/bottts/svg?seed=" + post.author.username;

    // Record post impression view on mount
    useEffect(() => {
        fetchGraphQL(MUTATIONS.RECORD_POST_VIEW, { postId: post.id }).catch(() => null);
    }, [post.id]);

    const timeAgo = (dateStr: string) => {
        const diff = Date.now() - new Date(dateStr).getTime();
        const mins = Math.floor(diff / (1000 * 60));
        if (mins < 1) return "방금 전";
        if (mins < 60) return `${mins}분 전`;
        const hours = Math.floor(mins / 60);
        if (hours < 24) return `${hours}시간 전`;
        const days = Math.floor(hours / 24);
        return `${days}일 전`;
    };

    // Toggle Like
    const handleLikeToggle = async () => {
        if (!user) {
            showToast("좋아요를 누르려면 먼저 로그인하세요.", "info");
            return;
        }
        if (likeLoading) return;

        setLikeLoading(true);
        const prevLiked = isLiked;
        const prevCount = likesCount;

        setIsLiked(!prevLiked);
        setLikesCount(prevLiked ? Math.max(0, prevCount - 1) : prevCount + 1);

        try {
            if (prevLiked) {
                await fetchGraphQL(MUTATIONS.UNLIKE_POST, { postId: post.id });
            } else {
                await fetchGraphQL(MUTATIONS.LIKE_POST, { postId: post.id });
            }
        } catch (err: any) {
            setIsLiked(prevLiked);
            setLikesCount(prevCount);
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLikeLoading(false);
        }
    };

    // Toggle Repost
    const handleRepostToggle = async () => {
        if (!user) {
            showToast("리포스트하려면 먼저 로그인하세요.", "info");
            return;
        }
        if (repostLoading) return;

        setRepostLoading(true);
        const prevReposted = isReposted;
        const prevCount = repostsCount;

        setIsReposted(!prevReposted);
        setRepostsCount(prevReposted ? Math.max(0, prevCount - 1) : prevCount + 1);

        try {
            if (prevReposted) {
                await fetchGraphQL(MUTATIONS.UNREPOST_POST, { postId: post.id });
                showToast("리포스트를 취소했습니다.", "info");
            } else {
                await fetchGraphQL(MUTATIONS.REPOST_POST, { postId: post.id });
                showToast("게시물을 리포스트했습니다!", "success");
            }
        } catch (err: any) {
            setIsReposted(prevReposted);
            setRepostsCount(prevCount);
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setRepostLoading(false);
        }
    };

    // Toggle Save / Bookmark
    const handleSaveToggle = async () => {
        if (!user) {
            showToast("게시물을 저장하려면 먼저 로그인하세요.", "info");
            return;
        }
        if (saveLoading) return;

        setSaveLoading(true);
        const prevSaved = isSaved;
        setIsSaved(!prevSaved);

        try {
            if (prevSaved) {
                await fetchGraphQL(MUTATIONS.UNSAVE_POST, { postId: post.id });
                showToast("북마크에서 제거되었습니다.", "info");
            } else {
                await fetchGraphQL(MUTATIONS.SAVE_POST, { postId: post.id });
                showToast("북마크에 저장되었습니다!", "success");
            }
        } catch (err: any) {
            setIsSaved(prevSaved);
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setSaveLoading(false);
        }
    };

    // Vote on Poll
    const handleVotePoll = async (optionId: string) => {
        if (!user) {
            showToast("투표하려면 먼저 로그인하세요.", "info");
            return;
        }
        if (!poll || poll.isExpired || poll.userVotedOptionId || votingOptionId) return;

        setVotingOptionId(optionId);
        try {
            const data = await fetchGraphQL<{ votePoll: PollOption }>(MUTATIONS.VOTE_POLL, {
                pollId: poll.id,
                optionId,
            });

            if (data?.votePoll) {
                showToast("투표가 정상 반영되었습니다!", "success");
                setPoll((prev) => {
                    if (!prev) return prev;
                    const newTotal = prev.totalVotes + 1;
                    const updatedOptions = prev.options.map((opt) => {
                        const count = opt.id === optionId ? opt.votesCount + 1 : opt.votesCount;
                        const pct = newTotal > 0 ? parseFloat(((count / newTotal) * 100).toFixed(1)) : 0;
                        return {
                            ...opt,
                            votesCount: count,
                            percentage: pct,
                            isVotedByMe: opt.id === optionId,
                        };
                    });
                    return {
                        ...prev,
                        totalVotes: newTotal,
                        userVotedOptionId: optionId,
                        options: updatedOptions,
                    };
                });
            }
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setVotingOptionId(null);
        }
    };

    // Create Comment / Reply
    const handleAddComment = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!user) {
            showToast("댓글을 작성하려면 먼저 로그인하세요.", "info");
            return;
        }
        if (!newComment.trim() || commentLoading) return;

        setCommentLoading(true);
        try {
            const data = await fetchGraphQL<{ createComment: Comment }>(
                MUTATIONS.CREATE_COMMENT,
                {
                    postId: post.id,
                    content: newComment.trim(),
                    parentId: replyToComment ? replyToComment.id : undefined,
                }
            );

            if (data?.createComment) {
                setComments((prev) => [...prev, data.createComment]);
                setNewComment("");
                setReplyToComment(null);
                showToast("댓글이 등록되었습니다.", "success");
            }
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setCommentLoading(false);
        }
    };

    // Like Comment
    const handleCommentLike = async (commentId: string, currentLiked: boolean) => {
        if (!user) {
            showToast("로그인이 필요합니다.", "info");
            return;
        }

        setComments((prev) =>
            prev.map((c) =>
                c.id === commentId
                    ? {
                          ...c,
                          isLikedByMe: !currentLiked,
                          likesCount: (c.likesCount || 0) + (currentLiked ? -1 : 1),
                      }
                    : c
            )
        );

        try {
            if (currentLiked) {
                await fetchGraphQL(MUTATIONS.UNLIKE_COMMENT, { commentId });
            } else {
                await fetchGraphQL(MUTATIONS.LIKE_COMMENT, { commentId });
            }
        } catch (err) {
            // rollback
            setComments((prev) =>
                prev.map((c) =>
                    c.id === commentId
                        ? {
                              ...c,
                              isLikedByMe: currentLiked,
                              likesCount: (c.likesCount || 0) + (currentLiked ? 1 : -1),
                          }
                        : c
                )
            );
        }
    };

    // Pin Comment
    const handlePinComment = async (commentId: string) => {
        try {
            if (pinnedComment?.id === commentId) {
                await fetchGraphQL(MUTATIONS.UNPIN_COMMENT, { postId: post.id });
                setPinnedComment(null);
                showToast("댓글 고정을 해제했습니다.", "info");
            } else {
                await fetchGraphQL(MUTATIONS.PIN_COMMENT, { postId: post.id, commentId });
                const found = comments.find((c) => c.id === commentId);
                setPinnedComment(found || null);
                showToast("댓글을 상단에 고정했습니다.", "success");
            }
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        }
    };

    // Delete Comment
    const handleDeleteComment = async (commentId: string) => {
        if (!window.confirm("댓글을 삭제하시겠습니까?")) return;
        try {
            await fetchGraphQL(MUTATIONS.DELETE_COMMENT, { commentId });
            setComments((prev) => prev.filter((c) => c.id !== commentId));
            if (pinnedComment?.id === commentId) setPinnedComment(null);
            showToast("댓글이 삭제되었습니다.", "info");
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        }
    };

    // Delete Post
    const handleDeletePost = async () => {
        if (!window.confirm("정말로 이 게시물을 삭제하시겠습니까?")) return;

        setDeleteLoading(true);
        try {
            await fetchGraphQL(MUTATIONS.DELETE_POST, { postId: post.id });
            showToast("게시물이 삭제되었습니다.", "info");
            onPostDeleted?.();
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setDeleteLoading(false);
            setShowMenu(false);
        }
    };

    const handleCopyLink = () => {
        if (typeof window !== "undefined") {
            navigator.clipboard.writeText(`${window.location.origin}/profile/${post.author.username}`);
            showToast("게시물 링크가 클립보드에 복사되었습니다.", "success");
            setShowMenu(false);
        }
    };

    // Text & Hashtag rendering
    const renderFormattedText = (text: string) => {
        const parts = text.split(/(#[\w가-힣]+|@[\w_]+)/g);
        return parts.map((part, i) => {
            if (part.startsWith("#")) {
                const tag = part.substring(1);
                return (
                    <Link
                        key={i}
                        href={`/explore?q=%23${encodeURIComponent(tag)}`}
                        style={{ color: "var(--accent-primary)", fontWeight: 600 }}
                        onClick={(e) => e.stopPropagation()}
                    >
                        {part}
                    </Link>
                );
            }
            if (part.startsWith("@")) {
                const handle = part.substring(1);
                return (
                    <Link
                        key={i}
                        href={`/profile/${handle}`}
                        style={{ color: "var(--accent-purple)", fontWeight: 600 }}
                        onClick={(e) => e.stopPropagation()}
                    >
                        {part}
                    </Link>
                );
            }
            return part;
        });
    };

    // Resolve media images
    const mediaList = post.media && post.media.length > 0
        ? post.media.map((m) => m.mediaUrl)
        : post.imageUrl
        ? [post.imageUrl]
        : [];

    return (
        <>
            <article className="post-card">
                {/* Author Avatar with story indicator */}
                <Link href={`/profile/${post.author.username}`} style={{ flexShrink: 0 }}>
                    <div
                        className={`story-avatar-wrapper ${post.author.hasActiveStories ? "unread" : ""}`}
                        style={{ width: 44, height: 44, padding: post.author.hasActiveStories ? "2px" : "0" }}
                    >
                        <div className="story-avatar-inner">
                            <img
                                src={post.author.avatarUrl || defaultAvatar}
                                alt={post.author.username}
                                className="story-avatar-img"
                            />
                        </div>
                    </div>
                </Link>

                {/* Main Content Area */}
                <div className="post-main">
                    {/* Header */}
                    <div className="post-header">
                        <div className="post-author-info">
                            <Link href={`/profile/${post.author.username}`} className="post-author-name">
                                {post.author.displayName || post.author.username}
                            </Link>
                            <span className="post-author-handle">@{post.author.username}</span>
                            <span style={{ color: "var(--text-muted)", fontSize: "12px" }}>·</span>
                            <time className="post-date">{timeAgo(post.createdAt)}</time>

                            {/* Audience Scope Badge */}
                            {post.audience === "FOLLOWERS_ONLY" && (
                                <span
                                    title="팔로워에게만 공개된 게시물"
                                    style={{
                                        display: "inline-flex",
                                        alignItems: "center",
                                        gap: "3px",
                                        fontSize: "10px",
                                        padding: "1px 6px",
                                        borderRadius: "var(--radius-full)",
                                        backgroundColor: "rgba(168, 85, 247, 0.15)",
                                        color: "var(--accent-purple)",
                                        fontWeight: 700,
                                    }}
                                >
                                    <Users size={10} /> 팔로워
                                </span>
                            )}
                            {post.audience === "CLOSE_FRIENDS" && (
                                <span
                                    title="친한 친구에게만 공개"
                                    style={{
                                        display: "inline-flex",
                                        alignItems: "center",
                                        gap: "3px",
                                        fontSize: "10px",
                                        padding: "1px 6px",
                                        borderRadius: "var(--radius-full)",
                                        backgroundColor: "rgba(16, 185, 129, 0.15)",
                                        color: "#10b981",
                                        fontWeight: 700,
                                    }}
                                >
                                    <Lock size={10} /> 친한 친구
                                </span>
                            )}
                        </div>

                        {/* More Menu */}
                        <div style={{ position: "relative" }}>
                            <button
                                type="button"
                                onClick={() => setShowMenu(!showMenu)}
                                style={{ color: "var(--text-muted)", padding: "4px", borderRadius: "50%" }}
                                aria-label="게시물 메뉴"
                            >
                                <MoreHorizontal size={16} />
                            </button>

                            {showMenu && (
                                <div
                                    style={{
                                        position: "absolute",
                                        right: 0,
                                        top: "100%",
                                        zIndex: 30,
                                        backgroundColor: "var(--bg-surface)",
                                        border: "1px solid var(--border-subtle)",
                                        borderRadius: "var(--radius-md)",
                                        boxShadow: "var(--shadow-lg)",
                                        minWidth: "160px",
                                        overflow: "hidden",
                                    }}
                                >
                                    <button
                                        type="button"
                                        onClick={handleCopyLink}
                                        style={{
                                            display: "flex",
                                            alignItems: "center",
                                            gap: "8px",
                                            width: "100%",
                                            padding: "10px 14px",
                                            fontSize: "13px",
                                            color: "var(--text-primary)",
                                            textAlign: "left",
                                        }}
                                        onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = "var(--bg-surface-hover)")}
                                        onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
                                    >
                                        <LinkIcon size={14} /> 링크 복사
                                    </button>

                                    <button
                                        type="button"
                                        onClick={() => {
                                            setShowAnalyticsModal(true);
                                            setShowMenu(false);
                                        }}
                                        style={{
                                            display: "flex",
                                            alignItems: "center",
                                            gap: "8px",
                                            width: "100%",
                                            padding: "10px 14px",
                                            fontSize: "13px",
                                            color: "var(--text-primary)",
                                            textAlign: "left",
                                            borderTop: "1px solid var(--border-subtle)",
                                        }}
                                        onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = "var(--bg-surface-hover)")}
                                        onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
                                    >
                                        <BarChart3 size={14} color="var(--accent-primary)" /> 통계 보기
                                    </button>

                                    <button
                                        type="button"
                                        onClick={() => {
                                            setShowReportModal(true);
                                            setShowMenu(false);
                                        }}
                                        style={{
                                            display: "flex",
                                            alignItems: "center",
                                            gap: "8px",
                                            width: "100%",
                                            padding: "10px 14px",
                                            fontSize: "13px",
                                            color: "var(--accent-secondary)",
                                            textAlign: "left",
                                            borderTop: "1px solid var(--border-subtle)",
                                        }}
                                        onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = "var(--bg-surface-hover)")}
                                        onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
                                    >
                                        <ShieldAlert size={14} /> 신고하기
                                    </button>

                                    {isAuthor && (
                                        <button
                                            type="button"
                                            onClick={handleDeletePost}
                                            disabled={deleteLoading}
                                            style={{
                                                display: "flex",
                                                alignItems: "center",
                                                gap: "8px",
                                                width: "100%",
                                                padding: "10px 14px",
                                                fontSize: "13px",
                                                color: "var(--accent-secondary)",
                                                textAlign: "left",
                                                borderTop: "1px solid var(--border-subtle)",
                                            }}
                                            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = "rgba(244, 63, 94, 0.1)")}
                                            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
                                        >
                                            <Trash2 size={14} /> 게시물 삭제
                                        </button>
                                    )}
                                </div>
                            )}
                        </div>
                    </div>

                    {/* Post Content */}
                    <div className="post-text">{renderFormattedText(post.content)}</div>

                    {/* Media Gallery / Grid */}
                    {mediaList.length > 0 && (
                        <div
                            style={{
                                display: "grid",
                                gridTemplateColumns: mediaList.length === 1 ? "1fr" : "repeat(2, 1fr)",
                                gap: "6px",
                                borderRadius: "var(--radius-md)",
                                overflow: "hidden",
                                marginBottom: "14px",
                                maxHeight: "420px",
                            }}
                        >
                            {mediaList.map((url, idx) => (
                                <div
                                    key={idx}
                                    onClick={() => setPreviewImage(url)}
                                    style={{
                                        position: "relative",
                                        maxHeight: "360px",
                                        cursor: "pointer",
                                        overflow: "hidden",
                                        backgroundColor: "#000",
                                    }}
                                >
                                    <img
                                        src={url}
                                        alt={`Media ${idx + 1}`}
                                        style={{
                                            width: "100%",
                                            height: "100%",
                                            objectFit: "cover",
                                            display: "block",
                                            transition: "transform 0.2s ease",
                                        }}
                                        onMouseEnter={(e) => (e.currentTarget.style.transform = "scale(1.02)")}
                                        onMouseLeave={(e) => (e.currentTarget.style.transform = "scale(1)")}
                                    />
                                </div>
                            ))}
                        </div>
                    )}

                    {/* Quoted Post Card */}
                    {post.quotePost && (
                        <div
                            style={{
                                padding: "12px 14px",
                                borderRadius: "var(--radius-md)",
                                border: "1px solid var(--border-subtle)",
                                backgroundColor: "var(--bg-surface)",
                                marginBottom: "14px",
                            }}
                        >
                            <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "6px" }}>
                                <img
                                    src={post.quotePost.author.avatarUrl || "https://api.dicebear.com/7.x/bottts/svg?seed=" + post.quotePost.author.username}
                                    alt={post.quotePost.author.username}
                                    style={{ width: 20, height: 20, borderRadius: "50%" }}
                                />
                                <span style={{ fontSize: "13px", fontWeight: 700, color: "var(--text-primary)" }}>
                                    {post.quotePost.author.displayName || post.quotePost.author.username}
                                </span>
                                <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                                    @{post.quotePost.author.username}
                                </span>
                            </div>
                            <div style={{ fontSize: "13px", color: "var(--text-secondary)", lineHeight: "1.5" }}>
                                {post.quotePost.content}
                            </div>
                        </div>
                    )}

                    {/* Interactive Poll */}
                    {poll && (
                        <div
                            style={{
                                padding: "14px",
                                borderRadius: "var(--radius-md)",
                                border: "1px solid var(--border-subtle)",
                                backgroundColor: "var(--bg-surface)",
                                marginBottom: "14px",
                            }}
                        >
                            <div style={{ fontSize: "14px", fontWeight: 700, color: "var(--text-primary)", marginBottom: "10px" }}>
                                {poll.question}
                            </div>

                            <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
                                {poll.options.map((option) => {
                                    const hasVoted = Boolean(poll.userVotedOptionId);
                                    const isSelected = poll.userVotedOptionId === option.id;

                                    return (
                                        <button
                                            key={option.id}
                                            type="button"
                                            disabled={hasVoted || poll.isExpired || Boolean(votingOptionId)}
                                            onClick={() => handleVotePoll(option.id)}
                                            style={{
                                                position: "relative",
                                                padding: "10px 14px",
                                                borderRadius: "var(--radius-md)",
                                                border: `1px solid ${isSelected ? "var(--accent-primary)" : "var(--border-subtle)"}`,
                                                backgroundColor: "var(--bg-input)",
                                                display: "flex",
                                                alignItems: "center",
                                                justifyContent: "space-between",
                                                overflow: "hidden",
                                                textAlign: "left",
                                                cursor: hasVoted || poll.isExpired ? "default" : "pointer",
                                            }}
                                        >
                                            {/* Percentage Fill Bar */}
                                            {hasVoted && (
                                                <div
                                                    style={{
                                                        position: "absolute",
                                                        left: 0,
                                                        top: 0,
                                                        bottom: 0,
                                                        width: `${option.percentage}%`,
                                                        backgroundColor: isSelected ? "rgba(56, 189, 248, 0.2)" : "rgba(255, 255, 255, 0.05)",
                                                        transition: "width 0.4s ease",
                                                    }}
                                                />
                                            )}

                                            <span style={{ position: "relative", fontSize: "13px", fontWeight: isSelected ? 700 : 500, color: "var(--text-primary)", display: "flex", alignItems: "center", gap: "6px" }}>
                                                {isSelected && <CheckCircle2 size={14} color="var(--accent-primary)" />}
                                                {option.optionText}
                                            </span>

                                            {hasVoted && (
                                                <span style={{ position: "relative", fontSize: "12px", fontWeight: 700, color: "var(--text-secondary)" }}>
                                                    {option.percentage}% ({option.votesCount}표)
                                                </span>
                                            )}
                                        </button>
                                    );
                                })}
                            </div>

                            <div style={{ marginTop: "10px", fontSize: "11px", color: "var(--text-muted)", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                                <span>총 {poll.totalVotes}표 참여</span>
                                <span>{poll.isExpired ? "투표 종료됨" : "진행 중인 투표"}</span>
                            </div>
                        </div>
                    )}

                    {/* Pinned Comment Highlight */}
                    {pinnedComment && (
                        <div
                            style={{
                                padding: "8px 12px",
                                borderRadius: "var(--radius-md)",
                                backgroundColor: "rgba(56, 189, 248, 0.07)",
                                border: "1px solid rgba(56, 189, 248, 0.2)",
                                marginBottom: "12px",
                                display: "flex",
                                alignItems: "center",
                                gap: "8px",
                                fontSize: "12px",
                            }}
                        >
                            <Pin size={13} color="var(--accent-primary)" style={{ flexShrink: 0 }} />
                            <div style={{ minWidth: 0, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                                <span style={{ fontWeight: 700, color: "var(--accent-primary)", marginRight: "6px" }}>
                                    고정된 댓글:
                                </span>
                                <span style={{ color: "var(--text-primary)" }}>{pinnedComment.content}</span>
                            </div>
                        </div>
                    )}

                    {/* Interactions Bar */}
                    <div className="post-interactions">
                        {/* Like */}
                        <button
                            type="button"
                            onClick={handleLikeToggle}
                            className={`interaction-btn ${isLiked ? "liked" : ""}`}
                            title="좋아요"
                        >
                            <Heart
                                size={18}
                                fill={isLiked ? "var(--accent-secondary)" : "none"}
                                color={isLiked ? "var(--accent-secondary)" : "currentColor"}
                                style={{
                                    transition: "transform 0.15s ease",
                                    transform: isLiked ? "scale(1.15)" : "scale(1)",
                                }}
                            />
                            <span>{likesCount > 0 ? likesCount : ""}</span>
                        </button>

                        {/* Repost */}
                        <button
                            type="button"
                            onClick={handleRepostToggle}
                            className={`interaction-btn ${isReposted ? "reposted" : ""}`}
                            title="리포스트"
                            style={{ color: isReposted ? "#10b981" : "currentColor" }}
                        >
                            <Repeat size={18} />
                            <span>{repostsCount > 0 ? repostsCount : ""}</span>
                        </button>

                        {/* Quote */}
                        <button
                            type="button"
                            onClick={() => setShowQuoteModal(true)}
                            className="interaction-btn"
                            title="게시물 인용"
                        >
                            <Quote size={17} />
                        </button>

                        {/* Comments Toggle */}
                        <button
                            type="button"
                            onClick={() => setShowComments(!showComments)}
                            className="interaction-btn"
                            title="댓글"
                        >
                            <MessageCircle size={18} />
                            <span>{comments.length > 0 ? comments.length : ""}</span>
                        </button>

                        {/* Bookmark / Save */}
                        <button
                            type="button"
                            onClick={handleSaveToggle}
                            className="interaction-btn"
                            title="북마크 저장"
                            style={{ color: isSaved ? "var(--accent-primary)" : "currentColor" }}
                        >
                            <Bookmark size={17} fill={isSaved ? "var(--accent-primary)" : "none"} />
                        </button>

                        {/* Share */}
                        <button
                            type="button"
                            onClick={handleCopyLink}
                            className="interaction-btn"
                            title="공유"
                        >
                            <Share2 size={17} />
                        </button>
                    </div>

                    {/* Comments & Replies Thread Section */}
                    {showComments && (
                        <div className="comments-section">
                            {comments.map((c) => (
                                <div
                                    key={c.id}
                                    className="comment-item"
                                    style={{
                                        marginLeft: c.parentId ? "24px" : "0px",
                                        position: "relative",
                                    }}
                                >
                                    {c.parentId && (
                                        <CornerDownRight
                                            size={14}
                                            color="var(--text-muted)"
                                            style={{ position: "absolute", left: "-18px", top: "8px" }}
                                        />
                                    )}

                                    <img
                                        src={c.author.avatarUrl || "https://api.dicebear.com/7.x/bottts/svg?seed=" + c.author.username}
                                        alt={c.author.username}
                                        style={{ width: 28, height: 28, borderRadius: "50%", objectFit: "cover" }}
                                    />

                                    <div className="comment-content" style={{ flex: 1 }}>
                                        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                                            <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                                                <Link href={`/profile/${c.author.username}`} className="comment-author-name">
                                                    {c.author.displayName || c.author.username}
                                                </Link>
                                                <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                                                    {timeAgo(c.createdAt)}
                                                </span>
                                                {pinnedComment?.id === c.id && (
                                                    <span style={{ fontSize: "10px", color: "var(--accent-primary)", display: "inline-flex", alignItems: "center", gap: "2px" }}>
                                                        <Pin size={10} /> 고정됨
                                                    </span>
                                                )}
                                            </div>

                                            {/* Comment action icons */}
                                            <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                                                <button
                                                    type="button"
                                                    onClick={() => handleCommentLike(c.id, Boolean(c.isLikedByMe))}
                                                    style={{
                                                        color: c.isLikedByMe ? "var(--accent-secondary)" : "var(--text-muted)",
                                                        display: "flex",
                                                        alignItems: "center",
                                                        gap: "3px",
                                                        fontSize: "11px",
                                                    }}
                                                >
                                                    <Heart size={12} fill={c.isLikedByMe ? "currentColor" : "none"} />
                                                    {c.likesCount ? c.likesCount : ""}
                                                </button>

                                                <button
                                                    type="button"
                                                    onClick={() => {
                                                        setReplyToComment(c);
                                                        setNewComment(`@${c.author.username} `);
                                                    }}
                                                    style={{ color: "var(--text-muted)", fontSize: "11px" }}
                                                >
                                                    답글
                                                </button>

                                                {isAuthor && (
                                                    <button
                                                        type="button"
                                                        onClick={() => handlePinComment(c.id)}
                                                        title="댓글 상단 고정"
                                                        style={{ color: pinnedComment?.id === c.id ? "var(--accent-primary)" : "var(--text-muted)" }}
                                                    >
                                                        <Pin size={12} />
                                                    </button>
                                                )}

                                                {user?.username === c.author.username && (
                                                    <button
                                                        type="button"
                                                        onClick={() => handleDeleteComment(c.id)}
                                                        title="댓글 삭제"
                                                        style={{ color: "var(--text-muted)" }}
                                                    >
                                                        <Trash2 size={12} />
                                                    </button>
                                                )}
                                            </div>
                                        </div>

                                        <span className="comment-text">{c.content}</span>
                                    </div>
                                </div>
                            ))}

                            {/* Comment / Reply Input Form */}
                            {user ? (
                                <form onSubmit={handleAddComment} className="comment-input-row" style={{ flexDirection: "column", alignItems: "stretch" }}>
                                    {replyToComment && (
                                        <div
                                            style={{
                                                fontSize: "11px",
                                                color: "var(--accent-purple)",
                                                display: "flex",
                                                alignItems: "center",
                                                justifyContent: "space-between",
                                                marginBottom: "4px",
                                            }}
                                        >
                                            <span>@{replyToComment.author.username} 님에게 답글 남기는 중</span>
                                            <button
                                                type="button"
                                                onClick={() => {
                                                    setReplyToComment(null);
                                                    setNewComment("");
                                                }}
                                                style={{ color: "var(--text-muted)" }}
                                            >
                                                <X size={12} />
                                            </button>
                                        </div>
                                    )}
                                    <div style={{ display: "flex", gap: "8px" }}>
                                        <input
                                            type="text"
                                            placeholder={replyToComment ? "답글 작성하기..." : "댓글 작성하기..."}
                                            value={newComment}
                                            onChange={(e) => setNewComment(e.target.value)}
                                            className="comment-input"
                                            style={{ flex: 1 }}
                                        />
                                        <button
                                            type="submit"
                                            disabled={!newComment.trim() || commentLoading}
                                            className="btn-primary"
                                            style={{ padding: "6px 14px", fontSize: "12px" }}
                                        >
                                            <Send size={12} />
                                        </button>
                                    </div>
                                </form>
                            ) : (
                                <div style={{ fontSize: "12px", color: "var(--text-muted)", padding: "4px" }}>
                                    <Link href="/login" style={{ color: "var(--accent-primary)" }}>
                                        로그인
                                    </Link> 후 댓글을 작성할 수 있습니다.
                                </div>
                            )}
                        </div>
                    )}
                </div>
            </article>

            {/* Lightbox image preview modal */}
            {previewImage && (
                <div
                    className="modal-overlay"
                    onClick={() => setPreviewImage(null)}
                    style={{ zIndex: 200, padding: "20px" }}
                >
                    <div style={{ position: "relative", maxWidth: "90vw", maxHeight: "90vh" }}>
                        <button
                            type="button"
                            onClick={() => setPreviewImage(null)}
                            style={{
                                position: "absolute",
                                top: -36,
                                right: 0,
                                color: "#fff",
                                backgroundColor: "rgba(0,0,0,0.6)",
                                padding: "6px",
                                borderRadius: "50%",
                            }}
                        >
                            <X size={20} />
                        </button>
                        <img
                            src={previewImage}
                            alt="Full Preview"
                            style={{
                                maxWidth: "100%",
                                maxHeight: "85vh",
                                borderRadius: "var(--radius-md)",
                                objectFit: "contain",
                            }}
                        />
                    </div>
                </div>
            )}

            {/* Modals */}
            {showQuoteModal && (
                <QuoteModal
                    post={post}
                    onClose={() => setShowQuoteModal(false)}
                    onPostCreated={onPostUpdated}
                />
            )}

            {showReportModal && (
                <ReportModal
                    targetType="POST"
                    targetId={post.id}
                    targetTitle={post.content.slice(0, 30)}
                    onClose={() => setShowReportModal(false)}
                />
            )}

            {showAnalyticsModal && (
                <PostAnalyticsModal
                    postId={post.id}
                    onClose={() => setShowAnalyticsModal(false)}
                />
            )}
        </>
    );
}
