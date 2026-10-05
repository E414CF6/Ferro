"use client";

import React, { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import { BookmarkCollection, Post } from "@/lib/types";
import { useAuth } from "@/lib/auth-context";
import { fetchGraphQL, MUTATIONS, QUERIES } from "@/lib/graphql";
import { useToast } from "@/lib/toast-context";
import { formatErrorMessage } from "@/lib/i18n";
import PostCard from "@/components/PostCard";
import BookmarkCollectionModal from "@/components/BookmarkCollectionModal";
import {
    Bookmark,
    Folder,
    FolderPlus,
    Lock,
    Sparkles,
    Trash2,
} from "lucide-react";

export default function BookmarksPage() {
    const { user } = useAuth();
    const { showToast } = useToast();
    const [collections, setCollections] = useState<BookmarkCollection[]>([]);
    // "all" or collection id
    const [activeTab, setActiveTab] = useState<string>("all");
    const [posts, setPosts] = useState<Post[]>([]);
    const [loadingPosts, setLoadingPosts] = useState(true);
    const [showCreateModal, setShowCreateModal] = useState(false);

    const loadCollections = useCallback(async () => {
        if (!user) return;
        try {
            const data = await fetchGraphQL<{ bookmarkCollections: BookmarkCollection[] }>(
                QUERIES.BOOKMARK_COLLECTIONS
            );
            if (data?.bookmarkCollections) {
                setCollections(data.bookmarkCollections);
            }
        } catch (err) {
            console.error("Failed to load bookmark collections:", err);
        }
    }, [user]);

    const loadPosts = useCallback(async () => {
        if (!user) return;
        setLoadingPosts(true);
        try {
            if (activeTab === "all") {
                const data = await fetchGraphQL<{
                    savedPostsConnection: { edges: { node: Post }[] };
                }>(QUERIES.SAVED_POSTS, { first: 30 });
                if (data?.savedPostsConnection?.edges) {
                    setPosts(data.savedPostsConnection.edges.map((e) => e.node));
                } else {
                    setPosts([]);
                }
            } else {
                const data = await fetchGraphQL<{
                    bookmarkCollection: {
                        postsConnection: { edges: { node: Post }[] };
                    };
                }>(QUERIES.BOOKMARK_COLLECTION, { id: activeTab, first: 30 });
                if (data?.bookmarkCollection?.postsConnection?.edges) {
                    setPosts(data.bookmarkCollection.postsConnection.edges.map((e) => e.node));
                } else {
                    setPosts([]);
                }
            }
        } catch (err) {
            console.error("Failed to load bookmark posts:", err);
            setPosts([]);
        } finally {
            setLoadingPosts(false);
        }
    }, [user, activeTab]);

    useEffect(() => {
        if (user) {
            loadCollections();
            loadPosts();
        } else {
            setLoadingPosts(false);
        }
    }, [user, loadCollections, loadPosts]);

    const handleDeleteCollection = async (collId: string, collName: string) => {
        if (!window.confirm(`정말로 "${collName}" 컬렉션을 삭제하시겠습니까?`)) return;
        try {
            await fetchGraphQL(MUTATIONS.DELETE_BOOKMARK_COLLECTION, { collectionId: collId });
            setCollections((prev) => prev.filter((c) => c.id !== collId));
            setActiveTab("all");
            showToast("컬렉션이 삭제되었습니다.", "info");
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        }
    };

    if (!user) {
        return (
            <div>
                <header className="sticky-header">
                    <h1 className="header-title">북마크</h1>
                </header>
                <div style={{ padding: "60px 20px", textAlign: "center" }}>
                    <Bookmark size={40} color="var(--text-muted)" style={{ margin: "0 auto 16px" }} />
                    <h3 style={{ fontSize: "18px", fontWeight: 800 }}>로그인이 필요합니다</h3>
                    <p style={{ fontSize: "14px", color: "var(--text-secondary)", marginTop: "8px" }}>
                        나만의 북마크 컬렉션을 관리하려면 로그인하세요.
                    </p>
                    <Link
                        href="/login"
                        className="btn-primary"
                        style={{ marginTop: "16px", display: "inline-flex" }}
                    >
                        로그인하기
                    </Link>
                </div>
            </div>
        );
    }

    const currentCollection = collections.find((c) => c.id === activeTab);

    return (
        <div>
            {/* Header */}
            <header className="sticky-header">
                <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                    <Bookmark size={20} color="var(--accent-primary)" />
                    <h1 className="header-title">북마크 컬렉션</h1>
                </div>

                <button
                    onClick={() => setShowCreateModal(true)}
                    className="btn-primary"
                    style={{ padding: "6px 14px", fontSize: "12px" }}
                >
                    <FolderPlus size={14} /> 새 컬렉션
                </button>
            </header>

            {/* Horizontal Collections Tabs Bar */}
            <div
                style={{
                    display: "flex",
                    gap: "8px",
                    overflowX: "auto",
                    padding: "14px 20px",
                    borderBottom: "1px solid var(--border-subtle)",
                    backgroundColor: "var(--bg-surface)",
                }}
            >
                {/* All Saved Posts Tab */}
                <button
                    onClick={() => setActiveTab("all")}
                    style={{
                        padding: "6px 14px",
                        borderRadius: "var(--radius-full)",
                        fontSize: "13px",
                        fontWeight: 700,
                        display: "flex",
                        alignItems: "center",
                        gap: "6px",
                        whiteSpace: "nowrap",
                        transition: "all var(--transition-fast)",
                        backgroundColor:
                            activeTab === "all" ? "var(--accent-primary)" : "var(--bg-input)",
                        color: activeTab === "all" ? "#fff" : "var(--text-primary)",
                        border: `1px solid ${
                            activeTab === "all" ? "var(--accent-primary)" : "var(--border-subtle)"
                        }`,
                    }}
                >
                    <Bookmark size={14} />
                    <span>전체 저장된 글</span>
                </button>

                {/* Individual Collections */}
                {collections.map((c) => (
                    <button
                        key={c.id}
                        onClick={() => setActiveTab(c.id)}
                        style={{
                            padding: "6px 14px",
                            borderRadius: "var(--radius-full)",
                            fontSize: "13px",
                            fontWeight: 700,
                            display: "flex",
                            alignItems: "center",
                            gap: "6px",
                            whiteSpace: "nowrap",
                            transition: "all var(--transition-fast)",
                            backgroundColor:
                                activeTab === c.id ? "var(--accent-primary)" : "var(--bg-input)",
                            color: activeTab === c.id ? "#fff" : "var(--text-primary)",
                            border: `1px solid ${
                                activeTab === c.id
                                    ? "var(--accent-primary)"
                                    : "var(--border-subtle)"
                            }`,
                        }}
                    >
                        <Folder size={14} />
                        <span>{c.name}</span>
                        {c.isPrivate && <Lock size={11} />}
                    </button>
                ))}
            </div>

            {/* Collection Description & Actions Banner */}
            {currentCollection && (
                <div
                    style={{
                        padding: "12px 20px",
                        backgroundColor: "rgba(56, 189, 248, 0.05)",
                        borderBottom: "1px solid var(--border-subtle)",
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "space-between",
                    }}
                >
                    <div>
                        <div style={{ fontSize: "14px", fontWeight: 800, color: "var(--text-primary)" }}>
                            {currentCollection.name}
                        </div>
                        {currentCollection.description && (
                            <div style={{ fontSize: "12px", color: "var(--text-muted)", marginTop: "2px" }}>
                                {currentCollection.description}
                            </div>
                        )}
                    </div>

                    <button
                        onClick={() => handleDeleteCollection(currentCollection.id, currentCollection.name)}
                        className="btn-secondary"
                        style={{ color: "var(--accent-secondary)", padding: "5px 10px", fontSize: "11px" }}
                    >
                        <Trash2 size={12} /> 컬렉션 삭제
                    </button>
                </div>
            )}

            {/* Posts List */}
            <div>
                {loadingPosts ? (
                    <div style={{ padding: "40px", textAlign: "center", color: "var(--text-muted)" }}>
                        저장된 게시물을 불러오는 중...
                    </div>
                ) : posts.length === 0 ? (
                    <div style={{ padding: "80px 20px", textAlign: "center", color: "var(--text-muted)" }}>
                        <Bookmark size={44} style={{ margin: "0 auto 14px", opacity: 0.3 }} />
                        <h3 style={{ fontSize: "17px", fontWeight: 700, color: "var(--text-primary)" }}>
                            저장된 게시물이 없습니다
                        </h3>
                        <p style={{ fontSize: "13px", color: "var(--text-secondary)", marginTop: "6px" }}>
                            피드에서 게시물의 북마크 아이콘을 눌러 관심 있는 글을 보관해보세요.
                        </p>
                    </div>
                ) : (
                    posts.map((p) => (
                        <PostCard
                            key={p.id}
                            post={p}
                            onPostDeleted={loadPosts}
                            onPostUpdated={loadPosts}
                        />
                    ))
                )}
            </div>

            {/* Create Collection Modal */}
            {showCreateModal && (
                <BookmarkCollectionModal
                    onClose={() => setShowCreateModal(false)}
                    onPostSaved={() => {
                        loadCollections();
                        loadPosts();
                    }}
                />
            )}
        </div>
    );
}
