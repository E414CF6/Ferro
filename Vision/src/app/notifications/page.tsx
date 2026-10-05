"use client";

import React, { useEffect, useState } from "react";
import Link from "next/link";
import { FollowRequest, Notification } from "@/lib/types";
import { useAuth } from "@/lib/auth-context";
import {
    fetchGraphQL,
    MUTATIONS,
    QUERIES,
    subscribeToNotifications,
} from "@/lib/graphql";
import { useToast } from "@/lib/toast-context";
import { formatErrorMessage } from "@/lib/i18n";
import {
    BarChart2,
    Bell,
    Check,
    CheckCheck,
    Heart,
    MessageCircle,
    Quote,
    Repeat,
    UserCheck,
    UserPlus,
    X,
} from "lucide-react";

export default function NotificationsPage() {
    const { user } = useAuth();
    const { showToast } = useToast();
    const [notifications, setNotifications] = useState<Notification[]>([]);
    const [pendingRequests, setPendingRequests] = useState<FollowRequest[]>([]);
    const [loading, setLoading] = useState(true);

    const loadData = async () => {
        if (!user) return;
        try {
            // Load Notifications
            const notifData = await fetchGraphQL<{ notifications: Notification[] }>(
                QUERIES.NOTIFICATIONS,
                { limit: 50, offset: 0 }
            ).catch(() => null);

            if (notifData?.notifications) {
                setNotifications(notifData.notifications);
            }

            // Load Pending Follow Requests if account is private
            const reqData = await fetchGraphQL<{ pendingFollowRequests: FollowRequest[] }>(
                QUERIES.PENDING_FOLLOW_REQUESTS
            ).catch(() => null);

            if (reqData?.pendingFollowRequests) {
                setPendingRequests(reqData.pendingFollowRequests);
            }
        } catch (err) {
            console.error("Failed to fetch notifications data:", err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        if (user) {
            loadData();

            // WebSocket real-time notification subscription
            const unsubscribe = subscribeToNotifications(user.id, (newNotif) => {
                setNotifications((prev) => [newNotif, ...prev]);
                showToast("새로운 알림이 도착했습니다!", "info");
            });

            return () => unsubscribe();
        } else {
            setLoading(false);
        }
    }, [user]);

    const handleMarkAllRead = async () => {
        try {
            await fetchGraphQL(MUTATIONS.MARK_ALL_NOTIFICATIONS_AS_READ);
            setNotifications((prev) => prev.map((n) => ({ ...n, isRead: true })));
            showToast("모든 알림을 읽음으로 처리했습니다.", "success");
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        }
    };

    const handleMarkSingleRead = async (notifId: string) => {
        try {
            await fetchGraphQL(MUTATIONS.MARK_NOTIFICATION_AS_READ, { notificationId: notifId });
            setNotifications((prev) =>
                prev.map((n) => (n.id === notifId ? { ...n, isRead: true } : n))
            );
        } catch {
            // silent
        }
    };

    const handleAcceptFollowRequest = async (req: FollowRequest) => {
        try {
            await fetchGraphQL(MUTATIONS.ACCEPT_FOLLOW_REQUEST, { requesterId: req.requesterId });
            setPendingRequests((prev) => prev.filter((r) => r.id !== req.id));
            showToast(`@${req.requester.username} 님의 팔로우 요청을 수락했습니다.`, "success");
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        }
    };

    const handleRejectFollowRequest = async (req: FollowRequest) => {
        try {
            await fetchGraphQL(MUTATIONS.REJECT_FOLLOW_REQUEST, { requesterId: req.requesterId });
            setPendingRequests((prev) => prev.filter((r) => r.id !== req.id));
            showToast("팔로우 요청을 거절했습니다.", "info");
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        }
    };

    const getNotificationIcon = (type: string) => {
        switch (type) {
            case "LIKE_POST":
            case "LIKE_COMMENT":
                return <Heart size={16} color="#f43f5e" fill="#f43f5e" />;
            case "COMMENT_POST":
            case "REPLY_COMMENT":
                return <MessageCircle size={16} color="#a855f7" />;
            case "REPOST":
                return <Repeat size={16} color="#10b981" />;
            case "QUOTE":
                return <Quote size={16} color="var(--accent-primary)" />;
            case "FOLLOW":
            case "FOLLOW_ACCEPTED":
            case "FOLLOW_REQUEST":
                return <UserPlus size={16} color="var(--accent-primary)" />;
            case "POLL_ENDED":
                return <BarChart2 size={16} color="#eab308" />;
            default:
                return <Bell size={16} color="var(--accent-primary)" />;
        }
    };

    const getNotificationText = (n: Notification) => {
        const actor = n.actor || n.sender;
        const actorName = actor?.displayName || actor?.username || "누군가";
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
            case "DIRECT_MESSAGE":
                return `${actorName} 님으로부터 새 메시지가 도착했습니다.`;
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
                <div style={{ padding: "60px 20px", textAlign: "center" }}>
                    <Bell size={40} color="var(--text-muted)" style={{ margin: "0 auto 16px" }} />
                    <h3 style={{ fontSize: "18px", fontWeight: 800 }}>로그인이 필요합니다</h3>
                    <p style={{ fontSize: "14px", color: "var(--text-secondary)", marginTop: "8px" }}>
                        알림을 확인하려면 계정에 로그인하세요.
                    </p>
                    <Link
                        href="/welcome?tab=login"
                        className="btn-primary"
                        style={{ marginTop: "16px", display: "inline-flex" }}
                    >
                        로그인하기
                    </Link>
                </div>
            </div>
        );
    }

    return (
        <div>
            <header className="sticky-header">
                <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                    <Bell size={20} color="var(--accent-primary)" />
                    <h1 className="header-title">알림</h1>
                </div>

                {notifications.some((n) => !n.isRead) && (
                    <button
                        onClick={handleMarkAllRead}
                        className="btn-secondary"
                        style={{ padding: "6px 12px", fontSize: "12px" }}
                    >
                        <CheckCheck size={14} /> 모두 읽음 표시
                    </button>
                )}
            </header>

            {/* Pending Follow Requests Drawer / Section */}
            {pendingRequests.length > 0 && (
                <div
                    style={{
                        padding: "16px 20px",
                        backgroundColor: "rgba(56, 189, 248, 0.08)",
                        borderBottom: "1px solid rgba(56, 189, 248, 0.2)",
                    }}
                >
                    <div style={{ fontSize: "13px", fontWeight: 800, color: "var(--accent-primary)", marginBottom: "10px", display: "flex", alignItems: "center", gap: "6px" }}>
                        <UserCheck size={16} /> 대기 중인 팔로우 요청 ({pendingRequests.length}건)
                    </div>

                    <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
                        {pendingRequests.map((req) => (
                            <div
                                key={req.id}
                                style={{
                                    display: "flex",
                                    alignItems: "center",
                                    justifyContent: "space-between",
                                    backgroundColor: "var(--bg-surface)",
                                    padding: "10px 14px",
                                    borderRadius: "var(--radius-md)",
                                    border: "1px solid var(--border-subtle)",
                                }}
                            >
                                <Link
                                    href={`/profile/${req.requester.username}`}
                                    style={{ display: "flex", alignItems: "center", gap: "10px" }}
                                >
                                    <img
                                        src={req.requester.avatarUrl || defaultAvatar}
                                        alt={req.requester.username}
                                        style={{ width: 36, height: 36, borderRadius: "50%" }}
                                    />
                                    <div>
                                        <div style={{ fontSize: "13px", fontWeight: 700, color: "var(--text-primary)" }}>
                                            {req.requester.displayName || req.requester.username}
                                        </div>
                                        <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                                            @{req.requester.username}
                                        </div>
                                    </div>
                                </Link>

                                <div style={{ display: "flex", gap: "6px" }}>
                                    <button
                                        onClick={() => handleAcceptFollowRequest(req)}
                                        className="btn-primary"
                                        style={{ padding: "5px 12px", fontSize: "12px" }}
                                    >
                                        <Check size={13} /> 수락
                                    </button>
                                    <button
                                        onClick={() => handleRejectFollowRequest(req)}
                                        className="btn-secondary"
                                        style={{ padding: "5px 10px", fontSize: "12px", color: "var(--accent-secondary)" }}
                                    >
                                        <X size={13} /> 거절
                                    </button>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            )}

            {/* Notifications List */}
            <div style={{ minHeight: "300px" }}>
                {loading ? (
                    <div style={{ padding: "40px", textAlign: "center", color: "var(--text-muted)" }}>
                        알림을 불러오는 중...
                    </div>
                ) : notifications.length === 0 ? (
                    <div style={{ padding: "80px 20px", textAlign: "center", color: "var(--text-muted)" }}>
                        <Bell size={44} style={{ opacity: 0.3, margin: "0 auto 14px" }} />
                        <h3 style={{ fontSize: "17px", fontWeight: 700, color: "var(--text-primary)" }}>
                            새로운 알림이 없습니다
                        </h3>
                        <p style={{ fontSize: "13px", marginTop: "6px" }}>
                            좋아요, 댓글, 팔로우 등의 활동이 여기에 실시간으로 표시됩니다.
                        </p>
                    </div>
                ) : (
                    notifications.map((n) => {
                        const actor = n.actor || n.sender;
                        const actorAvatar = actor?.avatarUrl || defaultAvatar;

                        return (
                            <div
                                key={n.id}
                                onClick={() => !n.isRead && handleMarkSingleRead(n.id)}
                                style={{
                                    display: "flex",
                                    alignItems: "flex-start",
                                    gap: "14px",
                                    padding: "16px 20px",
                                    borderBottom: "1px solid var(--border-subtle)",
                                    backgroundColor: n.isRead ? "transparent" : "rgba(56, 189, 248, 0.04)",
                                    transition: "background-color 0.15s ease",
                                }}
                            >
                                <div style={{ marginTop: "2px", flexShrink: 0 }}>
                                    {getNotificationIcon(n.notificationType)}
                                </div>

                                <div style={{ flex: 1 }}>
                                    <div style={{ display: "flex", alignItems: "center", gap: "10px", marginBottom: "4px" }}>
                                        {actor ? (
                                            <Link href={`/profile/${actor.username}`}>
                                                <img
                                                    src={actorAvatar}
                                                    alt={actor.username}
                                                    style={{ width: 28, height: 28, borderRadius: "50%", objectFit: "cover" }}
                                                />
                                            </Link>
                                        ) : null}
                                        <span style={{ fontSize: "14px", color: "var(--text-primary)" }}>
                                            {getNotificationText(n)}
                                        </span>
                                    </div>

                                    {/* Snippet preview */}
                                    {(n.targetPost || n.post) && (
                                        <div
                                            style={{
                                                fontSize: "12px",
                                                color: "var(--text-secondary)",
                                                backgroundColor: "var(--bg-surface)",
                                                padding: "6px 12px",
                                                borderRadius: "var(--radius-sm)",
                                                marginTop: "4px",
                                                borderLeft: "2px solid var(--accent-primary)",
                                                overflow: "hidden",
                                                textOverflow: "ellipsis",
                                                whiteSpace: "nowrap",
                                                maxWidth: "500px",
                                            }}
                                        >
                                            {(n.targetPost || n.post)?.content}
                                        </div>
                                    )}

                                    <div style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "4px" }}>
                                        {new Date(n.createdAt).toLocaleDateString("ko-KR", {
                                            month: "short",
                                            day: "numeric",
                                            hour: "2-digit",
                                            minute: "2-digit",
                                        })}
                                    </div>
                                </div>
                            </div>
                        );
                    })
                )}
            </div>
        </div>
    );
}
