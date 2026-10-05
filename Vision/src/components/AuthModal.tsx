"use client";

import React, { useState } from "react";
import { useAuth } from "@/lib/auth-context";
import { useToast } from "@/lib/toast-context";
import { formatErrorMessage } from "@/lib/i18n";
import { Check, Eye, EyeOff, Lock, LogIn, Mail, Sparkles, User as UserIcon, UserPlus, X } from "lucide-react";

interface AuthModalProps {
    onClose: () => void;
    initialMode?: "login" | "signup";
}

export default function AuthModal({ onClose, initialMode = "login" }: AuthModalProps) {
    const { login, signup } = useAuth();
    const { showToast } = useToast();

    const [mode, setMode] = useState<"login" | "signup">(initialMode);
    const [loading, setLoading] = useState(false);

    // Login form state
    const [loginEmail, setLoginEmail] = useState("");
    const [loginPassword, setLoginPassword] = useState("");
    const [showLoginPassword, setShowLoginPassword] = useState(false);

    // Signup form state
    const [signupUsername, setSignupUsername] = useState("");
    const [signupDisplayName, setSignupDisplayName] = useState("");
    const [signupEmail, setSignupEmail] = useState("");
    const [signupPassword, setSignupPassword] = useState("");
    const [signupConfirmPassword, setSignupConfirmPassword] = useState("");
    const [showSignupPassword, setShowSignupPassword] = useState(false);
    const [showSignupConfirmPassword, setShowSignupConfirmPassword] = useState(false);

    const handleLoginSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        const cleanEmail = loginEmail.trim().toLowerCase();
        if (!cleanEmail || !loginPassword) {
            showToast("이메일 주소와 비밀번호를 모두 입력해주세요.", "error");
            return;
        }

        setLoading(true);
        try {
            await login(cleanEmail, loginPassword);
            showToast("로그인되었습니다.", "success");
            onClose();
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLoading(false);
        }
    };

    const handleSignupSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        const cleanUsername = signupUsername.trim().toLowerCase();
        const cleanEmail = signupEmail.trim().toLowerCase();
        const cleanDisplayName = signupDisplayName.trim() || cleanUsername;

        if (!cleanUsername || !cleanEmail || !signupPassword || !cleanDisplayName) {
            showToast("필수 입력 항목을 모두 작성해주세요.", "error");
            return;
        }

        if (!/^[a-zA-Z0-9_]{3,30}$/.test(cleanUsername)) {
            showToast("아이디는 3~30자의 영문, 숫자, 밑줄(_)만 가능합니다.", "error");
            return;
        }

        if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(cleanEmail)) {
            showToast("유효한 이메일 형식을 입력해주세요.", "error");
            return;
        }

        if (signupPassword.length < 8) {
            showToast("비밀번호는 최소 8자 이상이어야 합니다.", "error");
            return;
        }

        if (signupPassword !== signupConfirmPassword) {
            showToast("비밀번호 확인이 일치하지 않습니다.", "error");
            return;
        }

        setLoading(true);
        try {
            await signup({
                username: cleanUsername,
                email: cleanEmail,
                password: signupPassword,
                displayName: cleanDisplayName,
            });

            showToast(`환영합니다! @${cleanUsername} 계정이 생성되었습니다.`, "success");
            onClose();
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="modal-overlay" onClick={onClose}>
            <div className="modal-card" onClick={(e) => e.stopPropagation()} style={{ maxWidth: "440px" }}>
                {/* Modal Header */}
                <div className="modal-header">
                    <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                        <div
                            className="logo-badge"
                            style={{
                                width: "36px",
                                height: "36px",
                                fontSize: "16px",
                                color: "#fff",
                                fontWeight: 900,
                                borderRadius: "8px",
                            }}
                        >
                            F
                        </div>
                        <div>
                            <h2 className="modal-title" style={{ fontSize: "18px" }}>
                                {mode === "login" ? "로그인" : "회원가입"}
                            </h2>
                            <p style={{ fontSize: "12px", color: "var(--text-muted)", marginTop: "2px" }}>
                                {mode === "login"
                                    ? "이메일 주소로 로그인하세요"
                                    : "새로운 계정을 생성하세요"}
                            </p>
                        </div>
                    </div>
                    <button onClick={onClose} className="modal-close-btn" aria-label="닫기">
                        <X size={20} />
                    </button>
                </div>

                <div className="modal-body">
                    {mode === "login" ? (
                        /* Login Mode */
                        <form onSubmit={handleLoginSubmit} style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
                            <div className="form-group" style={{ marginBottom: 0 }}>
                                <label className="form-label">
                                    <Mail size={13} style={{ display: "inline", marginRight: "4px" }} />
                                    이메일 주소
                                </label>
                                <input
                                    type="email"
                                    className="form-input"
                                    placeholder="name@example.com"
                                    value={loginEmail}
                                    onChange={(e) => setLoginEmail(e.target.value)}
                                    autoFocus
                                    required
                                />
                            </div>

                            <div className="form-group" style={{ marginBottom: 0 }}>
                                <label className="form-label">
                                    <Lock size={13} style={{ display: "inline", marginRight: "4px" }} />
                                    비밀번호
                                </label>
                                <div style={{ position: "relative" }}>
                                    <input
                                        type={showLoginPassword ? "text" : "password"}
                                        className="form-input"
                                        placeholder="비밀번호를 입력하세요"
                                        value={loginPassword}
                                        onChange={(e) => setLoginPassword(e.target.value)}
                                        style={{ paddingRight: "36px" }}
                                        required
                                    />
                                    <button
                                        type="button"
                                        onClick={() => setShowLoginPassword(!showLoginPassword)}
                                        style={{
                                            position: "absolute",
                                            right: 10,
                                            top: "50%",
                                            transform: "translateY(-50%)",
                                            color: "var(--text-muted)",
                                            background: "none",
                                            border: "none",
                                            cursor: "pointer",
                                        }}
                                        aria-label={showLoginPassword ? "비밀번호 숨기기" : "비밀번호 표시"}
                                    >
                                        {showLoginPassword ? <EyeOff size={15} /> : <Eye size={15} />}
                                    </button>
                                </div>
                            </div>

                            <button
                                type="submit"
                                disabled={loading || !loginEmail.trim() || !loginPassword}
                                className="btn-primary"
                                style={{ width: "100%", padding: "12px", marginTop: "8px", fontWeight: 700 }}
                            >
                                <LogIn size={16} />
                                {loading ? "로그인 중..." : "로그인"}
                            </button>

                            <div
                                style={{
                                    textAlign: "center",
                                    fontSize: "13px",
                                    color: "var(--text-secondary)",
                                    marginTop: "10px",
                                    paddingTop: "12px",
                                    borderTop: "1px solid var(--border-subtle)",
                                }}
                            >
                                계정이 아직 없으신가요?{" "}
                                <button
                                    type="button"
                                    onClick={() => setMode("signup")}
                                    style={{
                                        color: "var(--accent-primary)",
                                        fontWeight: 700,
                                        background: "none",
                                        border: "none",
                                        cursor: "pointer",
                                        padding: 0,
                                    }}
                                >
                                    회원가입
                                </button>
                            </div>
                        </form>
                    ) : (
                        /* Signup Mode */
                        <form onSubmit={handleSignupSubmit} style={{ display: "flex", flexDirection: "column", gap: "14px" }}>
                            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "10px" }}>
                                <div className="form-group" style={{ marginBottom: 0 }}>
                                    <label className="form-label">아이디 (핸들) *</label>
                                    <input
                                        type="text"
                                        className="form-input"
                                        placeholder="handle"
                                        value={signupUsername}
                                        onChange={(e) => setSignupUsername(e.target.value.toLowerCase().replace(/[^a-z0-9_]/g, ""))}
                                        maxLength={30}
                                        autoFocus
                                        required
                                    />
                                </div>

                                <div className="form-group" style={{ marginBottom: 0 }}>
                                    <label className="form-label">표시 이름 *</label>
                                    <input
                                        type="text"
                                        className="form-input"
                                        placeholder="홍길동"
                                        value={signupDisplayName}
                                        onChange={(e) => setSignupDisplayName(e.target.value)}
                                        maxLength={50}
                                        required
                                    />
                                </div>
                            </div>

                            <div className="form-group" style={{ marginBottom: 0 }}>
                                <label className="form-label">
                                    <Mail size={13} style={{ display: "inline", marginRight: "4px" }} />
                                    이메일 주소 *
                                </label>
                                <input
                                    type="email"
                                    className="form-input"
                                    placeholder="name@example.com"
                                    value={signupEmail}
                                    onChange={(e) => setSignupEmail(e.target.value)}
                                    required
                                />
                            </div>

                            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "10px" }}>
                                <div className="form-group" style={{ marginBottom: 0 }}>
                                    <label className="form-label">비밀번호 *</label>
                                    <div style={{ position: "relative" }}>
                                        <input
                                            type={showSignupPassword ? "text" : "password"}
                                            className="form-input"
                                            placeholder="8자 이상"
                                            value={signupPassword}
                                            onChange={(e) => setSignupPassword(e.target.value)}
                                            style={{ paddingRight: "30px" }}
                                            required
                                        />
                                        <button
                                            type="button"
                                            onClick={() => setShowSignupPassword(!showSignupPassword)}
                                            style={{
                                                position: "absolute",
                                                right: 8,
                                                top: "50%",
                                                transform: "translateY(-50%)",
                                                color: "var(--text-muted)",
                                                background: "none",
                                                border: "none",
                                                cursor: "pointer",
                                            }}
                                            aria-label={showSignupPassword ? "비밀번호 숨기기" : "비밀번호 표시"}
                                        >
                                            {showSignupPassword ? <EyeOff size={14} /> : <Eye size={14} />}
                                        </button>
                                    </div>
                                </div>

                                <div className="form-group" style={{ marginBottom: 0 }}>
                                    <label className="form-label">비밀번호 확인 *</label>
                                    <div style={{ position: "relative" }}>
                                        <input
                                            type={showSignupConfirmPassword ? "text" : "password"}
                                            className="form-input"
                                            placeholder="비밀번호 확인"
                                            value={signupConfirmPassword}
                                            onChange={(e) => setSignupConfirmPassword(e.target.value)}
                                            style={{ paddingRight: "30px" }}
                                            required
                                        />
                                        <button
                                            type="button"
                                            onClick={() => setShowSignupConfirmPassword(!showSignupConfirmPassword)}
                                            style={{
                                                position: "absolute",
                                                right: 8,
                                                top: "50%",
                                                transform: "translateY(-50%)",
                                                color: "var(--text-muted)",
                                                background: "none",
                                                border: "none",
                                                cursor: "pointer",
                                            }}
                                            aria-label={showSignupConfirmPassword ? "비밀번호 숨기기" : "비밀번호 표시"}
                                        >
                                            {showSignupConfirmPassword ? <EyeOff size={14} /> : <Eye size={14} />}
                                        </button>
                                    </div>
                                </div>
                            </div>

                            {signupConfirmPassword && signupPassword === signupConfirmPassword && (
                                <div style={{ fontSize: "11px", color: "#10b981", display: "flex", alignItems: "center", gap: "4px" }}>
                                    <Check size={12} /> 비밀번호가 일치합니다
                                </div>
                            )}

                            <button
                                type="submit"
                                disabled={
                                    loading ||
                                    !signupUsername.trim() ||
                                    !signupEmail.trim() ||
                                    !signupPassword ||
                                    signupPassword.length < 8 ||
                                    signupPassword !== signupConfirmPassword ||
                                    !signupDisplayName.trim()
                                }
                                className="btn-primary"
                                style={{ width: "100%", padding: "12px", marginTop: "4px", fontWeight: 700 }}
                            >
                                <UserPlus size={16} />
                                {loading ? "가입 처리 중..." : "회원가입"}
                            </button>

                            <div
                                style={{
                                    textAlign: "center",
                                    fontSize: "13px",
                                    color: "var(--text-secondary)",
                                    marginTop: "10px",
                                    paddingTop: "12px",
                                    borderTop: "1px solid var(--border-subtle)",
                                }}
                            >
                                이미 계정이 있으신가요?{" "}
                                <button
                                    type="button"
                                    onClick={() => setMode("login")}
                                    style={{
                                        color: "var(--accent-primary)",
                                        fontWeight: 700,
                                        background: "none",
                                        border: "none",
                                        cursor: "pointer",
                                        padding: 0,
                                    }}
                                >
                                    로그인
                                </button>
                            </div>
                        </form>
                    )}
                </div>
            </div>
        </div>
    );
}
