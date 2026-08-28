"use client";

import React, {useRef, useState} from "react";
import {Story} from "@/lib/types";
import {useAuth} from "@/lib/auth-context";
import {ChevronLeft, ChevronRight, Plus, Sparkles} from "lucide-react";
import StoryViewerModal from "./StoryViewerModal";
import StoryCreateModal from "./StoryCreateModal";

interface StoryBarProps {
    stories: Story[];
    onRefresh?: () => void;
}

export default function StoryBar({stories, onRefresh}: StoryBarProps) {
    const {user} = useAuth();
    const [selectedStoryIndex, setSelectedStoryIndex] = useState<number | null>(null);
    const [showCreateModal, setShowCreateModal] = useState(false);
    const scrollContainerRef = useRef<HTMLDivElement>(null);

    // Group stories by author username
    const authorStoryMap = new Map<string, Story[]>();
    stories.forEach((story) => {
        const key = story.author.username;
        if (!authorStoryMap.has(key)) {
            authorStoryMap.set(key, []);
        }
        authorStoryMap.get(key)!.push(story);
    });

    const storyGroups = Array.from(authorStoryMap.entries()).map(([username, items]) => {
        const author = items[0].author;
        const hasUnviewed = items.some((s) => !s.isViewedByMe);
        return {
            username,
            author,
            stories: items,
            hasUnviewed,
        };
    });

    const defaultAvatar = "https://api.dicebear.com/7.x/bottts/svg?seed=user";

    const handleOpenViewer = (index: number) => {
        setSelectedStoryIndex(index);
    };

    const scroll = (direction: "left" | "right") => {
        if (scrollContainerRef.current) {
            const offset = direction === "left" ? -240 : 240;
            scrollContainerRef.current.scrollBy({left: offset, behavior: "smooth"});
        }
    };

    return (
        <>
            <div style={{
                position: "relative",
                borderBottom: "1px solid var(--border-subtle)",
                background: "rgba(16, 23, 38, 0.25)"
            }}>
                {/* Left scroll chevron */}
                {storyGroups.length > 4 && (
                    <button
                        type="button"
                        onClick={() => scroll("left")}
                        style={{
                            position: "absolute",
                            left: 8,
                            top: "50%",
                            transform: "translateY(-50%)",
                            zIndex: 10,
                            backgroundColor: "rgba(11, 15, 25, 0.85)",
                            backdropFilter: "blur(8px)",
                            border: "1px solid var(--border-subtle)",
                            borderRadius: "50%",
                            width: 32,
                            height: 32,
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "center",
                            color: "var(--text-primary)",
                            boxShadow: "var(--shadow-sm)",
                        }}
                        aria-label="이전 스토리"
                    >
                        <ChevronLeft size={18}/>
                    </button>
                )}

                <div ref={scrollContainerRef} className="stories-container">
                    {/* Add My Story circle if logged in */}
                    {user && (
                        <div className="story-item" onClick={() => setShowCreateModal(true)}>
                            <div className="story-avatar-wrapper add-story">
                                <div className="story-avatar-inner">
                                    <img
                                        src={user.avatarUrl || defaultAvatar}
                                        alt="내 스토리"
                                        className="story-avatar-img"
                                    />
                                </div>
                                <div className="story-add-badge">
                                    <Plus size={12} strokeWidth={3}/>
                                </div>
                            </div>
                            <span className="story-username" style={{color: "var(--accent-primary)", fontWeight: 700}}>
                내 스토리
              </span>
                        </div>
                    )}

                    {/* List of Authors with Active Stories */}
                    {storyGroups.map((group, idx) => (
                        <div
                            key={group.username}
                            className="story-item"
                            onClick={() => handleOpenViewer(idx)}
                        >
                            <div
                                className={`story-avatar-wrapper ${group.hasUnviewed ? "unread" : "read"}`}
                            >
                                <div className="story-avatar-inner">
                                    <img
                                        src={group.author.avatarUrl || defaultAvatar}
                                        alt={group.username}
                                        className="story-avatar-img"
                                    />
                                </div>
                            </div>
                            <span className="story-username">
                {group.author.displayName || group.username}
              </span>
                        </div>
                    ))}

                    {storyGroups.length === 0 && !user && (
                        <div style={{
                            display: "flex",
                            alignItems: "center",
                            gap: "8px",
                            color: "var(--text-muted)",
                            fontSize: "13px",
                            padding: "10px 4px"
                        }}>
                            <Sparkles size={16} color="var(--accent-primary)"/>
                            <span>로그인하여 24시간 스토리를 공유하고 친구들의 일상을 확인하세요.</span>
                        </div>
                    )}
                </div>

                {/* Right scroll chevron */}
                {storyGroups.length > 4 && (
                    <button
                        type="button"
                        onClick={() => scroll("right")}
                        style={{
                            position: "absolute",
                            right: 8,
                            top: "50%",
                            transform: "translateY(-50%)",
                            zIndex: 10,
                            backgroundColor: "rgba(11, 15, 25, 0.85)",
                            backdropFilter: "blur(8px)",
                            border: "1px solid var(--border-subtle)",
                            borderRadius: "50%",
                            width: 32,
                            height: 32,
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "center",
                            color: "var(--text-primary)",
                            boxShadow: "var(--shadow-sm)",
                        }}
                        aria-label="다음 스토리"
                    >
                        <ChevronRight size={18}/>
                    </button>
                )}
            </div>

            {/* Story Viewer Modal */}
            {selectedStoryIndex !== null && storyGroups[selectedStoryIndex] && (
                <StoryViewerModal
                    storyGroups={storyGroups}
                    initialGroupIndex={selectedStoryIndex}
                    onClose={() => {
                        setSelectedStoryIndex(null);
                        onRefresh?.();
                    }}
                />
            )}

            {/* Story Create Modal */}
            {showCreateModal && (
                <StoryCreateModal
                    onClose={() => {
                        setShowCreateModal(false);
                        onRefresh?.();
                    }}
                />
            )}
        </>
    );
}
