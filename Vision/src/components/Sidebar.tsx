"use client";

import React, {useEffect, useState} from "react";
import Link from "next/link";
import {usePathname} from "next/navigation";
import {useAuth} from "@/lib/auth-context";
import {
    Bell,
    Bookmark,
    Camera,
    Compass,
    Globe,
    Home,
    ListFilter,
    LogIn,
    LogOut,
    MessageCircle,
    PlusCircle,
    Sparkles,
    User as UserIcon,
} from "lucide-react";
import AuthModal from "./AuthModal";
import StoryCreateModal from "./StoryCreateModal";
import PostComposerModal from "./PostComposerModal";
import {fetchGraphQL, QUERIES} from "@/lib/graphql";

export default function Sidebar() {
    const pathname = usePathname();
    const {user, logout} = useAuth();
    const [showAuthModal, setShowAuthModal] = useState(false);
    const [showStoryModal, setShowStoryModal] = useState(false);
    const [showPostModal, setShowPostModal] = useState(false);
    const [unreadDmCount, setUnreadDmCount] = useState(0);
    const [unreadNotifCount, setUnreadNotifCount] = useState(0);

    useEffect(() => {
        if (!user) {
            setUnreadDmCount(0);
            setUnreadNotifCount(0);
            return;
        }

        const checkBadges = async () => {
            try {
                const dmData = await fetchGraphQL<{ unreadDmCount: number }>(
                    QUERIES.UNREAD_DM_COUNT
                ).catch(() => null);
                if (typeof dmData?.unreadDmCount === "number") {
                    setUnreadDmCount(dmData.unreadDmCount);
                }

                const notifData = await fetchGraphQL<{ unreadNotificationsCount: number }>(
                    QUERIES.UNREAD_NOTIFICATIONS_COUNT
                ).catch(() => null);
                if (typeof notifData?.unreadNotificationsCount === "number") {
                    setUnreadNotifCount(notifData.unreadNotificationsCount);
                }
            } catch (e) {
                // silent
            }
        };

        checkBadges();
        const interval = setInterval(checkBadges, 8000);
        return () => clearInterval(interval);
    }, [user, pathname]);

    const defaultAvatar =
        "https://api.dicebear.com/7.x/bottts/svg?seed=" + (user?.username || "ferro");

    return (
        <>
            {/* Desktop & Tablet Sidebar */}
            <aside className="sidebar">
                <div>
                    {/* Logo */}
                    <Link href="/" className="logo-container">
                        <div className="logo-badge">🦀</div>
                        <div className="logo-text">
                            Ferro
                            <span className="logo-subtext">Vision SNS</span>
                        </div>
                    </Link>

                    {/* Navigation Links */}
                    <nav className="nav-links">
                        <Link
                            href="/"
                            className={`nav-item ${pathname === "/" ? "active" : ""}`}
                        >
                            <Home size={20}/>
                            <span>피드 (Home)</span>
                        </Link>

                        <Link
                            href="/explore"
                            className={`nav-item ${pathname === "/explore" ? "active" : ""}`}
                        >
                            <Compass size={20}/>
                            <span>탐색 (Explore)</span>
                        </Link>

                        {user && (
                            <>
                                <Link
                                    href="/notifications"
                                    className={`nav-item ${
                                        pathname === "/notifications" ? "active" : ""
                                    }`}
                                >
                                    <Bell size={20}/>
                                    <span>알림 (Notifications)</span>
                                    {unreadNotifCount > 0 && (
                                        <span className="badge-count">{unreadNotifCount}</span>
                                    )}
                                </Link>

                                <Link
                                    href="/messages"
                                    className={`nav-item ${
                                        pathname.startsWith("/messages") ? "active" : ""
                                    }`}
                                >
                                    <MessageCircle size={20}/>
                                    <span>메시지 (DM)</span>
                                    {unreadDmCount > 0 && (
                                        <span className="badge-count">{unreadDmCount}</span>
                                    )}
                                </Link>

                                <Link
                                    href="/bookmarks"
                                    className={`nav-item ${
                                        pathname === "/bookmarks" ? "active" : ""
                                    }`}
                                >
                                    <Bookmark size={20}/>
                                    <span>북마크 (Bookmarks)</span>
                                </Link>

                                <Link
                                    href="/lists"
                                    className={`nav-item ${pathname === "/lists" ? "active" : ""}`}
                                >
                                    <ListFilter size={20}/>
                                    <span>리스트 (Lists)</span>
                                </Link>

                                <Link
                                    href={`/profile/${user.username}`}
                                    className={`nav-item ${
                                        pathname === `/profile/${user.username}` ? "active" : ""
                                    }`}
                                >
                                    <UserIcon size={20}/>
                                    <span>내 프로필</span>
                                </Link>
                            </>
                        )}

                        {/* Welcome / Landing page link */}
                        <Link
                            href="/welcome"
                            className={`nav-item ${pathname === "/welcome" ? "active" : ""}`}
                        >
                            <Sparkles size={20} color="#a855f7"/>
                            <span>소개 & 가입</span>
                        </Link>

                        {/* Quick Actions for logged in user */}
                        {user && (
                            <button
                                onClick={() => setShowStoryModal(true)}
                                className="nav-item"
                                style={{textAlign: "left", width: "100%"}}
                            >
                                <Camera size={20} color="#ec4899"/>
                                <span>스토리 올리기</span>
                            </button>
                        )}

                        {user ? (
                            <button
                                onClick={() => setShowPostModal(true)}
                                className="create-btn-nav"
                            >
                                <PlusCircle size={18}/>
                                <span>새 게시물 작성</span>
                            </button>
                        ) : (
                            <div
                                style={{
                                    marginTop: "16px",
                                    display: "flex",
                                    flexDirection: "column",
                                    gap: "8px",
                                }}
                            >
                                <button
                                    onClick={() => setShowAuthModal(true)}
                                    className="create-btn-nav"
                                >
                                    <LogIn size={18}/>
                                    <span>로그인 / 가입</span>
                                </button>
                                <Link
                                    href="/welcome"
                                    className="btn-secondary"
                                    style={{
                                        width: "100%",
                                        justifyContent: "center",
                                        borderRadius: "var(--radius-full)",
                                    }}
                                >
                                    <Globe size={15}/> 랜딩 페이지 보기
                                </Link>
                            </div>
                        )}
                    </nav>
                </div>

                {/* User Card at bottom */}
                {user ? (
                    <div className="user-nav-card">
                        <Link
                            href={`/profile/${user.username}`}
                            style={{
                                display: "flex",
                                alignItems: "center",
                                gap: "10px",
                                flex: 1,
                                minWidth: 0,
                            }}
                        >
                            <div style={{position: "relative"}}>
                                <img
                                    src={user.avatarUrl || defaultAvatar}
                                    alt={user.username}
                                    style={{
                                        width: 40,
                                        height: 40,
                                        borderRadius: "50%",
                                        objectFit: "cover",
                                    }}
                                />
                                <div
                                    style={{
                                        position: "absolute",
                                        bottom: 0,
                                        right: 0,
                                        width: 10,
                                        height: 10,
                                        borderRadius: "50%",
                                        backgroundColor: "#10b981",
                                        border: "2px solid var(--bg-surface)",
                                    }}
                                />
                            </div>
                            <div
                                className="user-nav-details"
                                style={{overflow: "hidden", textOverflow: "ellipsis"}}
                            >
                                <div
                                    style={{
                                        fontWeight: 700,
                                        fontSize: "14px",
                                        color: "var(--text-primary)",
                                        whiteSpace: "nowrap",
                                        overflow: "hidden",
                                        textOverflow: "ellipsis",
                                    }}
                                >
                                    {user.displayName || user.username}
                                </div>
                                <div
                                    style={{
                                        fontSize: "12px",
                                        color: "var(--text-muted)",
                                        whiteSpace: "nowrap",
                                        overflow: "hidden",
                                        textOverflow: "ellipsis",
                                    }}
                                >
                                    @{user.username}
                                </div>
                            </div>
                        </Link>
                        <button
                            onClick={logout}
                            title="로그아웃"
                            style={{
                                color: "var(--text-muted)",
                                padding: "6px",
                                borderRadius: "8px",
                            }}
                            onMouseEnter={(e) => (e.currentTarget.style.color = "#f43f5e")}
                            onMouseLeave={(e) => (e.currentTarget.style.color = "var(--text-muted)")}
                        >
                            <LogOut size={18}/>
                        </button>
                    </div>
                ) : (
                    <div
                        onClick={() => setShowAuthModal(true)}
                        style={{
                            padding: "16px",
                            borderRadius: "var(--radius-lg)",
                            background:
                                "linear-gradient(135deg, rgba(56, 189, 248, 0.1) 0%, rgba(168, 85, 247, 0.1) 100%)",
                            border: "1px solid rgba(56, 189, 248, 0.3)",
                            cursor: "pointer",
                            textAlign: "center",
                            transition: "all var(--transition-fast)",
                        }}
                    >
                        <div
                            style={{
                                fontSize: "13px",
                                fontWeight: 700,
                                color: "var(--accent-primary)",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                                gap: "6px",
                                marginBottom: "4px",
                            }}
                        >
                            <Sparkles size={16}/> Ferro로 시작하기
                        </div>
                        <div style={{fontSize: "11px", color: "var(--text-secondary)"}}>
                            스토리 올리고 실시간 DM 나누기
                        </div>
                    </div>
                )}
            </aside>

            {/* Mobile Bottom Navigation */}
            <nav className="mobile-bottom-nav">
                <Link
                    href="/"
                    className={`mobile-nav-item ${pathname === "/" ? "active" : ""}`}
                >
                    <Home size={22}/>
                    <span>홈</span>
                </Link>
                <Link
                    href="/explore"
                    className={`mobile-nav-item ${pathname === "/explore" ? "active" : ""}`}
                >
                    <Compass size={22}/>
                    <span>탐색</span>
                </Link>
                {user ? (
                    <>
                        <button
                            onClick={() => setShowPostModal(true)}
                            className="mobile-nav-item"
                            style={{color: "var(--accent-primary)"}}
                        >
                            <PlusCircle size={26}/>
                            <span>작성</span>
                        </button>
                        <Link
                            href="/notifications"
                            className={`mobile-nav-item ${
                                pathname === "/notifications" ? "active" : ""
                            }`}
                        >
                            <Bell size={22}/>
                            {unreadNotifCount > 0 && (
                                <span
                                    className="badge-count"
                                    style={{
                                        position: "absolute",
                                        top: 4,
                                        right: 12,
                                    }}
                                >
                  {unreadNotifCount}
                </span>
                            )}
                            <span>알림</span>
                        </Link>
                        <Link
                            href={`/profile/${user.username}`}
                            className={`mobile-nav-item ${
                                pathname === `/profile/${user.username}` ? "active" : ""
                            }`}
                        >
                            <UserIcon size={22}/>
                            <span>프로필</span>
                        </Link>
                    </>
                ) : (
                    <button
                        onClick={() => setShowAuthModal(true)}
                        className="mobile-nav-item"
                    >
                        <LogIn size={22}/>
                        <span>로그인</span>
                    </button>
                )}
            </nav>

            {/* Modals */}
            {showAuthModal && <AuthModal onClose={() => setShowAuthModal(false)}/>}
            {showStoryModal && (
                <StoryCreateModal onClose={() => setShowStoryModal(false)}/>
            )}
            {showPostModal && (
                <PostComposerModal onClose={() => setShowPostModal(false)}/>
            )}
        </>
    );
}
