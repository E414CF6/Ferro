"use client";

import React, {useEffect, useState} from "react";
import Link from "next/link";
import {Post, UserList} from "@/lib/types";
import {useAuth} from "@/lib/auth-context";
import {fetchGraphQL, MUTATIONS, QUERIES} from "@/lib/graphql";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import PostCard from "@/components/PostCard";
import {ListFilter, Lock, Plus, Users, X,} from "lucide-react";

export default function ListsPage() {
    const {user} = useAuth();
    const {showToast} = useToast();
    const [lists, setLists] = useState<UserList[]>([]);
    const [selectedListId, setSelectedListId] = useState<string | null>(null);
    const [posts, setPosts] = useState<Post[]>([]);
    const [loadingLists, setLoadingLists] = useState(true);
    const [loadingPosts, setLoadingPosts] = useState(false);
    const [showCreateModal, setShowCreateModal] = useState(false);

    // New list form state
    const [name, setName] = useState("");
    const [description, setDescription] = useState("");
    const [isPrivate, setIsPrivate] = useState(false);
    const [creating, setCreating] = useState(false);

    const loadLists = async () => {
        try {
            const data = await fetchGraphQL<{ userLists: UserList[] }>(
                QUERIES.USER_LISTS
            );
            if (data?.userLists) {
                setLists(data.userLists);
                if (data.userLists.length > 0 && !selectedListId) {
                    setSelectedListId(data.userLists[0].id);
                }
            }
        } catch (err) {
            console.error("Failed to load user lists:", err);
        } finally {
            setLoadingLists(false);
        }
    };

    const loadListFeed = async (listId: string) => {
        setLoadingPosts(true);
        try {
            const data = await fetchGraphQL<{ listFeed: Post[] }>(QUERIES.LIST_FEED, {
                listId,
                limit: 30,
                offset: 0,
            });
            if (data?.listFeed) {
                setPosts(data.listFeed);
            }
        } catch (err) {
            console.error("Failed to load list feed:", err);
        } finally {
            setLoadingPosts(false);
        }
    };

    useEffect(() => {
        if (user) {
            loadLists();
        } else {
            setLoadingLists(false);
        }
    }, [user]);

    useEffect(() => {
        if (selectedListId) {
            loadListFeed(selectedListId);
        }
    }, [selectedListId]);

    const handleCreateList = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!name.trim()) return;

        setCreating(true);
        try {
            const data = await fetchGraphQL<{ createUserList: UserList }>(
                MUTATIONS.CREATE_USER_LIST,
                {
                    name: name.trim(),
                    description: description.trim() || undefined,
                    isPrivate,
                }
            );

            if (data?.createUserList) {
                showToast("사용자 리스트가 생성되었습니다.", "success");
                setLists((prev) => [data.createUserList, ...prev]);
                setSelectedListId(data.createUserList.id);
                setName("");
                setDescription("");
                setShowCreateModal(false);
            }
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setCreating(false);
        }
    };

    if (!user) {
        return (
            <div>
                <header className="sticky-header">
                    <h1 className="header-title">사용자 리스트</h1>
                </header>
                <div style={{padding: "60px 20px", textAlign: "center"}}>
                    <ListFilter size={40} color="var(--text-muted)" style={{margin: "0 auto 16px"}}/>
                    <h3 style={{fontSize: "18px", fontWeight: 800}}>로그인이 필요합니다</h3>
                    <p style={{fontSize: "14px", color: "var(--text-secondary)", marginTop: "8px"}}>
                        관심 있는 개발자들을 모아 맞춤형 피드를 관리해보세요.
                    </p>
                    <Link href="/welcome?tab=login" className="btn-primary"
                          style={{marginTop: "16px", display: "inline-flex"}}>
                        로그인하기
                    </Link>
                </div>
            </div>
        );
    }

    return (
        <div>
            <header className="sticky-header">
                <div style={{display: "flex", alignItems: "center", gap: "10px"}}>
                    <ListFilter size={20} color="var(--accent-primary)"/>
                    <h1 className="header-title">사용자 리스트 (Custom Lists)</h1>
                </div>

                <button
                    onClick={() => setShowCreateModal(true)}
                    className="btn-primary"
                    style={{padding: "6px 12px", fontSize: "12px"}}
                >
                    <Plus size={14}/> 새 리스트
                </button>
            </header>

            {/* Lists Selector Tray */}
            <div
                style={{
                    display: "flex",
                    gap: "8px",
                    overflowX: "auto",
                    padding: "14px 20px",
                    borderBottom: "1px solid var(--border-subtle)",
                    backgroundColor: "var(--bg-surface)",
                }}
            >
                {lists.map((l) => (
                    <button
                        key={l.id}
                        onClick={() => setSelectedListId(l.id)}
                        style={{
                            padding: "6px 14px",
                            borderRadius: "var(--radius-full)",
                            fontSize: "13px",
                            fontWeight: 700,
                            display: "flex",
                            alignItems: "center",
                            gap: "6px",
                            whiteSpace: "nowrap",
                            transition: "all var(--transition-fast)",
                            backgroundColor:
                                selectedListId === l.id
                                    ? "var(--accent-primary)"
                                    : "var(--bg-input)",
                            color: selectedListId === l.id ? "#fff" : "var(--text-primary)",
                            border: `1px solid ${
                                selectedListId === l.id
                                    ? "var(--accent-primary)"
                                    : "var(--border-subtle)"
                            }`,
                        }}
                    >
                        <Users size={14}/>
                        <span>{l.name}</span>
                        {l.isPrivate && <Lock size={11}/>}
                    </button>
                ))}
            </div>

            {/* List Feed Posts */}
            <div>
                {loadingPosts ? (
                    <div style={{padding: "40px", textAlign: "center", color: "var(--text-muted)"}}>
                        리스트 피드를 불러오는 중...
                    </div>
                ) : posts.length === 0 ? (
                    <div style={{padding: "60px 20px", textAlign: "center", color: "var(--text-muted)"}}>
                        <Users size={40} style={{margin: "0 auto 14px", opacity: 0.5}}/>
                        <h3 style={{fontSize: "16px", fontWeight: 700, color: "var(--text-primary)"}}>
                            리스트 피드에 게시물이 없습니다
                        </h3>
                        <p style={{fontSize: "13px", color: "var(--text-secondary)", marginTop: "6px"}}>
                            리스트에 관심 있는 크리에이터를 추가하여 피드를 채워보세요.
                        </p>
                    </div>
                ) : (
                    posts.map((p) => (
                        <PostCard
                            key={p.id}
                            post={p}
                            onPostDeleted={() =>
                                selectedListId && loadListFeed(selectedListId)
                            }
                        />
                    ))
                )}
            </div>

            {/* Create List Modal */}
            {showCreateModal && (
                <div className="modal-overlay" onClick={() => setShowCreateModal(false)}>
                    <div
                        className="modal-content"
                        onClick={(e) => e.stopPropagation()}
                        style={{maxWidth: "460px"}}
                    >
                        <div className="modal-header">
                            <div style={{display: "flex", alignItems: "center", gap: "8px"}}>
                                <ListFilter size={20} color="var(--accent-primary)"/>
                                <h3 className="modal-title">새 사용자 리스트 생성</h3>
                            </div>
                            <button
                                onClick={() => setShowCreateModal(false)}
                                className="modal-close-btn"
                            >
                                <X size={18}/>
                            </button>
                        </div>

                        <form onSubmit={handleCreateList} style={{padding: "20px"}}>
                            <div style={{marginBottom: "14px"}}>
                                <label
                                    style={{fontSize: "13px", fontWeight: 700, display: "block", marginBottom: "6px"}}>
                                    리스트 이름
                                </label>
                                <input
                                    type="text"
                                    placeholder="예: Rust 핵심 개발진, 디자이너 모음"
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
                                    placeholder="리스트에 대한 설명"
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
                    비공개 리스트로 설정 (나만 피드 보기)
                  </span>
                                </label>
                            </div>

                            <div style={{display: "flex", justifyContent: "flex-end", gap: "8px"}}>
                                <button
                                    type="button"
                                    onClick={() => setShowCreateModal(false)}
                                    className="btn-secondary"
                                >
                                    취소
                                </button>
                                <button
                                    type="submit"
                                    disabled={!name.trim() || creating}
                                    className="btn-primary"
                                >
                                    <Plus size={13}/>
                                    {creating ? "생성 중..." : "리스트 만들기"}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            )}
        </div>
    );
}
