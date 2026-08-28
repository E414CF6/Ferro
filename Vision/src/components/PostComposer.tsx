"use client";

import React, {useRef, useState} from "react";
import {useAuth} from "@/lib/auth-context";
import {fetchGraphQL, MUTATIONS} from "@/lib/graphql";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import {BarChart2, Globe, Image as ImageIcon, Lock, Plus, Send, Trash2, X,} from "lucide-react";

interface PostComposerProps {
    onPostCreated?: () => void;
}

export default function PostComposer({onPostCreated}: PostComposerProps) {
    const {user} = useAuth();
    const {showToast} = useToast();

    const [content, setContent] = useState("");
    const [loading, setLoading] = useState(false);
    const [attachedImages, setAttachedImages] = useState<string[]>([]);
    const [audience, setAudience] = useState<"PUBLIC" | "FOLLOWERS_ONLY">("PUBLIC");

    // Poll creator state
    const [showPollCreator, setShowPollCreator] = useState(false);
    const [pollQuestion, setPollQuestion] = useState("");
    const [pollOptions, setPollOptions] = useState<string[]>(["", ""]);
    const [pollDurationSeconds, setPollDurationSeconds] = useState<number>(86400); // 24h default

    const fileInputRef = useRef<HTMLInputElement>(null);

    if (!user) return null;

    const defaultAvatar =
        "https://api.dicebear.com/7.x/bottts/svg?seed=" + user.username;

    const handleFileSelect = (files: FileList) => {
        const validFiles = Array.from(files).filter((file) => {
            if (!file.type.startsWith("image/")) {
                showToast("이미지 파일만 첨부할 수 있습니다.", "error");
                return false;
            }
            if (file.size > 5 * 1024 * 1024) {
                showToast("각 이미지는 최대 5MB까지 업로드 가능합니다.", "error");
                return false;
            }
            return true;
        });

        if (attachedImages.length + validFiles.length > 4) {
            showToast("이미지는 최대 4장까지 첨부할 수 있습니다.", "error");
            return;
        }

        validFiles.forEach((file) => {
            const reader = new FileReader();
            reader.onload = (e) => {
                if (e.target?.result) {
                    setAttachedImages((prev) => [...prev, e.target!.result as string]);
                }
            };
            reader.readAsDataURL(file);
        });
    };

    const handleAddPollOption = () => {
        if (pollOptions.length >= 4) {
            showToast("투표 항목은 최대 4개까지 추가할 수 있습니다.", "info");
            return;
        }
        setPollOptions((prev) => [...prev, ""]);
    };

    const handleRemovePollOption = (index: number) => {
        if (pollOptions.length <= 2) {
            showToast("투표 항목은 최소 2개 이상이어야 합니다.", "info");
            return;
        }
        setPollOptions((prev) => prev.filter((_, i) => i !== index));
    };

    const handlePollOptionChange = (index: number, val: string) => {
        setPollOptions((prev) => {
            const updated = [...prev];
            updated[index] = val;
            return updated;
        });
    };

    const handleSubmit = async (e?: React.FormEvent) => {
        if (e) e.preventDefault();
        const trimmed = content.trim();

        if (!trimmed && attachedImages.length === 0 && !showPollCreator) return;

        // Validate poll if enabled
        let pollPayload = undefined;
        if (showPollCreator) {
            const q = pollQuestion.trim();
            const validOptions = pollOptions.map((o) => o.trim()).filter(Boolean);
            if (!q) {
                showToast("투표 질문을 입력해주세요.", "error");
                return;
            }
            if (validOptions.length < 2) {
                showToast("투표 항목을 2개 이상 입력해주세요.", "error");
                return;
            }
            pollPayload = {
                question: q,
                options: validOptions,
                durationSeconds: pollDurationSeconds,
            };
        }

        // Media payload
        const mediaPayload =
            attachedImages.length > 0
                ? attachedImages.map((url, idx) => ({
                    mediaUrl: url,
                    mediaType: "IMAGE",
                    sortOrder: idx,
                }))
                : undefined;

        setLoading(true);
        try {
            await fetchGraphQL(MUTATIONS.CREATE_POST, {
                content: trimmed || (pollPayload ? pollPayload.question : "새 게시물"),
                media: mediaPayload,
                poll: pollPayload,
                audience: audience === "FOLLOWERS_ONLY" ? "FOLLOWERS_ONLY" : "PUBLIC",
            });

            setContent("");
            setAttachedImages([]);
            setShowPollCreator(false);
            setPollQuestion("");
            setPollOptions(["", ""]);
            showToast("게시물이 성공적으로 등록되었습니다!", "success");
            onPostCreated?.();
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLoading(false);
        }
    };

    const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
        if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
            e.preventDefault();
            handleSubmit();
        }
    };

    const addTag = (tag: string) => {
        setContent((prev) => (prev ? `${prev} #${tag}` : `#${tag}`));
    };

    const isOverLimit = content.length > 500;

    return (
        <div className="composer-card">
            <img
                src={user.avatarUrl || defaultAvatar}
                alt={user.username}
                className="composer-avatar"
            />
            <div className="composer-main">
                <form onSubmit={handleSubmit}>
                    {/* Audience Selector Pill */}
                    <div style={{marginBottom: "8px", display: "flex", gap: "6px"}}>
                        <button
                            type="button"
                            onClick={() => setAudience("PUBLIC")}
                            style={{
                                display: "inline-flex",
                                alignItems: "center",
                                gap: "4px",
                                padding: "3px 9px",
                                borderRadius: "var(--radius-full)",
                                fontSize: "11px",
                                fontWeight: 600,
                                backgroundColor:
                                    audience === "PUBLIC"
                                        ? "rgba(56, 189, 248, 0.15)"
                                        : "var(--bg-surface)",
                                color:
                                    audience === "PUBLIC"
                                        ? "var(--accent-primary)"
                                        : "var(--text-muted)",
                                border: `1px solid ${
                                    audience === "PUBLIC"
                                        ? "var(--accent-primary)"
                                        : "var(--border-subtle)"
                                }`,
                            }}
                        >
                            <Globe size={12}/> 전체 공개
                        </button>
                        <button
                            type="button"
                            onClick={() => setAudience("FOLLOWERS_ONLY")}
                            style={{
                                display: "inline-flex",
                                alignItems: "center",
                                gap: "4px",
                                padding: "3px 9px",
                                borderRadius: "var(--radius-full)",
                                fontSize: "11px",
                                fontWeight: 600,
                                backgroundColor:
                                    audience === "FOLLOWERS_ONLY"
                                        ? "rgba(56, 189, 248, 0.15)"
                                        : "var(--bg-surface)",
                                color:
                                    audience === "FOLLOWERS_ONLY"
                                        ? "var(--accent-primary)"
                                        : "var(--text-muted)",
                                border: `1px solid ${
                                    audience === "FOLLOWERS_ONLY"
                                        ? "var(--accent-primary)"
                                        : "var(--border-subtle)"
                                }`,
                            }}
                        >
                            <Lock size={12}/> 팔로워 전용
                        </button>
                    </div>

                    <textarea
                        value={content}
                        onChange={(e) => setContent(e.target.value)}
                        onKeyDown={handleKeyDown}
                        placeholder="어떤 소식이 있나요? (Rust, GraphQL, 개발 이야기, 투표...)"
                        className="composer-textarea"
                        rows={3}
                    />

                    {/* Attached Images Grid Preview */}
                    {attachedImages.length > 0 && (
                        <div
                            style={{
                                display: "grid",
                                gridTemplateColumns:
                                    attachedImages.length === 1 ? "1fr" : "repeat(2, 1fr)",
                                gap: "6px",
                                borderRadius: "var(--radius-md)",
                                overflow: "hidden",
                                border: "1px solid var(--border-subtle)",
                                marginBottom: "10px",
                                backgroundColor: "#000",
                            }}
                        >
                            {attachedImages.map((img, idx) => (
                                <div
                                    key={idx}
                                    style={{
                                        position: "relative",
                                        height: attachedImages.length === 1 ? "180px" : "110px",
                                    }}
                                >
                                    <img
                                        src={img}
                                        alt="Upload preview"
                                        style={{
                                            width: "100%",
                                            height: "100%",
                                            objectFit: "cover",
                                        }}
                                    />
                                    <button
                                        type="button"
                                        onClick={() =>
                                            setAttachedImages((prev) =>
                                                prev.filter((_, i) => i !== idx)
                                            )
                                        }
                                        style={{
                                            position: "absolute",
                                            top: 6,
                                            right: 6,
                                            backgroundColor: "rgba(0,0,0,0.75)",
                                            color: "#fff",
                                            padding: "4px",
                                            borderRadius: "50%",
                                        }}
                                        title="이미지 제거"
                                    >
                                        <X size={12}/>
                                    </button>
                                </div>
                            ))}
                        </div>
                    )}

                    {/* Interactive Poll Creator Drawer */}
                    {showPollCreator && (
                        <div
                            style={{
                                padding: "14px",
                                borderRadius: "var(--radius-md)",
                                border: "1px solid var(--accent-primary)",
                                backgroundColor: "rgba(56, 189, 248, 0.04)",
                                marginBottom: "12px",
                            }}
                        >
                            <div
                                style={{
                                    display: "flex",
                                    alignItems: "center",
                                    justifyContent: "space-between",
                                    marginBottom: "10px",
                                }}
                            >
                <span
                    style={{
                        fontSize: "13px",
                        fontWeight: 700,
                        color: "var(--accent-primary)",
                        display: "flex",
                        alignItems: "center",
                        gap: "4px",
                    }}
                >
                  <BarChart2 size={15}/> 투표 만들기
                </span>
                                <button
                                    type="button"
                                    onClick={() => setShowPollCreator(false)}
                                    style={{color: "var(--text-muted)", padding: "2px"}}
                                >
                                    <X size={14}/>
                                </button>
                            </div>

                            <input
                                type="text"
                                placeholder="투표 질문을 입력하세요..."
                                value={pollQuestion}
                                onChange={(e) => setPollQuestion(e.target.value)}
                                className="composer-textarea"
                                style={{
                                    width: "100%",
                                    padding: "8px 12px",
                                    borderRadius: "var(--radius-md)",
                                    border: "1px solid var(--border-subtle)",
                                    fontSize: "13px",
                                    marginBottom: "8px",
                                }}
                            />

                            <div
                                style={{
                                    display: "flex",
                                    flexDirection: "column",
                                    gap: "6px",
                                    marginBottom: "10px",
                                }}
                            >
                                {pollOptions.map((opt, i) => (
                                    <div
                                        key={i}
                                        style={{
                                            display: "flex",
                                            alignItems: "center",
                                            gap: "6px",
                                        }}
                                    >
                                        <input
                                            type="text"
                                            placeholder={`항목 ${i + 1}`}
                                            value={opt}
                                            onChange={(e) =>
                                                handlePollOptionChange(i, e.target.value)
                                            }
                                            className="composer-textarea"
                                            style={{
                                                flex: 1,
                                                padding: "7px 10px",
                                                borderRadius: "var(--radius-md)",
                                                border: "1px solid var(--border-subtle)",
                                                fontSize: "12px",
                                            }}
                                        />
                                        {pollOptions.length > 2 && (
                                            <button
                                                type="button"
                                                onClick={() => handleRemovePollOption(i)}
                                                style={{color: "var(--accent-secondary)", padding: "4px"}}
                                            >
                                                <Trash2 size={13}/>
                                            </button>
                                        )}
                                    </div>
                                ))}
                            </div>

                            <div
                                style={{
                                    display: "flex",
                                    alignItems: "center",
                                    justifyContent: "space-between",
                                }}
                            >
                                {pollOptions.length < 4 ? (
                                    <button
                                        type="button"
                                        onClick={handleAddPollOption}
                                        className="btn-secondary"
                                        style={{padding: "4px 10px", fontSize: "11px"}}
                                    >
                                        <Plus size={12}/> 항목 추가
                                    </button>
                                ) : (
                                    <div/>
                                )}

                                <div style={{display: "flex", alignItems: "center", gap: "6px"}}>
                  <span style={{fontSize: "11px", color: "var(--text-muted)"}}>
                    투표 기간:
                  </span>
                                    <select
                                        value={pollDurationSeconds}
                                        onChange={(e) =>
                                            setPollDurationSeconds(Number(e.target.value))
                                        }
                                        style={{
                                            padding: "3px 8px",
                                            borderRadius: "var(--radius-sm)",
                                            backgroundColor: "var(--bg-surface)",
                                            border: "1px solid var(--border-subtle)",
                                            color: "var(--text-primary)",
                                            fontSize: "11px",
                                        }}
                                    >
                                        <option value={3600}>1시간</option>
                                        <option value={86400}>1일 (24시간)</option>
                                        <option value={259200}>3일</option>
                                        <option value={604800}>7일</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                    )}

                    {/* Quick Tags */}
                    <div
                        style={{
                            display: "flex",
                            gap: "6px",
                            flexWrap: "wrap",
                            marginBottom: "10px",
                        }}
                    >
                        {["Rust", "GraphQL", "PostgreSQL", "NextJS", "WebDev", "DevLife"].map(
                            (tag) => (
                                <button
                                    key={tag}
                                    type="button"
                                    onClick={() => addTag(tag)}
                                    style={{
                                        fontSize: "11px",
                                        fontWeight: 600,
                                        padding: "3px 9px",
                                        borderRadius: "var(--radius-full)",
                                        backgroundColor: "rgba(56, 189, 248, 0.08)",
                                        color: "var(--accent-primary)",
                                        border: "1px solid rgba(56, 189, 248, 0.2)",
                                        transition: "all var(--transition-fast)",
                                    }}
                                >
                                    #{tag}
                                </button>
                            )
                        )}
                    </div>

                    <div className="composer-actions">
                        <div className="composer-tools">
                            <input
                                ref={fileInputRef}
                                type="file"
                                accept="image/*"
                                multiple
                                style={{display: "none"}}
                                onChange={(e) => {
                                    if (e.target.files && e.target.files.length > 0) {
                                        handleFileSelect(e.target.files);
                                    }
                                }}
                            />
                            <button
                                type="button"
                                onClick={() => fileInputRef.current?.click()}
                                className="composer-tool-btn"
                                title="사진 첨부 (최대 4장)"
                            >
                                <ImageIcon size={18}/>
                            </button>

                            <button
                                type="button"
                                onClick={() => setShowPollCreator(!showPollCreator)}
                                className="composer-tool-btn"
                                style={{
                                    color: showPollCreator
                                        ? "var(--accent-primary)"
                                        : "var(--text-muted)",
                                }}
                                title="투표 생성"
                            >
                                <BarChart2 size={18}/>
                            </button>

                            <span
                                style={{
                                    fontSize: "11px",
                                    color: "var(--text-muted)",
                                    marginLeft: "4px",
                                }}
                            >
                (Ctrl + Enter로 바로 게시)
              </span>
                        </div>

                        <div style={{display: "flex", alignItems: "center", gap: "12px"}}>
              <span
                  style={{
                      fontSize: "12px",
                      fontWeight: 600,
                      color: isOverLimit
                          ? "var(--accent-secondary)"
                          : content.length > 400
                              ? "var(--accent-warning)"
                              : "var(--text-muted)",
                  }}
              >
                {content.length} / 500자
              </span>
                            <button
                                type="submit"
                                disabled={
                                    loading ||
                                    (!content.trim() &&
                                        attachedImages.length === 0 &&
                                        !showPollCreator) ||
                                    isOverLimit
                                }
                                className="btn-primary"
                                style={{padding: "8px 18px", fontSize: "13px"}}
                            >
                                <Send size={13}/>
                                {loading ? "게시 중..." : "게시하기"}
                            </button>
                        </div>
                    </div>
                </form>
            </div>
        </div>
    );
}
