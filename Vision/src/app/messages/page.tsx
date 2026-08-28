"use client";

import React, {Suspense, useCallback, useEffect, useRef, useState} from "react";
import Link from "next/link";
import {useSearchParams} from "next/navigation";
import {Conversation, DirectMessage, User} from "@/lib/types";
import {useAuth} from "@/lib/auth-context";
import {fetchGraphQL, MUTATIONS, QUERIES} from "@/lib/graphql";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import NewMessageModal from "@/components/NewMessageModal";
import {ArrowLeft, Check, CheckCheck, Edit, MessageSquare, Search, Send, Sparkles,} from "lucide-react";

function MessagesContent() {
    const {user} = useAuth();
    const {showToast} = useToast();
    const searchParams = useSearchParams();
    const targetUsernameParam = searchParams.get("user");

    const [conversations, setConversations] = useState<Conversation[]>([]);
    const [loadingConversations, setLoadingConversations] = useState(true);
    const [selectedUser, setSelectedUser] = useState<User | null>(null);
    const [messages, setMessages] = useState<DirectMessage[]>([]);
    const [loadingMessages, setLoadingMessages] = useState(false);
    const [inputMessage, setInputMessage] = useState("");
    const [sending, setSending] = useState(false);
    const [showNewMsgModal, setShowNewMsgModal] = useState(false);
    const [filterQuery, setFilterQuery] = useState("");

    const messagesEndRef = useRef<HTMLDivElement | null>(null);

    const scrollToBottom = () => {
        messagesEndRef.current?.scrollIntoView({behavior: "smooth"});
    };

    // Load conversations list
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

    // Load chat messages with selected user
    const loadMessages = useCallback(
        async (otherUser: User, markRead = true) => {
            if (!user) return;
            try {
                const data = await fetchGraphQL<{ directMessages: DirectMessage[] }>(
                    QUERIES.DIRECT_MESSAGES,
                    {otherUserId: otherUser.id}
                );
                if (data?.directMessages) {
                    setMessages(data.directMessages);
                }

                if (markRead) {
                    await fetchGraphQL(MUTATIONS.MARK_MESSAGES_AS_READ, {
                        otherUserId: otherUser.id,
                    });
                    // update local conversation unread count
                    setConversations((prev) =>
                        prev.map((c) =>
                            c.otherUser.id === otherUser.id ? {...c, unreadCount: 0} : c
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

    // Initialize selected user from URL query param if present
    useEffect(() => {
        if (targetUsernameParam && user) {
            const initTarget = async () => {
                try {
                    const data = await fetchGraphQL<{ profile: User }>(QUERIES.USER_PROFILE, {
                        username: targetUsernameParam,
                    });
                    if (data?.profile) {
                        setSelectedUser(data.profile);
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

    useEffect(() => {
        loadConversations();
        const interval = setInterval(loadConversations, 5000);
        return () => clearInterval(interval);
    }, [loadConversations]);

    // Poll messages every 3 seconds for active conversation
    useEffect(() => {
        if (!selectedUser) return;
        const interval = setInterval(() => {
            loadMessages(selectedUser, true);
        }, 3000);
        return () => clearInterval(interval);
    }, [selectedUser, loadMessages]);

    useEffect(() => {
        scrollToBottom();
    }, [messages]);

    const handleSelectConversation = (otherUser: User) => {
        setSelectedUser(otherUser);
        setLoadingMessages(true);
        loadMessages(otherUser, true);
    };

    const handleSendMessage = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!inputMessage.trim() || !selectedUser || sending) return;

        const content = inputMessage.trim();
        setInputMessage("");
        setSending(true);

        // Optimistic message
        const tempMsg: DirectMessage = {
            id: "temp-" + Date.now(),
            sender: user!,
            recipient: selectedUser,
            content,
            isRead: false,
            isMine: true,
            createdAt: new Date().toISOString(),
        };
        setMessages((prev) => [...prev, tempMsg]);

        try {
            const data = await fetchGraphQL<{ sendDirectMessage: DirectMessage }>(
                MUTATIONS.SEND_DIRECT_MESSAGE,
                {
                    recipientId: selectedUser.id,
                    content,
                }
            );
            if (data?.sendDirectMessage) {
                setMessages((prev) =>
                    prev.map((m) => (m.id === tempMsg.id ? data.sendDirectMessage : m))
                );
                loadConversations();
            }
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
            setMessages((prev) => prev.filter((m) => m.id !== tempMsg.id));
        } finally {
            setSending(false);
        }
    };

    const formatTime = (iso: string) => {
        const d = new Date(iso);
        return d.toLocaleTimeString("ko-KR", {hour: "2-digit", minute: "2-digit", hour12: false});
    };

    const defaultAvatar = "https://api.dicebear.com/7.x/bottts/svg?seed=user";

    const filteredConversations = conversations.filter((c) => {
        const q = filterQuery.toLowerCase();
        return (
            c.otherUser.username.toLowerCase().includes(q) ||
            (c.otherUser.displayName && c.otherUser.displayName.toLowerCase().includes(q))
        );
    });

    if (!user) {
        return (
            <div
                style={{
                    padding: "80px 20px",
                    textAlign: "center",
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    gap: "16px",
                }}
            >
                <div
                    style={{
                        width: 64,
                        height: 64,
                        borderRadius: "50%",
                        backgroundColor: "var(--bg-surface)",
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "center",
                        color: "var(--accent-primary)",
                        boxShadow: "var(--shadow-glow)",
                    }}
                >
                    <MessageSquare size={32}/>
                </div>
                <h2 style={{fontSize: "20px", fontWeight: 800}}>로그인이 필요한 서비스입니다</h2>
                <p style={{fontSize: "14px", color: "var(--text-secondary)", maxWidth: "360px"}}>
                    1:1 실시간 다이렉트 메시지를 주고받으려면 먼저 로그인하거나 가입해주세요.
                </p>
                <Link href="/welcome?tab=login" className="btn-primary" style={{marginTop: "8px"}}>
                    로그인 및 가입하기
                </Link>
            </div>
        );
    }

    return (
        <>
            <div className={`messages-layout ${selectedUser ? "chat-open" : ""}`}>
                {/* Left Pane: Conversations List */}
                <div className="conversations-pane">
                    {/* Header */}
                    <div
                        style={{
                            padding: "16px 20px",
                            borderBottom: "1px solid var(--border-subtle)",
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "space-between",
                            backgroundColor: "rgba(11, 15, 25, 0.8)",
                            backdropFilter: "blur(12px)",
                        }}
                    >
                        <h2 style={{fontSize: "18px", fontWeight: 800, color: "var(--text-primary)"}}>
                            메시지
                        </h2>
                        <button
                            onClick={() => setShowNewMsgModal(true)}
                            style={{
                                color: "var(--accent-primary)",
                                padding: "6px",
                                borderRadius: "50%",
                                backgroundColor: "rgba(56, 189, 248, 0.1)",
                            }}
                            title="새 메시지 작성"
                        >
                            <Edit size={16}/>
                        </button>
                    </div>

                    {/* Search Filter */}
                    <div style={{padding: "10px 16px", borderBottom: "1px solid var(--border-subtle)"}}>
                        <div style={{position: "relative", display: "flex", alignItems: "center"}}>
                            <Search
                                size={14}
                                style={{position: "absolute", left: 10, color: "var(--text-muted)"}}
                            />
                            <input
                                type="text"
                                placeholder="대화 검색..."
                                value={filterQuery}
                                onChange={(e) => setFilterQuery(e.target.value)}
                                style={{
                                    width: "100%",
                                    padding: "8px 12px 8px 32px",
                                    borderRadius: "var(--radius-full)",
                                    backgroundColor: "var(--bg-input)",
                                    border: "1px solid var(--border-subtle)",
                                    fontSize: "13px",
                                    color: "var(--text-primary)",
                                    outline: "none",
                                }}
                            />
                        </div>
                    </div>

                    {/* Conversations Items */}
                    <div style={{flex: 1, overflowY: "auto"}}>
                        {loadingConversations && conversations.length === 0 ? (
                            <div style={{
                                padding: "30px",
                                textAlign: "center",
                                color: "var(--text-muted)",
                                fontSize: "13px"
                            }}>
                                대화 목록을 불러오는 중...
                            </div>
                        ) : filteredConversations.length === 0 ? (
                            <div style={{padding: "40px 20px", textAlign: "center", color: "var(--text-muted)"}}>
                                <MessageSquare size={32} style={{margin: "0 auto 10px", opacity: 0.5}}/>
                                <div style={{fontSize: "14px", fontWeight: 700, color: "var(--text-primary)"}}>
                                    대화 내역이 없습니다
                                </div>
                                <div style={{fontSize: "12px", marginTop: "4px"}}>
                                    우측 상단 아이콘을 눌러 새 대화를 시작해보세요.
                                </div>
                            </div>
                        ) : (
                            filteredConversations.map((c) => {
                                const isActive = selectedUser?.id === c.otherUser.id;
                                return (
                                    <div
                                        key={c.otherUser.id}
                                        onClick={() => handleSelectConversation(c.otherUser)}
                                        className={`conversation-item ${isActive ? "active" : ""}`}
                                    >
                                        <div style={{position: "relative"}}>
                                            <img
                                                src={c.otherUser.avatarUrl || defaultAvatar}
                                                alt={c.otherUser.username}
                                                style={{width: 44, height: 44, borderRadius: "50%", objectFit: "cover"}}
                                            />
                                        </div>
                                        <div style={{flex: 1, minWidth: 0}}>
                                            <div style={{
                                                display: "flex",
                                                justifyContent: "space-between",
                                                alignItems: "baseline"
                                            }}>
                        <span
                            style={{
                                fontWeight: 700,
                                fontSize: "14px",
                                color: "var(--text-primary)",
                                whiteSpace: "nowrap",
                                overflow: "hidden",
                                textOverflow: "ellipsis",
                            }}
                        >
                          {c.otherUser.displayName || c.otherUser.username}
                        </span>
                                                {c.lastMessage && (
                                                    <span style={{fontSize: "11px", color: "var(--text-muted)"}}>
                            {formatTime(c.lastMessage.createdAt)}
                          </span>
                                                )}
                                            </div>
                                            <div style={{
                                                display: "flex",
                                                justifyContent: "space-between",
                                                alignItems: "center",
                                                marginTop: "2px"
                                            }}>
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
                          {c.lastMessage?.content || "새로운 대화"}
                        </span>
                                                {c.unreadCount > 0 && (
                                                    <span className="badge-count"
                                                          style={{fontSize: "10px", padding: "1px 6px"}}>
                            {c.unreadCount}
                          </span>
                                                )}
                                            </div>
                                        </div>
                                    </div>
                                );
                            })
                        )}
                    </div>
                </div>

                {/* Right Pane: Active Chat Room */}
                <div className="chat-pane">
                    {selectedUser ? (
                        <>
                            {/* Chat Header */}
                            <div
                                style={{
                                    padding: "14px 20px",
                                    borderBottom: "1px solid var(--border-subtle)",
                                    display: "flex",
                                    alignItems: "center",
                                    justifyContent: "space-between",
                                    backgroundColor: "rgba(11, 15, 25, 0.8)",
                                    backdropFilter: "blur(12px)",
                                }}
                            >
                                <div style={{display: "flex", alignItems: "center", gap: "12px"}}>
                                    <button
                                        onClick={() => setSelectedUser(null)}
                                        style={{color: "var(--text-muted)", padding: "4px", display: "none"}}
                                        className="mobile-back-btn"
                                    >
                                        <ArrowLeft size={20}/>
                                    </button>
                                    <Link href={`/profile/${selectedUser.username}`}
                                          style={{display: "flex", alignItems: "center", gap: "10px"}}>
                                        <img
                                            src={selectedUser.avatarUrl || defaultAvatar}
                                            alt={selectedUser.username}
                                            style={{width: 38, height: 38, borderRadius: "50%", objectFit: "cover"}}
                                        />
                                        <div>
                                            <div style={{
                                                fontWeight: 800,
                                                fontSize: "15px",
                                                color: "var(--text-primary)"
                                            }}>
                                                {selectedUser.displayName || selectedUser.username}
                                            </div>
                                            <div style={{
                                                fontSize: "11px",
                                                color: "var(--accent-success)",
                                                display: "flex",
                                                alignItems: "center",
                                                gap: "4px"
                                            }}>
                                                <span style={{
                                                    width: 6,
                                                    height: 6,
                                                    borderRadius: "50%",
                                                    backgroundColor: "var(--accent-success)"
                                                }}/>
                                                온라인 · @{selectedUser.username}
                                            </div>
                                        </div>
                                    </Link>
                                </div>
                            </div>

                            {/* Chat Messages Body */}
                            <div style={{
                                flex: 1,
                                overflowY: "auto",
                                padding: "20px",
                                display: "flex",
                                flexDirection: "column",
                                gap: "12px"
                            }}>
                                {loadingMessages ? (
                                    <div style={{
                                        padding: "40px",
                                        textAlign: "center",
                                        color: "var(--text-muted)",
                                        fontSize: "13px"
                                    }}>
                                        메시지 기록을 불러오는 중...
                                    </div>
                                ) : messages.length === 0 ? (
                                    <div style={{margin: "auto", textAlign: "center", color: "var(--text-muted)"}}>
                                        <Sparkles size={36} color="var(--accent-primary)"
                                                  style={{margin: "0 auto 12px"}}/>
                                        <div style={{fontSize: "16px", fontWeight: 800, color: "var(--text-primary)"}}>
                                            {selectedUser.displayName || selectedUser.username} 님과의 대화
                                        </div>
                                        <p style={{fontSize: "13px", marginTop: "4px"}}>
                                            첫 메시지를 보내서 인사를 나눠보세요! 💬
                                        </p>
                                    </div>
                                ) : (
                                    messages.map((m) => {
                                        const isMine = m.isMine ?? (m.sender?.id ? m.sender.id === user.id : m.sender?.username === user.username);
                                        return (
                                            <div
                                                key={m.id}
                                                style={{
                                                    display: "flex",
                                                    flexDirection: "column",
                                                    alignSelf: isMine ? "flex-end" : "flex-start",
                                                    maxWidth: "72%",
                                                }}
                                            >
                                                <div className={`chat-bubble ${isMine ? "mine" : "theirs"}`}>
                                                    {m.content}
                                                </div>
                                                <div
                                                    style={{
                                                        display: "flex",
                                                        alignItems: "center",
                                                        gap: "4px",
                                                        fontSize: "10px",
                                                        color: "var(--text-muted)",
                                                        marginTop: "3px",
                                                        alignSelf: isMine ? "flex-end" : "flex-start",
                                                        padding: "0 4px",
                                                    }}
                                                >
                                                    <span>{formatTime(m.createdAt)}</span>
                                                    {isMine && (
                                                        <span title={m.isRead ? "읽음" : "전송됨"}>
                              {m.isRead ? (
                                  <CheckCheck size={13} color="var(--accent-primary)"/>
                              ) : (
                                  <Check size={13} color="var(--text-muted)"/>
                              )}
                            </span>
                                                    )}
                                                </div>
                                            </div>
                                        );
                                    })
                                )}
                                <div ref={messagesEndRef}/>
                            </div>

                            {/* Chat Input Bar */}
                            <form
                                onSubmit={handleSendMessage}
                                style={{
                                    padding: "14px 20px",
                                    borderTop: "1px solid var(--border-subtle)",
                                    backgroundColor: "rgba(11, 15, 25, 0.8)",
                                    display: "flex",
                                    gap: "10px",
                                    alignItems: "center",
                                }}
                            >
                                <input
                                    type="text"
                                    placeholder={`${selectedUser.displayName || selectedUser.username} 님에게 보낼 메시지...`}
                                    value={inputMessage}
                                    onChange={(e) => setInputMessage(e.target.value)}
                                    style={{
                                        flex: 1,
                                        padding: "11px 18px",
                                        borderRadius: "var(--radius-full)",
                                        backgroundColor: "var(--bg-input)",
                                        border: "1px solid var(--border-subtle)",
                                        fontSize: "14px",
                                        color: "var(--text-primary)",
                                        outline: "none",
                                    }}
                                    autoFocus
                                />
                                <button
                                    type="submit"
                                    disabled={!inputMessage.trim() || sending}
                                    className="btn-primary"
                                    style={{padding: "11px 20px"}}
                                >
                                    <Send size={15}/>
                                </button>
                            </form>
                        </>
                    ) : (
                        <div
                            style={{
                                margin: "auto",
                                textAlign: "center",
                                display: "flex",
                                flexDirection: "column",
                                alignItems: "center",
                                gap: "14px",
                                padding: "20px",
                            }}
                        >
                            <div
                                style={{
                                    width: 64,
                                    height: 64,
                                    borderRadius: "50%",
                                    backgroundColor: "var(--bg-surface)",
                                    display: "flex",
                                    alignItems: "center",
                                    justifyContent: "center",
                                    color: "var(--accent-primary)",
                                    boxShadow: "var(--shadow-glow)",
                                }}
                            >
                                <MessageSquare size={30}/>
                            </div>
                            <h3 style={{fontSize: "18px", fontWeight: 800, color: "var(--text-primary)"}}>
                                대화를 선택하세요
                            </h3>
                            <p style={{fontSize: "13px", color: "var(--text-secondary)", maxWidth: "300px"}}>
                                왼쪽 목록에서 대화 상대를 선택하거나 새 메시지를 시작해보세요.
                            </p>
                            <button
                                onClick={() => setShowNewMsgModal(true)}
                                className="btn-secondary"
                                style={{marginTop: "4px"}}
                            >
                                <Edit size={14}/> 새 메시지 작성
                            </button>
                        </div>
                    )}
                </div>
            </div>

            {showNewMsgModal && (
                <NewMessageModal
                    onClose={() => setShowNewMsgModal(false)}
                    onSelectUser={(user) => {
                        setShowNewMsgModal(false);
                        handleSelectConversation(user);
                    }}
                />
            )}
        </>
    );
}

export default function MessagesPage() {
    return (
        <Suspense fallback={<div style={{padding: "40px", textAlign: "center"}}>Loading...</div>}>
            <MessagesContent/>
        </Suspense>
    );
}
