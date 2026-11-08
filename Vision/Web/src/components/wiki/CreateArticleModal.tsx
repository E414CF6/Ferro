"use client";

import React, { useState } from "react";
import { CreateArticleInput, WikiArticle } from "@/lib/types";
import { useAuth } from "@/lib/auth-context";
import { fetchGraphQL, MUTATIONS } from "@/lib/graphql";
import { useToast } from "@/lib/toast-context";
import { formatErrorMessage } from "@/lib/i18n";
import { Globe, MapPin, Tag, Type, X, Eye, FileText, Send } from "lucide-react";

interface CreateArticleModalProps {
    initialCoords?: { lat: number; lng: number } | null;
    onClose: () => void;
    onCreated: (article: WikiArticle) => void;
}

const CATEGORIES = [
    "장소/명소",
    "역사/문화",
    "기술/개발",
    "자연/지리",
    "맛집/카페",
    "커뮤니티",
    "기타",
];

export default function CreateArticleModal({
    initialCoords,
    onClose,
    onCreated,
}: CreateArticleModalProps) {
    const { user } = useAuth();
    const { showToast } = useToast();

    const [title, setTitle] = useState("");
    const [summary, setSummary] = useState("");
    const [content, setContent] = useState("");
    const [latitude, setLatitude] = useState<number>(initialCoords?.lat ?? 37.5665);
    const [longitude, setLongitude] = useState<number>(initialCoords?.lng ?? 126.9780);
    const [zoom, setZoom] = useState<number>(14);
    const [category, setCategory] = useState(CATEGORIES[0]);
    const [tagInput, setTagInput] = useState("");
    const [tags, setTags] = useState<string[]>([]);
    const [authorName, setAuthorName] = useState(user?.displayName || user?.username || "익명 기여자");
    const [activeTab, setActiveTab] = useState<"edit" | "preview">("edit");
    const [submitting, setSubmitting] = useState(false);

    const handleAddTag = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === "Enter" || e.key === ",") {
            e.preventDefault();
            const val = tagInput.trim().replace(/^#/, "");
            if (val && !tags.includes(val)) {
                if (tags.length >= 15) {
                    showToast("태그는 최대 15개까지 추가할 수 있습니다.", "info");
                    return;
                }
                setTags([...tags, val]);
                setTagInput("");
            }
        }
    };

    const handleRemoveTag = (tagToRemove: string) => {
        setTags(tags.filter((t) => t !== tagToRemove));
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();

        if (!title.trim()) {
            showToast("위키 문서 제목을 입력해주세요.", "error");
            return;
        }
        if (!content.trim()) {
            showToast("위키 본문 내용을 입력해주세요.", "error");
            return;
        }
        if (latitude < -90 || latitude > 90 || longitude < -180 || longitude > 180) {
            showToast("유효한 위도(-90~90) 및 경도(-180~180)를 입력해주세요.", "error");
            return;
        }

        const input: CreateArticleInput = {
            title: title.trim(),
            summary: summary.trim() || undefined,
            content: content.trim(),
            latitude: Number(latitude),
            longitude: Number(longitude),
            zoom: Number(zoom),
            tags: tags.length > 0 ? tags : [category],
            author: authorName.trim() || undefined,
        };

        setSubmitting(true);
        try {
            const data = await fetchGraphQL<{ createArticle: WikiArticle }>(
                MUTATIONS.CREATE_WIKI_ARTICLE,
                { input }
            );

            if (data?.createArticle) {
                showToast(`위키 문서 "${data.createArticle.title}" 등록 완료!`, "success");
                onCreated(data.createArticle);
                onClose();
            }
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <div className="modal-overlay" onClick={onClose} style={{ zIndex: 110 }}>
            <div
                className="modal-content"
                onClick={(e) => e.stopPropagation()}
                style={{
                    maxWidth: "720px",
                    width: "95%",
                    maxHeight: "90vh",
                    display: "flex",
                    flexDirection: "column",
                }}
            >
                {/* Header */}
                <div className="modal-header">
                    <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                        <div
                            style={{
                                width: 34,
                                height: 34,
                                borderRadius: "8px",
                                backgroundColor: "rgba(56, 189, 248, 0.15)",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                                color: "var(--accent-primary)",
                            }}
                        >
                            <Globe size={18} />
                        </div>
                        <div>
                            <h3 className="modal-title">새 공간 위키 문서 작성</h3>
                            <p style={{ fontSize: "12px", color: "var(--text-muted)" }}>
                                지도 좌표와 마크다운 지식을 결합한 wMap 아티클을 기여해보세요.
                            </p>
                        </div>
                    </div>
                    <button onClick={onClose} className="modal-close-btn">
                        <X size={18} />
                    </button>
                </div>

                {/* Form Body */}
                <form
                    onSubmit={handleSubmit}
                    style={{
                        padding: "20px",
                        overflowY: "auto",
                        display: "flex",
                        flexDirection: "column",
                        gap: "16px",
                    }}
                >
                    {/* Title */}
                    <div>
                        <label style={{ fontSize: "13px", fontWeight: 700, display: "block", marginBottom: "6px" }}>
                            문서 제목 *
                        </label>
                        <input
                            type="text"
                            placeholder="예: 경복궁 근정전, 판교 테크노밸리, 성수동 카페거리"
                            value={title}
                            onChange={(e) => setTitle(e.target.value)}
                            className="composer-textarea"
                            style={{ width: "100%", padding: "10px 14px", borderRadius: "var(--radius-md)", border: "1px solid var(--border-subtle)" }}
                            autoFocus
                        />
                    </div>

                    {/* Coordinates & Category Grid */}
                    <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))", gap: "12px" }}>
                        <div>
                            <label style={{ fontSize: "12px", fontWeight: 700, display: "flex", alignItems: "center", gap: "4px", marginBottom: "4px" }}>
                                <MapPin size={13} color="var(--accent-primary)" /> 위도 (Latitude)
                            </label>
                            <input
                                type="number"
                                step="any"
                                value={latitude}
                                onChange={(e) => setLatitude(parseFloat(e.target.value) || 0)}
                                className="composer-textarea"
                                style={{ width: "100%", padding: "8px 10px", borderRadius: "var(--radius-md)", border: "1px solid var(--border-subtle)" }}
                            />
                        </div>

                        <div>
                            <label style={{ fontSize: "12px", fontWeight: 700, display: "flex", alignItems: "center", gap: "4px", marginBottom: "4px" }}>
                                <MapPin size={13} color="var(--accent-primary)" /> 경도 (Longitude)
                            </label>
                            <input
                                type="number"
                                step="any"
                                value={longitude}
                                onChange={(e) => setLongitude(parseFloat(e.target.value) || 0)}
                                className="composer-textarea"
                                style={{ width: "100%", padding: "8px 10px", borderRadius: "var(--radius-md)", border: "1px solid var(--border-subtle)" }}
                            />
                        </div>

                        <div>
                            <label style={{ fontSize: "12px", fontWeight: 700, display: "block", marginBottom: "4px" }}>
                                카테고리
                            </label>
                            <select
                                value={category}
                                onChange={(e) => setCategory(e.target.value)}
                                style={{
                                    width: "100%",
                                    padding: "9px 10px",
                                    borderRadius: "var(--radius-md)",
                                    backgroundColor: "var(--bg-input)",
                                    border: "1px solid var(--border-subtle)",
                                    color: "var(--text-primary)",
                                    fontSize: "13px",
                                }}
                            >
                                {CATEGORIES.map((c) => (
                                    <option key={c} value={c}>{c}</option>
                                ))}
                            </select>
                        </div>
                    </div>

                    {/* Summary */}
                    <div>
                        <label style={{ fontSize: "13px", fontWeight: 700, display: "block", marginBottom: "6px" }}>
                            한 줄 요약 (선택)
                        </label>
                        <input
                            type="text"
                            placeholder="지도 핀 툴팁에 표시될 간결한 핵심 요약"
                            value={summary}
                            onChange={(e) => setSummary(e.target.value)}
                            className="composer-textarea"
                            style={{ width: "100%", padding: "8px 14px", borderRadius: "var(--radius-md)", border: "1px solid var(--border-subtle)" }}
                        />
                    </div>

                    {/* Tags Input */}
                    <div>
                        <label style={{ fontSize: "13px", fontWeight: 700, display: "flex", alignItems: "center", gap: "6px", marginBottom: "6px" }}>
                            <Tag size={14} color="#a855f7" /> 해시태그 (엔터로 추가, 최대 15개)
                        </label>
                        <div style={{ display: "flex", flexWrap: "wrap", gap: "6px", marginBottom: "6px" }}>
                            {tags.map((t) => (
                                <span
                                    key={t}
                                    style={{
                                        display: "inline-flex",
                                        alignItems: "center",
                                        gap: "4px",
                                        padding: "3px 8px",
                                        borderRadius: "var(--radius-full)",
                                        backgroundColor: "rgba(168, 85, 247, 0.15)",
                                        color: "var(--accent-purple)",
                                        fontSize: "12px",
                                        fontWeight: 600,
                                    }}
                                >
                                    #{t}
                                    <button
                                        type="button"
                                        onClick={() => handleRemoveTag(t)}
                                        style={{ color: "var(--text-muted)", marginLeft: "2px" }}
                                    >
                                        <X size={12} />
                                    </button>
                                </span>
                            ))}
                        </div>
                        <input
                            type="text"
                            placeholder="태그 입력 후 엔터..."
                            value={tagInput}
                            onChange={(e) => setTagInput(e.target.value)}
                            onKeyDown={handleAddTag}
                            className="composer-textarea"
                            style={{ width: "100%", padding: "8px 12px", borderRadius: "var(--radius-md)", border: "1px solid var(--border-subtle)" }}
                        />
                    </div>

                    {/* Markdown Content Editor with Preview Tabs */}
                    <div>
                        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "6px" }}>
                            <label style={{ fontSize: "13px", fontWeight: 700 }}>본문 내용 (Markdown) *</label>
                            <div style={{ display: "flex", gap: "4px" }}>
                                <button
                                    type="button"
                                    onClick={() => setActiveTab("edit")}
                                    className={`header-tab-pill ${activeTab === "edit" ? "active" : ""}`}
                                    style={{ padding: "3px 10px", fontSize: "11px" }}
                                >
                                    <FileText size={12} /> 편집
                                </button>
                                <button
                                    type="button"
                                    onClick={() => setActiveTab("preview")}
                                    className={`header-tab-pill ${activeTab === "preview" ? "active" : ""}`}
                                    style={{ padding: "3px 10px", fontSize: "11px" }}
                                >
                                    <Eye size={12} /> 미리보기
                                </button>
                            </div>
                        </div>

                        {activeTab === "edit" ? (
                            <textarea
                                placeholder="마크다운 형식으로 상세한 지리적, 역사적, 문화적 지식을 기술해주세요. (## 헤더, - 목록, **굵게**, [링크](url) 지원)"
                                value={content}
                                onChange={(e) => setContent(e.target.value)}
                                rows={8}
                                className="composer-textarea"
                                style={{
                                    width: "100%",
                                    padding: "12px",
                                    borderRadius: "var(--radius-md)",
                                    border: "1px solid var(--border-subtle)",
                                    fontFamily: "monospace",
                                    fontSize: "13px",
                                    lineHeight: "1.6",
                                }}
                            />
                        ) : (
                            <div
                                style={{
                                    minHeight: "180px",
                                    maxHeight: "260px",
                                    overflowY: "auto",
                                    padding: "14px",
                                    borderRadius: "var(--radius-md)",
                                    backgroundColor: "var(--bg-input)",
                                    border: "1px solid var(--border-subtle)",
                                    fontSize: "14px",
                                    lineHeight: "1.6",
                                    whiteSpace: "pre-wrap",
                                }}
                            >
                                {content || <span style={{ color: "var(--text-muted)" }}>미리보기할 내용이 없습니다.</span>}
                            </div>
                        )}
                    </div>

                    {/* Author Name */}
                    <div>
                        <label style={{ fontSize: "12px", color: "var(--text-secondary)", display: "block", marginBottom: "4px" }}>
                            기여자 이름 (익명 기여 지원)
                        </label>
                        <input
                            type="text"
                            value={authorName}
                            onChange={(e) => setAuthorName(e.target.value)}
                            className="composer-textarea"
                            style={{ width: "100%", padding: "8px 12px", borderRadius: "var(--radius-md)", border: "1px solid var(--border-subtle)", fontSize: "13px" }}
                        />
                    </div>

                    {/* Actions */}
                    <div style={{ display: "flex", justifyContent: "flex-end", gap: "10px", marginTop: "8px" }}>
                        <button type="button" onClick={onClose} className="btn-secondary">
                            취소
                        </button>
                        <button type="submit" disabled={submitting || !title.trim() || !content.trim()} className="btn-primary">
                            <Send size={14} /> {submitting ? "문서 등록 중..." : "위키 문서 등록"}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
}
