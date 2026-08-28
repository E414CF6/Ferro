"use client";

import React, {Suspense, useCallback, useEffect, useState} from "react";
import Link from "next/link";
import {useSearchParams} from "next/navigation";
import {Post, User} from "@/lib/types";
import {fetchGraphQL, MUTATIONS, QUERIES} from "@/lib/graphql";
import {useAuth} from "@/lib/auth-context";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import PostCard from "@/components/PostCard";
import {Compass, MessageCircle, Search, Users, X,} from "lucide-react";

const CATEGORIES = [
    {id: "all", label: "전체", query: ""},
    {id: "rust", label: "🦀 Rust 개발", query: "Rust"},
    {id: "graphql", label: "⚡ GraphQL", query: "GraphQL"},
    {id: "postgres", label: "🐘 PostgreSQL", query: "PostgreSQL"},
    {id: "design", label: "🎨 UI/UX 디자인", query: "Design"},
    {id: "trending", label: "🔥 최신 트렌드", query: "NextJS"},
];

function ExploreContent() {
    const {user} = useAuth();
    const {showToast} = useToast();
    const searchParams = useSearchParams();
    const initialQuery = searchParams.get("q") || "";

    const [query, setQuery] = useState(initialQuery);
    const [selectedCategory, setSelectedCategory] = useState("all");
    const [posts, setPosts] = useState<Post[]>([]);
    const [allUsers, setAllUsers] = useState<User[]>([]);
    const [loading, setLoading] = useState(false);
    const [followLoading, setFollowLoading] = useState<Record<string, boolean>>({});

    const handleSearch = useCallback(async (searchQuery: string) => {
        setLoading(true);
        try {
            if (searchQuery.trim()) {
                const data = await fetchGraphQL<{ searchPosts: Post[] }>(QUERIES.SEARCH_POSTS, {
                    query: searchQuery.trim(),
                });
                if (data?.searchPosts) {
                    setPosts(data.searchPosts);
                }
            } else {
                const data = await fetchGraphQL<{ posts: Post[] }>(QUERIES.GLOBAL_POSTS, {
                    limit: 30,
                });
                if (data?.posts) {
                    setPosts(data.posts);
                }
            }
        } catch (err) {
            console.error("Failed to search posts:", err);
        } finally {
            setLoading(false);
        }
    }, []);

    const loadUsers = useCallback(async () => {
        try {
            const data = await fetchGraphQL<{ users: User[] }>(QUERIES.USERS_LIST);
            if (data?.users) {
                setAllUsers(data.users);
            }
        } catch (err) {
            console.error("Failed to fetch users:", err);
        }
    }, []);

    useEffect(() => {
        if (initialQuery) {
            setQuery(initialQuery);
            handleSearch(initialQuery);
        } else {
            handleSearch("");
        }
        loadUsers();
    }, [initialQuery, handleSearch, loadUsers]);

    const onSearchSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        handleSearch(query);
    };

    const handleCategoryClick = (cat: typeof CATEGORIES[0]) => {
        setSelectedCategory(cat.id);
        setQuery(cat.query);
        handleSearch(cat.query);
    };

    const handleFollowToggle = async (targetUser: User) => {
        if (!user) {
            showToast("팔로우하려면 먼저 로그인하세요.", "info");
            return;
        }

        const userId = targetUser.id;
        setFollowLoading((prev) => ({...prev, [userId]: true}));

        const isFollowing = targetUser.isFollowedByMe;

        // Optimistic UI update
        setAllUsers((prev) =>
            prev.map((u) => (u.id === userId ? {...u, isFollowedByMe: !isFollowing} : u))
        );

        try {
            if (isFollowing) {
                await fetchGraphQL(MUTATIONS.UNFOLLOW_USER, {userId});
                showToast(`@${targetUser.username} 님을 언팔로우했습니다.`, "info");
            } else {
                await fetchGraphQL(MUTATIONS.FOLLOW_USER, {userId});
                showToast(`@${targetUser.username} 님을 팔로우했습니다.`, "success");
            }
        } catch (err: any) {
            setAllUsers((prev) =>
                prev.map((u) => (u.id === userId ? {...u, isFollowedByMe: isFollowing} : u))
            );
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setFollowLoading((prev) => ({...prev, [userId]: false}));
        }
    };

    const defaultAvatar = "https://api.dicebear.com/7.x/bottts/svg?seed=ferro";

    return (
        <div>
            {/* Sticky Search Header */}
            <header className="sticky-header" style={{flexDirection: "column", gap: "12px", alignItems: "stretch"}}>
                <div style={{display: "flex", alignItems: "center", justifyContent: "space-between"}}>
                    <h1 className="header-title">
                        <Compass size={22} color="var(--accent-primary)"/>
                        <span>탐색 (Explore)</span>
                    </h1>
                </div>

                {/* Search Bar */}
                <form onSubmit={onSearchSubmit} style={{position: "relative"}}>
                    <Search
                        size={16}
                        style={{
                            position: "absolute",
                            left: 14,
                            top: "50%",
                            transform: "translateY(-50%)",
                            color: "var(--text-muted)",
                        }}
                    />
                    <input
                        type="text"
                        placeholder="키워드, 해시태그(#), 기술 스택으로 검색..."
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        style={{
                            width: "100%",
                            padding: "10px 38px 10px 40px",
                            borderRadius: "var(--radius-full)",
                            backgroundColor: "var(--bg-input)",
                            border: "1px solid var(--border-subtle)",
                            color: "var(--text-primary)",
                            fontSize: "14px",
                            outline: "none",
                        }}
                    />
                    {query && (
                        <button
                            type="button"
                            onClick={() => {
                                setQuery("");
                                handleSearch("");
                            }}
                            style={{
                                position: "absolute",
                                right: 12,
                                top: "50%",
                                transform: "translateY(-50%)",
                                color: "var(--text-muted)",
                                padding: "4px",
                            }}
                        >
                            <X size={14}/>
                        </button>
                    )}
                </form>

                {/* Category Pills */}
                <div style={{
                    display: "flex",
                    gap: "8px",
                    overflowX: "auto",
                    paddingBottom: "2px",
                    scrollbarWidth: "none"
                }}>
                    {CATEGORIES.map((cat) => (
                        <button
                            key={cat.id}
                            onClick={() => handleCategoryClick(cat)}
                            className={`header-tab-pill ${selectedCategory === cat.id ? "active" : ""}`}
                            style={{flexShrink: 0, fontSize: "12px", padding: "5px 12px"}}
                        >
                            {cat.label}
                        </button>
                    ))}
                </div>
            </header>

            {/* Suggested Creators Carousel / Grid */}
            {allUsers.length > 0 && !query && (
                <div style={{
                    padding: "20px 20px 10px",
                    borderBottom: "1px solid var(--border-subtle)",
                    background: "rgba(16, 23, 38, 0.2)"
                }}>
                    <div style={{display: "flex", alignItems: "center", gap: "8px", marginBottom: "14px"}}>
                        <Users size={16} color="var(--accent-primary)"/>
                        <span style={{fontSize: "15px", fontWeight: 800, color: "var(--text-primary)"}}>
              주목할 만한 크리에이터
            </span>
                    </div>

                    <div
                        style={{
                            display: "grid",
                            gridTemplateColumns: "repeat(auto-fill, minmax(180px, 1fr))",
                            gap: "12px",
                            marginBottom: "10px",
                        }}
                    >
                        {allUsers.slice(0, 4).map((u) => (
                            <div
                                key={u.id}
                                style={{
                                    backgroundColor: "var(--bg-surface)",
                                    border: "1px solid var(--border-subtle)",
                                    borderRadius: "var(--radius-lg)",
                                    padding: "16px",
                                    display: "flex",
                                    flexDirection: "column",
                                    alignItems: "center",
                                    textAlign: "center",
                                    position: "relative",
                                    transition: "all var(--transition-fast)",
                                }}
                            >
                                <Link href={`/profile/${u.username}`}>
                                    <div
                                        className={`story-avatar-wrapper ${u.hasActiveStories ? "unread" : ""}`}
                                        style={{width: 56, height: 56, margin: "0 auto 10px"}}
                                    >
                                        <div className="story-avatar-inner">
                                            <img
                                                src={u.avatarUrl || defaultAvatar}
                                                alt={u.username}
                                                className="story-avatar-img"
                                            />
                                        </div>
                                    </div>
                                    <div
                                        style={{
                                            fontWeight: 800,
                                            fontSize: "14px",
                                            color: "var(--text-primary)",
                                            whiteSpace: "nowrap",
                                            overflow: "hidden",
                                            textOverflow: "ellipsis",
                                            maxWidth: "140px",
                                        }}
                                    >
                                        {u.displayName || u.username}
                                    </div>
                                    <div style={{fontSize: "12px", color: "var(--text-muted)", marginBottom: "8px"}}>
                                        @{u.username}
                                    </div>
                                </Link>

                                <p
                                    style={{
                                        fontSize: "11px",
                                        color: "var(--text-secondary)",
                                        lineHeight: "1.4",
                                        marginBottom: "12px",
                                        display: "-webkit-box",
                                        WebkitLineClamp: 2,
                                        WebkitBoxOrient: "vertical",
                                        overflow: "hidden",
                                        height: "30px",
                                    }}
                                >
                                    {u.bio || "Ferro 크리에이터"}
                                </p>

                                <div style={{display: "flex", gap: "6px", width: "100%", marginTop: "auto"}}>
                                    <button
                                        onClick={() => handleFollowToggle(u)}
                                        disabled={followLoading[u.id] || user?.username === u.username}
                                        style={{
                                            flex: 1,
                                            padding: "6px 0",
                                            borderRadius: "var(--radius-full)",
                                            fontSize: "12px",
                                            fontWeight: 700,
                                            backgroundColor: u.isFollowedByMe ? "var(--bg-surface-hover)" : "var(--accent-primary)",
                                            color: u.isFollowedByMe ? "var(--text-primary)" : "#fff",
                                            border: u.isFollowedByMe ? "1px solid var(--border-subtle)" : "none",
                                        }}
                                    >
                                        {u.isFollowedByMe ? "팔로잉" : "팔로우"}
                                    </button>

                                    {user && user.username !== u.username && (
                                        <Link
                                            href={`/messages?user=${u.username}`}
                                            style={{
                                                padding: "6px 10px",
                                                borderRadius: "var(--radius-full)",
                                                backgroundColor: "var(--bg-input)",
                                                border: "1px solid var(--border-subtle)",
                                                color: "var(--text-primary)",
                                                display: "flex",
                                                alignItems: "center",
                                                justifyContent: "center",
                                            }}
                                            title="1:1 메시지 보내기"
                                        >
                                            <MessageCircle size={14}/>
                                        </Link>
                                    )}
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            )}

            {/* Search Results / Post Feed */}
            <div>
                <div
                    style={{
                        padding: "14px 20px",
                        fontSize: "13px",
                        fontWeight: 700,
                        color: "var(--text-muted)",
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "space-between",
                        borderBottom: "1px solid var(--border-subtle)",
                    }}
                >
                    <span>{query ? `"${query}" 검색 결과` : "탐색 피드"}</span>
                    <span>{posts.length}개의 게시물</span>
                </div>

                {loading ? (
                    <div style={{padding: "20px"}}>
                        {[1, 2, 3].map((i) => (
                            <div
                                key={i}
                                style={{
                                    display: "flex",
                                    gap: "14px",
                                    padding: "20px 0",
                                    borderBottom: "1px solid var(--border-subtle)",
                                }}
                            >
                                <div className="skeleton" style={{width: 44, height: 44, borderRadius: "50%"}}/>
                                <div style={{flex: 1, display: "flex", flexDirection: "column", gap: "10px"}}>
                                    <div className="skeleton" style={{width: "120px", height: "16px"}}/>
                                    <div className="skeleton" style={{width: "100%", height: "40px"}}/>
                                </div>
                            </div>
                        ))}
                    </div>
                ) : posts.length === 0 ? (
                    <div
                        style={{
                            padding: "60px 20px",
                            textAlign: "center",
                            display: "flex",
                            flexDirection: "column",
                            alignItems: "center",
                            gap: "12px",
                        }}
                    >
                        <div
                            style={{
                                width: 60,
                                height: 60,
                                borderRadius: "50%",
                                backgroundColor: "var(--bg-surface)",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                                color: "var(--accent-primary)",
                            }}
                        >
                            <Search size={26}/>
                        </div>
                        <h3 style={{fontSize: "17px", fontWeight: 800, color: "var(--text-primary)"}}>
                            {query ? `"${query}"에 대한 검색 결과가 없습니다` : "게시물이 없습니다"}
                        </h3>
                        <p style={{fontSize: "13px", color: "var(--text-secondary)", maxWidth: "340px"}}>
                            다른 검색어를 입력하거나 상단 카테고리 태그를 클릭하여 새로운 주제를 탐색해보세요.
                        </p>
                    </div>
                ) : (
                    posts.map((p) => <PostCard key={p.id} post={p} onPostDeleted={() => handleSearch(query)}/>)
                )}
            </div>
        </div>
    );
}

export default function ExplorePage() {
    return (
        <Suspense fallback={<div style={{padding: "40px", textAlign: "center"}}>Loading...</div>}>
            <ExploreContent/>
        </Suspense>
    );
}
