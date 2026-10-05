"use client";

import React, { useRef, useState } from "react";
import { User } from "@/lib/types";
import { useAuth } from "@/lib/auth-context";
import { fetchGraphQL, MUTATIONS } from "@/lib/graphql";
import { useToast } from "@/lib/toast-context";
import { formatErrorMessage } from "@/lib/i18n";
import { uploadMedia } from "@/lib/upload";
import { Camera, Lock, Save, X } from "lucide-react";

interface EditProfileModalProps {
    user: User;
    onClose: () => void;
    onUpdated?: () => void;
}

export default function EditProfileModal({
    user,
    onClose,
    onUpdated,
}: EditProfileModalProps) {
    const { refreshUser } = useAuth();
    const { showToast } = useToast();

    const [displayName, setDisplayName] = useState(user.displayName || "");
    const [bio, setBio] = useState(user.bio || "");
    const [avatarUrl, setAvatarUrl] = useState(user.avatarUrl || "");
    const [headerImageUrl, setHeaderImageUrl] = useState(user.headerImageUrl || "");
    const [location, setLocation] = useState(user.location || "");
    const [website, setWebsite] = useState(user.website || "");
    const [isPrivate, setIsPrivate] = useState(user.isPrivate || false);
    const [loading, setLoading] = useState(false);
    const [uploadingAvatar, setUploadingAvatar] = useState(false);
    const [uploadingHeader, setUploadingHeader] = useState(false);

    const avatarFileRef = useRef<HTMLInputElement>(null);
    const headerFileRef = useRef<HTMLInputElement>(null);

    const handleFileSelect = async (file: File, target: "avatar" | "header") => {
        try {
            if (target === "avatar") setUploadingAvatar(true);
            else setUploadingHeader(true);

            const uploadedUrl = await uploadMedia(file);
            if (target === "avatar") {
                setAvatarUrl(uploadedUrl);
            } else {
                setHeaderImageUrl(uploadedUrl);
            }
            showToast("이미지가 업로드되었습니다.", "success");
        } catch (err: any) {
            showToast(err.message || "이미지 업로드에 실패했습니다.", "error");
        } finally {
            if (target === "avatar") setUploadingAvatar(false);
            else setUploadingHeader(false);
        }
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setLoading(true);

        try {
            await fetchGraphQL(MUTATIONS.UPDATE_PROFILE, {
                displayName: displayName.trim() || undefined,
                bio: bio.trim() || undefined,
                avatarUrl: avatarUrl.trim() || undefined,
                headerImageUrl: headerImageUrl.trim() || undefined,
                location: location.trim() || undefined,
                website: website.trim() || undefined,
            });

            if (isPrivate !== user.isPrivate) {
                await fetchGraphQL(MUTATIONS.UPDATE_USER_PRIVACY, {
                    isPrivate,
                });
            }

            showToast("프로필이 성공적으로 업데이트되었습니다!", "success");
            await refreshUser();
            onUpdated?.();
            onClose();
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLoading(false);
        }
    };

    const defaultAvatar = "https://api.dicebear.com/7.x/bottts/svg?seed=" + user.username;

    return (
        <div className="modal-overlay" onClick={onClose} style={{ zIndex: 110 }}>
            <div
                className="modal-content"
                onClick={(e) => e.stopPropagation()}
                style={{ maxWidth: "520px", maxHeight: "90vh", overflowY: "auto" }}
            >
                <div className="modal-header">
                    <h3 className="modal-title">프로필 정보 수정</h3>
                    <button onClick={onClose} className="modal-close-btn" aria-label="닫기">
                        <X size={18} />
                    </button>
                </div>

                <form onSubmit={handleSubmit} style={{ padding: "20px", display: "flex", flexDirection: "column", gap: "16px" }}>
                    {/* Header Image Picker */}
                    <div>
                        <label style={{ fontSize: "13px", fontWeight: 700, display: "block", marginBottom: "6px" }}>
                            헤더 배경 이미지
                        </label>
                        <div
                            style={{
                                width: "100%",
                                height: "120px",
                                backgroundColor: "var(--bg-input)",
                                borderRadius: "var(--radius-md)",
                                overflow: "hidden",
                                position: "relative",
                                border: "1px dashed var(--border-subtle)",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                                cursor: "pointer",
                            }}
                            onClick={() => headerFileRef.current?.click()}
                        >
                            {headerImageUrl ? (
                                <img
                                    src={headerImageUrl}
                                    alt="Header background"
                                    style={{ width: "100%", height: "100%", objectFit: "cover" }}
                                />
                            ) : (
                                <span style={{ fontSize: "12px", color: "var(--text-muted)", display: "flex", alignItems: "center", gap: "6px" }}>
                                    <Camera size={16} /> 배경 이미지 선택 (권장: 1500x500)
                                </span>
                            )}
                            <input
                                ref={headerFileRef}
                                type="file"
                                accept="image/*"
                                style={{ display: "none" }}
                                onChange={(e) => {
                                    if (e.target.files?.[0]) handleFileSelect(e.target.files[0], "header");
                                    e.target.value = "";
                                }}
                            />
                        </div>
                    </div>

                    {/* Avatar Picker */}
                    <div style={{ display: "flex", alignItems: "center", gap: "14px" }}>
                        <div
                            style={{
                                width: 64,
                                height: 64,
                                borderRadius: "50%",
                                overflow: "hidden",
                                position: "relative",
                                cursor: "pointer",
                                border: "2px solid var(--border-subtle)",
                            }}
                            onClick={() => avatarFileRef.current?.click()}
                        >
                            <img
                                src={avatarUrl || defaultAvatar}
                                alt="Avatar preview"
                                style={{ width: "100%", height: "100%", objectFit: "cover" }}
                            />
                            <div
                                style={{
                                    position: "absolute",
                                    inset: 0,
                                    backgroundColor: "rgba(0,0,0,0.4)",
                                    display: "flex",
                                    alignItems: "center",
                                    justifyContent: "center",
                                    color: "#fff",
                                }}
                            >
                                <Camera size={18} />
                            </div>
                        </div>
                        <div>
                            <div style={{ fontSize: "13px", fontWeight: 700, color: "var(--text-primary)" }}>
                                프로필 사진 변경
                            </div>
                            <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                                5MB 이하의 JPG, PNG, GIF, WEBP 지원
                            </div>
                        </div>
                        <input
                            ref={avatarFileRef}
                            type="file"
                            accept="image/*"
                            style={{ display: "none" }}
                            onChange={(e) => {
                                if (e.target.files?.[0]) handleFileSelect(e.target.files[0], "avatar");
                                e.target.value = "";
                            }}
                        />
                    </div>

                    {/* Display Name */}
                    <div>
                        <label style={{ fontSize: "13px", fontWeight: 700, display: "block", marginBottom: "4px" }}>
                            이름 (닉네임)
                        </label>
                        <input
                            type="text"
                            value={displayName}
                            onChange={(e) => setDisplayName(e.target.value)}
                            className="composer-textarea"
                            style={{ width: "100%", padding: "8px 12px", borderRadius: "var(--radius-md)", border: "1px solid var(--border-subtle)" }}
                        />
                    </div>

                    {/* Bio */}
                    <div>
                        <label style={{ fontSize: "13px", fontWeight: 700, display: "block", marginBottom: "4px" }}>
                            자기소개
                        </label>
                        <textarea
                            value={bio}
                            onChange={(e) => setBio(e.target.value)}
                            rows={3}
                            placeholder="자신을 소개하는 간단한 문장을 작성해보세요."
                            className="composer-textarea"
                            style={{ width: "100%", padding: "8px 12px", borderRadius: "var(--radius-md)", border: "1px solid var(--border-subtle)", resize: "none" }}
                        />
                    </div>

                    {/* Location & Website */}
                    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "12px" }}>
                        <div>
                            <label style={{ fontSize: "12px", fontWeight: 700, display: "block", marginBottom: "4px" }}>
                                위치
                            </label>
                            <input
                                type="text"
                                placeholder="예: 서울, 대한민국"
                                value={location}
                                onChange={(e) => setLocation(e.target.value)}
                                className="composer-textarea"
                                style={{ width: "100%", padding: "8px 12px", borderRadius: "var(--radius-md)", border: "1px solid var(--border-subtle)" }}
                            />
                        </div>
                        <div>
                            <label style={{ fontSize: "12px", fontWeight: 700, display: "block", marginBottom: "4px" }}>
                                웹사이트
                            </label>
                            <input
                                type="url"
                                placeholder="https://..."
                                value={website}
                                onChange={(e) => setWebsite(e.target.value)}
                                className="composer-textarea"
                                style={{ width: "100%", padding: "8px 12px", borderRadius: "var(--radius-md)", border: "1px solid var(--border-subtle)" }}
                            />
                        </div>
                    </div>

                    {/* Privacy Toggle */}
                    <div style={{ padding: "12px", borderRadius: "var(--radius-md)", backgroundColor: "var(--bg-input)", border: "1px solid var(--border-subtle)" }}>
                        <label style={{ display: "flex", alignItems: "center", justifyContent: "space-between", cursor: "pointer" }}>
                            <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                                <Lock size={16} color="var(--accent-primary)" />
                                <div>
                                    <div style={{ fontSize: "13px", fontWeight: 700, color: "var(--text-primary)" }}>
                                        비공개 계정 설정
                                    </div>
                                    <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                                        승인된 팔로워만 내 게시물과 활동을 볼 수 있습니다.
                                    </div>
                                </div>
                            </div>
                            <input
                                type="checkbox"
                                checked={isPrivate}
                                onChange={(e) => setIsPrivate(e.target.checked)}
                                style={{ width: "18px", height: "18px", accentColor: "var(--accent-primary)" }}
                            />
                        </label>
                    </div>

                    {/* Actions */}
                    <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px", marginTop: "10px" }}>
                        <button type="button" onClick={onClose} className="btn-secondary">
                            취소
                        </button>
                        <button
                            type="submit"
                            disabled={loading || uploadingAvatar || uploadingHeader}
                            className="btn-primary"
                        >
                            <Save size={14} /> {loading ? "저장 중..." : "변경사항 저장"}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
}
