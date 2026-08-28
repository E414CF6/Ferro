"use client";

import React, {useState} from "react";
import {Post} from "@/lib/types";
import {useAuth} from "@/lib/auth-context";
import {fetchGraphQL, MUTATIONS} from "@/lib/graphql";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import {Quote, Send, X} from "lucide-react";

interface QuoteModalProps {
    post: Post;
    onClose: () => void;
    onPostCreated?: () => void;
}

export default function QuoteModal({
                                       post,
                                       onClose,
                                       onPostCreated,
                                   }: QuoteModalProps) {
    const {user} = useAuth();
    const {showToast} = useToast();
    const [content, setContent] = useState("");
    const [loading, setLoading] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!content.trim()) return;

        setLoading(true);
        try {
            await fetchGraphQL(MUTATIONS.QUOTE_POST, {
                postId: post.id,
                content: content.trim(),
            });

            showToast("인용 게시물이 성공적으로 등록되었습니다!", "success");
            onPostCreated?.();
            onClose();
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLoading(false);
        }
    };

    const defaultAvatar =
        "https://api.dicebear.com/7.x/bottts/svg?seed=" + post.author.username;

    return (
        <div className="modal-overlay" onClick={onClose}>
            <div
                className="modal-content"
                onClick={(e) => e.stopPropagation()}
                style={{maxWidth: "520px"}}
            >
                <div className="modal-header">
                    <div style={{display: "flex", alignItems: "center", gap: "8px"}}>
                        <Quote size={20} color="var(--accent-primary)"/>
                        <h3 className="modal-title">게시물 인용하기</h3>
                    </div>
                    <button onClick={onClose} className="modal-close-btn">
                        <X size={18}/>
                    </button>
                </div>

                <form onSubmit={handleSubmit} style={{padding: "20px"}}>
          <textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              placeholder="이 게시물에 대한 생각을 덧붙여보세요..."
              rows={3}
              className="composer-textarea"
              style={{
                  width: "100%",
                  borderRadius: "var(--radius-md)",
                  marginBottom: "14px",
                  border: "1px solid var(--border-subtle)",
                  padding: "12px",
                  fontSize: "14px",
              }}
              autoFocus
          />

                    {/* Quoted Post Card Preview */}
                    <div
                        style={{
                            padding: "12px 14px",
                            borderRadius: "var(--radius-md)",
                            border: "1px solid var(--border-subtle)",
                            backgroundColor: "var(--bg-surface)",
                            marginBottom: "16px",
                        }}
                    >
                        <div
                            style={{
                                display: "flex",
                                alignItems: "center",
                                gap: "8px",
                                marginBottom: "6px",
                            }}
                        >
                            <img
                                src={post.author.avatarUrl || defaultAvatar}
                                alt={post.author.username}
                                style={{
                                    width: 20,
                                    height: 20,
                                    borderRadius: "50%",
                                    objectFit: "cover",
                                }}
                            />
                            <span
                                style={{
                                    fontSize: "13px",
                                    fontWeight: 700,
                                    color: "var(--text-primary)",
                                }}
                            >
                {post.author.displayName || post.author.username}
              </span>
                            <span style={{fontSize: "12px", color: "var(--text-muted)"}}>
                @{post.author.username}
              </span>
                        </div>
                        <p
                            style={{
                                fontSize: "13px",
                                color: "var(--text-secondary)",
                                lineHeight: "1.5",
                                whiteSpace: "pre-wrap",
                                overflow: "hidden",
                                textOverflow: "ellipsis",
                                display: "-webkit-box",
                                WebkitLineClamp: 3,
                                WebkitBoxOrient: "vertical",
                            }}
                        >
                            {post.content}
                        </p>
                    </div>

                    <div
                        style={{display: "flex", justifyContent: "flex-end", gap: "8px"}}
                    >
                        <button type="button" onClick={onClose} className="btn-secondary">
                            취소
                        </button>
                        <button
                            type="submit"
                            disabled={!content.trim() || loading}
                            className="btn-primary"
                        >
                            <Send size={13}/>
                            {loading ? "인용 중..." : "인용 게시하기"}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
}
