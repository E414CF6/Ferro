"use client";

import React, { useEffect, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth-context";
import { useToast } from "@/lib/toast-context";
import { formatErrorMessage } from "@/lib/i18n";
import { Check, Eye, EyeOff, Lock, Mail, Sparkles, User as UserIcon, UserPlus } from "lucide-react";

export default function SignupPage() {
    const router = useRouter();
    const { user, signup } = useAuth();
    const { showToast } = useToast();

    const [username, setUsername] = useState("");
    const [displayName, setDisplayName] = useState("");
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [confirmPassword, setConfirmPassword] = useState("");
    const [showPassword, setShowPassword] = useState(false);
    const [showConfirmPassword, setShowConfirmPassword] = useState(false);
    const [loading, setLoading] = useState(false);

    useEffect(() => {
        if (user) {
            router.push("/");
        }
    }, [user, router]);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        const cleanUsername = username.trim().toLowerCase();
        const cleanEmail = email.trim().toLowerCase();
        const cleanDisplayName = displayName.trim() || cleanUsername;

        if (!cleanUsername || !cleanEmail || !password) {
            showToast("필수 항목을 모두 작성해주세요.", "error");
            return;
        }

        if (!/^[a-zA-Z0-9_]{3,30}$/.test(cleanUsername)) {
            showToast("아이디는 3~30자의 영문, 숫자, 밑줄(_)만 가능합니다.", "error");
            return;
        }

        if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(cleanEmail)) {
            showToast("유효한 이메일 주소를 입력해주세요.", "error");
            return;
        }

        if (password.length < 8) {
            showToast("비밀번호는 최소 8자 이상이어야 합니다.", "error");
            return;
        }

        if (password !== confirmPassword) {
            showToast("비밀번호 확인이 일치하지 않습니다.", "error");
            return;
        }

        setLoading(true);
        try {
            await signup({
                username: cleanUsername,
                email: cleanEmail,
                password,
                displayName: cleanDisplayName,
            });
            showToast(`환영합니다! @${cleanUsername} 계정이 생성되었습니다.`, "success");
            router.push("/");
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLoading(false);
        }
    };

    return (
        <div
            style={{
                minHeight: "100vh",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                padding: "20px",
                backgroundColor: "var(--bg-primary)",
            }}
        >
            <div
                style={{
                    width: "100%",
                    maxWidth: "460px",
                    backgroundColor: "var(--bg-card)",
                    borderRadius: "var(--radius-lg)",
                    border: "1px solid var(--border-subtle)",
                    padding: "36px 30px",
                    boxShadow: "var(--shadow-lg)",
                }}
            >
                {/* Brand Header */}
                <div style={{ textAlign: "center", marginBottom: "26px" }}>
                    <Link
                        href="/"
                        style={{
                            display: "inline-flex",
                            alignItems: "center",
                            gap: "10px",
                            textDecoration: "none",
                            marginBottom: "16px",
                        }}
                    >
                        <div
                            className="logo-badge"
                            style={{
                                width: "40px",
                                height: "40px",
                                fontSize: "20px",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                                borderRadius: "10px",
                                background: "linear-gradient(135deg, rgba(56, 189, 248, 0.2), rgba(168, 85, 247, 0.2))",
                                border: "1px solid var(--border-subtle)",
                                color: "#fff",
                                fontWeight: 900,
                            }}
                        >
                            F
                        </div>
                        <span
                            style={{
                                fontSize: "24px",
                                fontWeight: 800,
                                color: "var(--text-primary)",
                                letterSpacing: "-0.5px",
                            }}
                        >
                            Ferro
                        </span>
                    </Link>
                    <h1 style={{ fontSize: "20px", fontWeight: 700, color: "var(--text-primary)", margin: 0 }}>
                        회원가입
                    </h1>
                    <p style={{ fontSize: "13px", color: "var(--text-muted)", marginTop: "6px", marginBottom: 0 }}>
                        새로운 계정을 생성하세요
                    </p>
                </div>

                {/* Form */}
                <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "14px" }}>
                    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "10px" }}>
                        <div>
                            <label
                                style={{
                                    display: "block",
                                    fontSize: "13px",
                                    fontWeight: 600,
                                    color: "var(--text-secondary)",
                                    marginBottom: "4px",
                                }}
                            >
                                아이디 (핸들) *
                            </label>
                            <input
                                type="text"
                                placeholder="handle"
                                value={username}
                                onChange={(e) => setUsername(e.target.value.toLowerCase().replace(/[^a-z0-9_]/g, ""))}
                                maxLength={30}
                                className="form-input"
                                required
                                autoFocus
                            />
                        </div>

                        <div>
                            <label
                                style={{
                                    display: "block",
                                    fontSize: "13px",
                                    fontWeight: 600,
                                    color: "var(--text-secondary)",
                                    marginBottom: "4px",
                                }}
                            >
                                이름 (닉네임) *
                            </label>
                            <input
                                type="text"
                                placeholder="홍길동"
                                value={displayName}
                                onChange={(e) => setDisplayName(e.target.value)}
                                maxLength={50}
                                className="form-input"
                                required
                            />
                        </div>
                    </div>

                    <div>
                        <label
                            style={{
                                display: "block",
                                fontSize: "13px",
                                fontWeight: 600,
                                color: "var(--text-secondary)",
                                marginBottom: "4px",
                            }}
                        >
                            이메일 주소 *
                        </label>
                        <div style={{ position: "relative", display: "flex", alignItems: "center" }}>
                            <Mail
                                size={15}
                                style={{
                                    position: "absolute",
                                    left: "12px",
                                    color: "var(--text-muted)",
                                    pointerEvents: "none",
                                }}
                            />
                            <input
                                type="email"
                                placeholder="name@example.com"
                                value={email}
                                onChange={(e) => setEmail(e.target.value)}
                                className="form-input"
                                style={{ width: "100%", paddingLeft: "34px" }}
                                required
                            />
                        </div>
                    </div>

                    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "10px" }}>
                        <div>
                            <label
                                style={{
                                    display: "block",
                                    fontSize: "13px",
                                    fontWeight: 600,
                                    color: "var(--text-secondary)",
                                    marginBottom: "4px",
                                }}
                            >
                                비밀번호 *
                            </label>
                            <div style={{ position: "relative", display: "flex", alignItems: "center" }}>
                                <input
                                    type={showPassword ? "text" : "password"}
                                    placeholder="8자 이상"
                                    value={password}
                                    onChange={(e) => setPassword(e.target.value)}
                                    className="form-input"
                                    style={{ width: "100%", paddingRight: "34px" }}
                                    required
                                />
                                <button
                                    type="button"
                                    onClick={() => setShowPassword(!showPassword)}
                                    style={{
                                        position: "absolute",
                                        right: "10px",
                                        background: "none",
                                        border: "none",
                                        color: "var(--text-muted)",
                                        cursor: "pointer",
                                        padding: 0,
                                    }}
                                    aria-label={showPassword ? "비밀번호 숨기기" : "비밀번호 표시"}
                                >
                                    {showPassword ? <EyeOff size={15} /> : <Eye size={15} />}
                                </button>
                            </div>
                        </div>

                        <div>
                            <label
                                style={{
                                    display: "block",
                                    fontSize: "13px",
                                    fontWeight: 600,
                                    color: "var(--text-secondary)",
                                    marginBottom: "4px",
                                }}
                            >
                                비밀번호 확인 *
                            </label>
                            <div style={{ position: "relative", display: "flex", alignItems: "center" }}>
                                <input
                                    type={showConfirmPassword ? "text" : "password"}
                                    placeholder="비밀번호 확인"
                                    value={confirmPassword}
                                    onChange={(e) => setConfirmPassword(e.target.value)}
                                    className="form-input"
                                    style={{ width: "100%", paddingRight: "34px" }}
                                    required
                                />
                                <button
                                    type="button"
                                    onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                                    style={{
                                        position: "absolute",
                                        right: "10px",
                                        background: "none",
                                        border: "none",
                                        color: "var(--text-muted)",
                                        cursor: "pointer",
                                        padding: 0,
                                    }}
                                    aria-label={showConfirmPassword ? "비밀번호 숨기기" : "비밀번호 표시"}
                                >
                                    {showConfirmPassword ? <EyeOff size={15} /> : <Eye size={15} />}
                                </button>
                            </div>
                        </div>
                    </div>

                    {confirmPassword && password === confirmPassword && (
                        <div style={{ fontSize: "11px", color: "#10b981", display: "flex", alignItems: "center", gap: "4px" }}>
                            <Check size={13} /> 비밀번호가 일치합니다
                        </div>
                    )}

                    <button
                        type="submit"
                        disabled={loading || !username.trim() || !email.trim() || !password || password.length < 8 || password !== confirmPassword}
                        className="btn-primary"
                        style={{
                            width: "100%",
                            padding: "12px",
                            marginTop: "6px",
                            justifyContent: "center",
                            fontSize: "14px",
                            fontWeight: 700,
                        }}
                    >
                        <UserPlus size={16} />
                        {loading ? "가입 처리 중..." : "회원가입"}
                    </button>
                </form>

                {/* Footer Switch */}
                <div
                    style={{
                        marginTop: "24px",
                        paddingTop: "20px",
                        borderTop: "1px solid var(--border-subtle)",
                        textAlign: "center",
                        fontSize: "13px",
                        color: "var(--text-secondary)",
                    }}
                >
                    이미 계정이 있으신가요?{" "}
                    <Link
                        href="/login"
                        style={{
                            color: "var(--accent-primary)",
                            fontWeight: 700,
                            textDecoration: "none",
                        }}
                    >
                        로그인
                    </Link>
                    <div style={{ marginTop: "12px" }}>
                        <Link
                            href="/"
                            style={{
                                color: "var(--text-muted)",
                                fontSize: "12px",
                                textDecoration: "none",
                            }}
                        >
                            ← 피드로 돌아가기
                        </Link>
                    </div>
                </div>
            </div>
        </div>
    );
}
