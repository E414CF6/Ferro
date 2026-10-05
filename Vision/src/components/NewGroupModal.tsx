"use client";

import React, { useEffect, useState } from "react";
import { GroupConversation, User } from "@/lib/types";
import { fetchGraphQL, MUTATIONS, QUERIES } from "@/lib/graphql";
import { useAuth } from "@/lib/auth-context";
import { useToast } from "@/lib/toast-context";
import { formatErrorMessage } from "@/lib/i18n";
import { Check, Search, Users, X } from "lucide-react";

interface NewGroupModalProps {
    onClose: () => void;
    onGroupCreated: (group: GroupConversation) => void;
}

export default function NewGroupModal({ onClose, onGroupCreated }: NewGroupModalProps) {
    const { user } = useAuth();
    const { showToast } = useToast();
    const [title, setTitle] = useState("");
    const [users, setUsers] = useState<User[]>([]);
    const [selectedUserIds, setSelectedUserIds] = useState<string[]>([]);
    const [search, setSearch] = useState("");
    const [loading, setLoading] = useState(true);
    const [creating, setCreating] = useState(false);

    useEffect(() => {
        async function loadUsers() {
            try {
                const data = await fetchGraphQL<{ users: User[] }>(QUERIES.USERS_LIST);
                if (data?.users) {
                    const filtered = data.users.filter((u) => u.username !== user?.username);
                    setUsers(filtered);
                }
            } catch (err) {
                console.error("Failed to load users:", err);
            } finally {
                setLoading(false);
            }
        }

        loadUsers();
    }, [user]);

    const toggleSelectUser = (userId: string) => {
        setSelectedUserIds((prev) =>
            prev.includes(userId) ? prev.filter((id) => id !== userId) : [...prev, userId]
        );
    };

    const handleCreate = async (e: React.FormEvent) => {
        e.preventDefault();
        if (selectedUserIds.length === 0) {
            showToast("그룹에 참여할 멤버를 최소 1명 이상 선택해주세요.", "info");
            return;
        }

        setCreating(true);
        try {
            const data = await fetchGraphQL<{ createGroupConversation: GroupConversation }>(
                MUTATIONS.CREATE_GROUP_CONVERSATION,
                {
                    title: title.trim() || undefined,
                    participantIds: selectedUserIds,
                }
            );

            if (data?.createGroupConversation) {
                showToast("그룹 대화방이 생성되었습니다!", "success");
                onGroupCreated(data.createGroupConversation);
                onClose();
            }
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setCreating(false);
        }
    };

    const filteredUsers = users.filter(
        (u) =>
            u.username.toLowerCase().includes(search.toLowerCase()) ||
            u.displayName.toLowerCase().includes(search.toLowerCase())
    );

    const defaultAvatar = "https://api.dicebear.com/7.x/bottts/svg?seed=user";

    return (
        <div className="modal-overlay" onClick={onClose} style={{ zIndex: 110 }}>
            <div
                className="modal-content"
                onClick={(e) => e.stopPropagation()}
                style={{ maxWidth: "480px" }}
            >
                <div className="modal-header">
                    <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                        <Users size={20} color="var(--accent-primary)" />
                        <h3 className="modal-title">새 그룹 대화방 만들기</h3>
                    </div>
                    <button onClick={onClose} className="modal-close-btn">
                        <X size={18} />
                    </button>
                </div>

                <form onSubmit={handleCreate} style={{ padding: "20px" }}>
                    {/* Group Title */}
                    <div style={{ marginBottom: "14px" }}>
                        <label style={{ fontSize: "13px", fontWeight: 700, display: "block", marginBottom: "6px" }}>
                            그룹 이름 (선택사항)
                        </label>
                        <input
                            type="text"
                            placeholder="예: Ferro Rust 스터디 그룹, 프로젝트 회의"
                            value={title}
                            onChange={(e) => setTitle(e.target.value)}
                            className="composer-textarea"
                            style={{
                                width: "100%",
                                padding: "8px 12px",
                                borderRadius: "var(--radius-md)",
                                border: "1px solid var(--border-subtle)",
                            }}
                            autoFocus
                        />
                    </div>

                    {/* Member Search */}
                    <div style={{ marginBottom: "10px", position: "relative" }}>
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
                            placeholder="추가할 멤버 검색..."
                            value={search}
                            onChange={(e) => setSearch(e.target.value)}
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

                    {/* Selected Pills */}
                    {selectedUserIds.length > 0 && (
                        <div style={{ display: "flex", flexWrap: "wrap", gap: "6px", marginBottom: "12px" }}>
                            {selectedUserIds.map((id) => {
                                const u = users.find((item) => item.id === id);
                                return (
                                    <span
                                        key={id}
                                        style={{
                                            display: "inline-flex",
                                            alignItems: "center",
                                            gap: "4px",
                                            fontSize: "12px",
                                            fontWeight: 600,
                                            padding: "2px 8px",
                                            borderRadius: "var(--radius-full)",
                                            backgroundColor: "rgba(56, 189, 248, 0.15)",
                                            color: "var(--accent-primary)",
                                        }}
                                    >
                                        @{u?.username || id}
                                        <button
                                            type="button"
                                            onClick={() => toggleSelectUser(id)}
                                            style={{ color: "var(--text-muted)" }}
                                        >
                                            <X size={12} />
                                        </button>
                                    </span>
                                );
                            })}
                        </div>
                    )}

                    {/* Users list */}
                    <div
                        style={{
                            display: "flex",
                            flexDirection: "column",
                            gap: "6px",
                            maxHeight: "240px",
                            overflowY: "auto",
                            marginBottom: "16px",
                            border: "1px solid var(--border-subtle)",
                            borderRadius: "var(--radius-md)",
                            padding: "6px",
                        }}
                    >
                        {loading ? (
                            <div style={{ padding: "20px", textAlign: "center", color: "var(--text-muted)", fontSize: "12px" }}>
                                사용자 목록을 불러오는 중...
                            </div>
                        ) : filteredUsers.length === 0 ? (
                            <div style={{ padding: "20px", textAlign: "center", color: "var(--text-muted)", fontSize: "12px" }}>
                                검색 결과가 없습니다.
                            </div>
                        ) : (
                            filteredUsers.map((u) => {
                                const isSelected = selectedUserIds.includes(u.id);
                                return (
                                    <div
                                        key={u.id}
                                        onClick={() => toggleSelectUser(u.id)}
                                        style={{
                                            display: "flex",
                                            alignItems: "center",
                                            justifyContent: "space-between",
                                            padding: "8px 10px",
                                            borderRadius: "var(--radius-md)",
                                            cursor: "pointer",
                                            backgroundColor: isSelected ? "rgba(56, 189, 248, 0.1)" : "transparent",
                                        }}
                                    >
                                        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                                            <img
                                                src={u.avatarUrl || defaultAvatar}
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

                                        <div
                                            style={{
                                                width: 20,
                                                height: 20,
                                                borderRadius: "4px",
                                                border: `1px solid ${isSelected ? "var(--accent-primary)" : "var(--border-subtle)"}`,
                                                backgroundColor: isSelected ? "var(--accent-primary)" : "transparent",
                                                display: "flex",
                                                alignItems: "center",
                                                justifyContent: "center",
                                                color: "#fff",
                                            }}
                                        >
                                            {isSelected && <Check size={14} />}
                                        </div>
                                    </div>
                                );
                            })
                        )}
                    </div>

                    <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
                        <button type="button" onClick={onClose} className="btn-secondary">
                            취소
                        </button>
                        <button
                            type="submit"
                            disabled={creating || selectedUserIds.length === 0}
                            className="btn-primary"
                        >
                            {creating ? "생성 중..." : `그룹방 만들기 (${selectedUserIds.length}명)`}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
}
