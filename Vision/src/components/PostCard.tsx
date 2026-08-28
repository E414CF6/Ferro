"use client";

import React, {useState} from "react";
import Link from "next/link";
import {Comment, Post} from "@/lib/types";
import {useAuth} from "@/lib/auth-context";
import {fetchGraphQL, MUTATIONS} from "@/lib/graphql";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import {Heart, Link as LinkIcon, MessageCircle, MoreHorizontal, Send, Share2, Trash2, X,} from "lucide-react";

interface PostCardProps {
    post: Post;
    onPostDeleted?: () => void;
}

export default function PostCard({post, onPostDeleted}: PostCardProps) {
    const {user} = useAuth();
    const {showToast} = useToast();

    const [isLiked, setIsLiked] = useState(post.isLikedByMe ?? post.isLikedBy ?? false);
    const [likesCount, setLikesCount] = useState(post.likesCount || 0);
    const [likeLoading, setLikeLoading] = useState(false);

    const [showComments, setShowComments] = useState(false);
    const [comments, setComments] = useState<Comment[]>(post.comments || []);
    const [newComment, setNewComment] = useState("");
    const [commentLoading, setCommentLoading] = useState(false);
    const [previewImage, setPreviewImage] = useState<string | null>(null);
    const [showMenu, setShowMenu] = useState(false);
    const [deleteLoading, setDeleteLoading] = useState(false);

    const isAuthor = user && user.username === post.author.username;
    const defaultAvatar = "https://api.dicebear.com/7.x/bottts/svg?seed=" + post.author.username;

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

    const handleLikeToggle = async () => {
        if (!user) {
            showToast("좋아요를 누르려면 먼저 로그인하세요.", "info");
            return;
        }
        if (likeLoading) return;

        setLikeLoading(true);
        const prevLiked = isLiked;
        const prevCount = likesCount;

        // Optimistic UI update
        setIsLiked(!prevLiked);
        setLikesCount(prevLiked ? Math.max(0, prevCount - 1) : prevCount + 1);

        try {
            if (prevLiked) {
                await fetchGraphQL(MUTATIONS.UNLIKE_POST, {postId: post.id});
            } else {
                await fetchGraphQL(MUTATIONS.LIKE_POST, {postId: post.id});
            }
        } catch (err: any) {
            setIsLiked(prevLiked);
            setLikesCount(prevCount);
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLikeLoading(false);
        }
    };

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
                }
            );

            if (data?.createComment) {
                setComments((prev) => [...prev, data.createComment]);
                setNewComment("");
                showToast("댓글이 등록되었습니다.", "success");
            }
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setCommentLoading(false);
        }
    };

    const handleDeletePost = async () => {
        if (!window.confirm("정말로 이 게시물을 삭제하시겠습니까?")) return;

        setDeleteLoading(true);
        try {
            await fetchGraphQL(MUTATIONS.DELETE_POST, {postId: post.id});
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
            showToast("게시물 프로필 링크가 클립보드에 복사되었습니다.", "success");
            setShowMenu(false);
        }
    };

    // Split content and image
    const contentParts = post.content.split(/\n\n(data:image\/[a-zA-Z]+;base64,[^\s]+|https?:\/\/[^\s]+)/);
    const textContent = contentParts[0] || post.content;
    const imageMatch = post.imageUrl || post.content.match(/(data:image\/[a-zA-Z]+;base64,[^\s]+|https?:\/\/[^\s]+\.(?:png|jpg|jpeg|gif|webp|svg))/i);
    const mediaUrl = post.imageUrl || (imageMatch ? imageMatch[0] : null);

    // Render text with clickable hashtags
    const renderFormattedText = (text: string) => {
        const parts = text.split(/(#[\w가-힣]+|@[\w_]+)/g);
        return parts.map((part, i) => {
            if (part.startsWith("#")) {
                const tag = part.substring(1);
                return (
                    <Link
                        key={i}
                        href={`/explore?q=%23${encodeURIComponent(tag)}`}
                        style={{color: "var(--accent-primary)", fontWeight: 600}}
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
                        style={{color: "var(--accent-purple)", fontWeight: 600}}
                        onClick={(e) => e.stopPropagation()}
                    >
                        {part}
                    </Link>
                );
            }
            return part;
        });
    };

    return (
        <>
            <article className="post-card">
                {/* Author Avatar with story ring indicator */}
                <Link
                    href={`/profile/${post.author.username}`}
                    style={{flexShrink: 0}}
                >
                    <div
                        className={`story-avatar-wrapper ${post.author.hasActiveStories ? "unread" : ""}`}
                        style={{width: 44, height: 44, padding: post.author.hasActiveStories ? "2px" : "0"}}
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

                {/* Post Content Area */}
                <div className="post-main">
                    {/* Header */}
                    <div className="post-header">
                        <div className="post-author-info">
                            <Link href={`/profile/${post.author.username}`} className="post-author-name">
                                {post.author.displayName || post.author.username}
                            </Link>
                            <span className="post-author-handle">@{post.author.username}</span>
                            <span style={{color: "var(--text-muted)", fontSize: "12px"}}>·</span>
                            <time className="post-date">{timeAgo(post.createdAt)}</time>
                        </div>

                        {/* Menu */}
                        <div style={{position: "relative"}}>
                            <button
                                type="button"
                                onClick={() => setShowMenu(!showMenu)}
                                style={{color: "var(--text-muted)", padding: "4px", borderRadius: "50%"}}
                                aria-label="게시물 메뉴"
                            >
                                <MoreHorizontal size={16}/>
                            </button>

                            {showMenu && (
                                <div
                                    style={{
                                        position: "absolute",
                                        right: 0,
                                        top: "100%",
                                        zIndex: 20,
                                        backgroundColor: "var(--bg-surface)",
                                        border: "1px solid var(--border-subtle)",
                                        borderRadius: "var(--radius-md)",
                                        boxShadow: "var(--shadow-lg)",
                                        minWidth: "150px",
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
                                        <LinkIcon size={14}/> 링크 복사
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
                                            <Trash2 size={14}/> 게시물 삭제
                                        </button>
                                    )}
                                </div>
                            )}
                        </div>
                    </div>

                    {/* Post Text */}
                    <div className="post-text">
                        {renderFormattedText(textContent)}
                    </div>

                    {/* Media image if present */}
                    {mediaUrl && (
                        <div
                            onClick={() => setPreviewImage(mediaUrl)}
                            style={{
                                position: "relative",
                                maxHeight: "420px",
                                borderRadius: "var(--radius-md)",
                                overflow: "hidden",
                                border: "1px solid var(--border-subtle)",
                                marginBottom: "14px",
                                cursor: "pointer",
                                backgroundColor: "#000",
                            }}
                        >
                            <img
                                src={mediaUrl}
                                alt="Post Media"
                                style={{
                                    width: "100%",
                                    maxHeight: "420px",
                                    objectFit: "cover",
                                    display: "block",
                                    transition: "transform 0.2s ease",
                                }}
                                onMouseEnter={(e) => (e.currentTarget.style.transform = "scale(1.01)")}
                                onMouseLeave={(e) => (e.currentTarget.style.transform = "scale(1)")}
                            />
                        </div>
                    )}

                    {/* Interactions */}
                    <div className="post-interactions">
                        {/* Like */}
                        <button
                            type="button"
                            onClick={handleLikeToggle}
                            className={`interaction-btn ${isLiked ? "liked" : ""}`}
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

                        {/* Comments Toggle */}
                        <button
                            type="button"
                            onClick={() => setShowComments(!showComments)}
                            className="interaction-btn"
                        >
                            <MessageCircle size={18}/>
                            <span>{comments.length > 0 ? comments.length : ""}</span>
                        </button>

                        {/* Share */}
                        <button
                            type="button"
                            onClick={handleCopyLink}
                            className="interaction-btn"
                            title="공유"
                        >
                            <Share2 size={17}/>
                        </button>
                    </div>

                    {/* Comments Section Drawer */}
                    {showComments && (
                        <div className="comments-section">
                            {comments.map((c) => (
                                <div key={c.id} className="comment-item">
                                    <img
                                        src={c.author.avatarUrl || "https://api.dicebear.com/7.x/bottts/svg?seed=" + c.author.username}
                                        alt={c.author.username}
                                        style={{width: 28, height: 28, borderRadius: "50%", objectFit: "cover"}}
                                    />
                                    <div className="comment-content">
                                        <Link
                                            href={`/profile/${c.author.username}`}
                                            className="comment-author-name"
                                        >
                                            {c.author.displayName || c.author.username}
                                        </Link>
                                        <span className="comment-text">{c.content}</span>
                                    </div>
                                </div>
                            ))}

                            {user ? (
                                <form onSubmit={handleAddComment} className="comment-input-row">
                                    <input
                                        type="text"
                                        placeholder="댓글 작성하기..."
                                        value={newComment}
                                        onChange={(e) => setNewComment(e.target.value)}
                                        className="comment-input"
                                    />
                                    <button
                                        type="submit"
                                        disabled={!newComment.trim() || commentLoading}
                                        className="btn-primary"
                                        style={{padding: "6px 14px", fontSize: "12px"}}
                                    >
                                        <Send size={12}/>
                                    </button>
                                </form>
                            ) : (
                                <div style={{fontSize: "12px", color: "var(--text-muted)", padding: "4px"}}>
                                    <Link href="/welcome?tab=login"
                                          style={{color: "var(--accent-primary)"}}>로그인</Link> 후 댓글을 작성할 수 있습니다.
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
                    style={{zIndex: 200, padding: "20px"}}
                >
                    <div style={{position: "relative", maxWidth: "90vw", maxHeight: "90vh"}}>
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
                            <X size={20}/>
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
        </>
    );
}
