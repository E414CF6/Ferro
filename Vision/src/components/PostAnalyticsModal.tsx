"use client";

import React, {useEffect, useState} from "react";
import {PostAnalytics} from "@/lib/types";
import {fetchGraphQL, QUERIES} from "@/lib/graphql";
import {BarChart3, Eye, Heart, MessageCircle, Repeat, TrendingUp, X,} from "lucide-react";

interface PostAnalyticsModalProps {
    postId: string;
    onClose: () => void;
}

export default function PostAnalyticsModal({
                                               postId,
                                               onClose,
                                           }: PostAnalyticsModalProps) {
    const [analytics, setAnalytics] = useState<PostAnalytics | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchAnalytics = async () => {
            try {
                const data = await fetchGraphQL<{ postAnalytics: PostAnalytics }>(
                    QUERIES.POST_ANALYTICS,
                    {postId}
                );
                if (data?.postAnalytics) {
                    setAnalytics(data.postAnalytics);
                }
            } catch (err) {
                console.error("Failed to load post analytics:", err);
            } finally {
                setLoading(false);
            }
        };

        fetchAnalytics();
    }, [postId]);

    return (
        <div className="modal-overlay" onClick={onClose}>
            <div
                className="modal-content"
                onClick={(e) => e.stopPropagation()}
                style={{maxWidth: "440px"}}
            >
                <div className="modal-header">
                    <div style={{display: "flex", alignItems: "center", gap: "8px"}}>
                        <BarChart3 size={20} color="var(--accent-primary)"/>
                        <h3 className="modal-title">게시물 성과 및 통계</h3>
                    </div>
                    <button onClick={onClose} className="modal-close-btn">
                        <X size={18}/>
                    </button>
                </div>

                <div style={{padding: "20px"}}>
                    {loading ? (
                        <div
                            style={{
                                textAlign: "center",
                                padding: "30px",
                                color: "var(--text-muted)",
                            }}
                        >
                            통계 지표를 집계하는 중...
                        </div>
                    ) : !analytics ? (
                        <div
                            style={{
                                textAlign: "center",
                                padding: "30px",
                                color: "var(--text-muted)",
                            }}
                        >
                            통계 데이터를 불러올 수 없습니다.
                        </div>
                    ) : (
                        <div>
                            {/* Highlight Card: Engagement Rate */}
                            <div
                                style={{
                                    padding: "16px",
                                    borderRadius: "var(--radius-lg)",
                                    background:
                                        "linear-gradient(135deg, rgba(56, 189, 248, 0.15) 0%, rgba(168, 85, 247, 0.15) 100%)",
                                    border: "1px solid rgba(56, 189, 248, 0.3)",
                                    display: "flex",
                                    alignItems: "center",
                                    justifyContent: "space-between",
                                    marginBottom: "20px",
                                }}
                            >
                                <div>
                                    <div
                                        style={{
                                            fontSize: "12px",
                                            fontWeight: 600,
                                            color: "var(--text-secondary)",
                                        }}
                                    >
                                        참여율 (Engagement Rate)
                                    </div>
                                    <div
                                        style={{
                                            fontSize: "26px",
                                            fontWeight: 900,
                                            color: "var(--accent-primary)",
                                            marginTop: "4px",
                                        }}
                                    >
                                        {analytics.engagementRate.toFixed(1)}%
                                    </div>
                                </div>
                                <div
                                    style={{
                                        width: 44,
                                        height: 44,
                                        borderRadius: "50%",
                                        backgroundColor: "rgba(56, 189, 248, 0.2)",
                                        display: "flex",
                                        alignItems: "center",
                                        justifyContent: "center",
                                        color: "var(--accent-primary)",
                                    }}
                                >
                                    <TrendingUp size={22}/>
                                </div>
                            </div>

                            {/* Metrics Grid */}
                            <div
                                style={{
                                    display: "grid",
                                    gridTemplateColumns: "1fr 1fr",
                                    gap: "12px",
                                }}
                            >
                                {/* Views */}
                                <div
                                    style={{
                                        padding: "14px",
                                        borderRadius: "var(--radius-md)",
                                        backgroundColor: "var(--bg-surface)",
                                        border: "1px solid var(--border-subtle)",
                                    }}
                                >
                                    <div
                                        style={{
                                            display: "flex",
                                            alignItems: "center",
                                            gap: "6px",
                                            color: "var(--text-muted)",
                                            fontSize: "12px",
                                            marginBottom: "6px",
                                        }}
                                    >
                                        <Eye size={14} color="var(--accent-primary)"/>
                                        <span>노출 및 조회수</span>
                                    </div>
                                    <div
                                        style={{
                                            fontSize: "20px",
                                            fontWeight: 800,
                                            color: "var(--text-primary)",
                                        }}
                                    >
                                        {analytics.viewsCount.toLocaleString()}
                                    </div>
                                </div>

                                {/* Likes */}
                                <div
                                    style={{
                                        padding: "14px",
                                        borderRadius: "var(--radius-md)",
                                        backgroundColor: "var(--bg-surface)",
                                        border: "1px solid var(--border-subtle)",
                                    }}
                                >
                                    <div
                                        style={{
                                            display: "flex",
                                            alignItems: "center",
                                            gap: "6px",
                                            color: "var(--text-muted)",
                                            fontSize: "12px",
                                            marginBottom: "6px",
                                        }}
                                    >
                                        <Heart size={14} color="#f43f5e"/>
                                        <span>좋아요</span>
                                    </div>
                                    <div
                                        style={{
                                            fontSize: "20px",
                                            fontWeight: 800,
                                            color: "var(--text-primary)",
                                        }}
                                    >
                                        {analytics.likesCount.toLocaleString()}
                                    </div>
                                </div>

                                {/* Reposts */}
                                <div
                                    style={{
                                        padding: "14px",
                                        borderRadius: "var(--radius-md)",
                                        backgroundColor: "var(--bg-surface)",
                                        border: "1px solid var(--border-subtle)",
                                    }}
                                >
                                    <div
                                        style={{
                                            display: "flex",
                                            alignItems: "center",
                                            gap: "6px",
                                            color: "var(--text-muted)",
                                            fontSize: "12px",
                                            marginBottom: "6px",
                                        }}
                                    >
                                        <Repeat size={14} color="#10b981"/>
                                        <span>리포스트</span>
                                    </div>
                                    <div
                                        style={{
                                            fontSize: "20px",
                                            fontWeight: 800,
                                            color: "var(--text-primary)",
                                        }}
                                    >
                                        {analytics.repostsCount.toLocaleString()}
                                    </div>
                                </div>

                                {/* Comments */}
                                <div
                                    style={{
                                        padding: "14px",
                                        borderRadius: "var(--radius-md)",
                                        backgroundColor: "var(--bg-surface)",
                                        border: "1px solid var(--border-subtle)",
                                    }}
                                >
                                    <div
                                        style={{
                                            display: "flex",
                                            alignItems: "center",
                                            gap: "6px",
                                            color: "var(--text-muted)",
                                            fontSize: "12px",
                                            marginBottom: "6px",
                                        }}
                                    >
                                        <MessageCircle size={14} color="#a855f7"/>
                                        <span>댓글</span>
                                    </div>
                                    <div
                                        style={{
                                            fontSize: "20px",
                                            fontWeight: 800,
                                            color: "var(--text-primary)",
                                        }}
                                    >
                                        {analytics.commentsCount.toLocaleString()}
                                    </div>
                                </div>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
