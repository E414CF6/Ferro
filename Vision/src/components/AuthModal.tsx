"use client";

import React, {useState} from "react";
import {useAuth} from "@/lib/auth-context";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import {Check, Eye, EyeOff, Lock, LogIn, Mail, Sparkles, User as UserIcon, UserPlus, X} from "lucide-react";

interface AuthModalProps {
    onClose: () => void;
}

export default function AuthModal({onClose}: AuthModalProps) {
    const {login, signup} = useAuth();
    const {showToast} = useToast();

    const [tab, setTab] = useState<"login" | "signup">("login");
    const [loading, setLoading] = useState(false);
    const [showPassword, setShowPassword] = useState(false);
    const [showConfirmPassword, setShowConfirmPassword] = useState(false);

    // Login form state
    const [loginUsername, setLoginUsername] = useState("");
    const [loginPassword, setLoginPassword] = useState("");

    // Signup form state
    const [signupUsername, setSignupUsername] = useState("");
    const [signupEmail, setSignupEmail] = useState("");
    const [signupPassword, setSignupPassword] = useState("");
    const [signupConfirmPassword, setSignupConfirmPassword] = useState("");
    const [signupDisplayName, setSignupDisplayName] = useState("");
    const [signupBio, setSignupBio] = useState("");
    const [signupAvatarUrl, setSignupAvatarUrl] = useState("");
    const [signupLocation, setSignupLocation] = useState("");

    // Password strength calculation
    const calculateStrength = (pwd: string): { score: number; label: string; color: string } => {
        if (!pwd) return {score: 0, label: "비밀번호를 입력하세요", color: "var(--text-muted)"};
        let score = 0;
        if (pwd.length >= 8) score += 1;
        if (pwd.length >= 12) score += 1;
        if (/[A-Z]/.test(pwd) || /[a-z]/.test(pwd)) score += 1;
        if (/[0-9]/.test(pwd)) score += 1;
        if (/[^A-Za-z0-9]/.test(pwd)) score += 1;

        if (score <= 2) return {score: 1, label: "보안 취약 (최소 8자 이상 권장)", color: "#f43f5e"};
        if (score <= 3) return {score: 2, label: "보안 보통 (숫자/특수문자 추가 권장)", color: "#f59e0b"};
        if (score <= 4) return {score: 3, label: "보안 안전", color: "#38bdf8"};
        return {score: 4, label: "매우 강력함", color: "#10b981"};
    };

    const passwordStrength = calculateStrength(signupPassword);
    const generatedAvatar = signupAvatarUrl.trim() || `https://api.dicebear.com/7.x/bottts/svg?seed=${signupUsername || "ferro"}`;

    const handleLoginSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!loginUsername.trim() || !loginPassword) {
            showToast("아이디 또는 비밀번호를 입력해주세요.", "error");
            return;
        }

        setLoading(true);
        try {
            await login(loginUsername.trim(), loginPassword);
            showToast(`환영합니다! @${loginUsername.trim()} 계정으로 로그인되었습니다.`, "success");
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
        const cleanDisplayName = signupDisplayName.trim();

        // Validations
        if (!cleanUsername || !cleanEmail || !signupPassword || !cleanDisplayName) {
            showToast("필수 입력 항목을 모두 작성해주세요.", "error");
            return;
        }

        if (!/^[a-zA-Z0-9_]{3,50}$/.test(cleanUsername)) {
            showToast("사용자 아이디는 3~50자의 영문 소문자, 숫자, 밑줄(_)만 가능합니다.", "error");
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
                bio: signupBio.trim() || undefined,
                avatarUrl: generatedAvatar,
                location: signupLocation.trim() || undefined,
            });

            showToast(`환영합니다, ${cleanDisplayName}님! 회원가입이 완료되었습니다.`, "success");
            onClose();
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="modal-overlay" onClick={onClose}>
            <div className="modal-card" onClick={(e) => e.stopPropagation()} style={{maxWidth: "480px"}}>
                {/* Modal Header */}
                <div className="modal-header">
                    <div style={{display: "flex", alignItems: "center", gap: "10px"}}>
                        <div className="logo-badge" style={{width: "36px", height: "36px", fontSize: "18px"}}>
                            🦀
                        </div>
                        <div>
                            <h2 className="modal-title" style={{fontSize: "18px"}}>
                                {tab === "login" ? "Ferro 로그인" : "새 계정 만들기"}
                            </h2>
                            <p style={{fontSize: "12px", color: "var(--text-muted)"}}>
                                {tab === "login"
                                    ? "계정에 로그인하여 피드와 스토리를 확인하세요"
                                    : "모던 실시간 SNS 커뮤니티에 참여하세요"}
                            </p>
                        </div>
                    </div>
                    <button onClick={onClose} className="modal-close-btn" aria-label="닫기">
                        <X size={20}/>
                    </button>
                </div>

                {/* Tab Switcher */}
                <div style={{display: "flex", borderBottom: "1px solid var(--border-subtle)", padding: "0 24px"}}>
                    <button
                        type="button"
                        onClick={() => setTab("login")}
                        style={{
                            flex: 1,
                            padding: "12px 0",
                            fontWeight: 700,
                            fontSize: "14px",
                            color: tab === "login" ? "var(--accent-primary)" : "var(--text-secondary)",
                            borderBottom: tab === "login" ? "2px solid var(--accent-primary)" : "2px solid transparent",
                            transition: "all var(--transition-fast)",
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "center",
                            gap: "6px",
                        }}
                    >
                        <LogIn size={15}/> 로그인
                    </button>
                    <button
                        type="button"
                        onClick={() => setTab("signup")}
                        style={{
                            flex: 1,
                            padding: "12px 0",
                            fontWeight: 700,
                            fontSize: "14px",
                            color: tab === "signup" ? "var(--accent-primary)" : "var(--text-secondary)",
                            borderBottom: tab === "signup" ? "2px solid var(--accent-primary)" : "2px solid transparent",
                            transition: "all var(--transition-fast)",
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "center",
                            gap: "6px",
                        }}
                    >
                        <UserPlus size={15}/> 회원가입
                    </button>
                </div>

                <div className="modal-body">
                    {/* Login Form */}
                    {tab === "login" ? (
                        <form onSubmit={handleLoginSubmit}
                              style={{display: "flex", flexDirection: "column", gap: "16px"}}>
                            <div className="form-group" style={{marginBottom: 0}}>
                                <label className="form-label">
                                    <UserIcon size={13} style={{display: "inline", marginRight: "4px"}}/>
                                    아이디 또는 이메일
                                </label>
                                <input
                                    type="text"
                                    className="form-input"
                                    placeholder="사용자 아이디 또는 이메일 주소"
                                    value={loginUsername}
                                    onChange={(e) => setLoginUsername(e.target.value)}
                                    autoFocus
                                    required
                                />
                            </div>

                            <div className="form-group" style={{marginBottom: 0}}>
                                <label className="form-label">
                                    <Lock size={13} style={{display: "inline", marginRight: "4px"}}/>
                                    비밀번호
                                </label>
                                <div style={{position: "relative"}}>
                                    <input
                                        type={showPassword ? "text" : "password"}
                                        className="form-input"
                                        placeholder="비밀번호 입력"
                                        value={loginPassword}
                                        onChange={(e) => setLoginPassword(e.target.value)}
                                        required
                                    />
                                    <button
                                        type="button"
                                        onClick={() => setShowPassword(!showPassword)}
                                        style={{
                                            position: "absolute",
                                            right: 12,
                                            top: "50%",
                                            transform: "translateY(-50%)",
                                            color: "var(--text-muted)",
                                        }}
                                        aria-label={showPassword ? "비밀번호 숨기기" : "비밀번호 표시"}
                                    >
                                        {showPassword ? <EyeOff size={16}/> : <Eye size={16}/>}
                                    </button>
                                </div>
                            </div>

                            <button
                                type="submit"
                                disabled={loading || !loginUsername.trim() || !loginPassword}
                                className="btn-primary"
                                style={{width: "100%", padding: "12px", marginTop: "8px"}}
                            >
                                <LogIn size={16}/>
                                {loading ? "로그인 중..." : "로그인"}
                            </button>

                            <div style={{
                                textAlign: "center",
                                fontSize: "13px",
                                color: "var(--text-secondary)",
                                marginTop: "4px"
                            }}>
                                계정이 없으신가요?{" "}
                                <button
                                    type="button"
                                    onClick={() => setTab("signup")}
                                    style={{color: "var(--accent-primary)", fontWeight: 700}}
                                >
                                    회원가입하기
                                </button>
                            </div>
                        </form>
                    ) : (
                        /* Signup Form */
                        <form onSubmit={handleSignupSubmit}
                              style={{display: "flex", flexDirection: "column", gap: "14px"}}>
                            {/* Avatar Preview */}
                            <div
                                style={{
                                    display: "flex",
                                    alignItems: "center",
                                    gap: "14px",
                                    padding: "12px",
                                    borderRadius: "var(--radius-md)",
                                    backgroundColor: "var(--bg-input)",
                                    border: "1px solid var(--border-subtle)",
                                }}
                            >
                                <img
                                    src={generatedAvatar}
                                    alt="Avatar Preview"
                                    style={{
                                        width: 52,
                                        height: 52,
                                        borderRadius: "50%",
                                        objectFit: "cover",
                                        border: "2px solid var(--accent-primary)",
                                    }}
                                />
                                <div style={{flex: 1, minWidth: 0}}>
                                    <div style={{fontSize: "13px", fontWeight: 700, color: "var(--text-primary)"}}>
                                        프로필 아바타 미리보기
                                    </div>
                                    <div style={{fontSize: "11px", color: "var(--text-muted)", marginTop: "2px"}}>
                                        아이디에 맞춰 고유 로봇 아바타가 자동 생성됩니다.
                                    </div>
                                </div>
                            </div>

                            <div style={{display: "grid", gridTemplateColumns: "1fr 1fr", gap: "10px"}}>
                                <div className="form-group" style={{marginBottom: 0}}>
                                    <label className="form-label">아이디 (영문/숫자/_) *</label>
                                    <input
                                        type="text"
                                        className="form-input"
                                        placeholder="예: rust_ace"
                                        value={signupUsername}
                                        onChange={(e) => setSignupUsername(e.target.value.toLowerCase().replace(/[^a-z0-9_]/g, ""))}
                                        maxLength={50}
                                        required
                                    />
                                </div>

                                <div className="form-group" style={{marginBottom: 0}}>
                                    <label className="form-label">표시 이름 (닉네임) *</label>
                                    <input
                                        type="text"
                                        className="form-input"
                                        placeholder="예: 러스트 개발자"
                                        value={signupDisplayName}
                                        onChange={(e) => setSignupDisplayName(e.target.value)}
                                        maxLength={50}
                                        required
                                    />
                                </div>
                            </div>

                            <div className="form-group" style={{marginBottom: 0}}>
                                <label className="form-label">
                                    <Mail size={13} style={{display: "inline", marginRight: "4px"}}/>
                                    이메일 주소 *
                                </label>
                                <input
                                    type="email"
                                    className="form-input"
                                    placeholder="user@example.com"
                                    value={signupEmail}
                                    onChange={(e) => setSignupEmail(e.target.value)}
                                    required
                                />
                            </div>

                            <div style={{display: "grid", gridTemplateColumns: "1fr 1fr", gap: "10px"}}>
                                <div className="form-group" style={{marginBottom: 0}}>
                                    <label className="form-label">비밀번호 *</label>
                                    <div style={{position: "relative"}}>
                                        <input
                                            type={showPassword ? "text" : "password"}
                                            className="form-input"
                                            placeholder="8자 이상 입력"
                                            value={signupPassword}
                                            onChange={(e) => setSignupPassword(e.target.value)}
                                            required
                                        />
                                        <button
                                            type="button"
                                            onClick={() => setShowPassword(!showPassword)}
                                            style={{
                                                position: "absolute",
                                                right: 10,
                                                top: "50%",
                                                transform: "translateY(-50%)",
                                                color: "var(--text-muted)",
                                            }}
                                            aria-label={showPassword ? "비밀번호 숨기기" : "비밀번호 표시"}
                                        >
                                            {showPassword ? <EyeOff size={15}/> : <Eye size={15}/>}
                                        </button>
                                    </div>
                                </div>

                                <div className="form-group" style={{marginBottom: 0}}>
                                    <label className="form-label">비밀번호 확인 *</label>
                                    <div style={{position: "relative"}}>
                                        <input
                                            type={showConfirmPassword ? "text" : "password"}
                                            className="form-input"
                                            placeholder="비밀번호 재입력"
                                            value={signupConfirmPassword}
                                            onChange={(e) => setSignupConfirmPassword(e.target.value)}
                                            required
                                        />
                                        <button
                                            type="button"
                                            onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                                            style={{
                                                position: "absolute",
                                                right: 10,
                                                top: "50%",
                                                transform: "translateY(-50%)",
                                                color: "var(--text-muted)",
                                            }}
                                            aria-label={showConfirmPassword ? "비밀번호 숨기기" : "비밀번호 표시"}
                                        >
                                            {showConfirmPassword ? <EyeOff size={15}/> : <Eye size={15}/>}
                                        </button>
                                    </div>
                                </div>
                            </div>

                            {/* Password strength indicator */}
                            {signupPassword && (
                                <div style={{
                                    fontSize: "11px",
                                    color: passwordStrength.color,
                                    display: "flex",
                                    alignItems: "center",
                                    gap: "6px"
                                }}>
                                    <Sparkles size={12}/>
                                    <span>{passwordStrength.label}</span>
                                    {signupConfirmPassword && signupPassword === signupConfirmPassword && (
                                        <span style={{
                                            color: "#10b981",
                                            marginLeft: "auto",
                                            display: "flex",
                                            alignItems: "center",
                                            gap: "2px"
                                        }}>
                                            <Check size={12}/> 일치함
                                        </span>
                                    )}
                                </div>
                            )}

                            <div className="form-group" style={{marginBottom: 0}}>
                                <label className="form-label">한 줄 소개 (선택)</label>
                                <input
                                    type="text"
                                    className="form-input"
                                    placeholder="예: Rust 백엔드 개발자"
                                    value={signupBio}
                                    onChange={(e) => setSignupBio(e.target.value)}
                                    maxLength={160}
                                />
                            </div>

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
                                style={{width: "100%", padding: "12px", marginTop: "6px"}}
                            >
                                <UserPlus size={16}/>
                                {loading ? "가입 처리 중..." : "회원가입 완료"}
                            </button>

                            <div style={{textAlign: "center", fontSize: "13px", color: "var(--text-secondary)"}}>
                                이미 계정이 있으신가요?{" "}
                                <button
                                    type="button"
                                    onClick={() => setTab("login")}
                                    style={{color: "var(--accent-primary)", fontWeight: 700}}
                                >
                                    로그인하기
                                </button>
                            </div>
                        </form>
                    )}
                </div>
            </div>
        </div>
    );
}
