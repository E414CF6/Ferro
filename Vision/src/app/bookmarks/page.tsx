"use client";

import React, {useEffect, useState} from "react";
import Link from "next/link";
import {BookmarkCollection, Post} from "@/lib/types";
import {useAuth} from "@/lib/auth-context";
import {fetchGraphQL, QUERIES} from "@/lib/graphql";
import PostCard from "@/components/PostCard";
import BookmarkCollectionModal from "@/components/BookmarkCollectionModal";
import {Bookmark, Folder, FolderPlus, Lock,} from "lucide-react";

export default function BookmarksPage() {
    const {user} = useAuth();
    const [collections, setCollections] = useState<BookmarkCollection[]>([]);
    const [selectedCollectionId, setSelectedCollectionId] = useState<string | null>(null);
    const [posts, setPosts] = useState<Post[]>([]);
    const [loadingCollections, setLoadingCollections] = useState(true);
    const [loadingPosts, setLoadingPosts] = useState(false);
    const [showCreateModal, setShowCreateModal] = useState(false);

    const loadCollections = async () => {
        try {
            const data = await fetchGraphQL<{ bookmarkCollections: BookmarkCollection[] }>(
                QUERIES.BOOKMARK_COLLECTIONS
            );
            if (data?.bookmarkCollections) {
                setCollections(data.bookmarkCollections);
                if (data.bookmarkCollections.length > 0 && !selectedCollectionId) {
                    setSelectedCollectionId(data.bookmarkCollections[0].id);
                }
            }
        } catch (err) {
            console.error("Failed to load bookmark collections:", err);
        } finally {
            setLoadingCollections(false);
        }
    };

    const loadCollectionPosts = async (collId: string) => {
        setLoadingPosts(true);
        try {
            const data = await fetchGraphQL<{ collectionPosts: Post[] }>(
                QUERIES.COLLECTION_POSTS,
                {collectionId: collId, limit: 30, offset: 0}
            );
            if (data?.collectionPosts) {
                setPosts(data.collectionPosts);
            }
        } catch (err) {
            console.error("Failed to load collection posts:", err);
        } finally {
            setLoadingPosts(false);
        }
    };

    useEffect(() => {
        if (user) {
            loadCollections();
        } else {
            setLoadingCollections(false);
        }
    }, [user]);

    useEffect(() => {
        if (selectedCollectionId) {
            loadCollectionPosts(selectedCollectionId);
        }
    }, [selectedCollectionId]);

    if (!user) {
        return (
            <div>
                <header className="sticky-header">
                    <h1 className="header-title">북마크 컬렉션</h1>
                </header>
                <div style={{padding: "60px 20px", textAlign: "center"}}>
                    <Bookmark size={40} color="var(--text-muted)" style={{margin: "0 auto 16px"}}/>
                    <h3 style={{fontSize: "18px", fontWeight: 800}}>로그인이 필요합니다</h3>
                    <p style={{fontSize: "14px", color: "var(--text-secondary)", marginTop: "8px"}}>
                        나만의 북마크 컬렉션을 관리하려면 로그인하세요.
                    </p>
                    <Link href="/welcome?tab=login" className="btn-primary"
                          style={{marginTop: "16px", display: "inline-flex"}}>
                        로그인하기
                    </Link>
                </div>
            </div>
        );
    }

    return (
        <div>
            <header className="sticky-header">
                <div style={{display: "flex", alignItems: "center", gap: "10px"}}>
                    <Bookmark size={20} color="var(--accent-primary)"/>
                    <h1 className="header-title">북마크 컬렉션</h1>
                </div>

                <button
                    onClick={() => setShowCreateModal(true)}
                    className="btn-primary"
                    style={{padding: "6px 12px", fontSize: "12px"}}
                >
                    <FolderPlus size={14}/> 새 컬렉션
                </button>
            </header>

            {/* Horizontal Collections Pills Bar */}
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
                {collections.map((c) => (
                    <button
                        key={c.id}
                        onClick={() => setSelectedCollectionId(c.id)}
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
                                selectedCollectionId === c.id
                                    ? "var(--accent-primary)"
                                    : "var(--bg-input)",
                            color: selectedCollectionId === c.id ? "#fff" : "var(--text-primary)",
                            border: `1px solid ${
                                selectedCollectionId === c.id
                                    ? "var(--accent-primary)"
                                    : "var(--border-subtle)"
                            }`,
                        }}
                    >
                        <Folder size={14}/>
                        <span>{c.name}</span>
                        {c.isPrivate && <Lock size={11}/>}
                    </button>
                ))}
            </div>

            {/* Posts List */}
            <div>
                {loadingPosts ? (
                    <div style={{padding: "40px", textAlign: "center", color: "var(--text-muted)"}}>
                        저장된 게시물을 불러오는 중...
                    </div>
                ) : posts.length === 0 ? (
                    <div style={{padding: "60px 20px", textAlign: "center", color: "var(--text-muted)"}}>
                        <Bookmark size={40} style={{margin: "0 auto 14px", opacity: 0.5}}/>
                        <h3 style={{fontSize: "16px", fontWeight: 700, color: "var(--text-primary)"}}>
                            이 컬렉션에 저장된 게시물이 없습니다
                        </h3>
                        <p style={{fontSize: "13px", color: "var(--text-secondary)", marginTop: "6px"}}>
                            피드에서 관심 있는 게시물의 메뉴를 눌러 이 컬렉션에 저장해보세요.
                        </p>
                    </div>
                ) : (
                    posts.map((p) => (
                        <PostCard
                            key={p.id}
                            post={p}
                            onPostDeleted={() =>
                                selectedCollectionId && loadCollectionPosts(selectedCollectionId)
                            }
                        />
                    ))
                )}
            </div>

            {showCreateModal && (
                <BookmarkCollectionModal
                    onClose={() => setShowCreateModal(false)}
                    onPostSaved={loadCollections}
                />
            )}
        </div>
    );
}
