"use client";

import React, {useCallback, useEffect, useState} from "react";
import {Post, Story} from "@/lib/types";
import {useAuth} from "@/lib/auth-context";
import {fetchGraphQL, QUERIES} from "@/lib/graphql";
import StoryBar from "@/components/StoryBar";
import PostComposer from "@/components/PostComposer";
import PostCard from "@/components/PostCard";
import {ArrowRight, Globe, RefreshCw, Sparkles, UserCheck,} from "lucide-react";
import Link from "next/link";

export default function HomePage() {
    const {user} = useAuth();
    const [activeTab, setActiveTab] = useState<"following" | "global">("following");
    const [stories, setStories] = useState<Story[]>([]);
    const [posts, setPosts] = useState<Post[]>([]);
    const [loadingPosts, setLoadingPosts] = useState(true);
    const [refreshing, setRefreshing] = useState(false);

    // Load stories
    const loadStories = useCallback(async () => {
        try {
            const data = await fetchGraphQL<{ storiesFeed: Story[] }>(QUERIES.STORIES_FEED).catch(
                () => null
            );
            if (data?.storiesFeed) {
                setStories(data.storiesFeed);
            }
        } catch (err) {
            console.error("Failed to fetch stories feed:", err);
        }
    }, []);

    // Load posts
    const loadPosts = useCallback(async () => {
        setLoadingPosts(true);
        try {
            if (activeTab === "following" && user) {
                const data = await fetchGraphQL<{ feed: Post[] }>(QUERIES.FEED, {
                    limit: 30,
                    offset: 0,
                });
                if (data?.feed) {
                    setPosts(data.feed);
                }
            } else {
                const data = await fetchGraphQL<{ posts: Post[] }>(QUERIES.GLOBAL_POSTS, {
                    limit: 30,
                    offset: 0,
                });
                if (data?.posts) {
                    setPosts(data.posts);
                }
            }
        } catch (err) {
            console.error("Failed to load posts:", err);
        } finally {
            setLoadingPosts(false);
        }
    }, [activeTab, user]);

    useEffect(() => {
        loadStories();
        loadPosts();
    }, [loadStories, loadPosts]);

    const handleRefresh = async () => {
        setRefreshing(true);
        await Promise.all([loadStories(), loadPosts()]);
        setRefreshing(false);
    };

    return (
        <div>
            {/* Sticky Top Header */}
            <header className="sticky-header">
                <div style={{display: "flex", alignItems: "center", gap: "16px"}}>
                    <h1 className="header-title">
                        <span>홈 피드</span>
                    </h1>

                    {user && (
                        <div
                            style={{
                                display: "flex",
                                backgroundColor: "var(--bg-input)",
                                borderRadius: "var(--radius-full)",
                                padding: "3px",
                                border: "1px solid var(--border-subtle)",
                            }}
                        >
                            <button
                                onClick={() => setActiveTab("following")}
                                className={`header-tab-pill ${activeTab === "following" ? "active" : ""}`}
                            >
                                <UserCheck size={14}/>
                                팔로잉
                            </button>
                            <button
                                onClick={() => setActiveTab("global")}
                                className={`header-tab-pill ${activeTab === "global" ? "active" : ""}`}
                            >
                                <Globe size={14}/>
                                전체 피드
                            </button>
                        </div>
                    )}
                </div>

                <button
                    onClick={handleRefresh}
                    disabled={refreshing}
                    title="새로고침"
                    style={{
                        color: "var(--text-secondary)",
                        padding: "8px",
                        borderRadius: "50%",
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "center",
                    }}
                    onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = "var(--bg-surface-hover)")}
                    onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
                >
                    <RefreshCw
                        size={18}
                        style={{
                            animation: refreshing ? "spin 1s linear infinite" : "none",
                        }}
                    />
                </button>
            </header>

            {/* Guest Banner if not logged in */}
            {!user && (
                <div
                    style={{
                        padding: "16px 20px",
                        background: "linear-gradient(135deg, rgba(56, 189, 248, 0.1) 0%, rgba(168, 85, 247, 0.1) 100%)",
                        borderBottom: "1px solid rgba(56, 189, 248, 0.2)",
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "space-between",
                        gap: "12px",
                        flexWrap: "wrap",
                    }}
                >
                    <div style={{display: "flex", alignItems: "center", gap: "10px"}}>
                        <Sparkles size={20} color="var(--accent-primary)"/>
                        <div>
                            <div style={{fontSize: "14px", fontWeight: 700, color: "var(--text-primary)"}}>
                                Ferro에 오신 것을 환영합니다!
                            </div>
                            <div style={{fontSize: "12px", color: "var(--text-secondary)"}}>
                                24시간 스토리, 1:1 비공개 DM, 초고속 피드를 모두 즐겨보세요.
                            </div>
                        </div>
                    </div>
                    <Link
                        href="/welcome"
                        className="btn-primary"
                        style={{padding: "7px 16px", fontSize: "12px"}}
                    >
                        소개 & 가입하기 <ArrowRight size={13}/>
                    </Link>
                </div>
            )}

            {/* Stories horizontal tray */}
            <StoryBar stories={stories} onRefresh={loadStories}/>

            {/* Post Composer (Only shown to authenticated users) */}
            <PostComposer onPostCreated={loadPosts}/>

            {/* Posts Feed list */}
            <div>
                {loadingPosts ? (
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
                                <div
                                    className="skeleton"
                                    style={{width: 44, height: 44, borderRadius: "50%", flexShrink: 0}}
                                />
                                <div style={{flex: 1, display: "flex", flexDirection: "column", gap: "10px"}}>
                                    <div style={{display: "flex", gap: "10px", alignItems: "center"}}>
                                        <div className="skeleton" style={{width: "120px", height: "16px"}}/>
                                        <div className="skeleton" style={{width: "80px", height: "14px"}}/>
                                    </div>
                                    <div className="skeleton" style={{width: "100%", height: "48px"}}/>
                                    <div style={{display: "flex", gap: "24px"}}>
                                        <div className="skeleton" style={{width: "40px", height: "18px"}}/>
                                        <div className="skeleton" style={{width: "40px", height: "18px"}}/>
                                    </div>
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
                            gap: "14px",
                        }}
                    >
                        <div
                            style={{
                                width: 64,
                                height: 64,
                                borderRadius: "50%",
                                backgroundColor: "var(--bg-surface)",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                                color: "var(--accent-primary)",
                                boxShadow: "var(--shadow-glow)",
                            }}
                        >
                            <Sparkles size={30}/>
                        </div>
                        <h3 style={{fontSize: "19px", fontWeight: 800, color: "var(--text-primary)"}}>
                            {activeTab === "following" ? "팔로우한 사용자의 게시물이 없습니다" : "첫 게시물을 작성해보세요!"}
                        </h3>
                        <p style={{
                            fontSize: "14px",
                            color: "var(--text-secondary)",
                            maxWidth: "380px",
                            lineHeight: "1.6"
                        }}>
                            {activeTab === "following"
                                ? "탐색 탭에서 흥미로운 개발자 및 크리에이터를 팔로우하거나 전체 피드로 전환해보세요."
                                : "상단 작성기를 통해 아이디어나 최신 기술 스택 이야기를 커뮤니티에 공유해보세요."}
                        </p>
                        {activeTab === "following" && (
                            <button
                                onClick={() => setActiveTab("global")}
                                className="btn-secondary"
                                style={{marginTop: "8px"}}
                            >
                                <Globe size={15}/> 전체 피드 보기
                            </button>
                        )}
                    </div>
                ) : (
                    posts.map((p) => <PostCard key={p.id} post={p} onPostDeleted={loadPosts}/>)
                )}
            </div>

            <style jsx>{`
                @keyframes spin {
                    100% {
                        transform: rotate(360deg);
                    }
                }
            `}</style>
        </div>
    );
}
