"use client";

import React, { useEffect, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth-context";
import { useToast } from "@/lib/toast-context";
import { formatErrorMessage } from "@/lib/i18n";
import { Eye, EyeOff, Lock, LogIn, Mail } from "lucide-react";

export default function LoginPage() {
    const router = useRouter();
    const { user, login } = useAuth();
    const { showToast } = useToast();

    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [showPassword, setShowPassword] = useState(false);
    const [loading, setLoading] = useState(false);

    useEffect(() => {
        if (user) {
            router.push("/");
        }
    }, [user, router]);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        const cleanEmail = email.trim().toLowerCase();
        if (!cleanEmail || !password) {
            showToast("이메일 주소와 비밀번호를 모두 입력해주세요.", "error");
            return;
        }

        setLoading(true);
        try {
            await login(cleanEmail, password);
            showToast("로그인되었습니다.", "success");
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
                    maxWidth: "420px",
                    backgroundColor: "var(--bg-card)",
                    borderRadius: "var(--radius-lg)",
                    border: "1px solid var(--border-subtle)",
                    padding: "36px 30px",
                    boxShadow: "var(--shadow-lg)",
                }}
            >
                {/* Brand Header */}
                <div style={{ textAlign: "center", marginBottom: "28px" }}>
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
                        로그인
                    </h1>
                    <p style={{ fontSize: "13px", color: "var(--text-muted)", marginTop: "6px", marginBottom: 0 }}>
                        이메일 주소로 로그인하세요
                    </p>
                </div>

                {/* Form */}
                <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
                    <div>
                        <label
                            style={{
                                display: "block",
                                fontSize: "13px",
                                fontWeight: 600,
                                color: "var(--text-secondary)",
                                marginBottom: "6px",
                            }}
                        >
                            이메일 주소
                        </label>
                        <div style={{ position: "relative", display: "flex", alignItems: "center" }}>
                            <Mail
                                size={16}
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
                                style={{ width: "100%", paddingLeft: "36px" }}
                                autoFocus
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
                                marginBottom: "6px",
                            }}
                        >
                            비밀번호
                        </label>
                        <div style={{ position: "relative", display: "flex", alignItems: "center" }}>
                            <Lock
                                size={16}
                                style={{
                                    position: "absolute",
                                    left: "12px",
                                    color: "var(--text-muted)",
                                    pointerEvents: "none",
                                }}
                            />
                            <input
                                type={showPassword ? "text" : "password"}
                                placeholder="비밀번호 입력"
                                value={password}
                                onChange={(e) => setPassword(e.target.value)}
                                className="form-input"
                                style={{ width: "100%", paddingLeft: "36px", paddingRight: "38px" }}
                                required
                            />
                            <button
                                type="button"
                                onClick={() => setShowPassword(!showPassword)}
                                style={{
                                    position: "absolute",
                                    right: "12px",
                                    background: "none",
                                    border: "none",
                                    color: "var(--text-muted)",
                                    cursor: "pointer",
                                    padding: 0,
                                }}
                                aria-label={showPassword ? "비밀번호 숨기기" : "비밀번호 표시"}
                            >
                                {showPassword ? <EyeOff size={16} /> : <Eye size={16} />}
                            </button>
                        </div>
                    </div>

                    <button
                        type="submit"
                        disabled={loading || !email.trim() || !password}
                        className="btn-primary"
                        style={{
                            width: "100%",
                            padding: "12px",
                            marginTop: "8px",
                            justifyContent: "center",
                            fontSize: "14px",
                            fontWeight: 700,
                        }}
                    >
                        <LogIn size={16} />
                        {loading ? "로그인 중..." : "로그인"}
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
                    계정이 없으신가요?{" "}
                    <Link
                        href="/signup"
                        style={{
                            color: "var(--accent-primary)",
                            fontWeight: 700,
                            textDecoration: "none",
                        }}
                    >
                        회원가입
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
