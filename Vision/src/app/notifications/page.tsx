"use client";

import React, {useEffect, useState} from "react";
import Link from "next/link";
import {Notification} from "@/lib/types";
import {useAuth} from "@/lib/auth-context";
import {fetchGraphQL, MUTATIONS, QUERIES} from "@/lib/graphql";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import {BarChart2, Bell, CheckCheck, Heart, MessageCircle, Quote, Repeat, UserPlus,} from "lucide-react";

export default function NotificationsPage() {
    const {user} = useAuth();
    const {showToast} = useToast();
    const [notifications, setNotifications] = useState<Notification[]>([]);
    const [loading, setLoading] = useState(true);

    const loadNotifications = async () => {
        try {
            const data = await fetchGraphQL<{ notifications: Notification[] }>(
                QUERIES.NOTIFICATIONS,
                {limit: 50, offset: 0}
            );
            if (data?.notifications) {
                setNotifications(data.notifications);
            }
        } catch (err) {
            console.error("Failed to fetch notifications:", err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        if (user) {
            loadNotifications();
        } else {
            setLoading(false);
        }
    }, [user]);

    const handleMarkAllRead = async () => {
        try {
            await fetchGraphQL(MUTATIONS.MARK_NOTIFICATIONS_AS_READ);
            setNotifications((prev) => prev.map((n) => ({...n, isRead: true})));
            showToast("모든 알림을 읽음으로 처리했습니다.", "success");
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        }
    };

    const getNotificationIcon = (type: string) => {
        switch (type) {
            case "LIKE_POST":
            case "LIKE_COMMENT":
                return <Heart size={16} color="#f43f5e" fill="#f43f5e"/>;
            case "COMMENT_POST":
            case "REPLY_COMMENT":
                return <MessageCircle size={16} color="#a855f7"/>;
            case "REPOST":
                return <Repeat size={16} color="#10b981"/>;
            case "QUOTE":
                return <Quote size={16} color="var(--accent-primary)"/>;
            case "FOLLOW":
            case "FOLLOW_ACCEPTED":
            case "FOLLOW_REQUEST":
                return <UserPlus size={16} color="var(--accent-primary)"/>;
            case "POLL_ENDED":
                return <BarChart2 size={16} color="#eab308"/>;
            default:
                return <Bell size={16} color="var(--accent-primary)"/>;
        }
    };

    const getNotificationText = (n: Notification) => {
        const actorName = n.actor?.displayName || n.actor?.username || "누군가";
        switch (n.notificationType) {
            case "LIKE_POST":
                return `${actorName} 님이 내 게시물을 좋아합니다.`;
            case "LIKE_COMMENT":
                return `${actorName} 님이 내 댓글을 좋아합니다.`;
            case "COMMENT_POST":
                return `${actorName} 님이 내 게시물에 댓글을 남겼습니다.`;
            case "REPLY_COMMENT":
                return `${actorName} 님이 내 댓글에 답글을 남겼습니다.`;
            case "REPOST":
                return `${actorName} 님이 내 게시물을 리포스트했습니다.`;
            case "QUOTE":
                return `${actorName} 님이 내 게시물을 인용했습니다.`;
            case "FOLLOW":
                return `${actorName} 님이 나를 팔로우하기 시작했습니다.`;
            case "FOLLOW_REQUEST":
                return `${actorName} 님이 팔로우를 요청했습니다.`;
            case "FOLLOW_ACCEPTED":
                return `${actorName} 님이 팔로우 요청을 수락했습니다.`;
            case "POLL_ENDED":
                return `내가 참여한 투표가 종료되었습니다.`;
            default:
                return `새로운 알림이 도착했습니다.`;
        }
    };

    const defaultAvatar =
        "https://api.dicebear.com/7.x/bottts/svg?seed=" + (user?.username || "user");

    if (!user) {
        return (
            <div>
                <header className="sticky-header">
                    <h1 className="header-title">알림</h1>
                </header>
                <div style={{padding: "60px 20px", textAlign: "center"}}>
                    <Bell size={40} color="var(--text-muted)" style={{margin: "0 auto 16px"}}/>
                    <h3 style={{fontSize: "18px", fontWeight: 800}}>로그인이 필요합니다</h3>
                    <p style={{fontSize: "14px", color: "var(--text-secondary)", marginTop: "8px"}}>
                        알림을 확인하려면 계정에 로그인하세요.
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
                    <Bell size={20} color="var(--accent-primary)"/>
                    <h1 className="header-title">알림</h1>
                </div>

                {notifications.some((n) => !n.isRead) && (
                    <button
                        onClick={handleMarkAllRead}
                        className="btn-secondary"
                        style={{padding: "6px 12px", fontSize: "12px"}}
                    >
                        <CheckCheck size={14}/> 모두 읽음 표시
                    </button>
                )}
            </header>

            <div style={{minHeight: "300px"}}>
                {loading ? (
                    <div style={{padding: "40px", textAlign: "center", color: "var(--text-muted)"}}>
                        알림을 불러오는 중...
                    </div>
                ) : notifications.length === 0 ? (
                    <div style={{padding: "80px 20px", textAlign: "center", color: "var(--text-muted)"}}>
                        <Bell size={40} style={{margin: "0 auto 16px", opacity: 0.5}}/>
                        <h3 style={{fontSize: "17px", fontWeight: 800, color: "var(--text-primary)"}}>
                            새로운 알림이 없습니다
                        </h3>
                        <p style={{fontSize: "13px", color: "var(--text-secondary)", marginTop: "6px"}}>
                            내 게시물에 인터랙션이 발생하면 실시간으로 알림을 받을 수 있습니다.
                        </p>
                    </div>
                ) : (
                    <div>
                        {notifications.map((n) => (
                            <div
                                key={n.id}
                                style={{
                                    padding: "16px 20px",
                                    borderBottom: "1px solid var(--border-subtle)",
                                    backgroundColor: n.isRead ? "transparent" : "rgba(56, 189, 248, 0.04)",
                                    display: "flex",
                                    alignItems: "flex-start",
                                    gap: "14px",
                                    transition: "background-color var(--transition-fast)",
                                }}
                            >
                                <div style={{marginTop: "3px"}}>{getNotificationIcon(n.notificationType)}</div>

                                <div style={{flex: 1}}>
                                    <div style={{
                                        display: "flex",
                                        alignItems: "center",
                                        gap: "8px",
                                        marginBottom: "4px"
                                    }}>
                                        {n.actor && (
                                            <Link href={`/profile/${n.actor.username}`}>
                                                <img
                                                    src={n.actor.avatarUrl || defaultAvatar}
                                                    alt={n.actor.username}
                                                    style={{
                                                        width: 28,
                                                        height: 28,
                                                        borderRadius: "50%",
                                                        objectFit: "cover"
                                                    }}
                                                />
                                            </Link>
                                        )}
                                        <span style={{fontSize: "14px", color: "var(--text-primary)"}}>
                      {getNotificationText(n)}
                    </span>
                                    </div>

                                    {n.comment && (
                                        <div
                                            style={{
                                                padding: "8px 12px",
                                                borderRadius: "var(--radius-md)",
                                                backgroundColor: "var(--bg-surface)",
                                                fontSize: "13px",
                                                color: "var(--text-secondary)",
                                                marginTop: "6px",
                                            }}
                                        >
                                            "{n.comment.content}"
                                        </div>
                                    )}

                                    {n.post && !n.comment && (
                                        <div
                                            style={{
                                                fontSize: "12px",
                                                color: "var(--text-muted)",
                                                marginTop: "4px",
                                                overflow: "hidden",
                                                textOverflow: "ellipsis",
                                                whiteSpace: "nowrap",
                                            }}
                                        >
                                            게시물: {n.post.content}
                                        </div>
                                    )}

                                    <div style={{fontSize: "11px", color: "var(--text-muted)", marginTop: "6px"}}>
                                        {new Date(n.createdAt).toLocaleString("ko-KR")}
                                    </div>
                                </div>

                                {!n.isRead && (
                                    <div
                                        style={{
                                            width: 8,
                                            height: 8,
                                            borderRadius: "50%",
                                            backgroundColor: "var(--accent-primary)",
                                            marginTop: "6px",
                                        }}
                                    />
                                )}
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
}
