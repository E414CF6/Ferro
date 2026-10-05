"use client";

import React, { useEffect, useState } from "react";
import { UpdateArticleInput, WikiArticle, WikiRevision } from "@/lib/types";
import { useAuth } from "@/lib/auth-context";
import { fetchGraphQL, MUTATIONS, QUERIES } from "@/lib/graphql";
import { useToast } from "@/lib/toast-context";
import { formatErrorMessage } from "@/lib/i18n";
import {
    Calendar,
    Edit3,
    Eye,
    Globe,
    History,
    MapPin,
    Tag,
    Trash2,
    User as UserIcon,
    X,
    Save,
} from "lucide-react";

interface ArticleDetailModalProps {
    slug: string;
    onClose: () => void;
    onArticleUpdated?: () => void;
    onArticleDeleted?: () => void;
}

export default function ArticleDetailModal({
    slug,
    onClose,
    onArticleUpdated,
    onArticleDeleted,
}: ArticleDetailModalProps) {
    const { user } = useAuth();
    const { showToast } = useToast();

    const [article, setArticle] = useState<WikiArticle | null>(null);
    const [loading, setLoading] = useState(true);
    const [activeTab, setActiveTab] = useState<"read" | "revisions" | "edit">("read");

    // Edit state
    const [editTitle, setEditTitle] = useState("");
    const [editContent, setEditContent] = useState("");
    const [editSummary, setEditSummary] = useState("");
    const [editRevisionSummary, setEditRevisionSummary] = useState("");
    const [editTags, setEditTags] = useState<string[]>([]);
    const [editTagInput, setEditTagInput] = useState("");
    const [saving, setSaving] = useState(false);
    const [deleting, setDeleting] = useState(false);

    useEffect(() => {
        const loadArticle = async () => {
            setLoading(true);
            try {
                const data = await fetchGraphQL<{ article: WikiArticle }>(
                    QUERIES.WIKI_ARTICLE,
                    { slug }
                );
                if (data?.article) {
                    setArticle(data.article);
                    setEditTitle(data.article.title);
                    setEditContent(data.article.content);
                    setEditSummary(data.article.summary || "");
                    setEditTags(data.article.tags || []);

                    // Record view count
                    fetchGraphQL(MUTATIONS.RECORD_ARTICLE_VIEW, {
                        articleId: data.article.id,
                    }).catch(() => null);
                }
            } catch (err) {
                console.error("Failed to load article:", err);
            } finally {
                setLoading(false);
            }
        };

        if (slug) {
            loadArticle();
        }
    }, [slug]);

    const handleSaveEdit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!article) return;

        if (!editTitle.trim() || !editContent.trim()) {
            showToast("제목과 내용을 입력해주세요.", "error");
            return;
        }

        const input: UpdateArticleInput = {
            title: editTitle.trim(),
            content: editContent.trim(),
            summary: editSummary.trim() || undefined,
            tags: editTags,
            editSummary: editRevisionSummary.trim() || "문서 내용 보완 및 갱신",
            author: user?.displayName || user?.username || undefined,
        };

        setSaving(true);
        try {
            const data = await fetchGraphQL<{ updateArticle: WikiArticle }>(
                MUTATIONS.UPDATE_WIKI_ARTICLE,
                { slug: article.slug, input }
            );

            if (data?.updateArticle) {
                showToast("문서가 성공적으로 수정 및 새 버전으로 기록되었습니다!", "success");
                setArticle((prev) => (prev ? { ...prev, ...data.updateArticle } : null));
                setActiveTab("read");
                onArticleUpdated?.();
            }
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setSaving(false);
        }
    };

    const handleDelete = async () => {
        if (!article) return;
        if (!window.confirm(`정말로 "${article.title}" 위키 문서를 삭제하시겠습니까?`)) {
            return;
        }

        setDeleting(true);
        try {
            await fetchGraphQL(MUTATIONS.DELETE_WIKI_ARTICLE, { slug: article.slug });
            showToast("위키 문서가 삭제되었습니다.", "info");
            onArticleDeleted?.();
            onClose();
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setDeleting(false);
        }
    };

    const formatDate = (dateStr: string) => {
        return new Date(dateStr).toLocaleDateString("ko-KR", {
            year: "numeric",
            month: "short",
            day: "numeric",
            hour: "2-digit",
            minute: "2-digit",
        });
    };

    return (
        <div className="modal-overlay" onClick={onClose} style={{ zIndex: 120 }}>
            <div
                className="modal-content"
                onClick={(e) => e.stopPropagation()}
                style={{
                    maxWidth: "800px",
                    width: "95%",
                    maxHeight: "92vh",
                    display: "flex",
                    flexDirection: "column",
                }}
            >
                {/* Modal Header */}
                <div className="modal-header">
                    <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                        <div
                            style={{
                                width: 36,
                                height: 36,
                                borderRadius: "8px",
                                backgroundColor: "rgba(56, 189, 248, 0.15)",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                                color: "var(--accent-primary)",
                            }}
                        >
                            <Globe size={20} />
                        </div>
                        <div>
                            <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                                <h3 className="modal-title" style={{ fontSize: "17px" }}>
                                    {loading ? "불러오는 중..." : article?.title}
                                </h3>
                                {article?.category && (
                                    <span
                                        style={{
                                            fontSize: "11px",
                                            padding: "2px 8px",
                                            borderRadius: "var(--radius-full)",
                                            backgroundColor: "rgba(56, 189, 248, 0.15)",
                                            color: "var(--accent-primary)",
                                            fontWeight: 700,
                                        }}
                                    >
                                        {article.category}
                                    </span>
                                )}
                            </div>
                            <span style={{ fontSize: "12px", color: "var(--text-muted)" }}>
                                /{article?.slug}
                            </span>
                        </div>
                    </div>

                    <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                        {/* Tabs */}
                        <div
                            style={{
                                display: "flex",
                                backgroundColor: "var(--bg-input)",
                                borderRadius: "var(--radius-full)",
                                padding: "3px",
                                border: "1px solid var(--border-subtle)",
                            }}
                        >
                            <button
                                type="button"
                                onClick={() => setActiveTab("read")}
                                className={`header-tab-pill ${activeTab === "read" ? "active" : ""}`}
                                style={{ padding: "4px 10px", fontSize: "12px" }}
                            >
                                <Eye size={12} /> 열람
                            </button>
                            <button
                                type="button"
                                onClick={() => setActiveTab("revisions")}
                                className={`header-tab-pill ${activeTab === "revisions" ? "active" : ""}`}
                                style={{ padding: "4px 10px", fontSize: "12px" }}
                            >
                                <History size={12} /> 역사 ({article?.revisions?.length || 0})
                            </button>
                            <button
                                type="button"
                                onClick={() => setActiveTab("edit")}
                                className={`header-tab-pill ${activeTab === "edit" ? "active" : ""}`}
                                style={{ padding: "4px 10px", fontSize: "12px" }}
                            >
                                <Edit3 size={12} /> 편집
                            </button>
                        </div>

                        <button onClick={onClose} className="modal-close-btn">
                            <X size={18} />
                        </button>
                    </div>
                </div>

                {/* Modal Body */}
                <div style={{ padding: "20px", overflowY: "auto", flex: 1 }}>
                    {loading ? (
                        <div style={{ padding: "40px", textAlign: "center", color: "var(--text-muted)" }}>
                            문서를 불러오는 중...
                        </div>
                    ) : !article ? (
                        <div style={{ padding: "40px", textAlign: "center", color: "var(--text-muted)" }}>
                            해당 위키 문서를 찾을 수 없습니다.
                        </div>
                    ) : activeTab === "read" ? (
                        <div>
                            {/* Meta banner */}
                            <div
                                style={{
                                    display: "flex",
                                    flexWrap: "wrap",
                                    alignItems: "center",
                                    justifyContent: "space-between",
                                    gap: "10px",
                                    padding: "12px 16px",
                                    backgroundColor: "var(--bg-surface)",
                                    border: "1px solid var(--border-subtle)",
                                    borderRadius: "var(--radius-md)",
                                    marginBottom: "18px",
                                    fontSize: "12px",
                                    color: "var(--text-secondary)",
                                }}
                            >
                                <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
                                    <span style={{ display: "inline-flex", alignItems: "center", gap: "4px" }}>
                                        <UserIcon size={13} color="var(--accent-primary)" /> {article.author}
                                    </span>
                                    <span style={{ display: "inline-flex", alignItems: "center", gap: "4px" }}>
                                        <Calendar size={13} /> {formatDate(article.updatedAt || article.createdAt)}
                                    </span>
                                    <span style={{ display: "inline-flex", alignItems: "center", gap: "4px" }}>
                                        <Eye size={13} color="#10b981" /> 조회수 {article.views}
                                    </span>
                                </div>

                                <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                                    <MapPin size={13} color="#ec4899" />
                                    <span>
                                        좌표: {article.latitude.toFixed(4)}, {article.longitude.toFixed(4)}
                                    </span>
                                </div>
                            </div>

                            {/* Summary callout if present */}
                            {article.summary && (
                                <div
                                    style={{
                                        padding: "12px 16px",
                                        borderRadius: "var(--radius-md)",
                                        backgroundColor: "rgba(56, 189, 248, 0.08)",
                                        borderLeft: "3px solid var(--accent-primary)",
                                        marginBottom: "20px",
                                        fontSize: "14px",
                                        color: "var(--text-primary)",
                                        fontStyle: "italic",
                                    }}
                                >
                                    "{article.summary}"
                                </div>
                            )}

                            {/* Article Body */}
                            <div
                                style={{
                                    fontSize: "15px",
                                    lineHeight: "1.8",
                                    color: "var(--text-primary)",
                                    whiteSpace: "pre-wrap",
                                    wordBreak: "break-word",
                                    marginBottom: "24px",
                                }}
                            >
                                {article.content}
                            </div>

                            {/* Tags list */}
                            {article.tags && article.tags.length > 0 && (
                                <div style={{ display: "flex", flexWrap: "wrap", gap: "6px", borderTop: "1px solid var(--border-subtle)", paddingTop: "14px" }}>
                                    {article.tags.map((t) => (
                                        <span
                                            key={t}
                                            style={{
                                                fontSize: "12px",
                                                fontWeight: 600,
                                                padding: "3px 10px",
                                                borderRadius: "var(--radius-full)",
                                                backgroundColor: "rgba(168, 85, 247, 0.12)",
                                                color: "var(--accent-purple)",
                                            }}
                                        >
                                            #{t}
                                        </span>
                                    ))}
                                </div>
                            )}

                            {/* Delete article button if creator / user */}
                            <div style={{ marginTop: "24px", display: "flex", justifyContent: "flex-end" }}>
                                <button
                                    onClick={handleDelete}
                                    disabled={deleting}
                                    className="btn-secondary"
                                    style={{ color: "var(--accent-secondary)", fontSize: "12px", padding: "6px 12px" }}
                                >
                                    <Trash2 size={13} /> {deleting ? "삭제 중..." : "문서 삭제"}
                                </button>
                            </div>
                        </div>
                    ) : activeTab === "revisions" ? (
                        <div>
                            <h4 style={{ fontSize: "15px", fontWeight: 700, marginBottom: "14px" }}>
                                판본 역사 (Revision History)
                            </h4>
                            {!article.revisions || article.revisions.length === 0 ? (
                                <div style={{ padding: "20px", textAlign: "center", color: "var(--text-muted)", fontSize: "13px" }}>
                                    기록된 판본 이력이 없습니다.
                                </div>
                            ) : (
                                <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
                                    {article.revisions.map((rev, idx) => (
                                        <div
                                            key={rev.id}
                                            style={{
                                                padding: "14px",
                                                borderRadius: "var(--radius-md)",
                                                backgroundColor: "var(--bg-surface)",
                                                border: "1px solid var(--border-subtle)",
                                                display: "flex",
                                                flexDirection: "column",
                                                gap: "6px",
                                            }}
                                        >
                                            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                                                <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                                                    <span style={{ fontWeight: 800, fontSize: "13px", color: "var(--accent-primary)" }}>
                                                        v{article.revisions!.length - idx}
                                                    </span>
                                                    <span style={{ fontSize: "13px", fontWeight: 700, color: "var(--text-primary)" }}>
                                                        {rev.author}
                                                    </span>
                                                </div>
                                                <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                                                    {formatDate(rev.createdAt)}
                                                </span>
                                            </div>
                                            <div style={{ fontSize: "12px", color: "var(--text-secondary)" }}>
                                                요약: {rev.editSummary || "내용 갱신"}
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            )}
                        </div>
                    ) : (
                        /* Edit Tab */
                        <form onSubmit={handleSaveEdit} style={{ display: "flex", flexDirection: "column", gap: "14px" }}>
                            <div>
                                <label style={{ fontSize: "13px", fontWeight: 700, display: "block", marginBottom: "4px" }}>
                                    제목
                                </label>
                                <input
                                    type="text"
                                    value={editTitle}
                                    onChange={(e) => setEditTitle(e.target.value)}
                                    className="composer-textarea"
                                    style={{ width: "100%", padding: "8px 12px", borderRadius: "var(--radius-md)", border: "1px solid var(--border-subtle)" }}
                                />
                            </div>

                            <div>
                                <label style={{ fontSize: "13px", fontWeight: 700, display: "block", marginBottom: "4px" }}>
                                    요약 (선택)
                                </label>
                                <input
                                    type="text"
                                    value={editSummary}
                                    onChange={(e) => setEditSummary(e.target.value)}
                                    className="composer-textarea"
                                    style={{ width: "100%", padding: "8px 12px", borderRadius: "var(--radius-md)", border: "1px solid var(--border-subtle)" }}
                                />
                            </div>

                            <div>
                                <label style={{ fontSize: "13px", fontWeight: 700, display: "block", marginBottom: "4px" }}>
                                    본문 내용 (Markdown)
                                </label>
                                <textarea
                                    value={editContent}
                                    onChange={(e) => setEditContent(e.target.value)}
                                    rows={10}
                                    className="composer-textarea"
                                    style={{
                                        width: "100%",
                                        padding: "10px",
                                        borderRadius: "var(--radius-md)",
                                        border: "1px solid var(--border-subtle)",
                                        fontFamily: "monospace",
                                        fontSize: "13px",
                                        lineHeight: "1.6",
                                    }}
                                />
                            </div>

                            <div>
                                <label style={{ fontSize: "13px", fontWeight: 700, display: "block", marginBottom: "4px" }}>
                                    수정 요약 (판본 기록용 설명)
                                </label>
                                <input
                                    type="text"
                                    placeholder="예: 최신 정보 반영 및 오타 수정"
                                    value={editRevisionSummary}
                                    onChange={(e) => setEditRevisionSummary(e.target.value)}
                                    className="composer-textarea"
                                    style={{ width: "100%", padding: "8px 12px", borderRadius: "var(--radius-md)", border: "1px solid var(--border-subtle)" }}
                                />
                            </div>

                            <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px", marginTop: "10px" }}>
                                <button type="button" onClick={() => setActiveTab("read")} className="btn-secondary">
                                    취소
                                </button>
                                <button type="submit" disabled={saving} className="btn-primary">
                                    <Save size={14} /> {saving ? "저장 중..." : "판본 저장"}
                                </button>
                            </div>
                        </form>
                    )}
                </div>
            </div>
        </div>
    );
}
