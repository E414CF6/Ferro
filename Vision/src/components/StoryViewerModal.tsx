"use client";

import React, {useCallback, useEffect, useState} from "react";
import {Story, User} from "@/lib/types";
import {useAuth} from "@/lib/auth-context";
import {fetchGraphQL, MUTATIONS, QUERIES} from "@/lib/graphql";
import {useToast} from "@/lib/toast-context";
import {ChevronLeft, ChevronRight, Clock, Eye, Trash2, Users, X,} from "lucide-react";

interface StoryGroup {
    username: string;
    author: {
        id?: string;
        username: string;
        displayName: string;
        avatarUrl?: string | null;
    };
    stories: Story[];
}

interface StoryViewerModalProps {
    storyGroups: StoryGroup[];
    initialGroupIndex: number;
    onClose: () => void;
}

export default function StoryViewerModal({
                                             storyGroups,
                                             initialGroupIndex,
                                             onClose,
                                         }: StoryViewerModalProps) {
    const {user} = useAuth();
    const {showToast} = useToast();

    const [groupIndex, setGroupIndex] = useState(initialGroupIndex);
    const [storyIndex, setStoryIndex] = useState(0);
    const [progress, setProgress] = useState(0);
    const [isPaused, setIsPaused] = useState(false);
    const [showViewersModal, setShowViewersModal] = useState(false);
    const [viewersList, setViewersList] = useState<User[]>([]);
    const [loadingViewers, setLoadingViewers] = useState(false);

    const currentGroup = storyGroups[groupIndex];
    const currentStory = currentGroup ? currentGroup.stories[storyIndex] : null;

    const DURATION_MS = 5000;
    const INTERVAL_MS = 50;

    // Mark current story as viewed
    const markAsViewed = useCallback(
        async (storyId: string) => {
            if (!user) return;
            try {
                await fetchGraphQL(MUTATIONS.VIEW_STORY, {storyId});
            } catch (err) {
                console.error("Failed to mark story as viewed:", err);
            }
        },
        [user]
    );

    useEffect(() => {
        if (currentStory) {
            markAsViewed(currentStory.id);
        }
    }, [currentStory, markAsViewed]);

    // Handle next story navigation
    const handleNext = useCallback(() => {
        setProgress(0);
        if (!currentGroup) return;

        if (storyIndex < currentGroup.stories.length - 1) {
            setStoryIndex((prev) => prev + 1);
        } else if (groupIndex < storyGroups.length - 1) {
            setGroupIndex((prev) => prev + 1);
            setStoryIndex(0);
        } else {
            onClose();
        }
    }, [storyIndex, currentGroup, groupIndex, storyGroups.length, onClose]);

    // Handle previous story navigation
    const handlePrev = useCallback(() => {
        setProgress(0);
        if (storyIndex > 0) {
            setStoryIndex((prev) => prev - 1);
        } else if (groupIndex > 0) {
            setGroupIndex((prev) => prev - 1);
            setStoryIndex(storyGroups[groupIndex - 1].stories.length - 1);
        }
    }, [storyIndex, groupIndex, storyGroups]);

    // Story Progress Timer
    useEffect(() => {
        if (isPaused || showViewersModal) return;

        const timer = setInterval(() => {
            setProgress((prev) => {
                if (prev >= 100) {
                    handleNext();
                    return 0;
                }
                return prev + (INTERVAL_MS / DURATION_MS) * 100;
            });
        }, INTERVAL_MS);

        return () => clearInterval(timer);
    }, [isPaused, showViewersModal, handleNext]);

    // Keyboard navigation
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === "Escape") onClose();
            if (e.key === "ArrowRight") handleNext();
            if (e.key === "ArrowLeft") handlePrev();
            if (e.key === " ") setIsPaused((prev) => !prev);
        };

        window.addEventListener("keydown", handleKeyDown);
        return () => window.removeEventListener("keydown", handleKeyDown);
    }, [onClose, handleNext, handlePrev]);

    // Delete story
    const handleDeleteStory = async () => {
        if (!currentStory) return;
        if (!confirm("이 스토리를 삭제하시겠습니까?")) return;

        try {
            await fetchGraphQL(MUTATIONS.DELETE_STORY, {storyId: currentStory.id});
            showToast("스토리가 삭제되었습니다.", "success");
            onClose();
        } catch (err: any) {
            showToast(err.message || "삭제 실패", "error");
        }
    };

    // View viewers list
    const handleFetchViewers = async () => {
        if (!currentStory) return;
        setIsPaused(true);
        setLoadingViewers(true);
        setShowViewersModal(true);

        try {
            const data = await fetchGraphQL<{ story: { viewers: User[] } }>(
                QUERIES.STORY_VIEWERS,
                {id: currentStory.id}
            );
            if (data?.story?.viewers) {
                setViewersList(data.story.viewers);
            }
        } catch (err: any) {
            showToast(err.message || "조회자 목록을 가져오지 못했습니다.", "error");
        } finally {
            setLoadingViewers(false);
        }
    };

    if (!currentGroup || !currentStory) return null;

    const isOwner = user && user.username === currentStory.author.username;
    const defaultAvatar = "https://api.dicebear.com/7.x/bottts/svg?seed=user";

    const timeAgo = (dateStr: string) => {
        const diff = Date.now() - new Date(dateStr).getTime();
        const hours = Math.floor(diff / (1000 * 60 * 60));
        if (hours < 1) {
            const mins = Math.floor(diff / (1000 * 60));
            return `${mins}분 전`;
        }
        return `${hours}시간 전`;
    };

    const getRemainingHours = (expiresAtStr: string) => {
        const diff = new Date(expiresAtStr).getTime() - Date.now();
        const hours = Math.max(0, Math.floor(diff / (1000 * 60 * 60)));
        return `${hours}시간 후 만료`;
    };

    return (
        <div className="story-modal-overlay" onClick={onClose}>
            {/* Prev Group Button */}
            {(groupIndex > 0 || storyIndex > 0) && (
                <button
                    className="story-nav-btn story-nav-prev"
                    onClick={(e) => {
                        e.stopPropagation();
                        handlePrev();
                    }}
                    title="이전 스토리"
                >
                    <ChevronLeft size={28}/>
                </button>
            )}

            {/* Main Story Viewer Card */}
            <div
                className="story-viewer-card"
                onClick={(e) => e.stopPropagation()}
                onMouseDown={() => setIsPaused(true)}
                onMouseUp={() => setIsPaused(false)}
                onTouchStart={() => setIsPaused(true)}
                onTouchEnd={() => setIsPaused(false)}
            >
                {/* Progress Bars */}
                <div className="story-progress-bar-container">
                    {currentGroup.stories.map((s, idx) => {
                        let fillWidth = 0;
                        if (idx < storyIndex) fillWidth = 100;
                        else if (idx === storyIndex) fillWidth = progress;

                        return (
                            <div key={s.id} className="story-progress-bar-bg">
                                <div
                                    className="story-progress-bar-fill"
                                    style={{width: `${fillWidth}%`}}
                                />
                            </div>
                        );
                    })}
                </div>

                {/* Header */}
                <div className="story-viewer-header">
                    <div className="story-viewer-author">
                        <img
                            src={currentStory.author.avatarUrl || defaultAvatar}
                            alt={currentStory.author.username}
                            style={{
                                width: 38,
                                height: 38,
                                borderRadius: "50%",
                                border: "2px solid #fff",
                                objectFit: "cover",
                            }}
                        />
                        <div>
                            <div style={{fontWeight: 700, fontSize: "14px", color: "#fff"}}>
                                {currentStory.author.displayName || currentStory.author.username}
                            </div>
                            <div
                                style={{
                                    fontSize: "11px",
                                    color: "rgba(255, 255, 255, 0.75)",
                                    display: "flex",
                                    alignItems: "center",
                                    gap: "4px",
                                }}
                            >
                                <span>{timeAgo(currentStory.createdAt)}</span>
                                <span>•</span>
                                <span style={{color: "#fbbf24"}}>
                  {getRemainingHours(currentStory.expiresAt)}
                </span>
                            </div>
                        </div>
                    </div>

                    <div style={{display: "flex", alignItems: "center", gap: "8px"}}>
                        {isOwner && (
                            <button
                                onClick={handleDeleteStory}
                                style={{
                                    color: "#f43f5e",
                                    padding: "6px",
                                    borderRadius: "50%",
                                    background: "rgba(0,0,0,0.4)",
                                }}
                                title="스토리 삭제"
                            >
                                <Trash2 size={18}/>
                            </button>
                        )}
                        <button
                            onClick={onClose}
                            style={{
                                color: "#fff",
                                padding: "6px",
                                borderRadius: "50%",
                                background: "rgba(0,0,0,0.4)",
                            }}
                            title="닫기"
                        >
                            <X size={20}/>
                        </button>
                    </div>
                </div>

                {/* Story Media (Image) */}
                <img
                    src={currentStory.mediaUrl}
                    alt={currentStory.caption || "Story media"}
                    className="story-viewer-media"
                />

                {/* Left & Right Tap Overlays for instant slide navigation */}
                <div
                    style={{
                        position: "absolute",
                        top: 60,
                        left: 0,
                        width: "35%",
                        bottom: 80,
                        cursor: "pointer",
                    }}
                    onClick={handlePrev}
                />
                <div
                    style={{
                        position: "absolute",
                        top: 60,
                        right: 0,
                        width: "65%",
                        bottom: 80,
                        cursor: "pointer",
                    }}
                    onClick={handleNext}
                />

                {/* Story Bottom Caption & Viewers */}
                <div className="story-viewer-caption-box">
                    {currentStory.caption && (
                        <p className="story-viewer-caption">{currentStory.caption}</p>
                    )}

                    <div className="story-viewer-stats">
                        <div
                            style={{
                                display: "flex",
                                alignItems: "center",
                                gap: "6px",
                                cursor: isOwner ? "pointer" : "default",
                                padding: "4px 8px",
                                borderRadius: "6px",
                                backgroundColor: isOwner ? "rgba(255,255,255,0.15)" : "transparent",
                            }}
                            onClick={isOwner ? handleFetchViewers : undefined}
                        >
                            <Eye size={15}/>
                            <span>조회 {currentStory.viewsCount}명</span>
                            {isOwner && (
                                <span style={{fontSize: "11px", color: "var(--accent-primary)"}}>
                  (조회자 보기)
                </span>
                            )}
                        </div>

                        <div style={{display: "flex", alignItems: "center", gap: "4px"}}>
                            <Clock size={14}/>
                            <span>24시간 자동 만료</span>
                        </div>
                    </div>
                </div>

                {/* Story Viewers Popup (Owner only) */}
                {showViewersModal && (
                    <div
                        style={{
                            position: "absolute",
                            inset: 0,
                            backgroundColor: "rgba(15, 20, 31, 0.95)",
                            backdropFilter: "blur(10px)",
                            zIndex: 30,
                            padding: "20px",
                            display: "flex",
                            flexDirection: "column",
                        }}
                    >
                        <div
                            style={{
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "space-between",
                                marginBottom: "16px",
                                borderBottom: "1px solid var(--border-subtle)",
                                paddingBottom: "10px",
                            }}
                        >
                            <div style={{display: "flex", alignItems: "center", gap: "8px", fontWeight: 700}}>
                                <Users size={18} color="var(--accent-primary)"/>
                                <span>스토리 시청자 ({viewersList.length}명)</span>
                            </div>
                            <button
                                onClick={() => {
                                    setShowViewersModal(false);
                                    setIsPaused(false);
                                }}
                                style={{color: "#fff", padding: "4px"}}
                            >
                                <X size={18}/>
                            </button>
                        </div>

                        <div
                            style={{flex: 1, overflowY: "auto", display: "flex", flexDirection: "column", gap: "10px"}}>
                            {loadingViewers && (
                                <div style={{textAlign: "center", padding: "20px", color: "var(--text-muted)"}}>
                                    조회자 목록을 불러오는 중...
                                </div>
                            )}

                            {!loadingViewers && viewersList.length === 0 && (
                                <div style={{textAlign: "center", padding: "20px", color: "var(--text-muted)"}}>
                                    아직 시청한 사용자가 없습니다.
                                </div>
                            )}

                            {viewersList.map((viewer) => (
                                <div
                                    key={viewer.id}
                                    style={{
                                        display: "flex",
                                        alignItems: "center",
                                        gap: "10px",
                                        padding: "8px",
                                        borderRadius: "var(--radius-md)",
                                        backgroundColor: "var(--bg-surface)",
                                    }}
                                >
                                    <img
                                        src={viewer.avatarUrl || defaultAvatar}
                                        alt={viewer.username}
                                        style={{width: 36, height: 36, borderRadius: "50%", objectFit: "cover"}}
                                    />
                                    <div>
                                        <div style={{fontWeight: 600, fontSize: "13px"}}>
                                            {viewer.displayName || viewer.username}
                                        </div>
                                        <div style={{fontSize: "11px", color: "var(--text-muted)"}}>
                                            @{viewer.username}
                                        </div>
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>
                )}
            </div>

            {/* Next Group Button */}
            {(groupIndex < storyGroups.length - 1 ||
                storyIndex < currentGroup.stories.length - 1) && (
                <button
                    className="story-nav-btn story-nav-next"
                    onClick={(e) => {
                        e.stopPropagation();
                        handleNext();
                    }}
                    title="다음 스토리"
                >
                    <ChevronRight size={28}/>
                </button>
            )}
        </div>
    );
}
