"use client";

import React, {useCallback, useEffect, useState} from "react";
import Link from "next/link";
import {useParams, useRouter} from "next/navigation";
import {User} from "@/lib/types";
import {useAuth} from "@/lib/auth-context";
import {fetchGraphQL, MUTATIONS, QUERIES} from "@/lib/graphql";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import PostCard from "@/components/PostCard";
import StoryViewerModal from "@/components/StoryViewerModal";
import EditProfileModal from "@/components/EditProfileModal";
import TwoFactorModal from "@/components/TwoFactorModal";
import ReportModal from "@/components/ReportModal";
import {
    ArrowLeft,
    Calendar,
    Camera,
    Clock,
    Edit3,
    Flame,
    Grid,
    Heart,
    Link as LinkIcon,
    Lock,
    MapPin,
    MessageCircle,
    MoreHorizontal,
    Share2,
    ShieldAlert,
    ShieldCheck,
    UserCheck,
    UserPlus,
    UserX,
    VolumeX,
} from "lucide-react";

export default function ProfilePage() {
    const router = useRouter();
    const params = useParams();
    const username = params.username as string;
    const {user: currentUser} = useAuth();
    const {showToast} = useToast();

    const [profileUser, setProfileUser] = useState<User | null>(null);
    const [loading, setLoading] = useState(true);
    const [activeTab, setActiveTab] = useState<"posts" | "liked" | "stories">("posts");
    const [showEditModal, setShowEditModal] = useState(false);
    const [showStoryViewer, setShowStoryViewer] = useState(false);
    const [show2faModal, setShow2faModal] = useState(false);
    const [showReportModal, setShowReportModal] = useState(false);
    const [showMenu, setShowMenu] = useState(false);
    const [followLoading, setFollowLoading] = useState(false);

    const loadProfile = useCallback(async () => {
        if (!username) return;
        setLoading(true);
        try {
            const data = await fetchGraphQL<{ profile: User }>(QUERIES.USER_PROFILE, {
                username,
            });
            if (data?.profile) {
                setProfileUser(data.profile);
            }
        } catch (err) {
            console.error("Failed to load user profile:", err);
        } finally {
            setLoading(false);
        }
    }, [username]);

    useEffect(() => {
        loadProfile();
    }, [loadProfile]);

    const handleFollowToggle = async () => {
        if (!currentUser) {
            showToast("팔로우하려면 먼저 로그인하세요.", "info");
            return;
        }
        if (!profileUser || followLoading) return;

        setFollowLoading(true);
        const isFollowing = profileUser.isFollowedByMe;

        setProfileUser((prev) =>
            prev
                ? {
                    ...prev,
                    isFollowedByMe: !isFollowing,
                    followersCount: (prev.followersCount || 0) + (isFollowing ? -1 : 1),
                }
                : null
        );

        try {
            if (isFollowing) {
                await fetchGraphQL(MUTATIONS.UNFOLLOW_USER, {followeeId: profileUser.id});
                showToast(`@${profileUser.username} 님을 언팔로우했습니다.`, "info");
            } else {
                const res = await fetchGraphQL<{ followUser: { hasPendingFollowRequest?: boolean } }>(
                    MUTATIONS.FOLLOW_USER,
                    {followeeId: profileUser.id}
                );
                if (res?.followUser?.hasPendingFollowRequest) {
                    showToast(`비공개 계정입니다. @${profileUser.username} 님에게 팔로우 요청을 보냈습니다.`, "info");
                    setProfileUser((prev) => prev ? {
                        ...prev,
                        hasPendingFollowRequest: true,
                        isFollowedByMe: false
                    } : null);
                } else {
                    showToast(`@${profileUser.username} 님을 팔로우했습니다.`, "success");
                }
            }
        } catch (err: any) {
            setProfileUser((prev) =>
                prev
                    ? {
                        ...prev,
                        isFollowedByMe: isFollowing,
                        followersCount: (prev.followersCount || 0) + (isFollowing ? 1 : -1),
                    }
                    : null
            );
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setFollowLoading(false);
        }
    };

    const handleBlockToggle = async () => {
        if (!profileUser) return;
        try {
            if (profileUser.isBlockedByMe) {
                await fetchGraphQL(MUTATIONS.UNBLOCK_USER, {userId: profileUser.id});
                setProfileUser((prev) => prev ? {...prev, isBlockedByMe: false} : null);
                showToast(`@${profileUser.username} 님의 차단을 해제했습니다.`, "info");
            } else {
                await fetchGraphQL(MUTATIONS.BLOCK_USER, {userId: profileUser.id});
                setProfileUser((prev) => prev ? {...prev, isBlockedByMe: true} : null);
                showToast(`@${profileUser.username} 님을 차단했습니다.`, "info");
            }
            setShowMenu(false);
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        }
    };

    const handleMuteToggle = async () => {
        if (!profileUser) return;
        try {
            if (profileUser.isMutedByMe) {
                await fetchGraphQL(MUTATIONS.UNMUTE_USER, {userId: profileUser.id});
                setProfileUser((prev) => prev ? {...prev, isMutedByMe: false} : null);
                showToast(`@${profileUser.username} 님의 뮤트를 해제했습니다.`, "info");
            } else {
                await fetchGraphQL(MUTATIONS.MUTE_USER, {userId: profileUser.id});
                setProfileUser((prev) => prev ? {...prev, isMutedByMe: true} : null);
                showToast(`@${profileUser.username} 님을 뮤트했습니다.`, "info");
            }
            setShowMenu(false);
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        }
    };

    const handleShareProfile = () => {
        if (typeof window !== "undefined") {
            navigator.clipboard.writeText(window.location.href);
            showToast("프로필 링크가 복사되었습니다.", "success");
        }
    };

    const isMyProfile = currentUser && profileUser && currentUser.id === profileUser.id;
    const defaultAvatar = "https://api.dicebear.com/7.x/bottts/svg?seed=" + (username || "ferro");
    const hasStories = (profileUser?.stories && profileUser.stories.length > 0) || profileUser?.hasActiveStories;
    const isPrivateLocked = profileUser?.isPrivate && !profileUser.isFollowedByMe && !isMyProfile;

    if (loading && !profileUser) {
        return (
            <div>
                <header className="sticky-header">
                    <button onClick={() => router.back()} style={{color: "var(--text-secondary)", padding: "6px"}}>
                        <ArrowLeft size={20}/>
                    </button>
                    <div className="header-title">프로필</div>
                    <div style={{width: 20}}/>
                </header>
                <div style={{padding: "40px", textAlign: "center", color: "var(--text-muted)"}}>
                    프로필을 불러오는 중입니다...
                </div>
            </div>
        );
    }

    if (!profileUser) {
        return (
            <div>
                <header className="sticky-header">
                    <button onClick={() => router.back()} style={{color: "var(--text-secondary)", padding: "6px"}}>
                        <ArrowLeft size={20}/>
                    </button>
                    <div className="header-title">프로필</div>
                    <div style={{width: 20}}/>
                </header>
                <div style={{padding: "60px 20px", textAlign: "center"}}>
                    <h2 style={{fontSize: "18px", fontWeight: 800}}>사용자를 찾을 수 없습니다</h2>
                    <p style={{fontSize: "13px", color: "var(--text-muted)", marginTop: "8px"}}>
                        존재하지 않거나 삭제된 계정입니다.
                    </p>
                    <button onClick={() => router.push("/")} className="btn-primary" style={{marginTop: "16px"}}>
                        홈 피드로 돌아가기
                    </button>
                </div>
            </div>
        );
    }

    return (
        <div>
            {/* Sticky Header */}
            <header className="sticky-header">
                <div style={{display: "flex", alignItems: "center", gap: "14px"}}>
                    <button
                        onClick={() => router.back()}
                        style={{color: "var(--text-primary)", padding: "4px", borderRadius: "50%"}}
                        title="뒤로 가기"
                    >
                        <ArrowLeft size={20}/>
                    </button>
                    <div>
                        <div style={{display: "flex", alignItems: "center", gap: 6}}>
                            <h1 className="header-title" style={{fontSize: "18px"}}>
                                {profileUser.displayName || profileUser.username}
                            </h1>
                            {profileUser.isPrivate && <Lock size={14} color="var(--text-muted)"/>}
                        </div>
                        <div style={{fontSize: "12px", color: "var(--text-muted)"}}>
                            {profileUser.postsCount || 0}개의 게시물
                        </div>
                    </div>
                </div>

                <div style={{display: "flex", alignItems: "center", gap: "6px"}}>
                    <button onClick={handleShareProfile} title="프로필 공유"
                            style={{color: "var(--text-secondary)", padding: "8px"}}>
                        <Share2 size={18}/>
                    </button>

                    {!isMyProfile && (
                        <div style={{position: "relative"}}>
                            <button
                                onClick={() => setShowMenu(!showMenu)}
                                style={{color: "var(--text-secondary)", padding: "8px"}}
                            >
                                <MoreHorizontal size={18}/>
                            </button>

                            {showMenu && (
                                <div
                                    style={{
                                        position: "absolute",
                                        right: 0,
                                        top: "100%",
                                        zIndex: 20,
                                        backgroundColor: "var(--bg-surface)",
                                        border: "1px solid var(--border-subtle)",
                                        borderRadius: "var(--radius-md)",
                                        boxShadow: "var(--shadow-lg)",
                                        minWidth: "160px",
                                        overflow: "hidden",
                                    }}
                                >
                                    <button
                                        type="button"
                                        onClick={handleMuteToggle}
                                        className="dropdown-item"
                                        style={{
                                            display: "flex",
                                            alignItems: "center",
                                            gap: "8px",
                                            width: "100%",
                                            padding: "10px 14px",
                                            fontSize: "13px",
                                            color: "var(--text-primary)",
                                            textAlign: "left",
                                        }}
                                    >
                                        <VolumeX size={14}/>
                                        {profileUser.isMutedByMe ? "뮤트 해제" : "뮤트하기"}
                                    </button>

                                    <button
                                        type="button"
                                        onClick={handleBlockToggle}
                                        className="dropdown-item"
                                        style={{
                                            display: "flex",
                                            alignItems: "center",
                                            gap: "8px",
                                            width: "100%",
                                            padding: "10px 14px",
                                            fontSize: "13px",
                                            color: "var(--accent-secondary)",
                                            textAlign: "left",
                                        }}
                                    >
                                        <UserX size={14}/>
                                        {profileUser.isBlockedByMe ? "차단 해제" : "차단하기"}
                                    </button>

                                    <button
                                        type="button"
                                        onClick={() => {
                                            setShowReportModal(true);
                                            setShowMenu(false);
                                        }}
                                        className="dropdown-item"
                                        style={{
                                            display: "flex",
                                            alignItems: "center",
                                            gap: "8px",
                                            width: "100%",
                                            padding: "10px 14px",
                                            fontSize: "13px",
                                            color: "var(--accent-secondary)",
                                            textAlign: "left",
                                            borderTop: "1px solid var(--border-subtle)",
                                        }}
                                    >
                                        <ShieldAlert size={14}/>
                                        사용자 신고
                                    </button>
                                </div>
                            )}
                        </div>
                    )}
                </div>
            </header>

            {/* Cover Header */}
            <div
                className="profile-cover"
                style={{
                    backgroundImage: profileUser.headerImageUrl
                        ? `url(${profileUser.headerImageUrl})`
                        : "linear-gradient(135deg, #0b0f19 0%, #1e293b 50%, #0284c7 100%)",
                }}
            />

            {/* Avatar & Action Buttons Row */}
            <div className="profile-avatar-row">
                {/* Avatar */}
                <div
                    onClick={() => {
                        if (hasStories) setShowStoryViewer(true);
                    }}
                    style={{cursor: hasStories ? "pointer" : "default", position: "relative"}}
                    title={hasStories ? "24시간 스토리 보기" : ""}
                >
                    <div
                        className={`story-avatar-wrapper ${hasStories ? "unread" : ""}`}
                        style={{
                            width: 128,
                            height: 128,
                            padding: hasStories ? "3.5px" : "0",
                            boxShadow: "var(--shadow-md)",
                        }}
                    >
                        <div className="story-avatar-inner">
                            <img
                                src={profileUser.avatarUrl || defaultAvatar}
                                alt={profileUser.username}
                                className="story-avatar-img"
                            />
                        </div>
                    </div>
                    {hasStories && (
                        <div
                            style={{
                                position: "absolute",
                                bottom: 4,
                                right: 4,
                                backgroundColor: "#ec4899",
                                color: "#fff",
                                borderRadius: "50%",
                                padding: "4px",
                                border: "2px solid var(--bg-main)",
                                display: "flex",
                            }}
                            title="스토리 활성"
                        >
                            <Flame size={14}/>
                        </div>
                    )}
                </div>

                {/* Action Buttons */}
                <div style={{display: "flex", gap: "8px", alignItems: "center"}}>
                    {isMyProfile ? (
                        <>
                            <button
                                onClick={() => setShow2faModal(true)}
                                className="btn-secondary"
                                style={{
                                    fontWeight: 700,
                                    color: profileUser.is2faEnabled ? "#10b981" : "inherit",
                                }}
                                title="2단계 인증(2FA) 보안 설정"
                            >
                                <ShieldCheck size={15} color={profileUser.is2faEnabled ? "#10b981" : "currentColor"}/>
                                2FA 보안
                            </button>
                            <button
                                onClick={() => setShowEditModal(true)}
                                className="btn-secondary"
                                style={{fontWeight: 700}}
                            >
                                <Edit3 size={15}/> 프로필 수정
                            </button>
                        </>
                    ) : (
                        <>
                            {currentUser && (
                                <Link
                                    href={`/messages?user=${profileUser.username}`}
                                    className="btn-secondary"
                                    style={{padding: "9px 14px"}}
                                    title="1:1 메시지"
                                >
                                    <MessageCircle size={16}/>
                                </Link>
                            )}

                            <button
                                onClick={handleFollowToggle}
                                disabled={followLoading}
                                className={profileUser.isFollowedByMe ? "btn-secondary" : "btn-primary"}
                                style={{minWidth: "100px"}}
                            >
                                {profileUser.isFollowedByMe ? (
                                    <span style={{display: "flex", alignItems: "center", gap: 4}}>
                    <UserCheck size={14}/> 팔로잉
                  </span>
                                ) : profileUser.hasPendingFollowRequest ? (
                                    <span style={{display: "flex", alignItems: "center", gap: 4}}>
                    <Clock size={14}/> 요청됨
                  </span>
                                ) : (
                                    <span style={{display: "flex", alignItems: "center", gap: 4}}>
                    <UserPlus size={14}/> 팔로우
                  </span>
                                )}
                            </button>
                        </>
                    )}
                </div>
            </div>

            {/* Profile Details */}
            <div style={{padding: "16px 20px 0"}}>
                <h2 style={{fontSize: "22px", fontWeight: 800, color: "var(--text-primary)"}}>
                    {profileUser.displayName || profileUser.username}
                </h2>
                <div style={{fontSize: "14px", color: "var(--text-muted)", marginTop: "2px"}}>
                    @{profileUser.username}
                </div>

                {profileUser.bio && (
                    <p style={{fontSize: "14px", lineHeight: "1.6", color: "var(--text-primary)", marginTop: "12px"}}>
                        {profileUser.bio}
                    </p>
                )}

                <div className="profile-meta-row">
                    {profileUser.location && (
                        <div style={{display: "flex", alignItems: "center", gap: "6px"}}>
                            <MapPin size={14} color="var(--text-muted)"/>
                            <span>{profileUser.location}</span>
                        </div>
                    )}

                    {profileUser.website && (
                        <div style={{display: "flex", alignItems: "center", gap: "6px"}}>
                            <LinkIcon size={14} color="var(--accent-primary)"/>
                            <a
                                href={profileUser.website.startsWith("http") ? profileUser.website : `https://${profileUser.website}`}
                                target="_blank"
                                rel="noreferrer"
                                style={{color: "var(--accent-primary)", fontWeight: 600}}
                            >
                                {profileUser.website.replace(/^https?:\/\//, "")}
                            </a>
                        </div>
                    )}

                    <div style={{display: "flex", alignItems: "center", gap: "6px"}}>
                        <Calendar size={14} color="var(--text-muted)"/>
                        <span>가입일: {new Date(profileUser.createdAt).toLocaleDateString("ko-KR")}</span>
                    </div>
                </div>

                {/* Stats */}
                <div className="profile-stat-box">
                    <div>
            <span style={{fontWeight: 800, color: "var(--text-primary)"}}>
              {profileUser.followingCount || 0}
            </span>{" "}
                        <span style={{color: "var(--text-muted)"}}>팔로잉</span>
                    </div>
                    <div>
            <span style={{fontWeight: 800, color: "var(--text-primary)"}}>
              {profileUser.followersCount || 0}
            </span>{" "}
                        <span style={{color: "var(--text-muted)"}}>팔로워</span>
                    </div>
                    <div>
            <span style={{fontWeight: 800, color: "var(--text-primary)"}}>
              {profileUser.postsCount || 0}
            </span>{" "}
                        <span style={{color: "var(--text-muted)"}}>게시물</span>
                    </div>
                </div>
            </div>

            {/* Private Account Notice */}
            {isPrivateLocked ? (
                <div
                    style={{
                        padding: "80px 20px",
                        textAlign: "center",
                        borderTop: "1px solid var(--border-subtle)",
                        marginTop: "20px",
                    }}
                >
                    <div
                        style={{
                            width: 56,
                            height: 56,
                            borderRadius: "50%",
                            backgroundColor: "var(--bg-surface)",
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "center",
                            margin: "0 auto 16px",
                            color: "var(--text-secondary)",
                        }}
                    >
                        <Lock size={28}/>
                    </div>
                    <h3 style={{fontSize: "17px", fontWeight: 800, color: "var(--text-primary)"}}>
                        비공개 계정입니다
                    </h3>
                    <p style={{
                        fontSize: "13px",
                        color: "var(--text-secondary)",
                        marginTop: "6px",
                        maxWidth: "340px",
                        marginInline: "auto"
                    }}>
                        이 사용자의 게시물과 사진을 보려면 팔로우 요청을 보내고 승인을 받아야 합니다.
                    </p>
                </div>
            ) : (
                <>
                    {/* Tabs */}
                    <div className="profile-tabs">
                        <button
                            onClick={() => setActiveTab("posts")}
                            className={`profile-tab-btn ${activeTab === "posts" ? "active" : ""}`}
                        >
                            <Grid size={16}/>
                            <span>게시물 ({profileUser.posts?.length || profileUser.postsCount || 0})</span>
                        </button>

                        <button
                            onClick={() => setActiveTab("liked")}
                            className={`profile-tab-btn ${activeTab === "liked" ? "active" : ""}`}
                        >
                            <Heart size={16}/>
                            <span>좋아요한 글</span>
                        </button>

                        {hasStories && (
                            <button
                                onClick={() => setActiveTab("stories")}
                                className={`profile-tab-btn ${activeTab === "stories" ? "active" : ""}`}
                            >
                                <Camera size={16} color="#ec4899"/>
                                <span>24h 스토리</span>
                            </button>
                        )}
                    </div>

                    {/* Tab Content */}
                    <div style={{minHeight: "200px"}}>
                        {activeTab === "posts" && (
                            <div>
                                {!profileUser.posts || profileUser.posts.length === 0 ? (
                                    <div
                                        style={{padding: "60px 20px", textAlign: "center", color: "var(--text-muted)"}}>
                                        작성한 게시물이 없습니다.
                                    </div>
                                ) : (
                                    profileUser.posts.map((p) => (
                                        <PostCard key={p.id} post={p} onPostDeleted={loadProfile}
                                                  onPostUpdated={loadProfile}/>
                                    ))
                                )}
                            </div>
                        )}

                        {activeTab === "liked" && (
                            <div>
                                {!profileUser.likedPosts || profileUser.likedPosts.length === 0 ? (
                                    <div
                                        style={{padding: "60px 20px", textAlign: "center", color: "var(--text-muted)"}}>
                                        좋아요를 표시한 게시물이 없습니다.
                                    </div>
                                ) : (
                                    profileUser.likedPosts.map((p) => (
                                        <PostCard key={p.id} post={p} onPostDeleted={loadProfile}
                                                  onPostUpdated={loadProfile}/>
                                    ))
                                )}
                            </div>
                        )}

                        {activeTab === "stories" && (
                            <div style={{padding: "20px"}}>
                                {profileUser.stories && profileUser.stories.length > 0 ? (
                                    <div style={{
                                        display: "grid",
                                        gridTemplateColumns: "repeat(auto-fill, minmax(140px, 1fr))",
                                        gap: "12px"
                                    }}>
                                        {profileUser.stories.map((s) => (
                                            <div
                                                key={s.id}
                                                onClick={() => setShowStoryViewer(true)}
                                                style={{
                                                    position: "relative",
                                                    height: "200px",
                                                    borderRadius: "var(--radius-md)",
                                                    overflow: "hidden",
                                                    cursor: "pointer",
                                                    backgroundColor: "var(--bg-surface)",
                                                    border: "1px solid var(--border-subtle)",
                                                }}
                                            >
                                                <img
                                                    src={s.mediaUrl}
                                                    alt="Story"
                                                    style={{width: "100%", height: "100%", objectFit: "cover"}}
                                                />
                                                <div
                                                    style={{
                                                        position: "absolute",
                                                        bottom: 0,
                                                        left: 0,
                                                        right: 0,
                                                        padding: "8px",
                                                        background: "linear-gradient(transparent, rgba(0,0,0,0.8))",
                                                        fontSize: "11px",
                                                        color: "#fff",
                                                    }}
                                                >
                                                    {s.caption || "24시간 스토리"}
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                ) : (
                                    <div style={{padding: "40px", textAlign: "center", color: "var(--text-muted)"}}>
                                        활성화된 24시간 스토리가 없습니다.
                                    </div>
                                )}
                            </div>
                        )}
                    </div>
                </>
            )}

            {/* Modals */}
            {showEditModal && (
                <EditProfileModal
                    user={profileUser}
                    onClose={() => setShowEditModal(false)}
                    onUpdated={loadProfile}
                />
            )}

            {show2faModal && (
                <TwoFactorModal
                    onClose={() => setShow2faModal(false)}
                    onStatusChanged={loadProfile}
                />
            )}

            {showReportModal && (
                <ReportModal
                    targetType="USER"
                    targetId={profileUser.id}
                    targetTitle={`@${profileUser.username}`}
                    onClose={() => setShowReportModal(false)}
                />
            )}

            {showStoryViewer && profileUser.stories && profileUser.stories.length > 0 && (
                <StoryViewerModal
                    storyGroups={[
                        {
                            username: profileUser.username,
                            author: profileUser,
                            stories: profileUser.stories,
                        },
                    ]}
                    initialGroupIndex={0}
                    onClose={() => setShowStoryViewer(false)}
                />
            )}
        </div>
    );
}
