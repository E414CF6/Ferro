"use client";

import React, {Suspense, useEffect, useState} from "react";
import Link from "next/link";
import {useRouter, useSearchParams} from "next/navigation";
import {useAuth} from "@/lib/auth-context";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import {
    ArrowRight,
    Camera,
    Check,
    CheckCircle2,
    Cpu,
    Dices,
    Eye,
    EyeOff,
    Flame,
    Globe,
    Layers,
    Lock,
    Mail,
    MessageCircle,
    ShieldCheck,
    Sparkles,
    User as UserIcon,
    Zap,
} from "lucide-react";

function WelcomeContent() {
    const router = useRouter();
    const searchParams = useSearchParams();
    const initialTab = searchParams.get("tab") === "login" ? "login" : "signup";

    const {user, login, signup} = useAuth();
    const {showToast} = useToast();

    const [authTab, setAuthTab] = useState<"signup" | "login">(initialTab);
    const [loading, setLoading] = useState(false);

    // If already logged in, redirect to home
    useEffect(() => {
        if (user) {
            router.push("/");
        }
    }, [user, router]);

    // Login form state
    const [loginUsername, setLoginUsername] = useState("");
    const [loginPassword, setLoginPassword] = useState("");
    const [showLoginPassword, setShowLoginPassword] = useState(false);

    // Signup form state
    const [signupUsername, setSignupUsername] = useState("");
    const [signupEmail, setSignupEmail] = useState("");
    const [signupPassword, setSignupPassword] = useState("");
    const [signupDisplayName, setSignupDisplayName] = useState("");
    const [signupBio, setSignupBio] = useState("");
    const [signupAvatarSeed, setSignupAvatarSeed] = useState("ferro");
    const [showSignupPassword, setShowSignupPassword] = useState(false);

    const avatarSeeds = ["ferro", "rustacean", "tokio", "axum", "graphql", "cyber", "pixel", "spark"];

    const randomizeSeed = () => {
        const random = "user_" + Math.random().toString(36).substring(2, 8);
        setSignupAvatarSeed(random);
    };

    const currentAvatarUrl = `https://api.dicebear.com/7.x/bottts/svg?seed=${signupAvatarSeed || signupUsername || "ferro"}`;

    // Password strength calculation
    const calculateStrength = (pwd: string) => {
        if (!pwd) return {score: 0, label: "비밀번호를 입력하세요", color: "var(--text-muted)"};
        let score = 0;
        if (pwd.length >= 8) score += 1;
        if (pwd.length >= 12) score += 1;
        if (/[A-Z]/.test(pwd) || /[a-z]/.test(pwd)) score += 1;
        if (/[0-9]/.test(pwd)) score += 1;
        if (/[^A-Za-z0-9]/.test(pwd)) score += 1;

        if (score <= 2) return {score: 1, label: "보안 보통 (8자 이상)", color: "#f43f5e"};
        if (score <= 3) return {score: 2, label: "보안 안전", color: "#f59e0b"};
        if (score <= 4) return {score: 3, label: "강력한 보안", color: "#38bdf8"};
        return {score: 4, label: "최고 수준 보안 🔒", color: "#10b981"};
    };

    const pwdStrength = calculateStrength(signupPassword);

    const handleLogin = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!loginUsername.trim() || !loginPassword) {
            showToast("아이디와 비밀번호를 모두 입력해주세요.", "error");
            return;
        }

        setLoading(true);
        try {
            await login(loginUsername.trim(), loginPassword);
            showToast(`환영합니다! @${loginUsername.trim()} 계정으로 로그인되었습니다.`, "success");
            router.push("/");
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLoading(false);
        }
    };

    const handleSignup = async (e: React.FormEvent) => {
        e.preventDefault();
        const cleanUsername = signupUsername.trim().toLowerCase();
        const cleanEmail = signupEmail.trim().toLowerCase();
        const cleanDisplayName = signupDisplayName.trim() || cleanUsername;

        if (!cleanUsername || !cleanEmail || !signupPassword) {
            showToast("필수 항목(아이디, 이메일, 비밀번호)을 작성해주세요.", "error");
            return;
        }

        if (!/^[a-zA-Z0-9_]{3,50}$/.test(cleanUsername)) {
            showToast("아이디는 3~50자의 영문, 숫자, 밑줄(_)만 가능합니다.", "error");
            return;
        }

        if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(cleanEmail)) {
            showToast("유효한 이메일 주소를 입력해주세요.", "error");
            return;
        }

        if (signupPassword.length < 8) {
            showToast("비밀번호는 최소 8자 이상이어야 합니다.", "error");
            return;
        }

        setLoading(true);
        try {
            await signup({
                username: cleanUsername,
                email: cleanEmail,
                password: signupPassword,
                displayName: cleanDisplayName,
                bio: signupBio.trim() || "Hello, Ferro!",
                avatarUrl: currentAvatarUrl,
            });
            showToast(`환영합니다! @${cleanUsername} 계정이 생성되었습니다. 🎉`, "success");
            router.push("/");
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="landing-page">
            {/* Background Decorative Glows */}
            <div className="landing-glow glow-1"/>
            <div className="landing-glow glow-2"/>
            <div className="landing-glow glow-3"/>

            {/* Top Navbar */}
            <header className="landing-header">
                <div className="landing-nav-container">
                    <Link href="/" className="logo-container" style={{margin: 0}}>
                        <div className="logo-badge">🦀</div>
                        <div className="logo-text">
                            Ferro
                            <span className="logo-subtext">Vision SNS</span>
                        </div>
                    </Link>

                    <nav className="landing-nav-links">
                        <a href="#features" className="landing-nav-link">기능 소개</a>
                        <a href="#architecture" className="landing-nav-link">기술 스택</a>
                        <a href="#join" className="landing-nav-link">가입 & 로그인</a>
                    </nav>

                    <div style={{display: "flex", alignItems: "center", gap: "10px"}}>
                        <Link href="/" className="btn-secondary" style={{padding: "8px 16px"}}>
                            <Globe size={15}/> 피드 둘러보기
                        </Link>
                        <a
                            href="#join"
                            onClick={() => setAuthTab("signup")}
                            className="btn-primary"
                            style={{padding: "8px 18px"}}
                        >
                            <Sparkles size={15}/> 무료 시작하기
                        </a>
                    </div>
                </div>
            </header>

            {/* Hero Section */}
            <section className="landing-hero-section">
                <div className="landing-hero-content">
                    <div className="hero-badge">
                        <Zap size={14} className="hero-badge-icon"/>
                        <span>Built with Rust 2024 & Next.js 15</span>
                    </div>

                    <h1 className="hero-headline">
                        Where Speed Meets <br/>
                        <span className="gradient-text">Social Connection.</span>
                    </h1>

                    <p className="hero-subhead">
                        초고성능 Rust Axum 백엔드, 24시간 인스타그램 스타일 스토리, 실시간 1:1 다이렉트 메시지,
                        완벽한 프라이버시가 결합된 차세대 소셜 네트워크 <strong>Ferro</strong>에 오신 것을 환영합니다.
                    </p>

                    <div className="hero-cta-group">
                        <a href="#join" className="hero-cta-primary" onClick={() => setAuthTab("signup")}>
                            <span>지금 무료로 시작하기</span>
                            <ArrowRight size={18}/>
                        </a>
                        <Link href="/" className="hero-cta-secondary">
                            <Globe size={18}/>
                            <span>게스트로 피드 탐색</span>
                        </Link>
                    </div>

                    <div className="hero-metrics">
                        <div className="metric-item">
                            <div className="metric-val">&lt; 1ms</div>
                            <div className="metric-label">Rust API 응답 속도</div>
                        </div>
                        <div className="metric-divider"/>
                        <div className="metric-item">
                            <div className="metric-val">24h</div>
                            <div className="metric-label">스마트 자동 만료 스토리</div>
                        </div>
                        <div className="metric-divider"/>
                        <div className="metric-item">
                            <div className="metric-val">100%</div>
                            <div className="metric-label">Argon2id 암호화 보안</div>
                        </div>
                    </div>
                </div>

                {/* Hero Interactive Floating Visuals */}
                <div className="hero-visuals">
                    {/* Card 1: Interactive Story Preview */}
                    <div className="floating-card card-story">
                        <div className="floating-card-header">
                            <div className="story-avatar-wrapper unread"
                                 style={{width: 44, height: 44, padding: "2px"}}>
                                <div className="story-avatar-inner">
                                    <img
                                        src="https://api.dicebear.com/7.x/bottts/svg?seed=ferro_dev"
                                        alt="Dev"
                                        className="story-avatar-img"
                                    />
                                </div>
                            </div>
                            <div>
                                <div style={{fontWeight: 800, fontSize: "14px", color: "var(--text-primary)"}}>
                                    ferro_dev
                                </div>
                                <div style={{
                                    fontSize: "11px",
                                    color: "var(--accent-primary)",
                                    display: "flex",
                                    alignItems: "center",
                                    gap: "4px"
                                }}>
                                    <Flame size={12}/> 24시간 스토리 활성
                                </div>
                            </div>
                            <div className="live-pill">LIVE</div>
                        </div>
                        <div className="card-story-preview">
                            <img
                                src="https://images.unsplash.com/photo-1518770660439-4636190af475?w=500&q=80"
                                alt="Story Preview"
                                style={{width: "100%", height: "130px", objectFit: "cover", borderRadius: "10px"}}
                            />
                            <div className="card-story-caption">오늘의 Rust 풀스택 빌드 스냅샷 📸🦀</div>
                        </div>
                    </div>

                    {/* Card 2: Real-time Direct Message Bubble */}
                    <div className="floating-card card-dm">
                        <div className="floating-card-header">
                            <div style={{display: "flex", alignItems: "center", gap: "8px"}}>
                                <div style={{width: 8, height: 8, borderRadius: "50%", backgroundColor: "#10b981"}}/>
                                <span style={{fontSize: "13px", fontWeight: 700}}>1:1 실시간 메시지</span>
                            </div>
                            <span style={{fontSize: "11px", color: "var(--text-muted)"}}>방금 전</span>
                        </div>
                        <div style={{display: "flex", flexDirection: "column", gap: "8px", marginTop: "10px"}}>
                            <div className="chat-bubble theirs" style={{fontSize: "12px", padding: "8px 12px"}}>
                                Ferro 백엔드 GraphQL 쿼리 속도 정말 빠르네요! ⚡
                            </div>
                            <div className="chat-bubble mine" style={{fontSize: "12px", padding: "8px 12px"}}>
                                Axum + SQLx 풀링 덕분입니다 🚀 <Check size={12} style={{display: "inline", marginLeft: 4}}/>
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            {/* Interactive Registration / Login Section */}
            <section id="join" className="landing-auth-section">
                <div className="landing-auth-container">
                    <div className="landing-auth-intro">
                        <div className="hero-badge" style={{marginBottom: "16px"}}>
                            <Sparkles size={14} className="hero-badge-icon"/>
                            <span>간편하고 빠른 시작</span>
                        </div>
                        <h2 style={{fontSize: "32px", fontWeight: 800, letterSpacing: "-0.03em", marginBottom: "14px"}}>
                            지금 Ferro 커뮤니티에 <br/>
                            <span className="gradient-text">합류하세요.</span>
                        </h2>
                        <p style={{
                            color: "var(--text-secondary)",
                            fontSize: "15px",
                            lineHeight: "1.6",
                            marginBottom: "24px"
                        }}>
                            단 10초 만에 프로필을 생성하고 개발자, 크리에이터들과 실시간으로 연결되세요.
                            복잡한 인증 절차 없이 즉시 나만의 피드와 스토리를 시작할 수 있습니다.
                        </p>

                        <div className="auth-feature-list">
                            <div className="auth-feature-item">
                                <CheckCircle2 size={18} color="#38bdf8"/>
                                <span>커스텀 봇츠 아바타 1-클릭 생성</span>
                            </div>
                            <div className="auth-feature-item">
                                <CheckCircle2 size={18} color="#38bdf8"/>
                                <span>스토리 조회자 통계 & 1:1 비공개 채팅</span>
                            </div>
                            <div className="auth-feature-item">
                                <CheckCircle2 size={18} color="#38bdf8"/>
                                <span>팔로잉 / 글로벌 전체 피드 실시간 토글</span>
                            </div>
                        </div>
                    </div>

                    {/* Form Card */}
                    <div className="landing-auth-card">
                        {/* Tabs */}
                        <div className="landing-auth-tabs">
                            <button
                                type="button"
                                className={`landing-auth-tab ${authTab === "signup" ? "active" : ""}`}
                                onClick={() => setAuthTab("signup")}
                            >
                                회원가입 (Sign Up)
                            </button>
                            <button
                                type="button"
                                className={`landing-auth-tab ${authTab === "login" ? "active" : ""}`}
                                onClick={() => setAuthTab("login")}
                            >
                                로그인 (Sign In)
                            </button>
                        </div>

                        {/* Signup Form */}
                        {authTab === "signup" ? (
                            <form onSubmit={handleSignup} className="landing-form">
                                {/* Interactive Avatar Generator */}
                                <div className="avatar-picker-section">
                                    <div className="avatar-preview-box">
                                        <img src={currentAvatarUrl} alt="Avatar Preview"
                                             className="avatar-preview-img"/>
                                    </div>
                                    <div className="avatar-picker-info">
                                        <div style={{fontSize: "13px", fontWeight: 700, color: "var(--text-primary)"}}>
                                            프로필 아바타 선택
                                        </div>
                                        <div style={{display: "flex", gap: "6px", flexWrap: "wrap", marginTop: "6px"}}>
                                            {avatarSeeds.map((seed) => (
                                                <button
                                                    key={seed}
                                                    type="button"
                                                    onClick={() => setSignupAvatarSeed(seed)}
                                                    className={`seed-pill ${signupAvatarSeed === seed ? "active" : ""}`}
                                                >
                                                    {seed}
                                                </button>
                                            ))}
                                            <button
                                                type="button"
                                                onClick={randomizeSeed}
                                                className="seed-pill random"
                                                title="랜덤 아바타 생성"
                                            >
                                                <Dices size={12}/> 랜덤
                                            </button>
                                        </div>
                                    </div>
                                </div>

                                <div className="form-row-2">
                                    <div className="form-group">
                                        <label className="form-label">아이디 (Username) *</label>
                                        <div className="input-with-icon">
                                            <UserIcon size={16} className="input-icon"/>
                                            <input
                                                type="text"
                                                placeholder="ferro_dev"
                                                value={signupUsername}
                                                onChange={(e) => setSignupUsername(e.target.value)}
                                                className="form-input with-icon"
                                                required
                                            />
                                        </div>
                                    </div>

                                    <div className="form-group">
                                        <label className="form-label">표시 이름 (Display Name) *</label>
                                        <div className="input-with-icon">
                                            <Sparkles size={16} className="input-icon"/>
                                            <input
                                                type="text"
                                                placeholder="홍길동"
                                                value={signupDisplayName}
                                                onChange={(e) => setSignupDisplayName(e.target.value)}
                                                className="form-input with-icon"
                                                required
                                            />
                                        </div>
                                    </div>
                                </div>

                                <div className="form-group">
                                    <label className="form-label">이메일 (Email) *</label>
                                    <div className="input-with-icon">
                                        <Mail size={16} className="input-icon"/>
                                        <input
                                            type="email"
                                            placeholder="developer@ferro.dev"
                                            value={signupEmail}
                                            onChange={(e) => setSignupEmail(e.target.value)}
                                            className="form-input with-icon"
                                            required
                                        />
                                    </div>
                                </div>

                                <div className="form-group">
                                    <div style={{display: "flex", justifyContent: "space-between"}}>
                                        <label className="form-label">비밀번호 (Password) *</label>
                                        <span style={{fontSize: "11px", color: pwdStrength.color, fontWeight: 700}}>
                      {pwdStrength.label}
                    </span>
                                    </div>
                                    <div className="input-with-icon">
                                        <Lock size={16} className="input-icon"/>
                                        <input
                                            type={showSignupPassword ? "text" : "password"}
                                            placeholder="8자 이상 안전한 비밀번호"
                                            value={signupPassword}
                                            onChange={(e) => setSignupPassword(e.target.value)}
                                            className="form-input with-icon"
                                            required
                                        />
                                        <button
                                            type="button"
                                            onClick={() => setShowSignupPassword(!showSignupPassword)}
                                            className="input-eye-btn"
                                        >
                                            {showSignupPassword ? <EyeOff size={16}/> : <Eye size={16}/>}
                                        </button>
                                    </div>
                                    {/* Strength bar */}
                                    {signupPassword && (
                                        <div className="strength-meter-bar">
                                            <div
                                                className="strength-meter-fill"
                                                style={{
                                                    width: `${(pwdStrength.score / 4) * 100}%`,
                                                    backgroundColor: pwdStrength.color,
                                                }}
                                            />
                                        </div>
                                    )}
                                </div>

                                <div className="form-group">
                                    <label className="form-label">한 줄 소개 (Bio)</label>
                                    <input
                                        type="text"
                                        placeholder="Rust & Next.js 풀스택 개발자 🦀"
                                        value={signupBio}
                                        onChange={(e) => setSignupBio(e.target.value)}
                                        className="form-input"
                                    />
                                </div>

                                <button type="submit" disabled={loading} className="btn-auth-submit">
                                    {loading ? "계정 생성 중..." : "무료 계정 생성 완료 🚀"}
                                </button>
                            </form>
                        ) : (
                            /* Login Form */
                            <form onSubmit={handleLogin} className="landing-form">
                                <div className="form-group">
                                    <label className="form-label">아이디 또는 이메일</label>
                                    <div className="input-with-icon">
                                        <UserIcon size={16} className="input-icon"/>
                                        <input
                                            type="text"
                                            placeholder="아이디 또는 이메일 입력"
                                            value={loginUsername}
                                            onChange={(e) => setLoginUsername(e.target.value)}
                                            className="form-input with-icon"
                                            required
                                            autoFocus
                                        />
                                    </div>
                                </div>

                                <div className="form-group">
                                    <label className="form-label">비밀번호</label>
                                    <div className="input-with-icon">
                                        <Lock size={16} className="input-icon"/>
                                        <input
                                            type={showLoginPassword ? "text" : "password"}
                                            placeholder="비밀번호를 입력하세요"
                                            value={loginPassword}
                                            onChange={(e) => setLoginPassword(e.target.value)}
                                            className="form-input with-icon"
                                            required
                                        />
                                        <button
                                            type="button"
                                            onClick={() => setShowLoginPassword(!showLoginPassword)}
                                            className="input-eye-btn"
                                        >
                                            {showLoginPassword ? <EyeOff size={16}/> : <Eye size={16}/>}
                                        </button>
                                    </div>
                                </div>

                                <button type="submit" disabled={loading} className="btn-auth-submit">
                                    {loading ? "로그인 중..." : "Ferro 로그인 🔓"}
                                </button>
                            </form>
                        )}
                    </div>
                </div>
            </section>

            {/* Feature Pillars */}
            <section id="features" className="landing-section">
                <div className="landing-section-header">
                    <div className="hero-badge">
                        <Layers size={14} className="hero-badge-icon"/>
                        <span>CORE FEATURES</span>
                    </div>
                    <h2 className="section-title">소셜 경험의 모든 것을 완벽하게</h2>
                    <p className="section-subtitle">
                        빠른 속도, 세련된 인터랙션, 완벽한 사용자 제어권을 제공합니다.
                    </p>
                </div>

                <div className="feature-grid">
                    <div className="feature-card">
                        <div className="feature-icon-wrapper"
                             style={{background: "rgba(236, 72, 153, 0.15)", color: "#ec4899"}}>
                            <Camera size={26}/>
                        </div>
                        <h3 className="feature-card-title">24시간 자동 만료 스토리</h3>
                        <p className="feature-card-desc">
                            게시 후 24시간 동안만 유지되는 스토리 기능. 5초 자동 전환 뷰어, 실시간 시청자 카운트,
                            그리고 <strong>작성자 본인만 시청자 목록을 조회할 수 있는 보안 검증</strong>이 적용되어 있습니다.
                        </p>
                    </div>

                    <div className="feature-card">
                        <div className="feature-icon-wrapper"
                             style={{background: "rgba(56, 189, 248, 0.15)", color: "#38bdf8"}}>
                            <MessageCircle size={26}/>
                        </div>
                        <h3 className="feature-card-title">1:1 실시간 다이렉트 메시지</h3>
                        <p className="feature-card-desc">
                            대화 상대별 최근 메시지 스니펫, 안 읽은 메시지 뱃지, 실시간 읽음 확인(Read Receipts) 체크마크로
                            언제 어디서나 신속하고 안전하게 소통할 수 있습니다.
                        </p>
                    </div>

                    <div className="feature-card">
                        <div className="feature-icon-wrapper"
                             style={{background: "rgba(16, 185, 129, 0.15)", color: "#10b981"}}>
                            <Zap size={26}/>
                        </div>
                        <h3 className="feature-card-title">Rust Axum 기반 초고속 백엔드</h3>
                        <p className="feature-card-desc">
                            단일 프로세스에서 비동기 I/O를 극한으로 활용하는 Tokio & Axum 0.8 엔진.
                            SQLx 0.9 커넥션 풀링과 async-graphql 7.2로 sub-millisecond 쿼리 레이턴시를 보장합니다.
                        </p>
                    </div>

                    <div className="feature-card">
                        <div className="feature-icon-wrapper"
                             style={{background: "rgba(168, 85, 247, 0.15)", color: "#a855f7"}}>
                            <ShieldCheck size={26}/>
                        </div>
                        <h3 className="feature-card-title">기업급 보안 & 국제화 (i18n)</h3>
                        <p className="feature-card-desc">
                            Argon2id 메모리 하드 해싱과 무상태 JWT 토큰 인증.
                            백엔드 표준 도메인 에러 코드(ErrorCode)와 클라이언트 다국어 템플릿 보간 엔진을 탑재했습니다.
                        </p>
                    </div>
                </div>
            </section>

            {/* Tech Stack Matrix */}
            <section id="architecture" className="landing-section"
                     style={{borderTop: "1px solid var(--border-subtle)"}}>
                <div className="landing-section-header">
                    <div className="hero-badge">
                        <Cpu size={14} className="hero-badge-icon"/>
                        <span>MODERN TECH STACK</span>
                    </div>
                    <h2 className="section-title">타협 없는 기술적 완성도</h2>
                    <p className="section-subtitle">
                        최신 Rust 2024와 Next.js 15 App Router로 설계된 모던 풀스택 아키텍처
                    </p>
                </div>

                <div className="tech-stack-row">
                    <div className="tech-badge">🦀 Rust 2024</div>
                    <div className="tech-badge">⚡ Axum 0.8</div>
                    <div className="tech-badge">🌐 async-graphql 7.2</div>
                    <div className="tech-badge">🐘 PostgreSQL 16 Alpine</div>
                    <div className="tech-badge">🛡️ SQLx 0.9</div>
                    <div className="tech-badge">⚛️ Next.js 15 (React 19)</div>
                    <div className="tech-badge">🎨 Tailwind CSS</div>
                    <div className="tech-badge">🔒 Argon2id & JWT</div>
                </div>
            </section>

            {/* Footer */}
            <footer className="landing-footer">
                <div className="landing-footer-inner">
                    <div style={{display: "flex", alignItems: "center", gap: "10px"}}>
                        <div className="logo-badge" style={{width: 32, height: 32, fontSize: 16}}>🦀</div>
                        <span style={{fontWeight: 800, fontSize: "16px", color: "var(--text-primary)"}}>Ferro</span>
                        <span style={{color: "var(--text-muted)", fontSize: "13px"}}>© 2026 Ferro Social Universe. All rights reserved.</span>
                    </div>
                    <div style={{display: "flex", gap: "18px", fontSize: "13px", color: "var(--text-secondary)"}}>
                        <Link href="/" style={{color: "var(--accent-primary)"}}>홈 피드</Link>
                        <Link href="/explore">탐색</Link>
                        <a href="#join">회원가입</a>
                    </div>
                </div>
            </footer>
        </div>
    );
}

export default function WelcomePage() {
    return (
        <Suspense fallback={<div style={{padding: "40px", textAlign: "center"}}>Loading...</div>}>
            <WelcomeContent/>
        </Suspense>
    );
}
