"use client";

import React, {useEffect, useState} from "react";
import {User} from "@/lib/types";
import {fetchGraphQL, QUERIES} from "@/lib/graphql";
import {useAuth} from "@/lib/auth-context";
import {MessageSquarePlus, Search, X} from "lucide-react";

interface NewMessageModalProps {
    onClose: () => void;
    onSelectUser: (user: User) => void;
}

export default function NewMessageModal({
                                            onClose,
                                            onSelectUser,
                                        }: NewMessageModalProps) {
    const {user} = useAuth();
    const [users, setUsers] = useState<User[]>([]);
    const [search, setSearch] = useState("");
    const [loading, setLoading] = useState(true);

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

    const filteredUsers = users.filter(
        (u) =>
            u.username.toLowerCase().includes(search.toLowerCase()) ||
            u.displayName.toLowerCase().includes(search.toLowerCase())
    );

    const defaultAvatar = "https://api.dicebear.com/7.x/bottts/svg?seed=user";

    return (
        <div className="modal-overlay" onClick={onClose}>
            <div
                className="modal-card"
                onClick={(e) => e.stopPropagation()}
                style={{maxWidth: "460px"}}
            >
                <div className="modal-header">
                    <div style={{display: "flex", alignItems: "center", gap: "8px"}}>
                        <MessageSquarePlus size={20} color="var(--accent-primary)"/>
                        <h2 className="modal-title">새 메시지</h2>
                    </div>
                    <button onClick={onClose} className="modal-close-btn">
                        <X size={20}/>
                    </button>
                </div>

                <div className="modal-body" style={{display: "flex", flexDirection: "column", gap: "14px"}}>
                    {/* Search input */}
                    <div style={{position: "relative"}}>
                        <Search
                            size={16}
                            style={{
                                position: "absolute",
                                left: 14,
                                top: "50%",
                                transform: "translateY(-50%)",
                                color: "var(--text-muted)",
                            }}
                        />
                        <input
                            type="text"
                            className="form-input"
                            style={{paddingLeft: "38px"}}
                            placeholder="받는 사람 검색 (이름 또는 @아이디)..."
                            value={search}
                            onChange={(e) => setSearch(e.target.value)}
                            autoFocus
                        />
                    </div>

                    {/* Users list */}
                    <div
                        style={{
                            display: "flex",
                            flexDirection: "column",
                            gap: "6px",
                            maxHeight: "360px",
                            overflowY: "auto",
                        }}
                    >
                        {loading ? (
                            <div style={{
                                textAlign: "center",
                                padding: "30px",
                                color: "var(--text-muted)",
                                fontSize: "13px"
                            }}>
                                사용자 목록을 불러오는 중...
                            </div>
                        ) : filteredUsers.length === 0 ? (
                            <div style={{
                                textAlign: "center",
                                padding: "30px",
                                color: "var(--text-muted)",
                                fontSize: "13px"
                            }}>
                                검색 결과가 없습니다.
                            </div>
                        ) : (
                            filteredUsers.map((u) => (
                                <div
                                    key={u.id}
                                    onClick={() => {
                                        onSelectUser(u);
                                        onClose();
                                    }}
                                    style={{
                                        display: "flex",
                                        alignItems: "center",
                                        gap: "12px",
                                        padding: "10px 12px",
                                        borderRadius: "var(--radius-md)",
                                        cursor: "pointer",
                                        backgroundColor: "var(--bg-input)",
                                        border: "1px solid var(--border-subtle)",
                                        transition: "all var(--transition-fast)",
                                    }}
                                    onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = "var(--bg-surface-hover)")}
                                    onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = "var(--bg-input)")}
                                >
                                    <img
                                        src={u.avatarUrl || defaultAvatar}
                                        alt={u.username}
                                        style={{width: 40, height: 40, borderRadius: "50%", objectFit: "cover"}}
                                    />
                                    <div style={{flex: 1, minWidth: 0}}>
                                        <div style={{fontWeight: 700, fontSize: "14px", color: "var(--text-primary)"}}>
                                            {u.displayName || u.username}
                                        </div>
                                        <div style={{fontSize: "12px", color: "var(--text-muted)"}}>
                                            @{u.username}
                                        </div>
                                    </div>
                                </div>
                            ))
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
