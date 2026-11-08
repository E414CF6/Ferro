"use client";

import React, { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import { Post, User, UserList } from "@/lib/types";
import { useAuth } from "@/lib/auth-context";
import { fetchGraphQL, MUTATIONS, QUERIES } from "@/lib/graphql";
import { useToast } from "@/lib/toast-context";
import { formatErrorMessage } from "@/lib/i18n";
import PostCard from "@/components/PostCard";
import {
    Check,
    ListFilter,
    Lock,
    Plus,
    Search,
    Trash2,
    UserPlus,
    Users,
    X,
} from "lucide-react";

export default function ListsPage() {
    const { user } = useAuth();
    const { showToast } = useToast();
    const [lists, setLists] = useState<UserList[]>([]);
    const [selectedListId, setSelectedListId] = useState<string | null>(null);
    const [posts, setPosts] = useState<Post[]>([]);
    const [loadingLists, setLoadingLists] = useState(true);
    const [loadingPosts, setLoadingPosts] = useState(false);
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [showMembersModal, setShowMembersModal] = useState(false);

    // New list form state
    const [name, setName] = useState("");
    const [description, setDescription] = useState("");
    const [isPrivate, setIsPrivate] = useState(false);
    const [creating, setCreating] = useState(false);

    // Member management state
    const [allUsers, setAllUsers] = useState<User[]>([]);
    const [memberSearch, setMemberSearch] = useState("");

    const loadLists = useCallback(async () => {
        if (!user) return;
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
    }, [user, selectedListId]);

    const loadListFeed = useCallback(async (listId: string) => {
        setLoadingPosts(true);
        try {
            const data = await fetchGraphQL<{
                listFeedConnection: { edges: { node: Post }[] };
            }>(QUERIES.LIST_FEED, {
                listId,
                first: 30,
            });
            if (data?.listFeedConnection?.edges) {
                setPosts(data.listFeedConnection.edges.map((e) => e.node));
            } else {
                setPosts([]);
            }
        } catch (err) {
            console.error("Failed to load list feed:", err);
            setPosts([]);
        } finally {
            setLoadingPosts(false);
        }
    }, []);

    const loadAllUsers = async () => {
        try {
            const data = await fetchGraphQL<{ users: User[] }>(QUERIES.USERS_LIST);
            if (data?.users) {
                setAllUsers(data.users);
            }
        } catch {
            // silent
        }
    };

    useEffect(() => {
        if (user) {
            loadLists();
            loadAllUsers();
        } else {
            setLoadingLists(false);
        }
    }, [user, loadLists]);

    useEffect(() => {
        if (selectedListId) {
            loadListFeed(selectedListId);
        }
    }, [selectedListId, loadListFeed]);

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

    const handleDeleteList = async (listId: string, listName: string) => {
        if (!window.confirm(`정말로 "${listName}" 리스트를 삭제하시겠습니까?`)) return;
        try {
            await fetchGraphQL(MUTATIONS.DELETE_USER_LIST, { listId });
            setLists((prev) => prev.filter((l) => l.id !== listId));
            setSelectedListId(lists.find((l) => l.id !== listId)?.id || null);
            showToast("리스트가 삭제되었습니다.", "info");
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        }
    };

    const handleToggleMember = async (targetUser: User, isMember: boolean) => {
        if (!selectedListId) return;
        try {
            if (isMember) {
                await fetchGraphQL(MUTATIONS.REMOVE_USER_FROM_LIST, {
                    listId: selectedListId,
                    userId: targetUser.id,
                });
                setLists((prev) =>
                    prev.map((l) =>
                        l.id === selectedListId
                            ? {
                                  ...l,
                                  membersCount: Math.max(0, l.membersCount - 1),
                                  members: l.members?.filter((m) => m.id !== targetUser.id),
                              }
                            : l
                    )
                );
                showToast(`@${targetUser.username} 님을 리스트에서 제외했습니다.`, "info");
            } else {
                await fetchGraphQL(MUTATIONS.ADD_USER_TO_LIST, {
                    listId: selectedListId,
                    userId: targetUser.id,
                });
                setLists((prev) =>
                    prev.map((l) =>
                        l.id === selectedListId
                            ? {
                                  ...l,
                                  membersCount: l.membersCount + 1,
                                  members: [...(l.members || []), targetUser],
                              }
                            : l
                    )
                );
                showToast(`@${targetUser.username} 님을 리스트에 추가했습니다.`, "success");
            }
            loadListFeed(selectedListId);
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        }
    };

    if (!user) {
        return (
            <div>
                <header className="sticky-header">
                    <h1 className="header-title">사용자 리스트</h1>
                </header>
                <div style={{ padding: "60px 20px", textAlign: "center" }}>
                    <ListFilter size={40} color="var(--text-muted)" style={{ margin: "0 auto 16px" }} />
                    <h3 style={{ fontSize: "18px", fontWeight: 800 }}>로그인이 필요합니다</h3>
                    <p style={{ fontSize: "14px", color: "var(--text-secondary)", marginTop: "8px" }}>
                        관심 있는 개발자들을 모아 맞춤형 피드를 관리해보세요.
                    </p>
                    <Link
                        href="/login"
                        className="btn-primary"
                        style={{ marginTop: "16px", display: "inline-flex" }}
                    >
                        로그인하기
                    </Link>
                </div>
            </div>
        );
    }

    const currentList = lists.find((l) => l.id === selectedListId);

    return (
        <div>
            {/* Header */}
            <header className="sticky-header">
                <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                    <ListFilter size={20} color="var(--accent-primary)" />
                    <h1 className="header-title">사용자 리스트</h1>
                </div>

                <button
                    onClick={() => setShowCreateModal(true)}
                    className="btn-primary"
                    style={{ padding: "6px 14px", fontSize: "12px" }}
                >
                    <Plus size={14} /> 새 리스트
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
                        <Users size={14} />
                        <span>{l.name}</span>
                        {l.isPrivate && <Lock size={11} />}
                    </button>
                ))}
            </div>

            {/* List Details & Actions Bar */}
            {currentList && (
                <div
                    style={{
                        padding: "12px 20px",
                        backgroundColor: "rgba(56, 189, 248, 0.05)",
                        borderBottom: "1px solid var(--border-subtle)",
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "space-between",
                    }}
                >
                    <div>
                        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                            <span style={{ fontSize: "15px", fontWeight: 800, color: "var(--text-primary)" }}>
                                {currentList.name}
                            </span>
                            <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                                멤버 {currentList.membersCount}명
                            </span>
                        </div>
                        {currentList.description && (
                            <div style={{ fontSize: "12px", color: "var(--text-muted)", marginTop: "2px" }}>
                                {currentList.description}
                            </div>
                        )}
                    </div>

                    <div style={{ display: "flex", gap: "8px" }}>
                        <button
                            onClick={() => setShowMembersModal(true)}
                            className="btn-secondary"
                            style={{ padding: "5px 12px", fontSize: "12px" }}
                        >
                            <UserPlus size={13} /> 멤버 관리
                        </button>
                        <button
                            onClick={() => handleDeleteList(currentList.id, currentList.name)}
                            className="btn-secondary"
                            style={{ color: "var(--accent-secondary)", padding: "5px 10px", fontSize: "12px" }}
                        >
                            <Trash2 size={13} /> 리스트 삭제
                        </button>
                    </div>
                </div>
            )}

            {/* List Feed Posts */}
            <div>
                {loadingPosts ? (
                    <div style={{ padding: "40px", textAlign: "center", color: "var(--text-muted)" }}>
                        리스트 피드를 불러오는 중...
                    </div>
                ) : posts.length === 0 ? (
                    <div style={{ padding: "80px 20px", textAlign: "center", color: "var(--text-muted)" }}>
                        <Users size={44} style={{ margin: "0 auto 14px", opacity: 0.3 }} />
                        <h3 style={{ fontSize: "17px", fontWeight: 700, color: "var(--text-primary)" }}>
                            리스트 피드에 게시물이 없습니다
                        </h3>
                        <p style={{ fontSize: "13px", color: "var(--text-secondary)", marginTop: "6px" }}>
                            상단 '멤버 관리' 버튼을 눌러 관심 있는 크리에이터들을 추가해보세요.
                        </p>
                    </div>
                ) : (
                    posts.map((p) => (
                        <PostCard
                            key={p.id}
                            post={p}
                            onPostDeleted={() => selectedListId && loadListFeed(selectedListId)}
                            onPostUpdated={() => selectedListId && loadListFeed(selectedListId)}
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
                        style={{ maxWidth: "460px" }}
                    >
                        <div className="modal-header">
                            <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                                <ListFilter size={20} color="var(--accent-primary)" />
                                <h3 className="modal-title">새 사용자 리스트 생성</h3>
                            </div>
                            <button onClick={() => setShowCreateModal(false)} className="modal-close-btn">
                                <X size={18} />
                            </button>
                        </div>

                        <form onSubmit={handleCreateList} style={{ padding: "20px" }}>
                            <div style={{ marginBottom: "14px" }}>
                                <label style={{ fontSize: "13px", fontWeight: 700, display: "block", marginBottom: "6px" }}>
                                    리스트 이름
                                </label>
                                <input
                                    type="text"
                                    placeholder="예: 크리에이터, 디자이너, 친구들"
                                    value={name}
                                    onChange={(e) => setName(e.target.value)}
                                    className="composer-textarea"
                                    style={{
                                        width: "100%",
                                        padding: "10px",
                                        borderRadius: "var(--radius-md)",
                                        border: "1px solid var(--border-subtle)",
                                    }}
                                    autoFocus
                                />
                            </div>

                            <div style={{ marginBottom: "14px" }}>
                                <label style={{ fontSize: "13px", fontWeight: 700, display: "block", marginBottom: "6px" }}>
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
                                        border: "1px solid var(--border-subtle)",
                                    }}
                                />
                            </div>

                            <div style={{ marginBottom: "16px" }}>
                                <label style={{ display: "flex", alignItems: "center", gap: "8px", cursor: "pointer" }}>
                                    <input
                                        type="checkbox"
                                        checked={isPrivate}
                                        onChange={(e) => setIsPrivate(e.target.checked)}
                                        style={{ accentColor: "var(--accent-primary)" }}
                                    />
                                    <span style={{ fontSize: "13px", color: "var(--text-primary)" }}>
                                        비공개 리스트 (나만 피드 보기)
                                    </span>
                                </label>
                            </div>

                            <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
                                <button type="button" onClick={() => setShowCreateModal(false)} className="btn-secondary">
                                    취소
                                </button>
                                <button type="submit" disabled={!name.trim() || creating} className="btn-primary">
                                    <Plus size={13} />
                                    {creating ? "생성 중..." : "리스트 만들기"}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            )}

            {/* Manage Members Modal */}
            {showMembersModal && currentList && (
                <div className="modal-overlay" onClick={() => setShowMembersModal(false)}>
                    <div
                        className="modal-content"
                        onClick={(e) => e.stopPropagation()}
                        style={{ maxWidth: "480px" }}
                    >
                        <div className="modal-header">
                            <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                                <Users size={20} color="var(--accent-primary)" />
                                <h3 className="modal-title">"{currentList.name}" 멤버 관리</h3>
                            </div>
                            <button onClick={() => setShowMembersModal(false)} className="modal-close-btn">
                                <X size={18} />
                            </button>
                        </div>

                        <div style={{ padding: "20px" }}>
                            {/* Search */}
                            <div style={{ position: "relative", marginBottom: "14px" }}>
                                <Search
                                    size={15}
                                    style={{
                                        position: "absolute",
                                        left: 12,
                                        top: "50%",
                                        transform: "translateY(-50%)",
                                        color: "var(--text-muted)",
                                    }}
                                />
                                <input
                                    type="text"
                                    placeholder="사용자 검색..."
                                    value={memberSearch}
                                    onChange={(e) => setMemberSearch(e.target.value)}
                                    className="composer-textarea"
                                    style={{
                                        width: "100%",
                                        padding: "8px 12px 8px 34px",
                                        borderRadius: "var(--radius-md)",
                                        border: "1px solid var(--border-subtle)",
                                        fontSize: "13px",
                                    }}
                                />
                            </div>

                            {/* Users list */}
                            <div style={{ display: "flex", flexDirection: "column", gap: "6px", maxHeight: "300px", overflowY: "auto" }}>
                                {allUsers
                                    .filter(
                                        (u) =>
                                            u.username.toLowerCase().includes(memberSearch.toLowerCase()) ||
                                            u.displayName.toLowerCase().includes(memberSearch.toLowerCase())
                                    )
                                    .map((u) => {
                                        const isMember = currentList.members?.some((m) => m.id === u.id) ?? false;
                                        return (
                                            <div
                                                key={u.id}
                                                style={{
                                                    display: "flex",
                                                    alignItems: "center",
                                                    justifyContent: "space-between",
                                                    padding: "8px 12px",
                                                    borderRadius: "var(--radius-md)",
                                                    backgroundColor: "var(--bg-surface)",
                                                    border: "1px solid var(--border-subtle)",
                                                }}
                                            >
                                                <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                                                    <img
                                                        src={u.avatarUrl || "https://api.dicebear.com/7.x/bottts/svg?seed=" + u.username}
                                                        alt={u.username}
                                                        style={{ width: 32, height: 32, borderRadius: "50%" }}
                                                    />
                                                    <div>
                                                        <div style={{ fontSize: "13px", fontWeight: 700, color: "var(--text-primary)" }}>
                                                            {u.displayName || u.username}
                                                        </div>
                                                        <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                                                            @{u.username}
                                                        </div>
                                                    </div>
                                                </div>

                                                <button
                                                    onClick={() => handleToggleMember(u, isMember)}
                                                    className={isMember ? "btn-secondary" : "btn-primary"}
                                                    style={{ padding: "4px 12px", fontSize: "11px" }}
                                                >
                                                    {isMember ? "제외" : "추가"}
                                                </button>
                                            </div>
                                        );
                                    })}
                            </div>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
