"use client";

import React, {useState} from "react";
import {useAuth} from "@/lib/auth-context";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import {ArrowLeft, Eye, EyeOff, Loader2, X} from "lucide-react";

interface AuthModalProps {
    onClose: () => void;
    initialMode?: "login" | "signup";
}

export default function AuthModal({onClose}: AuthModalProps) {
    const {login, signup, googleLogin} = useAuth();
    const { showToast } = useToast();

    const [step, setStep] = useState<1 | 2>(1);
    const [identifier, setIdentifier] = useState("");
    const [password, setPassword] = useState("");
    const [displayName, setDisplayName] = useState("");
    const [isNewUser, setIsNewUser] = useState(false);
    const [showPassword, setShowPassword] = useState(false);
    const [loading, setLoading] = useState(false);

    // Google modal state
    const [showGooglePrompt, setShowGooglePrompt] = useState(false);
    const [googleEmail, setGoogleEmail] = useState("");
    const [googleName, setGoogleName] = useState("");
    const [googleLoading, setGoogleLoading] = useState(false);

    const handleProceed = (e: React.FormEvent) => {
        e.preventDefault();
        const cleanId = identifier.trim().toLowerCase();
        if (!cleanId) return;
        setDisplayName(cleanId.includes("@") ? cleanId.split("@")[0] : cleanId);
        setStep(2);
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        const cleanId = identifier.trim().toLowerCase();
        if (!cleanId || !password) return;

        setLoading(true);
        try {
            try {
                await login(cleanId, password);
                showToast("로그인되었습니다.", "success");
                onClose();
                return;
            } catch (loginErr: any) {
                const errMsg = String(loginErr?.message || "");
                const isNotFound =
                    errMsg.toLowerCase().includes("not found") ||
                    errMsg.toLowerCase().includes("usernotfound") ||
                    errMsg.includes("사용자") ||
                    isNewUser;

                if (isNotFound) {
                    const isEmail = cleanId.includes("@");
                    const username = isEmail
                        ? cleanId.split("@")[0].replace(/[^a-zA-Z0-9_]/g, "_").slice(0, 24)
                        : cleanId;
                    const email = isEmail ? cleanId : `${cleanId}@ferro.app`;
                    const name = displayName.trim() || username;

                    await signup({
                        username: username.length >= 3 ? username : `user_${username}`,
                        email,
                        password,
                        displayName: name,
                    });
                    showToast("환영합니다! 계정이 생성되었습니다.", "success");
                    onClose();
                    return;
                }

                showToast(formatErrorMessage(loginErr, "ko"), "error");
            }
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLoading(false);
        }
    };

    const handleGoogleAuth = async (emailToUse?: string, nameToUse?: string) => {
        const targetEmail = (emailToUse || googleEmail).trim().toLowerCase();
        const targetName = (nameToUse || googleName || targetEmail.split("@")[0]).trim();

        if (!targetEmail || !targetEmail.includes("@")) {
            showToast("올바른 Google 이메일을 입력해주세요.", "error");
            return;
        }

        setGoogleLoading(true);
        try {
            await googleLogin({
                email: targetEmail,
                name: targetName,
            });
            showToast("Google 계정으로 로그인되었습니다.", "success");
            onClose();
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setGoogleLoading(false);
        }
    };

    return (
        <div className="modal-overlay" onClick={onClose}>
            <div className="modal-card" onClick={(e) => e.stopPropagation()}
                 style={{maxWidth: "400px", padding: "28px"}}>
                {/* Header */}
                <div style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    marginBottom: "20px"
                }}>
                    <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                        <div
                            style={{
                                width: "32px",
                                height: "32px",
                                borderRadius: "8px",
                                background: "#000000",
                                border: "1px solid #333639",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                                fontWeight: 900,
                                color: "#ffffff",
                                fontSize: "16px",
                            }}
                        >
                            F
                        </div>
                        <div style={{fontSize: "18px", fontWeight: 800, color: "#ffffff"}}>
                            Ferro 시작하기
                        </div>
                    </div>
                    <button onClick={onClose} className="modal-close-btn" aria-label="닫기">
                        <X size={20} />
                    </button>
                </div>

                {!showGooglePrompt ? (
                    <div>
                        {step === 1 ? (
                            <div style={{display: "flex", flexDirection: "column", gap: "12px"}}>
                                {/* Google Sign-In */}
                                <button
                                    type="button"
                                    onClick={() => setShowGooglePrompt(true)}
                                    className="x-pill-btn x-btn-google"
                                >
                                    <svg className="x-google-icon" viewBox="0 0 24 24" width="18" height="18">
                                        <path
                                            fill="#4285F4"
                                            d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
                                        />
                                        <path
                                            fill="#34A853"
                                            d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
                                        />
                                        <path
                                            fill="#FBBC05"
                                            d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l2.85-2.22.81-.63z"
                                        />
                                        <path
                                            fill="#EA4335"
                                            d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.52 6.16-4.52z"
                                        />
                                    </svg>
                                    <span>Google 계정으로 계속하기</span>
                                </button>

                                <div className="x-divider">
                                    <span className="x-divider-line"/>
                                    <span className="x-divider-text">또는</span>
                                    <span className="x-divider-line"/>
                                </div>

                                <form onSubmit={handleProceed}
                                      style={{display: "flex", flexDirection: "column", gap: "12px"}}>
                                    <input
                                        type="text"
                                        placeholder="이메일 또는 사용자 이름"
                                        value={identifier}
                                        onChange={(e) => setIdentifier(e.target.value)}
                                        className="x-text-input"
                                        autoFocus
                                        required
                                    />
                                    <button
                                        type="submit"
                                        disabled={!identifier.trim()}
                                        className="x-pill-btn x-btn-primary"
                                    >
                                        계속
                                    </button>
                                </form>
                            </div>
                        ) : (
                            <form onSubmit={handleSubmit}
                                  style={{display: "flex", flexDirection: "column", gap: "12px"}}>
                                <div className="x-identifier-chip">
                                    <div className="x-chip-content">
                                        <span className="x-chip-label">계정</span>
                                        <span className="x-chip-val">{identifier}</span>
                                    </div>
                                    <button
                                        type="button"
                                        onClick={() => setStep(1)}
                                        className="x-chip-change-btn"
                                    >
                                        <ArrowLeft size={13}/> 변경
                                    </button>
                                </div>

                                {isNewUser && (
                                    <input
                                        type="text"
                                        placeholder="닉네임"
                                        value={displayName}
                                        onChange={(e) => setDisplayName(e.target.value)}
                                        className="x-text-input"
                                    />
                                )}

                                <div className="x-input-with-icon">
                                    <input
                                        type={showPassword ? "text" : "password"}
                                        placeholder="비밀번호"
                                        value={password}
                                        onChange={(e) => setPassword(e.target.value)}
                                        className="x-text-input"
                                        autoFocus
                                        required
                                    />
                                    <button
                                        type="button"
                                        onClick={() => setShowPassword(!showPassword)}
                                        className="x-input-action-btn"
                                    >
                                        {showPassword ? <EyeOff size={16}/> : <Eye size={16}/>}
                                    </button>
                                </div>

                                <button
                                    type="submit"
                                    disabled={!password || loading}
                                    className="x-pill-btn x-btn-primary"
                                >
                                    {loading ? <Loader2 size={16} className="x-spin"/> : "계속"}
                                </button>
                            </form>
                        )}

                        <p className="x-legal-notice" style={{marginTop: "14px", textAlign: "center"}}>
                            로그인 / 회원가입 구분 없이 계정이 없으면 자동으로 생성됩니다.
                        </p>
                    </div>
                ) : (
                    /* Google Prompt */
                    <div style={{display: "flex", flexDirection: "column", gap: "12px"}}>
                        <div style={{display: "flex", alignItems: "center", gap: "8px", marginBottom: "4px"}}>
                            <button
                                type="button"
                                onClick={() => setShowGooglePrompt(false)}
                                style={{
                                    background: "none",
                                    border: "none",
                                    color: "#71767b",
                                    cursor: "pointer",
                                    display: "flex",
                                    alignItems: "center"
                                }}
                            >
                                <ArrowLeft size={16}/>
                            </button>
                            <span style={{fontSize: "14px", fontWeight: 700, color: "#ffffff"}}>Google 계정 선택</span>
                        </div>

                        <div className="x-google-accounts-list">
                            <button
                                type="button"
                                disabled={googleLoading}
                                onClick={() => handleGoogleAuth("ferro@example.com", "Ferro Dev")}
                                className="x-google-account-item"
                            >
                                <div className="x-google-avatar">F</div>
                                <div className="x-google-account-info">
                                    <div className="x-google-account-name">Ferro Dev</div>
                                    <div className="x-google-account-email">ferro@example.com</div>
                                </div>
                            </button>
                            <button
                                type="button"
                                disabled={googleLoading}
                                onClick={() => handleGoogleAuth("alex@example.com", "Alex Coder")}
                                className="x-google-account-item"
                            >
                                <div className="x-google-avatar" style={{background: "#8b5cf6"}}>A</div>
                                <div className="x-google-account-info">
                                    <div className="x-google-account-name">Alex Coder</div>
                                    <div className="x-google-account-email">alex@example.com</div>
                                </div>
                            </button>
                        </div>

                        <div className="x-divider">
                            <span className="x-divider-line"/>
                            <span className="x-divider-text">다른 Google 이메일</span>
                            <span className="x-divider-line"/>
                        </div>

                        <form
                            onSubmit={(e) => {
                                e.preventDefault();
                                handleGoogleAuth();
                            }}
                            style={{display: "flex", flexDirection: "column", gap: "8px"}}
                        >
                            <input
                                type="email"
                                placeholder="name@gmail.com"
                                value={googleEmail}
                                onChange={(e) => setGoogleEmail(e.target.value)}
                                className="x-text-input"
                                required
                            />
                            <button
                                type="submit"
                                disabled={googleLoading || !googleEmail.includes("@")}
                                className="x-pill-btn x-btn-primary"
                            >
                                {googleLoading ? <Loader2 size={16} className="x-spin"/> : "계속"}
                            </button>
                        </form>
                    </div>
                )}
            </div>
        </div>
    );
}
