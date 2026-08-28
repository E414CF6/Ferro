"use client";

import React, {useRef, useState} from "react";
import {fetchGraphQL, MUTATIONS} from "@/lib/graphql";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import {Link as LinkIcon, Send, Upload, X} from "lucide-react";

interface StoryCreateModalProps {
    onClose: () => void;
}

export default function StoryCreateModal({onClose}: StoryCreateModalProps) {
    const {showToast} = useToast();
    const [uploadMode, setUploadMode] = useState<"file" | "url">("file");
    const [mediaUrl, setMediaUrl] = useState("");
    const [caption, setCaption] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const [isDragging, setIsDragging] = useState(false);
    const fileInputRef = useRef<HTMLInputElement>(null);

    const handleFileSelect = (file: File) => {
        if (!file.type.startsWith("image/")) {
            showToast("이미지 파일만 업로드할 수 있습니다.", "error");
            return;
        }

        // Limit to 5MB
        if (file.size > 5 * 1024 * 1024) {
            showToast("이미지 파일 크기는 최대 5MB까지 가능합니다.", "error");
            return;
        }

        const reader = new FileReader();
        reader.onload = (e) => {
            if (e.target?.result) {
                setMediaUrl(e.target.result as string);
            }
        };
        reader.readAsDataURL(file);
    };

    const handleDrop = (e: React.DragEvent) => {
        e.preventDefault();
        setIsDragging(false);
        if (e.dataTransfer.files && e.dataTransfer.files[0]) {
            handleFileSelect(e.dataTransfer.files[0]);
        }
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!mediaUrl.trim()) {
            showToast("스토리에 등록할 이미지를 선택하거나 URL을 입력해주세요.", "error");
            return;
        }

        setSubmitting(true);
        try {
            await fetchGraphQL(MUTATIONS.CREATE_STORY, {
                mediaUrl: mediaUrl.trim(),
                caption: caption.trim() ? caption.trim() : null,
            });

            showToast("24시간 스토리가 성공적으로 공유되었습니다!", "success");
            onClose();
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <div className="modal-overlay" onClick={onClose}>
            <div className="modal-card" onClick={(e) => e.stopPropagation()} style={{maxWidth: "500px"}}>
                <div className="modal-header">
                    <div style={{display: "flex", alignItems: "center", gap: "10px"}}>
                        <div className="logo-badge" style={{
                            width: "36px",
                            height: "36px",
                            fontSize: "18px",
                            background: "var(--story-ring-unread)"
                        }}>
                            📸
                        </div>
                        <div>
                            <h2 className="modal-title" style={{fontSize: "18px"}}>새 24시간 스토리</h2>
                            <p style={{fontSize: "12px", color: "var(--text-muted)"}}>
                                게시 후 24시간 동안 피드 상단에 활성화됩니다.
                            </p>
                        </div>
                    </div>
                    <button onClick={onClose} className="modal-close-btn" aria-label="닫기">
                        <X size={20}/>
                    </button>
                </div>

                <form onSubmit={handleSubmit} className="modal-body"
                      style={{display: "flex", flexDirection: "column", gap: "16px"}}>
                    {/* Upload Mode Selector */}
                    <div style={{
                        display: "flex",
                        gap: "8px",
                        padding: "4px",
                        backgroundColor: "var(--bg-input)",
                        borderRadius: "var(--radius-md)"
                    }}>
                        <button
                            type="button"
                            onClick={() => setUploadMode("file")}
                            style={{
                                flex: 1,
                                padding: "8px",
                                borderRadius: "var(--radius-sm)",
                                fontSize: "13px",
                                fontWeight: 700,
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                                gap: "6px",
                                backgroundColor: uploadMode === "file" ? "var(--bg-surface)" : "transparent",
                                color: uploadMode === "file" ? "var(--accent-primary)" : "var(--text-secondary)",
                            }}
                        >
                            <Upload size={14}/> 사진 파일 업로드
                        </button>
                        <button
                            type="button"
                            onClick={() => setUploadMode("url")}
                            style={{
                                flex: 1,
                                padding: "8px",
                                borderRadius: "var(--radius-sm)",
                                fontSize: "13px",
                                fontWeight: 700,
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                                gap: "6px",
                                backgroundColor: uploadMode === "url" ? "var(--bg-surface)" : "transparent",
                                color: uploadMode === "url" ? "var(--accent-primary)" : "var(--text-secondary)",
                            }}
                        >
                            <LinkIcon size={14}/> 이미지 URL
                        </button>
                    </div>

                    {/* Drag and Drop File Upload Area */}
                    {uploadMode === "file" ? (
                        <div
                            onDragOver={(e) => {
                                e.preventDefault();
                                setIsDragging(true);
                            }}
                            onDragLeave={() => setIsDragging(false)}
                            onDrop={handleDrop}
                            onClick={() => fileInputRef.current?.click()}
                            style={{
                                border: `2px dashed ${isDragging ? "var(--accent-primary)" : "var(--border-subtle)"}`,
                                borderRadius: "var(--radius-lg)",
                                padding: "28px 20px",
                                textAlign: "center",
                                cursor: "pointer",
                                backgroundColor: isDragging ? "rgba(56, 189, 248, 0.08)" : "var(--bg-input)",
                                transition: "all var(--transition-fast)",
                            }}
                        >
                            <input
                                ref={fileInputRef}
                                type="file"
                                accept="image/*"
                                style={{display: "none"}}
                                onChange={(e) => {
                                    if (e.target.files && e.target.files[0]) {
                                        handleFileSelect(e.target.files[0]);
                                    }
                                }}
                            />
                            <div style={{
                                width: 48,
                                height: 48,
                                borderRadius: "50%",
                                backgroundColor: "var(--bg-surface)",
                                margin: "0 auto 12px",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                                color: "var(--accent-primary)"
                            }}>
                                <Upload size={22}/>
                            </div>
                            <div style={{fontWeight: 700, fontSize: "14px", color: "var(--text-primary)"}}>
                                사진을 드래그하거나 클릭하여 업로드
                            </div>
                            <div style={{fontSize: "12px", color: "var(--text-muted)", marginTop: "4px"}}>
                                PNG, JPG, WEBP, GIF (최대 5MB)
                            </div>
                        </div>
                    ) : (
                        /* Direct Image URL Input */
                        <div className="form-group" style={{marginBottom: 0}}>
                            <label className="form-label">이미지 주소 (Web URL)</label>
                            <input
                                type="url"
                                className="form-input"
                                value={mediaUrl}
                                onChange={(e) => setMediaUrl(e.target.value)}
                                placeholder="https://example.com/photo.jpg"
                                autoFocus
                            />
                        </div>
                    )}

                    {/* Media Preview Box */}
                    {mediaUrl && (
                        <div
                            style={{
                                position: "relative",
                                height: "200px",
                                borderRadius: "var(--radius-lg)",
                                overflow: "hidden",
                                border: "1px solid var(--border-subtle)",
                                backgroundColor: "#000",
                            }}
                        >
                            <img
                                src={mediaUrl}
                                alt="Story Preview"
                                style={{width: "100%", height: "100%", objectFit: "contain"}}
                            />
                            <button
                                type="button"
                                onClick={() => setMediaUrl("")}
                                style={{
                                    position: "absolute",
                                    top: 10,
                                    right: 10,
                                    backgroundColor: "rgba(0,0,0,0.65)",
                                    color: "#fff",
                                    padding: "6px",
                                    borderRadius: "50%",
                                }}
                                title="이미지 제거"
                            >
                                <X size={16}/>
                            </button>
                            {caption && (
                                <div
                                    style={{
                                        position: "absolute",
                                        bottom: 0,
                                        left: 0,
                                        right: 0,
                                        padding: "10px 14px",
                                        background: "linear-gradient(to top, rgba(0,0,0,0.85) 0%, transparent 100%)",
                                        color: "#fff",
                                        fontSize: "13px",
                                        fontWeight: 600,
                                    }}
                                >
                                    {caption}
                                </div>
                            )}
                        </div>
                    )}

                    {/* Caption Input */}
                    <div className="form-group" style={{marginBottom: 0}}>
                        <div style={{
                            display: "flex",
                            justifyContent: "space-between",
                            alignItems: "center",
                            marginBottom: "6px"
                        }}>
                            <label className="form-label">스토리 캡션 (선택)</label>
                            <span style={{fontSize: "11px", color: "var(--text-muted)"}}>{caption.length}/100</span>
                        </div>
                        <input
                            type="text"
                            className="form-input"
                            value={caption}
                            onChange={(e) => setCaption(e.target.value)}
                            placeholder="스토리에 담길 한 줄 메시지 (최대 100자)..."
                            maxLength={100}
                        />
                    </div>

                    <div style={{display: "flex", justifyContent: "flex-end", gap: "10px", marginTop: "4px"}}>
                        <button type="button" onClick={onClose} className="btn-secondary">
                            취소
                        </button>
                        <button type="submit" disabled={submitting || !mediaUrl.trim()} className="btn-primary">
                            <Send size={14}/>
                            {submitting ? "업로드 중..." : "스토리 공유하기"}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
}
