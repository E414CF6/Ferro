"use client";

import React, { useRef, useState } from "react";
import { useAuth } from "@/lib/auth-context";
import { fetchGraphQL, MUTATIONS } from "@/lib/graphql";
import { useToast } from "@/lib/toast-context";
import { formatErrorMessage } from "@/lib/i18n";
import { uploadMedia } from "@/lib/upload";
import {
    BarChart2,
    Globe,
    Image as ImageIcon,
    Lock,
    Plus,
    Send,
    Trash2,
    Users,
    X,
} from "lucide-react";

interface PostComposerProps {
    onPostCreated?: () => void;
}

export default function PostComposer({ onPostCreated }: PostComposerProps) {
    const { user } = useAuth();
    const { showToast } = useToast();

    const [content, setContent] = useState("");
    const [loading, setLoading] = useState(false);
    const [uploadingImage, setUploadingImage] = useState(false);
    const [attachedImages, setAttachedImages] = useState<string[]>([]);
    const [audience, setAudience] = useState<"PUBLIC" | "FOLLOWERS_ONLY" | "CLOSE_FRIENDS">("PUBLIC");

    // Poll creator state
    const [showPollCreator, setShowPollCreator] = useState(false);
    const [pollQuestion, setPollQuestion] = useState("");
    const [pollOptions, setPollOptions] = useState<string[]>(["", ""]);
    const [pollDurationSeconds, setPollDurationSeconds] = useState<number>(86400); // 24h default

    const fileInputRef = useRef<HTMLInputElement>(null);

    if (!user) return null;

    const defaultAvatar =
        "https://api.dicebear.com/7.x/bottts/svg?seed=" + user.username;

    // Handle real file upload to Serve /api/upload
    const handleFileSelect = async (files: FileList) => {
        const fileList = Array.from(files);

        if (attachedImages.length + fileList.length > 4) {
            showToast("이미지는 최대 4장까지 첨부할 수 있습니다.", "info");
            return;
        }

        const validFiles = fileList.filter((file) => {
            const isImage =
                file.type.startsWith("image/") ||
                /\.(heic|heif|avif|jpe?g|png|gif|webp)$/i.test(file.name);
            if (!isImage) {
                showToast("이미지 파일만 첨부할 수 있습니다.", "error");
                return false;
            }
            if (file.size > 50 * 1024 * 1024) {
                showToast("각 이미지는 최대 50MB까지 업로드 가능합니다.", "error");
                return false;
            }
            return true;
        });

        if (validFiles.length === 0) return;

        setUploadingImage(true);
        for (const file of validFiles) {
            try {
                const uploadedUrl = await uploadMedia(file);
                setAttachedImages((prev) => [...prev, uploadedUrl]);
            } catch (err: any) {
                showToast(err.message || "이미지 업로드에 실패했습니다.", "error");
            }
        }
        setUploadingImage(false);
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
                audience,
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

    return (
        <section className="post-composer">
            <img
                src={user.avatarUrl || defaultAvatar}
                alt={user.username}
                className="composer-avatar"
            />

            <div className="composer-body">
                <textarea
                    placeholder="무슨 일이 일어나고 있나요? (#해시태그, @멘션 입력 가능)"
                    value={content}
                    onChange={(e) => setContent(e.target.value)}
                    onKeyDown={handleKeyDown}
                    rows={3}
                    className="composer-textarea"
                />

                {/* Attached Images Preview Grid */}
                {attachedImages.length > 0 && (
                    <div
                        style={{
                            display: "grid",
                            gridTemplateColumns:
                                attachedImages.length === 1
                                    ? "1fr"
                                    : "repeat(2, 1fr)",
                            gap: "8px",
                            marginBottom: "12px",
                        }}
                    >
                        {attachedImages.map((img, idx) => (
                            <div
                                key={idx}
                                style={{
                                    position: "relative",
                                    borderRadius: "var(--radius-md)",
                                    overflow: "hidden",
                                    maxHeight: "220px",
                                }}
                            >
                                <img
                                    src={img}
                                    alt={`Upload preview ${idx + 1}`}
                                    style={{
                                        width: "100%",
                                        height: "100%",
                                        objectFit: "cover",
                                        display: "block",
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
                                        top: 8,
                                        right: 8,
                                        backgroundColor: "rgba(0,0,0,0.6)",
                                        color: "#fff",
                                        borderRadius: "50%",
                                        padding: "4px",
                                    }}
                                >
                                    <X size={14} />
                                </button>
                            </div>
                        ))}
                    </div>
                )}

                {/* Poll Creator Box */}
                {showPollCreator && (
                    <div
                        style={{
                            padding: "14px",
                            borderRadius: "var(--radius-md)",
                            backgroundColor: "var(--bg-input)",
                            border: "1px solid var(--border-subtle)",
                            marginBottom: "14px",
                            display: "flex",
                            flexDirection: "column",
                            gap: "10px",
                        }}
                    >
                        <div
                            style={{
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "space-between",
                            }}
                        >
                            <span
                                style={{
                                    fontSize: "13px",
                                    fontWeight: 700,
                                    color: "var(--text-primary)",
                                    display: "flex",
                                    alignItems: "center",
                                    gap: "6px",
                                }}
                            >
                                <BarChart2 size={16} color="var(--accent-primary)" /> 투표 만들기
                            </span>
                            <button
                                type="button"
                                onClick={() => setShowPollCreator(false)}
                                style={{ color: "var(--text-muted)" }}
                            >
                                <X size={16} />
                            </button>
                        </div>

                        <input
                            type="text"
                            placeholder="투표 질문 입력..."
                            value={pollQuestion}
                            onChange={(e) => setPollQuestion(e.target.value)}
                            style={{
                                width: "100%",
                                padding: "8px 12px",
                                borderRadius: "var(--radius-md)",
                                backgroundColor: "var(--bg-surface)",
                                border: "1px solid var(--border-subtle)",
                                color: "var(--text-primary)",
                                fontSize: "13px",
                            }}
                        />

                        {pollOptions.map((opt, i) => (
                            <div key={i} style={{ display: "flex", gap: "8px", alignItems: "center" }}>
                                <input
                                    type="text"
                                    placeholder={`항목 ${i + 1}`}
                                    value={opt}
                                    onChange={(e) => handlePollOptionChange(i, e.target.value)}
                                    style={{
                                        flex: 1,
                                        padding: "7px 12px",
                                        borderRadius: "var(--radius-md)",
                                        backgroundColor: "var(--bg-surface)",
                                        border: "1px solid var(--border-subtle)",
                                        color: "var(--text-primary)",
                                        fontSize: "13px",
                                    }}
                                />
                                {pollOptions.length > 2 && (
                                    <button
                                        type="button"
                                        onClick={() => handleRemovePollOption(i)}
                                        style={{ color: "var(--text-muted)", padding: "4px" }}
                                    >
                                        <Trash2 size={14} />
                                    </button>
                                )}
                            </div>
                        ))}

                        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                            {pollOptions.length < 4 && (
                                <button
                                    type="button"
                                    onClick={handleAddPollOption}
                                    style={{
                                        fontSize: "12px",
                                        color: "var(--accent-primary)",
                                        display: "inline-flex",
                                        alignItems: "center",
                                        gap: "4px",
                                        fontWeight: 600,
                                    }}
                                >
                                    <Plus size={13} /> 항목 추가 (최대 4개)
                                </button>
                            )}

                            {/* Poll Duration */}
                            <select
                                value={pollDurationSeconds}
                                onChange={(e) => setPollDurationSeconds(Number(e.target.value))}
                                style={{
                                    padding: "4px 8px",
                                    borderRadius: "var(--radius-sm)",
                                    backgroundColor: "var(--bg-surface)",
                                    border: "1px solid var(--border-subtle)",
                                    color: "var(--text-secondary)",
                                    fontSize: "11px",
                                }}
                            >
                                <option value={3600}>1시간</option>
                                <option value={21600}>6시간</option>
                                <option value={86400}>24시간</option>
                                <option value={259200}>3일</option>
                                <option value={604800}>7일</option>
                            </select>
                        </div>
                    </div>
                )}

                {/* Footer Controls */}
                <div className="composer-footer">
                    <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                        {/* Hidden file input */}
                        <input
                            ref={fileInputRef}
                            type="file"
                            accept="image/*,.heic,.heif,.avif"
                            multiple
                            style={{ display: "none" }}
                            onChange={(e) => {
                                if (e.target.files) handleFileSelect(e.target.files);
                                e.target.value = "";
                            }}
                        />

                        {/* Image upload button */}
                        <button
                            type="button"
                            onClick={() => fileInputRef.current?.click()}
                            disabled={uploadingImage || attachedImages.length >= 4}
                            title="이미지 첨부"
                            style={{
                                color: "var(--accent-primary)",
                                padding: "6px",
                                borderRadius: "var(--radius-sm)",
                                display: "flex",
                                alignItems: "center",
                                opacity: uploadingImage ? 0.6 : 1,
                            }}
                        >
                            <ImageIcon size={19} />
                        </button>

                        {/* Poll creator toggle */}
                        <button
                            type="button"
                            onClick={() => setShowPollCreator(!showPollCreator)}
                            title="투표 생성"
                            style={{
                                color: showPollCreator ? "var(--accent-primary)" : "var(--text-muted)",
                                padding: "6px",
                                borderRadius: "var(--radius-sm)",
                                display: "flex",
                                alignItems: "center",
                            }}
                        >
                            <BarChart2 size={19} />
                        </button>

                        {/* Audience Selector */}
                        <div style={{ display: "flex", alignItems: "center", gap: "4px" }}>
                            <button
                                type="button"
                                onClick={() =>
                                    setAudience((prev) =>
                                        prev === "PUBLIC"
                                            ? "FOLLOWERS_ONLY"
                                            : prev === "FOLLOWERS_ONLY"
                                            ? "CLOSE_FRIENDS"
                                            : "PUBLIC"
                                    )
                                }
                                style={{
                                    padding: "4px 8px",
                                    borderRadius: "var(--radius-full)",
                                    backgroundColor: "var(--bg-input)",
                                    border: "1px solid var(--border-subtle)",
                                    fontSize: "11px",
                                    color: "var(--text-secondary)",
                                    display: "flex",
                                    alignItems: "center",
                                    gap: "4px",
                                    fontWeight: 600,
                                }}
                            >
                                {audience === "PUBLIC" && (
                                    <>
                                        <Globe size={12} color="var(--accent-primary)" /> 전체 공개
                                    </>
                                )}
                                {audience === "FOLLOWERS_ONLY" && (
                                    <>
                                        <Users size={12} color="var(--accent-purple)" /> 팔로워 공개
                                    </>
                                )}
                                {audience === "CLOSE_FRIENDS" && (
                                    <>
                                        <Lock size={12} color="#10b981" /> 친한 친구
                                    </>
                                )}
                            </button>
                        </div>
                    </div>

                    <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
                        <span style={{ fontSize: "12px", color: content.length > 4500 ? "#f43f5e" : "var(--text-muted)" }}>
                            {content.length > 0 && `${content.length}/5,000`}
                        </span>

                        <button
                            type="button"
                            onClick={() => handleSubmit()}
                            disabled={
                                loading ||
                                uploadingImage ||
                                (!content.trim() && attachedImages.length === 0 && !showPollCreator)
                            }
                            className="btn-primary"
                            style={{ padding: "8px 20px" }}
                        >
                            <Send size={15} />
                            <span>{loading ? "게시 중..." : "게시하기"}</span>
                        </button>
                    </div>
                </div>
            </div>
        </section>
    );
}
