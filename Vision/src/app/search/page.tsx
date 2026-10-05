"use client";

import React, { Suspense, useCallback, useEffect, useState } from "react";
import { useSearchParams } from "next/navigation";
import { HashtagTrend, Post, User } from "@/lib/types";
import { fetchGraphQL, QUERIES } from "@/lib/graphql";
import PostCard from "@/components/PostCard";
import { Search as SearchIcon, Users, X, Sparkles } from "lucide-react";
import Link from "next/link";

function SearchContent() {
    const searchParams = useSearchParams();
    const initialQuery = searchParams.get("q") || "";

    const [query, setQuery] = useState(initialQuery);
    const [posts, setPosts] = useState<Post[]>([]);
    const [matchingUsers, setMatchingUsers] = useState<User[]>([]);
    const [trendingHashtags, setTrendingHashtags] = useState<HashtagTrend[]>([]);
    const [loading, setLoading] = useState(false);
    const [hasSearched, setHasSearched] = useState(false);

    const performSearch = useCallback(async (searchQueryStr: string) => {
        const trimmed = searchQueryStr.trim();
        if (!trimmed) {
            setPosts([]);
            setMatchingUsers([]);
            setHasSearched(false);
            return;
        }

        setLoading(true);
        setHasSearched(true);

        try {
            if (trimmed.startsWith("#")) {
                const hashtag = trimmed.substring(1);
                const data = await fetchGraphQL<{
                    postsByHashtag: { edges: { node: Post }[] };
                }>(QUERIES.POSTS_BY_HASHTAG, { hashtag, first: 30 });

                setPosts(data?.postsByHashtag?.edges?.map((e) => e.node) || []);
                setMatchingUsers([]);
            } else {
                const [postsData, usersData] = await Promise.all([
                    fetchGraphQL<{
                        searchPostsConnection: { edges: { node: Post }[] };
                    }>(QUERIES.SEARCH_POSTS, { query: trimmed, first: 30 }).catch(() => null),
                    fetchGraphQL<{ users: User[] }>(QUERIES.USERS_LIST).catch(() => null),
                ]);

                if (postsData?.searchPostsConnection?.edges) {
                    setPosts(postsData.searchPostsConnection.edges.map((e) => e.node));
                } else {
                    setPosts([]);
                }

                if (usersData?.users) {
                    const qLower = trimmed.toLowerCase();
                    const matched = usersData.users.filter(
                        (u) =>
                            u.username.toLowerCase().includes(qLower) ||
                            u.displayName.toLowerCase().includes(qLower)
                    );
                    setMatchingUsers(matched.slice(0, 5));
                } else {
                    setMatchingUsers([]);
                }
            }
        } catch (err) {
            console.error("Search error:", err);
            setPosts([]);
            setMatchingUsers([]);
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        // Load trending hashtags for search recommendations
        fetchGraphQL<{ trendingHashtags: HashtagTrend[] }>(QUERIES.TRENDING_HASHTAGS, {
            limit: 8,
        })
            .then((data) => {
                if (data?.trendingHashtags) {
                    setTrendingHashtags(data.trendingHashtags);
                }
            })
            .catch(() => null);
    }, []);

    useEffect(() => {
        if (initialQuery) {
            setQuery(initialQuery);
            performSearch(initialQuery);
        }
    }, [initialQuery, performSearch]);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        performSearch(query);
    };

    return (
        <div style={{ minHeight: "100vh", paddingBottom: "100px" }}>
            {/* Sticky Search Header */}
            <div
                style={{
                    position: "sticky",
                    top: 0,
                    zIndex: 20,
                    backgroundColor: "rgba(7, 10, 18, 0.88)",
                    backdropFilter: "blur(16px)",
                    borderBottom: "1px solid var(--border-subtle)",
                    padding: "16px 20px",
                }}
            >
                <form onSubmit={handleSubmit} style={{ position: "relative" }}>
                    <SearchIcon
                        size={18}
                        style={{
                            position: "absolute",
                            left: "14px",
                            top: "50%",
                            transform: "translateY(-50%)",
                            color: "var(--text-muted)",
                            pointerEvents: "none",
                        }}
                    />
                    <input
                        type="text"
                        placeholder="게시물, 키워드, 사용자 검색..."
                        value={query}
                        onChange={(e) => {
                            setQuery(e.target.value);
                            if (!e.target.value.trim()) {
                                setPosts([]);
                                setMatchingUsers([]);
                                setHasSearched(false);
                            }
                        }}
                        autoFocus
                        className="form-input"
                        style={{
                            width: "100%",
                            paddingLeft: "42px",
                            paddingRight: query ? "38px" : "14px",
                            borderRadius: "var(--radius-full)",
                            backgroundColor: "var(--bg-surface)",
                            border: "1px solid var(--border-subtle)",
                            fontSize: "14px",
                            height: "44px",
                        }}
                    />
                    {query && (
                        <button
                            type="button"
                            onClick={() => {
                                setQuery("");
                                setPosts([]);
                                setMatchingUsers([]);
                                setHasSearched(false);
                            }}
                            style={{
                                position: "absolute",
                                right: "12px",
                                top: "50%",
                                transform: "translateY(-50%)",
                                background: "none",
                                border: "none",
                                color: "var(--text-muted)",
                                cursor: "pointer",
                                padding: "4px",
                            }}
                            aria-label="지우기"
                        >
                            <X size={16} />
                        </button>
                    )}
                </form>

                {/* Trending quick tags */}
                {trendingHashtags.length > 0 && (
                    <div
                        style={{
                            display: "flex",
                            gap: "8px",
                            overflowX: "auto",
                            marginTop: "12px",
                            paddingBottom: "4px",
                            scrollbarWidth: "none",
                        }}
                    >
                        {trendingHashtags.map((t) => (
                            <button
                                key={t.hashtag}
                                onClick={() => {
                                    const tagQuery = `#${t.hashtag}`;
                                    setQuery(tagQuery);
                                    performSearch(tagQuery);
                                }}
                                style={{
                                    padding: "6px 14px",
                                    borderRadius: "var(--radius-full)",
                                    fontSize: "12px",
                                    fontWeight: 600,
                                    whiteSpace: "nowrap",
                                    border: "1px solid var(--border-subtle)",
                                    backgroundColor:
                                        query === `#${t.hashtag}`
                                            ? "var(--accent-primary)"
                                            : "var(--bg-surface)",
                                    color:
                                        query === `#${t.hashtag}`
                                            ? "#fff"
                                            : "var(--text-secondary)",
                                    cursor: "pointer",
                                    transition: "all 0.2s ease",
                                }}
                            >
                                #{t.hashtag}
                            </button>
                        ))}
                    </div>
                )}
            </div>

            {/* Matching Users Preview */}
            {matchingUsers.length > 0 && (
                <div
                    style={{
                        padding: "16px 20px",
                        borderBottom: "1px solid var(--border-subtle)",
                        backgroundColor: "var(--bg-surface)",
                    }}
                >
                    <div
                        style={{
                            fontSize: "12px",
                            fontWeight: 700,
                            color: "var(--text-muted)",
                            textTransform: "uppercase",
                            letterSpacing: "0.5px",
                            marginBottom: "12px",
                            display: "flex",
                            alignItems: "center",
                            gap: "6px",
                        }}
                    >
                        <Users size={14} /> 사용자
                    </div>
                    <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
                        {matchingUsers.map((u) => (
                            <Link
                                key={u.id}
                                href={`/profile/${u.username}`}
                                style={{
                                    display: "flex",
                                    alignItems: "center",
                                    gap: "12px",
                                    textDecoration: "none",
                                    padding: "6px 8px",
                                    borderRadius: "var(--radius-md)",
                                    transition: "background-color 0.2s ease",
                                }}
                            >
                                <img
                                    src={
                                        u.avatarUrl ||
                                        `https://api.dicebear.com/7.x/bottts/svg?seed=${u.username}`
                                    }
                                    alt={u.username}
                                    style={{
                                        width: "36px",
                                        height: "36px",
                                        borderRadius: "50%",
                                        objectFit: "cover",
                                    }}
                                />
                                <div>
                                    <div
                                        style={{
                                            fontWeight: 700,
                                            fontSize: "14px",
                                            color: "var(--text-primary)",
                                        }}
                                    >
                                        {u.displayName || u.username}
                                    </div>
                                    <div style={{ fontSize: "12px", color: "var(--text-muted)" }}>
                                        @{u.username}
                                    </div>
                                </div>
                            </Link>
                        ))}
                    </div>
                </div>
            )}

            {/* Search Results */}
            <div style={{ padding: "0" }}>
                {loading ? (
                    <div
                        style={{
                            padding: "48px 20px",
                            textAlign: "center",
                            color: "var(--text-muted)",
                            fontSize: "14px",
                        }}
                    >
                        검색 중...
                    </div>
                ) : hasSearched && posts.length === 0 ? (
                    <div
                        style={{
                            padding: "60px 20px",
                            textAlign: "center",
                            color: "var(--text-muted)",
                        }}
                    >
                        <SearchIcon size={36} style={{ marginBottom: "12px", opacity: 0.4 }} />
                        <h3
                            style={{
                                fontSize: "16px",
                                fontWeight: 700,
                                color: "var(--text-primary)",
                                marginBottom: "4px",
                            }}
                        >
                            검색 결과가 없습니다
                        </h3>
                        <p style={{ fontSize: "13px", color: "var(--text-muted)" }}>
                            다른 검색어나 키워드로 검색해 보세요.
                        </p>
                    </div>
                ) : !hasSearched ? (
                    <div
                        style={{
                            padding: "60px 20px",
                            textAlign: "center",
                            color: "var(--text-muted)",
                        }}
                    >
                        <Sparkles size={36} style={{ marginBottom: "12px", opacity: 0.4 }} />
                        <h3
                            style={{
                                fontSize: "16px",
                                fontWeight: 700,
                                color: "var(--text-primary)",
                                marginBottom: "4px",
                            }}
                        >
                            키워드나 사람을 검색해 보세요
                        </h3>
                        <p style={{ fontSize: "13px", color: "var(--text-muted)" }}>
                            관심 있는 주제나 트렌드 태그를 입력하세요.
                        </p>
                    </div>
                ) : (
                    posts.map((post) => (
                        <PostCard
                            key={post.id}
                            post={post}
                            onPostDeleted={() => performSearch(query)}
                            onPostUpdated={() => performSearch(query)}
                        />
                    ))
                )}
            </div>
        </div>
    );
}

export default function SearchPage() {
    return (
        <Suspense
            fallback={
                <div style={{ padding: "40px 20px", textAlign: "center", color: "var(--text-muted)" }}>
                    검색 페이지 로딩 중...
                </div>
            }
        >
            <SearchContent />
        </Suspense>
    );
}
