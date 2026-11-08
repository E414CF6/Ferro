"use client";

import React, {useEffect, useState} from "react";
import {useRouter} from "next/navigation";
import {useAuth} from "@/lib/auth-context";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import {fetchGraphQL, QUERIES} from "@/lib/graphql";
import {ArrowLeft, Eye, EyeOff, Loader2, X} from "lucide-react";

export default function LoginPage() {
    const router = useRouter();
    const {user, login, signup, googleLogin} = useAuth();
    const { showToast } = useToast();

    // Unified step: 1 = Identifier input, 2 = Password / Finalize
    const [step, setStep] = useState<1 | 2>(1);
    const [identifier, setIdentifier] = useState("");
    const [password, setPassword] = useState("");
    const [displayName, setDisplayName] = useState("");
    const [isNewUser, setIsNewUser] = useState(false);
    const [showPassword, setShowPassword] = useState(false);
    const [loading, setLoading] = useState(false);

    // Google Sign-In modal / selection dialog
    const [showGoogleModal, setShowGoogleModal] = useState(false);
    const [googleEmail, setGoogleEmail] = useState("");
    const [googleName, setGoogleName] = useState("");
    const [googleLoading, setGoogleLoading] = useState(false);

    useEffect(() => {
        if (user) {
            router.replace("/");
        }
    }, [user, router]);

    // Handle Step 1 -> Step 2
    const handleProceedToStep2 = async (e: React.FormEvent) => {
        e.preventDefault();
        const cleanId = identifier.trim().toLowerCase();
        if (!cleanId) return;

        setLoading(true);
        try {
            // Check if identifier is an existing username
            const isEmail = cleanId.includes("@");
            if (!isEmail) {
                const data = await fetchGraphQL<{
                    profile: any
                }>(QUERIES.USER_PROFILE, {username: cleanId}).catch(() => null);
                if (data?.profile) {
                    setIsNewUser(false);
                    setDisplayName(data.profile.displayName || cleanId);
                } else {
                    setIsNewUser(true);
                    setDisplayName(cleanId);
                }
            } else {
                // Email format: default display name derived from username part
                const suggestedName = cleanId.split("@")[0];
                setDisplayName(suggestedName);
            }
            setStep(2);
        } catch (err) {
            setStep(2);
        } finally {
            setLoading(false);
        }
    };

    // Handle final submission (Login or auto-signup without distinction)
    const handleAuthSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        const cleanId = identifier.trim().toLowerCase();
        if (!cleanId || !password) return;

        setLoading(true);
        try {
            // Attempt login first
            try {
                await login(cleanId, password);
                showToast("로그인되었습니다.", "success");
                router.replace("/");
                return;
            } catch (loginErr: any) {
                const errMsg = String(loginErr?.message || "");
                const isUserNotFound =
                    errMsg.toLowerCase().includes("not found") ||
                    errMsg.toLowerCase().includes("usernotfound") ||
                    errMsg.includes("사용자") ||
                    isNewUser;

                // If account does not exist, automatically sign up seamlessly!
                if (isUserNotFound) {
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
                    showToast(`환영합니다! 계정이 생성되었습니다.`, "success");
                    router.replace("/");
                    return;
                }

                // If user exists and password was wrong
                showToast(formatErrorMessage(loginErr, "ko"), "error");
            }
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLoading(false);
        }
    };

    // Google Sign-In handler
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
            showToast(`Google 계정으로 로그인되었습니다.`, "success");
            setShowGoogleModal(false);
            router.replace("/");
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setGoogleLoading(false);
        }
    };

    return (
        <div className="x-page-root">
            {/* Main Stage */}
            <main className="x-main-stage">
                {/* Left Side: Editorial & Auth Container */}
                <div className="x-left-column">
                    <h1 className="x-headline">
                        Happening<br/>now.
                    </h1>

                    <div className="x-auth-container">
                        {step === 1 ? (
                            <>
                                {/* Google Sign-In Pill Button */}
                                <button
                                    type="button"
                                    onClick={() => setShowGoogleModal(true)}
                                    className="x-pill-btn x-btn-google"
                                    aria-label="Google 계정으로 계속하기"
                                >
                                    <svg className="x-google-icon" viewBox="0 0 24 24" width="20" height="20">
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

                                {/* Divider */}
                                <div className="x-divider">
                                    <span className="x-divider-line"/>
                                    <span className="x-divider-text">또는</span>
                                    <span className="x-divider-line"/>
                                </div>

                                {/* Unified Identifier Form */}
                                <form onSubmit={handleProceedToStep2} className="x-auth-form">
                                    <div className="x-input-wrap">
                                        <input
                                            type="text"
                                            placeholder="이메일 또는 사용자 이름"
                                            value={identifier}
                                            onChange={(e) => setIdentifier(e.target.value)}
                                            className="x-text-input"
                                            autoFocus
                                            required
                                        />
                                    </div>

                                    <button
                                        type="submit"
                                        disabled={!identifier.trim() || loading}
                                        className="x-pill-btn x-btn-primary"
                                    >
                                        {loading ? (
                                            <Loader2 size={18} className="x-spin"/>
                                        ) : (
                                            "계속"
                                        )}
                                    </button>
                                </form>
                            </>
                        ) : (
                            /* Step 2: Password & Instant Entry */
                            <form onSubmit={handleAuthSubmit} className="x-auth-form">
                                <div className="x-identifier-chip">
                                    <div className="x-chip-content">
                                        <span className="x-chip-label">계정</span>
                                        <span className="x-chip-val">{identifier}</span>
                                    </div>
                                    <button
                                        type="button"
                                        onClick={() => {
                                            setStep(1);
                                            setPassword("");
                                        }}
                                        className="x-chip-change-btn"
                                        title="계정 변경"
                                    >
                                        <ArrowLeft size={14}/> 변경
                                    </button>
                                </div>

                                {isNewUser && (
                                    <div className="x-input-wrap">
                                        <label className="x-field-label">표시 이름 (닉네임)</label>
                                        <input
                                            type="text"
                                            placeholder="닉네임"
                                            value={displayName}
                                            onChange={(e) => setDisplayName(e.target.value)}
                                            className="x-text-input"
                                            maxLength={50}
                                        />
                                    </div>
                                )}

                                <div className="x-input-wrap">
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
                                            aria-label={showPassword ? "비밀번호 숨기기" : "비밀번호 표시"}
                                        >
                                            {showPassword ? <EyeOff size={18}/> : <Eye size={18}/>}
                                        </button>
                                    </div>
                                </div>

                                <button
                                    type="submit"
                                    disabled={!password || loading}
                                    className="x-pill-btn x-btn-primary"
                                >
                                    {loading ? (
                                        <Loader2 size={18} className="x-spin"/>
                                    ) : (
                                        "계속"
                                    )}
                                </button>
                            </form>
                        )}

                        {/* Legal Notice */}
                        <p className="x-legal-notice">
                            계속 진행하면 Ferro의{" "}
                            <a href="#" onClick={(e) => e.preventDefault()}>
                                서비스 약관
                            </a>
                            ,{" "}
                            <a href="#" onClick={(e) => e.preventDefault()}>
                                개인정보 처리방침
                            </a>{" "}
                            및{" "}
                            <a href="#" onClick={(e) => e.preventDefault()}>
                                쿠키 사용
                            </a>
                            에 동의하게 됩니다.
                        </p>
                    </div>
                </div>

                {/* Right Side: Massive Wireframe Ferro Outline Logo & App QR Card */}
                <div className="x-right-column">
                    <div className="x-wireframe-logo-stage">
                        {/* Chrome Geometric Wireframe 'F' Logo */}
                        <svg
                            className="x-giant-logo-svg"
                            viewBox="0 0 600 600"
                            fill="none"
                            xmlns="http://www.w3.org/2000/svg"
                            aria-hidden="true"
                        >
                            <defs>
                                {/* Metallic Chrome Stroke Gradients */}
                                <linearGradient id="chromeOuter" x1="0%" y1="0%" x2="100%" y2="100%">
                                    <stop offset="0%" stopColor="#ffffff" stopOpacity="0.95"/>
                                    <stop offset="25%" stopColor="#71767b" stopOpacity="0.8"/>
                                    <stop offset="45%" stopColor="#1e2229" stopOpacity="0.85"/>
                                    <stop offset="70%" stopColor="#cbd5e1" stopOpacity="0.9"/>
                                    <stop offset="100%" stopColor="#ffffff" stopOpacity="0.95"/>
                                </linearGradient>
                                <linearGradient id="chromeInner" x1="100%" y1="0%" x2="0%" y2="100%">
                                    <stop offset="0%" stopColor="#ffffff" stopOpacity="0.85"/>
                                    <stop offset="30%" stopColor="#475569" stopOpacity="0.7"/>
                                    <stop offset="55%" stopColor="#0f172a" stopOpacity="0.9"/>
                                    <stop offset="85%" stopColor="#94a3b8" stopOpacity="0.8"/>
                                    <stop offset="100%" stopColor="#ffffff" stopOpacity="0.85"/>
                                </linearGradient>
                                <linearGradient id="edgeGlow" x1="0%" y1="0%" x2="100%" y2="0%">
                                    <stop offset="0%" stopColor="#ffffff" stopOpacity="0.7"/>
                                    <stop offset="100%" stopColor="#38bdf8" stopOpacity="0.4"/>
                                </linearGradient>
                            </defs>

                            {/* Outer Geometric Wireframe F Outline */}
                            <path
                                d="M120 70 L480 70 L460 160 L240 160 L230 250 L410 250 L390 330 L220 330 L195 530 L105 530 Z"
                                stroke="url(#chromeOuter)"
                                strokeWidth="5"
                                strokeLinejoin="miter"
                                strokeMiterlimit="4"
                                opacity="0.9"
                            />

                            {/* Inner Parallel Precision Wireframe F Contour */}
                            <path
                                d="M145 95 L445 95 L433 140 L260 140 L250 270 L380 270 L368 310 L240 310 L215 505 L135 505 Z"
                                stroke="url(#chromeInner)"
                                strokeWidth="2.5"
                                strokeLinejoin="miter"
                                strokeMiterlimit="4"
                                opacity="0.65"
                            />

                            {/* Bevel Chamfer Connectors creating the double-stroke 3D Wireframe feel */}
                            <line x1="120" y1="70" x2="145" y2="95" stroke="url(#chromeOuter)" strokeWidth="2"
                                  opacity="0.5"/>
                            <line x1="480" y1="70" x2="445" y2="95" stroke="url(#chromeOuter)" strokeWidth="2"
                                  opacity="0.5"/>
                            <line x1="460" y1="160" x2="433" y2="140" stroke="url(#chromeOuter)" strokeWidth="2"
                                  opacity="0.5"/>
                            <line x1="240" y1="160" x2="260" y2="140" stroke="url(#chromeOuter)" strokeWidth="2"
                                  opacity="0.5"/>
                            <line x1="230" y1="250" x2="250" y2="270" stroke="url(#chromeOuter)" strokeWidth="2"
                                  opacity="0.5"/>
                            <line x1="410" y1="250" x2="380" y2="270" stroke="url(#chromeOuter)" strokeWidth="2"
                                  opacity="0.5"/>
                            <line x1="390" y1="330" x2="368" y2="310" stroke="url(#chromeOuter)" strokeWidth="2"
                                  opacity="0.5"/>
                            <line x1="220" y1="330" x2="240" y2="310" stroke="url(#chromeOuter)" strokeWidth="2"
                                  opacity="0.5"/>
                            <line x1="195" y1="530" x2="215" y2="505" stroke="url(#chromeOuter)" strokeWidth="2"
                                  opacity="0.5"/>
                            <line x1="105" y1="530" x2="135" y2="505" stroke="url(#chromeOuter)" strokeWidth="2"
                                  opacity="0.5"/>
                        </svg>
                    </div>

                    {/* QR Code App Download Box (Bottom Right) */}
                    <aside className="x-qr-card" aria-label="Ferro 앱 다운로드 QR">
                        <div className="x-qr-title">Scan to get the app</div>
                        <div className="x-qr-frame">
                            <svg viewBox="0 0 100 100" width="76" height="76" className="x-qr-svg">
                                {/* QR Corners and Pattern */}
                                <rect width="100" height="100" fill="#ffffff" rx="6"/>
                                <rect x="8" y="8" width="26" height="26" fill="#000000" rx="3"/>
                                <rect x="13" y="13" width="16" height="16" fill="#ffffff" rx="2"/>
                                <rect x="17" y="17" width="8" height="8" fill="#000000" rx="1"/>

                                <rect x="66" y="8" width="26" height="26" fill="#000000" rx="3"/>
                                <rect x="71" y="13" width="16" height="16" fill="#ffffff" rx="2"/>
                                <rect x="75" y="17" width="8" height="8" fill="#000000" rx="1"/>

                                <rect x="8" y="66" width="26" height="26" fill="#000000" rx="3"/>
                                <rect x="13" y="71" width="16" height="16" fill="#ffffff" rx="2"/>
                                <rect x="17" y="77" width="8" height="8" fill="#000000" rx="1"/>

                                {/* Matrix Modules */}
                                <rect x="38" y="10" width="5" height="5" fill="#000000"/>
                                <rect x="48" y="10" width="5" height="5" fill="#000000"/>
                                <rect x="56" y="10" width="5" height="5" fill="#000000"/>
                                <rect x="38" y="22" width="5" height="5" fill="#000000"/>
                                <rect x="48" y="28" width="5" height="5" fill="#000000"/>
                                <rect x="10" y="44" width="5" height="5" fill="#000000"/>
                                <rect x="22" y="48" width="5" height="5" fill="#000000"/>
                                <rect x="70" y="42" width="5" height="5" fill="#000000"/>
                                <rect x="80" y="52" width="5" height="5" fill="#000000"/>
                                <rect x="40" y="74" width="5" height="5" fill="#000000"/>
                                <rect x="52" y="84" width="5" height="5" fill="#000000"/>
                                <rect x="74" y="74" width="5" height="5" fill="#000000"/>
                                <rect x="84" y="84" width="5" height="5" fill="#000000"/>

                                {/* Center Badge */}
                                <rect x="40" y="40" width="20" height="20" fill="#000000" rx="4"/>
                                <text
                                    x="50"
                                    y="54"
                                    fontSize="12"
                                    fontWeight="900"
                                    fill="#ffffff"
                                    textAnchor="middle"
                                    fontFamily="system-ui, sans-serif"
                                >
                                    F
                                </text>
                            </svg>
                        </div>
                    </aside>
                </div>
            </main>

            {/* Bottom Minimal Footer Links */}
            <footer className="x-footer">
                <nav className="x-footer-nav">
                    <a href="#">About</a>
                    <a href="#">Get App</a>
                    <a href="#">Help</a>
                    <a href="#">Terms</a>
                    <a href="#">Privacy</a>
                    <a href="#">Cookies</a>
                    <a href="#">Careers</a>
                    <a href="#">Ads & Business</a>
                    <a href="#">Developers</a>
                    <a href="#">News</a>
                    <a href="#">Accessibility</a>
                    <span>© 2026 Ferro Corp.</span>
                </nav>
            </footer>

            {/* Google Fast Sign-In Modal */}
            {showGoogleModal && (
                <div className="x-modal-overlay" onClick={() => !googleLoading && setShowGoogleModal(false)}>
                    <div className="x-google-modal-card" onClick={(e) => e.stopPropagation()}>
                        <div className="x-google-modal-header">
                            <div className="x-google-brand">
                                <svg viewBox="0 0 24 24" width="24" height="24">
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
                                <span>Google 계정으로 Ferro 로그인</span>
                            </div>
                            <button
                                type="button"
                                onClick={() => setShowGoogleModal(false)}
                                className="x-modal-close"
                                disabled={googleLoading}
                            >
                                <X size={20}/>
                            </button>
                        </div>

                        <div className="x-google-modal-body">
                            <p className="x-google-desc">
                                로그인하거나 새로 가입할 Google 계정을 선택하거나 입력하세요.
                            </p>

                            {/* Preset Quick Select Accounts */}
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

                            <div className="x-divider" style={{margin: "16px 0"}}>
                                <span className="x-divider-line"/>
                                <span className="x-divider-text">다른 Google 계정</span>
                                <span className="x-divider-line"/>
                            </div>

                            <form
                                onSubmit={(e) => {
                                    e.preventDefault();
                                    handleGoogleAuth();
                                }}
                                style={{display: "flex", flexDirection: "column", gap: "10px"}}
                            >
                                <input
                                    type="email"
                                    placeholder="your-google-email@gmail.com"
                                    value={googleEmail}
                                    onChange={(e) => setGoogleEmail(e.target.value)}
                                    className="x-text-input"
                                    required
                                />
                                <input
                                    type="text"
                                    placeholder="이름 (선택)"
                                    value={googleName}
                                    onChange={(e) => setGoogleName(e.target.value)}
                                    className="x-text-input"
                                />
                                <button
                                    type="submit"
                                    disabled={googleLoading || !googleEmail.includes("@")}
                                    className="x-pill-btn x-btn-primary"
                                    style={{marginTop: "4px"}}
                                >
                                    {googleLoading ? (
                                        <Loader2 size={16} className="x-spin"/>
                                    ) : (
                                        "이 Google 계정으로 계속"
                                    )}
                                </button>
                            </form>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
