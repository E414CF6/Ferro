"use client";

import React, {useCallback, useEffect, useState} from "react";
import Link from "next/link";
import {useRouter} from "next/navigation";
import {Post, User} from "@/lib/types";
import {useAuth} from "@/lib/auth-context";
import {fetchGraphQL, MUTATIONS, QUERIES} from "@/lib/graphql";
import {useToast} from "@/lib/toast-context";
import {ArrowRight, Hash, RefreshCw, Search, TrendingUp, UserPlus, Users, X,} from "lucide-react";

export default function RightSidebar() {
    const router = useRouter();
    const {user} = useAuth();
    const {showToast} = useToast();
    const [searchQuery, setSearchQuery] = useState("");
    const [suggestedUsers, setSuggestedUsers] = useState<User[]>([]);
    const [trendingTags, setTrendingTags] = useState<{ tag: string; count: number }[]>([]);
    const [loadingUsers, setLoadingUsers] = useState(false);
    const [followLoading, setFollowLoading] = useState<Record<string, boolean>>({});

    const loadSuggestedUsers = useCallback(async () => {
        setLoadingUsers(true);
        try {
            const data = await fetchGraphQL<{ users: User[] }>(QUERIES.USERS_LIST);
            if (data?.users) {
                const filtered = data.users.filter((u) => u.username !== user?.username);
                setSuggestedUsers(filtered.slice(0, 5));
            }
        } catch (err) {
            console.error("Failed to load suggested users:", err);
        } finally {
            setLoadingUsers(false);
        }
    }, [user]);

    const loadTrendingTopics = useCallback(async () => {
        try {
            const data = await fetchGraphQL<{ posts: Post[] }>(QUERIES.GLOBAL_POSTS, {limit: 50});
            if (data?.posts) {
                const tagCountMap = new Map<string, number>();
                data.posts.forEach((p) => {
                    const matches = p.content.match(/#([\w가-힣]+)/g);
                    if (matches) {
                        matches.forEach((m) => {
                            const tag = m.substring(1);
                            tagCountMap.set(tag, (tagCountMap.get(tag) || 0) + 1);
                        });
                    }
                });

                const sortedTags = Array.from(tagCountMap.entries())
                    .map(([tag, count]) => ({tag, count}))
                    .sort((a, b) => b.count - a.count)
                    .slice(0, 5);

                if (sortedTags.length > 0) {
                    setTrendingTags(sortedTags);
                } else {
                    setTrendingTags([
                        {tag: "Rust", count: 18},
                        {tag: "Axum", count: 12},
                        {tag: "GraphQL", count: 9},
                        {tag: "NextJS", count: 8},
                        {tag: "DevLife", count: 5},
                    ]);
                }
            }
        } catch (err) {
            console.error("Failed to load trending topics:", err);
        }
    }, []);

    useEffect(() => {
        loadSuggestedUsers();
        loadTrendingTopics();
    }, [loadSuggestedUsers, loadTrendingTopics]);

    const handleSearchSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (searchQuery.trim()) {
            router.push(`/explore?q=${encodeURIComponent(searchQuery.trim())}`);
        }
    };

    const handleFollowToggle = async (targetUser: User) => {
        if (!user) {
            showToast("팔로우하려면 먼저 로그인하세요.", "info");
            return;
        }

        const userId = targetUser.id;
        setFollowLoading((prev) => ({...prev, [userId]: true}));

        const isFollowing = targetUser.isFollowedByMe;

        setSuggestedUsers((prev) =>
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
            setSuggestedUsers((prev) =>
                prev.map((u) => (u.id === userId ? {...u, isFollowedByMe: isFollowing} : u))
            );
            showToast("팔로우 처리에 실패했습니다.", "error");
        } finally {
            setFollowLoading((prev) => ({...prev, [userId]: false}));
        }
    };

    const defaultAvatar = "https://api.dicebear.com/7.x/bottts/svg?seed=ferro";

    return (
        <aside className="right-sidebar">
            {/* Search Input Box */}
            <form onSubmit={handleSearchSubmit}>
                <div style={{position: "relative", display: "flex", alignItems: "center"}}>
                    <Search
                        size={16}
                        style={{
                            position: "absolute",
                            left: 14,
                            color: "var(--text-muted)",
                            pointerEvents: "none",
                        }}
                    />
                    <input
                        type="text"
                        placeholder="포스트, 태그, 크리에이터 검색..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        style={{
                            width: "100%",
                            padding: "11px 40px 11px 40px",
                            borderRadius: "var(--radius-full)",
                            backgroundColor: "var(--bg-surface)",
                            border: "1px solid var(--border-subtle)",
                            color: "var(--text-primary)",
                            fontSize: "14px",
                            outline: "none",
                            transition: "all var(--transition-fast)",
                        }}
                        onFocus={(e) => {
                            e.currentTarget.style.borderColor = "var(--border-focus)";
                            e.currentTarget.style.boxShadow = "0 0 0 3px rgba(56, 189, 248, 0.15)";
                        }}
                        onBlur={(e) => {
                            e.currentTarget.style.borderColor = "var(--border-subtle)";
                            e.currentTarget.style.boxShadow = "none";
                        }}
                    />
                    {searchQuery && (
                        <button
                            type="button"
                            onClick={() => setSearchQuery("")}
                            style={{
                                position: "absolute",
                                right: 12,
                                color: "var(--text-muted)",
                                padding: "2px",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                            }}
                        >
                            <X size={14}/>
                        </button>
                    )}
                </div>
            </form>

            {/* Suggested Users Widget */}
            <div className="sidebar-widget">
                <div
                    style={{
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "space-between",
                        marginBottom: "14px",
                    }}
                >
                    <div className="widget-title" style={{margin: 0}}>
                        <Users size={16} color="var(--accent-primary)"/>
                        <span>추천 크리에이터</span>
                    </div>
                    <button
                        onClick={loadSuggestedUsers}
                        title="새로고침"
                        style={{color: "var(--text-muted)", padding: "4px"}}
                        onMouseEnter={(e) => (e.currentTarget.style.color = "var(--text-primary)")}
                        onMouseLeave={(e) => (e.currentTarget.style.color = "var(--text-muted)")}
                    >
                        <RefreshCw
                            size={13}
                            style={{animation: loadingUsers ? "spin 1s linear infinite" : "none"}}
                        />
                    </button>
                </div>

                <div>
                    {loadingUsers && suggestedUsers.length === 0 ? (
                        <div style={{
                            padding: "12px 0",
                            textAlign: "center",
                            color: "var(--text-muted)",
                            fontSize: "12px"
                        }}>
                            크리에이터 목록을 불러오는 중...
                        </div>
                    ) : suggestedUsers.length === 0 ? (
                        <div style={{
                            padding: "12px 0",
                            textAlign: "center",
                            color: "var(--text-muted)",
                            fontSize: "12px"
                        }}>
                            추천할 다른 사용자가 없습니다.
                        </div>
                    ) : (
                        suggestedUsers.map((u) => (
                            <div key={u.id} className="user-card-compact">
                                <Link
                                    href={`/profile/${u.username}`}
                                    className="user-compact-info"
                                >
                                    <img
                                        src={u.avatarUrl || defaultAvatar}
                                        alt={u.username}
                                        style={{width: 38, height: 38, borderRadius: "50%", objectFit: "cover"}}
                                    />
                                    <div className="user-compact-names">
                    <span
                        style={{
                            fontSize: "13px",
                            fontWeight: 700,
                            color: "var(--text-primary)",
                        }}
                    >
                      {u.displayName || u.username}
                    </span>
                                        <span style={{fontSize: "11px", color: "var(--text-muted)"}}>
                      @{u.username}
                    </span>
                                    </div>
                                </Link>

                                <button
                                    onClick={() => handleFollowToggle(u)}
                                    disabled={followLoading[u.id]}
                                    style={{
                                        padding: "5px 12px",
                                        borderRadius: "var(--radius-full)",
                                        fontSize: "12px",
                                        fontWeight: 700,
                                        transition: "all var(--transition-fast)",
                                        backgroundColor: u.isFollowedByMe
                                            ? "var(--bg-surface-hover)"
                                            : "var(--accent-primary)",
                                        color: u.isFollowedByMe ? "var(--text-primary)" : "#fff",
                                        border: u.isFollowedByMe ? "1px solid var(--border-subtle)" : "none",
                                    }}
                                >
                                    {u.isFollowedByMe ? (
                                        "팔로잉"
                                    ) : (
                                        <span style={{display: "inline-flex", alignItems: "center", gap: 3}}>
                      <UserPlus size={11}/> 팔로우
                    </span>
                                    )}
                                </button>
                            </div>
                        ))
                    )}
                </div>
            </div>

            {/* Trending Topics Widget */}
            <div className="sidebar-widget">
                <div className="widget-title">
                    <TrendingUp size={16} color="#ec4899"/>
                    <span>실시간 인기 태그</span>
                </div>

                <div style={{display: "flex", flexDirection: "column", gap: "10px"}}>
                    {trendingTags.map((item) => (
                        <Link
                            key={item.tag}
                            href={`/explore?q=%23${encodeURIComponent(item.tag)}`}
                            style={{
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "space-between",
                                padding: "6px 8px",
                                borderRadius: "var(--radius-sm)",
                                transition: "background-color var(--transition-fast)",
                            }}
                            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = "var(--bg-surface-hover)")}
                            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
                        >
                            <div style={{display: "flex", alignItems: "center", gap: "8px"}}>
                                <div
                                    style={{
                                        width: 24,
                                        height: 24,
                                        borderRadius: "6px",
                                        backgroundColor: "rgba(56, 189, 248, 0.1)",
                                        display: "flex",
                                        alignItems: "center",
                                        justifyContent: "center",
                                        color: "var(--accent-primary)",
                                    }}
                                >
                                    <Hash size={13}/>
                                </div>
                                <div>
                                    <div style={{fontSize: "13px", fontWeight: 700, color: "var(--text-primary)"}}>
                                        #{item.tag}
                                    </div>
                                    <div style={{fontSize: "11px", color: "var(--text-muted)"}}>
                                        {item.count}개의 게시물
                                    </div>
                                </div>
                            </div>
                            <ArrowRight size={13} color="var(--text-muted)"/>
                        </Link>
                    ))}
                </div>
            </div>

            {/* Guest Welcome Box if not logged in */}
            {!user && (
                <div
                    style={{
                        padding: "18px",
                        borderRadius: "var(--radius-lg)",
                        background: "linear-gradient(135deg, rgba(56, 189, 248, 0.12) 0%, rgba(168, 85, 247, 0.12) 100%)",
                        border: "1px solid rgba(56, 189, 248, 0.3)",
                    }}
                >
                    <div style={{fontSize: "14px", fontWeight: 800, color: "var(--text-primary)", marginBottom: "6px"}}>
                        ✨ Ferro에 오신 것을 환영합니다!
                    </div>
                    <p style={{
                        fontSize: "12px",
                        color: "var(--text-secondary)",
                        lineHeight: "1.5",
                        marginBottom: "12px"
                    }}>
                        24시간 스토리, 1:1 다이렉트 메시지, 초고속 Rust 피드를 경험해보세요.
                    </p>
                    <Link
                        href="/welcome"
                        className="btn-primary"
                        style={{width: "100%", justifyContent: "center", fontSize: "12px", padding: "8px"}}
                    >
                        소개 & 가입 페이지로 이동 <ArrowRight size={13}/>
                    </Link>
                </div>
            )}

            {/* Footer Info */}
            <div style={{fontSize: "11px", color: "var(--text-muted)", lineHeight: "1.6", padding: "0 6px"}}>
                <span>© 2026 Ferro. Rust 2024 & Next.js 15.</span>
                <div style={{display: "flex", gap: "10px", marginTop: "4px"}}>
                    <Link href="/welcome" style={{color: "var(--text-secondary)"}}>소개</Link>
                    <span>·</span>
                    <Link href="/explore" style={{color: "var(--text-secondary)"}}>탐색</Link>
                    <span>·</span>
                    <a href="https://github.com" target="_blank" rel="noreferrer"
                       style={{color: "var(--text-secondary)"}}>GitHub</a>
                </div>
            </div>
        </aside>
    );
}
