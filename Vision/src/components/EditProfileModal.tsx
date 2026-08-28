"use client";

import React, {useRef, useState} from "react";
import {User} from "@/lib/types";
import {useAuth} from "@/lib/auth-context";
import {fetchGraphQL, MUTATIONS} from "@/lib/graphql";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import {Camera, Save, X} from "lucide-react";

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
    const {refreshUser} = useAuth();
    const {showToast} = useToast();

    const [displayName, setDisplayName] = useState(user.displayName || "");
    const [bio, setBio] = useState(user.bio || "");
    const [avatarUrl, setAvatarUrl] = useState(user.avatarUrl || "");
    const [headerImageUrl, setHeaderImageUrl] = useState(user.headerImageUrl || "");
    const [location, setLocation] = useState(user.location || "");
    const [website, setWebsite] = useState(user.website || "");
    const [loading, setLoading] = useState(false);

    const avatarFileRef = useRef<HTMLInputElement>(null);
    const headerFileRef = useRef<HTMLInputElement>(null);

    const handleFileSelect = (file: File, target: "avatar" | "header") => {
        if (!file.type.startsWith("image/")) {
            showToast("이미지 파일만 선택할 수 있습니다.", "error");
            return;
        }
        if (file.size > 5 * 1024 * 1024) {
            showToast("이미지 크기는 최대 5MB까지 가능합니다.", "error");
            return;
        }

        const reader = new FileReader();
        reader.onload = (e) => {
            if (e.target?.result) {
                if (target === "avatar") {
                    setAvatarUrl(e.target.result as string);
                } else {
                    setHeaderImageUrl(e.target.result as string);
                }
            }
        };
        reader.readAsDataURL(file);
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
        <div className="modal-overlay" onClick={onClose}>
            <div className="modal-card" onClick={(e) => e.stopPropagation()} style={{maxWidth: "520px"}}>
                <div className="modal-header">
                    <h2 className="modal-title">프로필 정보 수정</h2>
                    <button onClick={onClose} className="modal-close-btn" aria-label="닫기">
                        <X size={20}/>
                    </button>
                </div>

                <form onSubmit={handleSubmit} className="modal-body"
                      style={{display: "flex", flexDirection: "column", gap: "16px"}}>
                    {/* Header Image Picker */}
                    <div style={{position: "relative"}}>
                        <label className="form-label" style={{marginBottom: "6px"}}>헤더 배경 이미지</label>
                        <div
                            style={{
                                height: "120px",
                                borderRadius: "var(--radius-md)",
                                overflow: "hidden",
                                backgroundColor: "var(--bg-surface-hover)",
                                backgroundImage: headerImageUrl ? `url(${headerImageUrl})` : "linear-gradient(135deg, #1e293b 0%, #0f172a 100%)",
                                backgroundSize: "cover",
                                backgroundPosition: "center",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                                position: "relative",
                                border: "1px solid var(--border-subtle)",
                            }}
                        >
                            <input
                                ref={headerFileRef}
                                type="file"
                                accept="image/*"
                                style={{display: "none"}}
                                onChange={(e) => {
                                    if (e.target.files && e.target.files[0]) {
                                        handleFileSelect(e.target.files[0], "header");
                                    }
                                }}
                            />
                            <button
                                type="button"
                                onClick={() => headerFileRef.current?.click()}
                                style={{
                                    backgroundColor: "rgba(0,0,0,0.65)",
                                    color: "#fff",
                                    padding: "8px 14px",
                                    borderRadius: "var(--radius-full)",
                                    fontSize: "12px",
                                    fontWeight: 600,
                                    display: "flex",
                                    alignItems: "center",
                                    gap: "6px",
                                }}
                            >
                                <Camera size={15}/> 배경 변경
                            </button>
                        </div>
                    </div>

                    {/* Avatar Image Picker */}
                    <div style={{display: "flex", alignItems: "center", gap: "16px"}}>
                        <input
                            ref={avatarFileRef}
                            type="file"
                            accept="image/*"
                            style={{display: "none"}}
                            onChange={(e) => {
                                if (e.target.files && e.target.files[0]) {
                                    handleFileSelect(e.target.files[0], "avatar");
                                }
                            }}
                        />
                        <div style={{position: "relative", cursor: "pointer"}}
                             onClick={() => avatarFileRef.current?.click()}>
                            <img
                                src={avatarUrl || defaultAvatar}
                                alt="Avatar"
                                style={{
                                    width: 64,
                                    height: 64,
                                    borderRadius: "50%",
                                    objectFit: "cover",
                                    border: "2px solid var(--accent-primary)",
                                }}
                            />
                            <div
                                style={{
                                    position: "absolute",
                                    inset: 0,
                                    backgroundColor: "rgba(0,0,0,0.45)",
                                    borderRadius: "50%",
                                    display: "flex",
                                    alignItems: "center",
                                    justifyContent: "center",
                                    color: "#fff",
                                }}
                            >
                                <Camera size={20}/>
                            </div>
                        </div>
                        <div>
                            <div style={{fontWeight: 700, fontSize: "14px", color: "var(--text-primary)"}}>프로필 사진</div>
                            <button
                                type="button"
                                onClick={() => avatarFileRef.current?.click()}
                                style={{
                                    fontSize: "12px",
                                    color: "var(--accent-primary)",
                                    fontWeight: 600,
                                    marginTop: "2px"
                                }}
                            >
                                사진 업로드 변경
                            </button>
                        </div>
                    </div>

                    <div className="form-group" style={{marginBottom: 0}}>
                        <label className="form-label">표시 이름 (닉네임) *</label>
                        <input
                            type="text"
                            className="form-input"
                            value={displayName}
                            onChange={(e) => setDisplayName(e.target.value)}
                            placeholder="표시될 이름"
                            required
                        />
                    </div>

                    <div className="form-group" style={{marginBottom: 0}}>
                        <div style={{
                            display: "flex",
                            justifyContent: "space-between",
                            alignItems: "center",
                            marginBottom: "6px"
                        }}>
                            <label className="form-label" style={{marginBottom: 0}}>자기소개 (Bio)</label>
                            <span style={{fontSize: "11px", color: "var(--text-muted)"}}>{bio.length}/160</span>
                        </div>
                        <textarea
                            className="form-input"
                            style={{minHeight: 70, resize: "none"}}
                            value={bio}
                            onChange={(e) => setBio(e.target.value)}
                            placeholder="나를 소개하는 한 마디..."
                            maxLength={160}
                            rows={3}
                        />
                    </div>

                    <div style={{display: "grid", gridTemplateColumns: "1fr 1fr", gap: "12px"}}>
                        <div className="form-group" style={{marginBottom: 0}}>
                            <label className="form-label">거주지 / 위치</label>
                            <input
                                type="text"
                                className="form-input"
                                value={location}
                                onChange={(e) => setLocation(e.target.value)}
                                placeholder="예: Seoul, Korea"
                            />
                        </div>
                        <div className="form-group" style={{marginBottom: 0}}>
                            <label className="form-label">웹사이트 URL</label>
                            <input
                                type="url"
                                className="form-input"
                                value={website}
                                onChange={(e) => setWebsite(e.target.value)}
                                placeholder="https://github.com/..."
                            />
                        </div>
                    </div>

                    <div style={{display: "flex", justifyContent: "flex-end", gap: "10px", marginTop: "8px"}}>
                        <button type="button" onClick={onClose} className="btn-secondary">
                            취소
                        </button>
                        <button type="submit" disabled={loading || !displayName.trim()} className="btn-primary">
                            <Save size={15}/>
                            {loading ? "저장 중..." : "변경사항 저장"}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
}
