"use client";

import React, { Suspense, useCallback, useEffect, useRef, useState } from "react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { Conversation, DirectMessage, GroupConversation, User } from "@/lib/types";
import { useAuth } from "@/lib/auth-context";
import {
    fetchGraphQL,
    MUTATIONS,
    QUERIES,
    subscribeToDirectMessages,
    subscribeToTypingStatus,
} from "@/lib/graphql";
import { useToast } from "@/lib/toast-context";
import { formatErrorMessage } from "@/lib/i18n";
import NewMessageModal from "@/components/NewMessageModal";
import NewGroupModal from "@/components/NewGroupModal";
import {
    ArrowLeft,
    Check,
    CheckCheck,
    Edit2,
    MessageCircle,
    MessageSquare,
    Search,
    Send,
    Sparkles,
    Trash2,
    UserPlus,
    Users,
    X,
} from "lucide-react";

function MessagesContent() {
    const { user } = useAuth();
    const { showToast } = useToast();
    const searchParams = useSearchParams();
    const targetUsernameParam = searchParams.get("user");

    // Chat mode: 1:1 or Group
    const [chatMode, setChatMode] = useState<"direct" | "group">("direct");

    // 1:1 Conversations state
    const [conversations, setConversations] = useState<Conversation[]>([]);
    const [loadingConversations, setLoadingConversations] = useState(true);
    const [selectedUser, setSelectedUser] = useState<User | null>(null);

    // Group Conversations state
    const [groupConversations, setGroupConversations] = useState<GroupConversation[]>([]);
    const [selectedGroup, setSelectedGroup] = useState<GroupConversation | null>(null);

    // Messages state
    const [messages, setMessages] = useState<DirectMessage[]>([]);
    const [loadingMessages, setLoadingMessages] = useState(false);
    const [inputMessage, setInputMessage] = useState("");
    const [sending, setSending] = useState(false);

    // Edit message state
    const [editingMessageId, setEditingMessageId] = useState<string | null>(null);
    const [editingContent, setEditingContent] = useState("");

    // Typing state
    const [isOtherTyping, setIsOtherTyping] = useState(false);
    const typingTimeoutRef = useRef<any>(null);
    const broadcastTypingTimeoutRef = useRef<any>(null);

    // Modals
    const [showNewMsgModal, setShowNewMsgModal] = useState(false);
    const [showNewGroupModal, setShowNewGroupModal] = useState(false);
    const [filterQuery, setFilterQuery] = useState("");

    const messagesEndRef = useRef<HTMLDivElement | null>(null);

    const scrollToBottom = () => {
        messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    };

    // Load 1:1 conversations
    const loadConversations = useCallback(async () => {
        if (!user) return;
        try {
            const data = await fetchGraphQL<{ conversations: Conversation[] }>(
                QUERIES.CONVERSATIONS
            );
            if (data?.conversations) {
                setConversations(data.conversations);
            }
        } catch (err) {
            console.error("Failed to load conversations:", err);
        } finally {
            setLoadingConversations(false);
        }
    }, [user]);

    // Load Group conversations
    const loadGroupConversations = useCallback(async () => {
        if (!user) return;
        try {
            const data = await fetchGraphQL<{ groupConversations: GroupConversation[] }>(
                QUERIES.GROUP_CONVERSATIONS
            );
            if (data?.groupConversations) {
                setGroupConversations(data.groupConversations);
            }
        } catch (err) {
            console.error("Failed to load group conversations:", err);
        }
    }, [user]);

    // Load 1:1 messages
    const loadMessages = useCallback(
        async (otherUser: User, markRead = true) => {
            if (!user) return;
            try {
                const data = await fetchGraphQL<{ directMessages: DirectMessage[] }>(
                    QUERIES.DIRECT_MESSAGES,
                    { otherUserId: otherUser.id }
                );
                if (data?.directMessages) {
                    setMessages(data.directMessages);
                }

                if (markRead) {
                    await fetchGraphQL(MUTATIONS.MARK_MESSAGES_AS_READ, {
                        senderId: otherUser.id,
                    }).catch(() => null);

                    setConversations((prev) =>
                        prev.map((c) =>
                            c.otherUser.id === otherUser.id ? { ...c, unreadCount: 0 } : c
                        )
                    );
                }
            } catch (err) {
                console.error("Failed to load messages:", err);
            } finally {
                setLoadingMessages(false);
            }
        },
        [user]
    );

    // Load Group messages
    const loadGroupMessages = useCallback(
        async (group: GroupConversation) => {
            if (!user) return;
            try {
                const data = await fetchGraphQL<{ groupConversation: GroupConversation }>(
                    QUERIES.GROUP_CONVERSATION,
                    { id: group.id }
                );
                if (data?.groupConversation?.messages) {
                    setMessages(data.groupConversation.messages);
                }
            } catch (err) {
                console.error("Failed to load group messages:", err);
            } finally {
                setLoadingMessages(false);
            }
        },
        [user]
    );

    // Initialize target user from URL
    useEffect(() => {
        if (targetUsernameParam && user) {
            const initTarget = async () => {
                try {
                    const data = await fetchGraphQL<{ profile: User }>(QUERIES.USER_PROFILE, {
                        username: targetUsernameParam,
                    });
                    if (data?.profile) {
                        setSelectedUser(data.profile);
                        setSelectedGroup(null);
                        setChatMode("direct");
                        setLoadingMessages(true);
                        loadMessages(data.profile);
                    }
                } catch (e) {
                    console.error("Failed to load target user:", e);
                }
            };
            initTarget();
        }
    }, [targetUsernameParam, user, loadMessages]);

    // Initial load & Polling fallback
    useEffect(() => {
        loadConversations();
        loadGroupConversations();
        const interval = setInterval(() => {
            loadConversations();
            loadGroupConversations();
        }, 10000);
        return () => clearInterval(interval);
    }, [loadConversations, loadGroupConversations]);

    // WebSocket real-time subscription for live messages
    useEffect(() => {
        if (!user) return;

        const unsubscribe = subscribeToDirectMessages(user.id, (newMsg) => {
            // If the incoming message belongs to the current chat
            if (
                selectedUser &&
                (newMsg.sender.id === selectedUser.id || newMsg.recipient?.id === selectedUser.id)
            ) {
                setMessages((prev) => {
                    if (prev.some((m) => m.id === newMsg.id)) return prev;
                    return [...prev, newMsg];
                });
                // mark as read
                fetchGraphQL(MUTATIONS.MARK_MESSAGES_AS_READ, {
                    senderId: selectedUser.id,
                }).catch(() => null);
            } else if (selectedGroup && newMsg.conversationId === selectedGroup.id) {
                setMessages((prev) => {
                    if (prev.some((m) => m.id === newMsg.id)) return prev;
                    return [...prev, newMsg];
                });
            }

            // Update conversation list
            loadConversations();
            loadGroupConversations();
        });

        return () => unsubscribe();
    }, [user, selectedUser, selectedGroup, loadConversations, loadGroupConversations]);

    // WebSocket subscription for typing indicator
    useEffect(() => {
        if (!user) return;
        const subParams = selectedGroup
            ? { conversationId: selectedGroup.id }
            : selectedUser
            ? { recipientId: user.id }
            : null;

        if (!subParams) return;

        const unsubTyping = subscribeToTypingStatus(subParams, (evt) => {
            if (evt.userId !== user.id) {
                setIsOtherTyping(evt.isTyping);
                if (typingTimeoutRef.current) clearTimeout(typingTimeoutRef.current);
                if (evt.isTyping) {
                    typingTimeoutRef.current = setTimeout(() => {
                        setIsOtherTyping(false);
                    }, 4000);
                }
            }
        });

        return () => {
            unsubTyping();
            if (typingTimeoutRef.current) clearTimeout(typingTimeoutRef.current);
        };
    }, [user, selectedUser, selectedGroup]);

    useEffect(() => {
        scrollToBottom();
    }, [messages, isOtherTyping]);

    // Broadcast typing indicator
    const handleInputChange = (val: string) => {
        setInputMessage(val);
        if (!user || (!selectedUser && !selectedGroup)) return;

        if (!broadcastTypingTimeoutRef.current) {
            fetchGraphQL(MUTATIONS.SEND_TYPING_INDICATOR, {
                recipientId: selectedUser ? selectedUser.id : undefined,
                conversationId: selectedGroup ? selectedGroup.id : undefined,
                isTyping: true,
            }).catch(() => null);
        }

        if (broadcastTypingTimeoutRef.current) clearTimeout(broadcastTypingTimeoutRef.current);
        broadcastTypingTimeoutRef.current = setTimeout(() => {
            broadcastTypingTimeoutRef.current = null;
            fetchGraphQL(MUTATIONS.SEND_TYPING_INDICATOR, {
                recipientId: selectedUser ? selectedUser.id : undefined,
                conversationId: selectedGroup ? selectedGroup.id : undefined,
                isTyping: false,
            }).catch(() => null);
        }, 2000);
    };

    const handleSelectConversation = (otherUser: User) => {
        setSelectedUser(otherUser);
        setSelectedGroup(null);
        setLoadingMessages(true);
        loadMessages(otherUser, true);
    };

    const handleSelectGroup = (group: GroupConversation) => {
        setSelectedGroup(group);
        setSelectedUser(null);
        setLoadingMessages(true);
        loadGroupMessages(group);
    };

    // Send Message
    const handleSendMessage = async (e: React.FormEvent) => {
        e.preventDefault();
        const content = inputMessage.trim();
        if (!content || sending || (!selectedUser && !selectedGroup)) return;

        setInputMessage("");
        setSending(true);

        const tempId = "temp-" + Date.now();
        const optimisticMsg: DirectMessage = {
            id: tempId,
            content,
            isRead: false,
            isMine: true,
            createdAt: new Date().toISOString(),
            sender: user!,
            recipient: selectedUser || undefined,
            conversationId: selectedGroup ? selectedGroup.id : undefined,
        };
        setMessages((prev) => [...prev, optimisticMsg]);

        try {
            if (selectedGroup) {
                const res = await fetchGraphQL<{ sendGroupMessage: DirectMessage }>(
                    MUTATIONS.SEND_GROUP_MESSAGE,
                    {
                        conversationId: selectedGroup.id,
                        content,
                    }
                );
                if (res?.sendGroupMessage) {
                    setMessages((prev) =>
                        prev.map((m) => (m.id === tempId ? res.sendGroupMessage : m))
                    );
                }
            } else if (selectedUser) {
                const res = await fetchGraphQL<{ sendDirectMessage: DirectMessage }>(
                    MUTATIONS.SEND_DIRECT_MESSAGE,
                    {
                        recipientId: selectedUser.id,
                        content,
                    }
                );
                if (res?.sendDirectMessage) {
                    setMessages((prev) =>
                        prev.map((m) => (m.id === tempId ? res.sendDirectMessage : m))
                    );
                }
            }
        } catch (err: any) {
            setMessages((prev) => prev.filter((m) => m.id !== tempId));
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setSending(false);
        }
    };

    // Edit message
    const handleSaveEditMessage = async (msgId: string) => {
        if (!editingContent.trim()) return;
        try {
            await fetchGraphQL(MUTATIONS.EDIT_DIRECT_MESSAGE, {
                messageId: msgId,
                content: editingContent.trim(),
            });
            setMessages((prev) =>
                prev.map((m) => (m.id === msgId ? { ...m, content: editingContent.trim() } : m))
            );
            setEditingMessageId(null);
            showToast("메시지가 수정되었습니다.", "info");
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        }
    };

    // Delete message
    const handleDeleteMessage = async (msgId: string) => {
        if (!window.confirm("이 메시지를 삭제하시겠습니까?")) return;
        try {
            await fetchGraphQL(MUTATIONS.DELETE_DIRECT_MESSAGE, { messageId: msgId });
            setMessages((prev) => prev.filter((m) => m.id !== msgId));
            showToast("메시지가 삭제되었습니다.", "info");
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        }
    };

    const formatMessageTime = (dateStr: string) => {
        const d = new Date(dateStr);
        return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    };

    const defaultAvatar = "https://api.dicebear.com/7.x/bottts/svg?seed=user";

    if (!user) {
        return (
            <div style={{ padding: "80px 20px", textAlign: "center" }}>
                <MessageSquare size={48} color="var(--text-muted)" style={{ margin: "0 auto 16px" }} />
                <h2 style={{ fontSize: "20px", fontWeight: 800, marginBottom: "8px" }}>
                    메시지를 확인하려면 로그인하세요
                </h2>
                <p style={{ color: "var(--text-secondary)", fontSize: "14px", marginBottom: "20px" }}>
                    다른 사용자와 실시간 1:1 및 그룹 대화를 나눌 수 있습니다.
                </p>
                <Link href="/login" className="btn-primary" style={{ display: "inline-flex" }}>
                    로그인하기
                </Link>
            </div>
        );
    }

    return (
        <div style={{ display: "flex", height: "calc(100vh - 60px)", overflow: "hidden" }}>
            {/* Left Conversations Sidebar */}
            <div
                style={{
                    width: "320px",
                    borderRight: "1px solid var(--border-subtle)",
                    display: selectedUser || selectedGroup ? "none" : "flex",
                    flexDirection: "column",
                    backgroundColor: "var(--bg-surface)",
                    flexShrink: 0,
                }}
                className="md:flex"
            >
                {/* Header */}
                <div style={{ padding: "14px 16px", borderBottom: "1px solid var(--border-subtle)" }}>
                    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "12px" }}>
                        <h2 style={{ fontSize: "17px", fontWeight: 800, color: "var(--text-primary)" }}>
                            메시지
                        </h2>
                        <div style={{ display: "flex", gap: "6px" }}>
                            <button
                                onClick={() => setShowNewGroupModal(true)}
                                title="새 그룹 대화방"
                                className="btn-secondary"
                                style={{ padding: "6px", borderRadius: "var(--radius-full)" }}
                            >
                                <Users size={16} />
                            </button>
                            <button
                                onClick={() => setShowNewMsgModal(true)}
                                title="새 1:1 메시지"
                                className="btn-primary"
                                style={{ padding: "6px", borderRadius: "var(--radius-full)" }}
                            >
                                <UserPlus size={16} />
                            </button>
                        </div>
                    </div>

                    {/* Mode Toggle Pills: 1:1 vs Groups */}
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
                            onClick={() => setChatMode("direct")}
                            className={`header-tab-pill ${chatMode === "direct" ? "active" : ""}`}
                            style={{ flex: 1, justifyContent: "center", fontSize: "12px" }}
                        >
                            <MessageCircle size={13} /> 1:1 대화
                        </button>
                        <button
                            onClick={() => setChatMode("group")}
                            className={`header-tab-pill ${chatMode === "group" ? "active" : ""}`}
                            style={{ flex: 1, justifyContent: "center", fontSize: "12px" }}
                        >
                            <Users size={13} /> 그룹방 ({groupConversations.length})
                        </button>
                    </div>
                </div>

                {/* Conversation List */}
                <div style={{ overflowY: "auto", flex: 1 }}>
                    {chatMode === "direct" ? (
                        loadingConversations ? (
                            <div style={{ padding: "20px", textAlign: "center", color: "var(--text-muted)", fontSize: "13px" }}>
                                대화 목록 불러오는 중...
                            </div>
                        ) : conversations.length === 0 ? (
                            <div style={{ padding: "40px 20px", textAlign: "center", color: "var(--text-muted)", fontSize: "13px" }}>
                                활성화된 대화가 없습니다. 상단 + 버튼을 눌러 대화를 시작해보세요!
                            </div>
                        ) : (
                            conversations.map((c) => {
                                const isSelected = selectedUser?.id === c.otherUser.id;
                                return (
                                    <div
                                        key={c.otherUser.id}
                                        onClick={() => handleSelectConversation(c.otherUser)}
                                        style={{
                                            padding: "12px 16px",
                                            display: "flex",
                                            alignItems: "center",
                                            gap: "12px",
                                            cursor: "pointer",
                                            backgroundColor: isSelected ? "var(--bg-surface-hover)" : "transparent",
                                            borderBottom: "1px solid var(--border-subtle)",
                                            transition: "background-color 0.15s ease",
                                        }}
                                    >
                                        <img
                                            src={c.otherUser.avatarUrl || defaultAvatar}
                                            alt={c.otherUser.username}
                                            style={{ width: 44, height: 44, borderRadius: "50%", objectFit: "cover" }}
                                        />
                                        <div style={{ flex: 1, minWidth: 0 }}>
                                            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "2px" }}>
                                                <span style={{ fontSize: "14px", fontWeight: 700, color: "var(--text-primary)" }}>
                                                    {c.otherUser.displayName || c.otherUser.username}
                                                </span>
                                                <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                                                    {c.lastMessage ? formatMessageTime(c.lastMessage.createdAt) : ""}
                                                </span>
                                            </div>
                                            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                                                <span
                                                    style={{
                                                        fontSize: "12px",
                                                        color: c.unreadCount > 0 ? "var(--text-primary)" : "var(--text-muted)",
                                                        fontWeight: c.unreadCount > 0 ? 700 : 400,
                                                        whiteSpace: "nowrap",
                                                        overflow: "hidden",
                                                        textOverflow: "ellipsis",
                                                        maxWidth: "180px",
                                                    }}
                                                >
                                                    {c.lastMessage ? c.lastMessage.content : "메시지 없음"}
                                                </span>
                                                {c.unreadCount > 0 && (
                                                    <span className="badge-count" style={{ position: "static" }}>
                                                        {c.unreadCount}
                                                    </span>
                                                )}
                                            </div>
                                        </div>
                                    </div>
                                );
                            })
                        )
                    ) : (
                        /* Group Conversations List */
                        groupConversations.length === 0 ? (
                            <div style={{ padding: "40px 20px", textAlign: "center", color: "var(--text-muted)", fontSize: "13px" }}>
                                참여 중인 그룹 대화방이 없습니다. 상단 그룹 아이콘을 눌러 새 방을 만들어보세요!
                            </div>
                        ) : (
                            groupConversations.map((g) => {
                                const isSelected = selectedGroup?.id === g.id;
                                return (
                                    <div
                                        key={g.id}
                                        onClick={() => handleSelectGroup(g)}
                                        style={{
                                            padding: "12px 16px",
                                            display: "flex",
                                            alignItems: "center",
                                            gap: "12px",
                                            cursor: "pointer",
                                            backgroundColor: isSelected ? "var(--bg-surface-hover)" : "transparent",
                                            borderBottom: "1px solid var(--border-subtle)",
                                        }}
                                    >
                                        <div
                                            style={{
                                                width: 44,
                                                height: 44,
                                                borderRadius: "12px",
                                                backgroundColor: "rgba(168, 85, 247, 0.15)",
                                                display: "flex",
                                                alignItems: "center",
                                                justifyContent: "center",
                                                color: "var(--accent-purple)",
                                                flexShrink: 0,
                                            }}
                                        >
                                            <Users size={22} />
                                        </div>
                                        <div style={{ flex: 1, minWidth: 0 }}>
                                            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                                                <span style={{ fontSize: "14px", fontWeight: 700, color: "var(--text-primary)" }}>
                                                    {g.title || "그룹 대화방"}
                                                </span>
                                                <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                                                    {g.members.length}명
                                                </span>
                                            </div>
                                            <div style={{ fontSize: "12px", color: "var(--text-muted)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                                                {g.members.map((m) => m.displayName || m.username).join(", ")}
                                            </div>
                                        </div>
                                    </div>
                                );
                            })
                        )
                    )}
                </div>
            </div>

            {/* Right Chat Area */}
            <div style={{ flex: 1, display: "flex", flexDirection: "column", backgroundColor: "var(--bg-surface)" }}>
                {selectedUser || selectedGroup ? (
                    <>
                        {/* Chat Header */}
                        <div
                            style={{
                                padding: "12px 18px",
                                borderBottom: "1px solid var(--border-subtle)",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "space-between",
                                backgroundColor: "var(--bg-surface)",
                            }}
                        >
                            <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                                <button
                                    onClick={() => {
                                        setSelectedUser(null);
                                        setSelectedGroup(null);
                                    }}
                                    className="md:hidden"
                                    style={{ color: "var(--text-muted)", padding: "4px" }}
                                >
                                    <ArrowLeft size={18} />
                                </button>

                                {selectedUser ? (
                                    <Link href={`/profile/${selectedUser.username}`} style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                                        <img
                                            src={selectedUser.avatarUrl || defaultAvatar}
                                            alt={selectedUser.username}
                                            style={{ width: 38, height: 38, borderRadius: "50%", objectFit: "cover" }}
                                        />
                                        <div>
                                            <div style={{ fontSize: "14px", fontWeight: 700, color: "var(--text-primary)" }}>
                                                {selectedUser.displayName || selectedUser.username}
                                            </div>
                                            <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                                                @{selectedUser.username}
                                            </div>
                                        </div>
                                    </Link>
                                ) : (
                                    <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                                        <div
                                            style={{
                                                width: 38,
                                                height: 38,
                                                borderRadius: "10px",
                                                backgroundColor: "rgba(168, 85, 247, 0.15)",
                                                display: "flex",
                                                alignItems: "center",
                                                justifyContent: "center",
                                                color: "var(--accent-purple)",
                                            }}
                                        >
                                            <Users size={18} />
                                        </div>
                                        <div>
                                            <div style={{ fontSize: "14px", fontWeight: 700, color: "var(--text-primary)" }}>
                                                {selectedGroup?.title || "그룹 대화방"}
                                            </div>
                                            <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                                                참여자 {selectedGroup?.members.length}명
                                            </div>
                                        </div>
                                    </div>
                                )}
                            </div>
                        </div>

                        {/* Messages Stream Body */}
                        <div style={{ flex: 1, padding: "20px", overflowY: "auto", display: "flex", flexDirection: "column", gap: "12px" }}>
                            {loadingMessages ? (
                                <div style={{ textAlign: "center", color: "var(--text-muted)", padding: "40px" }}>
                                    메시지를 불러오는 중...
                                </div>
                            ) : messages.length === 0 ? (
                                <div style={{ textAlign: "center", color: "var(--text-muted)", padding: "60px 20px" }}>
                                    <Sparkles size={36} color="var(--accent-primary)" style={{ margin: "0 auto 12px" }} />
                                    <p style={{ fontSize: "14px", fontWeight: 700, color: "var(--text-primary)" }}>
                                        새로운 대화가 시작되었습니다!
                                    </p>
                                    <p style={{ fontSize: "12px", marginTop: "4px" }}>
                                        첫 번째 메시지를 전송하여 인사를 건네보세요.
                                    </p>
                                </div>
                            ) : (
                                messages.map((m) => {
                                    const isMine = m.isMine ?? (m.sender && m.sender.username === user.username);
                                    const isEditing = editingMessageId === m.id;

                                    return (
                                        <div
                                            key={m.id}
                                            style={{
                                                display: "flex",
                                                flexDirection: "column",
                                                alignItems: isMine ? "flex-end" : "flex-start",
                                                maxWidth: "75%",
                                                alignSelf: isMine ? "flex-end" : "flex-start",
                                            }}
                                        >
                                            {!isMine && (
                                                <span style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "3px", marginLeft: "4px" }}>
                                                    {m.sender?.displayName || m.sender?.username}
                                                </span>
                                            )}

                                            <div
                                                style={{
                                                    padding: "10px 14px",
                                                    borderRadius: isMine ? "16px 16px 2px 16px" : "16px 16px 16px 2px",
                                                    backgroundColor: isMine ? "var(--accent-primary)" : "var(--bg-input)",
                                                    color: isMine ? "#fff" : "var(--text-primary)",
                                                    fontSize: "14px",
                                                    lineHeight: "1.5",
                                                    wordBreak: "break-word",
                                                    position: "relative",
                                                    boxShadow: "0 1px 3px rgba(0,0,0,0.1)",
                                                }}
                                            >
                                                {isEditing ? (
                                                    <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
                                                        <input
                                                            type="text"
                                                            value={editingContent}
                                                            onChange={(e) => setEditingContent(e.target.value)}
                                                            className="composer-textarea"
                                                            style={{
                                                                padding: "6px 8px",
                                                                borderRadius: "6px",
                                                                color: "#000",
                                                                backgroundColor: "#fff",
                                                                fontSize: "13px",
                                                            }}
                                                            autoFocus
                                                        />
                                                        <div style={{ display: "flex", justifyContent: "flex-end", gap: "4px" }}>
                                                            <button
                                                                type="button"
                                                                onClick={() => setEditingMessageId(null)}
                                                                style={{ fontSize: "11px", padding: "2px 6px" }}
                                                            >
                                                                취소
                                                            </button>
                                                            <button
                                                                type="button"
                                                                onClick={() => handleSaveEditMessage(m.id)}
                                                                style={{ fontSize: "11px", padding: "2px 6px", fontWeight: 700 }}
                                                            >
                                                                저장
                                                            </button>
                                                        </div>
                                                    </div>
                                                ) : (
                                                    m.content
                                                )}
                                            </div>

                                            {/* Meta & Actions */}
                                            <div style={{ display: "flex", alignItems: "center", gap: "6px", marginTop: "3px", fontSize: "11px", color: "var(--text-muted)" }}>
                                                <span>{formatMessageTime(m.createdAt)}</span>
                                                {isMine && (
                                                    <>
                                                        {m.isRead ? (
                                                            <span title="읽음" style={{ display: "inline-flex" }}>
                                                                <CheckCheck size={13} color="var(--accent-primary)" />
                                                            </span>
                                                        ) : (
                                                            <span title="전송됨" style={{ display: "inline-flex" }}>
                                                                <Check size={13} />
                                                            </span>
                                                        )}
                                                        <button
                                                            onClick={() => {
                                                                setEditingMessageId(m.id);
                                                                setEditingContent(m.content);
                                                            }}
                                                            title="수정"
                                                            style={{ color: "var(--text-muted)", marginLeft: "4px" }}
                                                        >
                                                            <Edit2 size={11} />
                                                        </button>
                                                        <button
                                                            onClick={() => handleDeleteMessage(m.id)}
                                                            title="삭제"
                                                            style={{ color: "var(--text-muted)" }}
                                                        >
                                                            <Trash2 size={11} />
                                                        </button>
                                                    </>
                                                )}
                                            </div>
                                        </div>
                                    );
                                })
                            )}

                            {/* Typing Indicator */}
                            {isOtherTyping && (
                                <div
                                    style={{
                                        alignSelf: "flex-start",
                                        padding: "6px 12px",
                                        borderRadius: "var(--radius-full)",
                                        backgroundColor: "var(--bg-input)",
                                        fontSize: "12px",
                                        color: "var(--text-muted)",
                                        display: "inline-flex",
                                        alignItems: "center",
                                        gap: "6px",
                                    }}
                                >
                                    <span style={{ fontStyle: "italic" }}>상대방이 입력 중입니다...</span>
                                    <span className="typing-dots">···</span>
                                </div>
                            )}

                            <div ref={messagesEndRef} />
                        </div>

                        {/* Input Footer */}
                        <form
                            onSubmit={handleSendMessage}
                            style={{
                                padding: "14px 18px",
                                borderTop: "1px solid var(--border-subtle)",
                                display: "flex",
                                gap: "10px",
                                alignItems: "center",
                                backgroundColor: "var(--bg-surface)",
                            }}
                        >
                            <input
                                type="text"
                                placeholder={selectedGroup ? "그룹 메시지 입력..." : `${selectedUser?.displayName || selectedUser?.username} 님에게 메시지 전송...`}
                                value={inputMessage}
                                onChange={(e) => handleInputChange(e.target.value)}
                                style={{
                                    flex: 1,
                                    padding: "10px 16px",
                                    borderRadius: "var(--radius-full)",
                                    backgroundColor: "var(--bg-input)",
                                    border: "1px solid var(--border-subtle)",
                                    color: "var(--text-primary)",
                                    fontSize: "14px",
                                    outline: "none",
                                }}
                            />
                            <button
                                type="submit"
                                disabled={!inputMessage.trim() || sending}
                                className="btn-primary"
                                style={{
                                    padding: "10px 18px",
                                    borderRadius: "var(--radius-full)",
                                    fontSize: "13px",
                                }}
                            >
                                <Send size={15} />
                            </button>
                        </form>
                    </>
                ) : (
                    <div style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", color: "var(--text-muted)", padding: "20px" }}>
                        <MessageSquare size={54} style={{ opacity: 0.3, marginBottom: "16px" }} />
                        <h3 style={{ fontSize: "17px", fontWeight: 700, color: "var(--text-primary)" }}>
                            대화를 선택하세요
                        </h3>
                        <p style={{ fontSize: "13px", marginTop: "6px" }}>
                            왼쪽 목록에서 1:1 대화 또는 그룹 채팅방을 선택하여 메시지를 주고받으세요.
                        </p>
                    </div>
                )}
            </div>

            {/* Modals */}
            {showNewMsgModal && (
                <NewMessageModal
                    onClose={() => setShowNewMsgModal(false)}
                    onSelectUser={(u) => {
                        setSelectedUser(u);
                        setSelectedGroup(null);
                        setChatMode("direct");
                        setShowNewMsgModal(false);
                        loadMessages(u, true);
                    }}
                />
            )}

            {showNewGroupModal && (
                <NewGroupModal
                    onClose={() => setShowNewGroupModal(false)}
                    onGroupCreated={(g) => {
                        setGroupConversations((prev) => [g, ...prev]);
                        setSelectedGroup(g);
                        setSelectedUser(null);
                        setChatMode("group");
                        loadGroupMessages(g);
                    }}
                />
            )}
        </div>
    );
}

export default function MessagesPage() {
    return (
        <Suspense fallback={<div style={{ padding: "40px", textAlign: "center" }}>Loading...</div>}>
            <MessagesContent />
        </Suspense>
    );
}
