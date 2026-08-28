"use client";

import React, {useEffect, useState} from "react";
import {BookmarkCollection} from "@/lib/types";
import {fetchGraphQL, MUTATIONS, QUERIES} from "@/lib/graphql";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import {Bookmark, FolderPlus, Globe, Lock, Plus, X} from "lucide-react";

interface BookmarkCollectionModalProps {
    postId?: string;
    onClose: () => void;
    onPostSaved?: () => void;
}

export default function BookmarkCollectionModal({
                                                    postId,
                                                    onClose,
                                                    onPostSaved,
                                                }: BookmarkCollectionModalProps) {
    const {showToast} = useToast();
    const [collections, setCollections] = useState<BookmarkCollection[]>([]);
    const [loading, setLoading] = useState(true);
    const [showCreate, setShowCreate] = useState(false);
    const [name, setName] = useState("");
    const [description, setDescription] = useState("");
    const [isPrivate, setIsPrivate] = useState(false);
    const [creating, setCreating] = useState(false);

    const loadCollections = async () => {
        try {
            const data = await fetchGraphQL<{ bookmarkCollections: BookmarkCollection[] }>(
                QUERIES.BOOKMARK_COLLECTIONS
            );
            if (data?.bookmarkCollections) {
                setCollections(data.bookmarkCollections);
            }
        } catch (err) {
            console.error("Failed to load bookmark collections:", err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        loadCollections();
    }, []);

    const handleCreateCollection = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!name.trim()) return;

        setCreating(true);
        try {
            const data = await fetchGraphQL<{ createBookmarkCollection: BookmarkCollection }>(
                MUTATIONS.CREATE_BOOKMARK_COLLECTION,
                {
                    name: name.trim(),
                    description: description.trim() || undefined,
                    isPrivate,
                }
            );

            if (data?.createBookmarkCollection) {
                showToast("컬렉션이 생성되었습니다.", "success");
                setCollections((prev) => [data.createBookmarkCollection, ...prev]);
                setName("");
                setDescription("");
                setShowCreate(false);
            }
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setCreating(false);
        }
    };

    const handleSaveToCollection = async (collectionId: string) => {
        if (!postId) return;

        try {
            await fetchGraphQL(MUTATIONS.ADD_POST_TO_COLLECTION, {
                collectionId,
                postId,
            });

            showToast("컬렉션에 게시물이 저장되었습니다!", "success");
            onPostSaved?.();
            onClose();
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        }
    };

    return (
        <div className="modal-overlay" onClick={onClose}>
            <div
                className="modal-content"
                onClick={(e) => e.stopPropagation()}
                style={{maxWidth: "460px"}}
            >
                <div className="modal-header">
                    <div style={{display: "flex", alignItems: "center", gap: "8px"}}>
                        <Bookmark size={20} color="var(--accent-primary)"/>
                        <h3 className="modal-title">북마크 컬렉션</h3>
                    </div>
                    <button onClick={onClose} className="modal-close-btn">
                        <X size={18}/>
                    </button>
                </div>

                <div style={{padding: "20px"}}>
                    {!showCreate ? (
                        <div>
                            <div
                                style={{
                                    display: "flex",
                                    justifyContent: "space-between",
                                    alignItems: "center",
                                    marginBottom: "14px",
                                }}
                            >
                <span
                    style={{
                        fontSize: "13px",
                        fontWeight: 700,
                        color: "var(--text-secondary)",
                    }}
                >
                  {postId ? "저장할 컬렉션을 선택하세요" : "내 컬렉션 목록"}
                </span>
                                <button
                                    type="button"
                                    onClick={() => setShowCreate(true)}
                                    className="btn-secondary"
                                    style={{padding: "5px 10px", fontSize: "12px"}}
                                >
                                    <Plus size={13}/> 새 컬렉션
                                </button>
                            </div>

                            {loading ? (
                                <div style={{
                                    textAlign: "center",
                                    padding: "20px",
                                    color: "var(--text-muted)",
                                    fontSize: "12px"
                                }}>
                                    컬렉션을 불러오는 중...
                                </div>
                            ) : collections.length === 0 ? (
                                <div style={{
                                    textAlign: "center",
                                    padding: "30px 10px",
                                    color: "var(--text-muted)",
                                    fontSize: "13px"
                                }}>
                                    생성된 컬렉션이 없습니다. 새 컬렉션을 만들어보세요!
                                </div>
                            ) : (
                                <div style={{
                                    display: "flex",
                                    flexDirection: "column",
                                    gap: "8px",
                                    maxHeight: "260px",
                                    overflowY: "auto"
                                }}>
                                    {collections.map((c) => (
                                        <div
                                            key={c.id}
                                            onClick={() => postId && handleSaveToCollection(c.id)}
                                            style={{
                                                padding: "12px 14px",
                                                borderRadius: "var(--radius-md)",
                                                border: "1px solid var(--border-subtle)",
                                                backgroundColor: "var(--bg-surface)",
                                                cursor: postId ? "pointer" : "default",
                                                display: "flex",
                                                alignItems: "center",
                                                justifyContent: "space-between",
                                                transition: "all var(--transition-fast)",
                                            }}
                                            onMouseEnter={(e) => {
                                                if (postId) e.currentTarget.style.borderColor = "var(--accent-primary)";
                                            }}
                                            onMouseLeave={(e) => {
                                                if (postId) e.currentTarget.style.borderColor = "var(--border-subtle)";
                                            }}
                                        >
                                            <div>
                                                <div style={{display: "flex", alignItems: "center", gap: "6px"}}>
                          <span style={{fontSize: "14px", fontWeight: 700, color: "var(--text-primary)"}}>
                            {c.name}
                          </span>
                                                    {c.isPrivate ? <Lock size={12} color="var(--text-muted)"/> :
                                                        <Globe size={12} color="var(--text-muted)"/>}
                                                </div>
                                                {c.description && (
                                                    <div style={{
                                                        fontSize: "11px",
                                                        color: "var(--text-muted)",
                                                        marginTop: "2px"
                                                    }}>
                                                        {c.description}
                                                    </div>
                                                )}
                                            </div>
                                            {postId && (
                                                <button
                                                    type="button"
                                                    className="btn-primary"
                                                    style={{padding: "4px 10px", fontSize: "11px"}}
                                                >
                                                    저장
                                                </button>
                                            )}
                                        </div>
                                    ))}
                                </div>
                            )}
                        </div>
                    ) : (
                        <form onSubmit={handleCreateCollection}>
                            <div style={{marginBottom: "14px"}}>
                                <label
                                    style={{fontSize: "13px", fontWeight: 700, display: "block", marginBottom: "6px"}}>
                                    컬렉션 이름
                                </label>
                                <input
                                    type="text"
                                    placeholder="예: 유용한 Rust 팁, 디자인 영감"
                                    value={name}
                                    onChange={(e) => setName(e.target.value)}
                                    className="composer-textarea"
                                    style={{
                                        width: "100%",
                                        padding: "10px",
                                        borderRadius: "var(--radius-md)",
                                        border: "1px solid var(--border-subtle)"
                                    }}
                                    autoFocus
                                />
                            </div>

                            <div style={{marginBottom: "14px"}}>
                                <label
                                    style={{fontSize: "13px", fontWeight: 700, display: "block", marginBottom: "6px"}}>
                                    설명 (선택사항)
                                </label>
                                <input
                                    type="text"
                                    placeholder="컬렉션에 대한 간단한 설명"
                                    value={description}
                                    onChange={(e) => setDescription(e.target.value)}
                                    className="composer-textarea"
                                    style={{
                                        width: "100%",
                                        padding: "10px",
                                        borderRadius: "var(--radius-md)",
                                        border: "1px solid var(--border-subtle)"
                                    }}
                                />
                            </div>

                            <div style={{marginBottom: "16px"}}>
                                <label style={{display: "flex", alignItems: "center", gap: "8px", cursor: "pointer"}}>
                                    <input
                                        type="checkbox"
                                        checked={isPrivate}
                                        onChange={(e) => setIsPrivate(e.target.checked)}
                                        style={{accentColor: "var(--accent-primary)"}}
                                    />
                                    <span style={{fontSize: "13px", color: "var(--text-primary)"}}>
                    비공개 컬렉션으로 설정 (나만 보기)
                  </span>
                                </label>
                            </div>

                            <div style={{display: "flex", justifyContent: "flex-end", gap: "8px"}}>
                                <button
                                    type="button"
                                    onClick={() => setShowCreate(false)}
                                    className="btn-secondary"
                                >
                                    취소
                                </button>
                                <button
                                    type="submit"
                                    disabled={!name.trim() || creating}
                                    className="btn-primary"
                                >
                                    <FolderPlus size={13}/>
                                    {creating ? "생성 중..." : "컬렉션 만들기"}
                                </button>
                            </div>
                        </form>
                    )}
                </div>
            </div>
        </div>
    );
}
