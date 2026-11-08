"use client";

import React, {useState} from "react";
import {useAuth} from "@/lib/auth-context";
import {fetchGraphQL, MUTATIONS} from "@/lib/graphql";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import {Check, Copy, Key, Lock, ShieldCheck, Unlock, X,} from "lucide-react";

interface TwoFactorModalProps {
    onClose: () => void;
    onStatusChanged?: () => void;
}

export default function TwoFactorModal({
                                           onClose,
                                           onStatusChanged,
                                       }: TwoFactorModalProps) {
    const {user} = useAuth();
    const {showToast} = useToast();

    const [step, setStep] = useState<"status" | "setup">("status");
    const [totpSecret, setTotpSecret] = useState("");
    const [otpauthUri, setOtpauthUri] = useState("");
    const [verificationCode, setVerificationCode] = useState("");
    const [disableCode, setDisableCode] = useState("");
    const [loading, setLoading] = useState(false);
    const [copied, setCopied] = useState(false);

    const handleStartSetup = async () => {
        setLoading(true);
        try {
            const data = await fetchGraphQL<{ setup2fa: { secret: string; otpauthUri: string } }>(
                MUTATIONS.SETUP_2FA
            );
            if (data?.setup2fa) {
                setTotpSecret(data.setup2fa.secret);
                setOtpauthUri(data.setup2fa.otpauthUri);
                setStep("setup");
            }
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLoading(false);
        }
    };

    const handleEnable2fa = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!verificationCode.trim()) return;

        setLoading(true);
        try {
            const success = await fetchGraphQL<{ enable2fa: boolean }>(
                MUTATIONS.ENABLE_2FA,
                {code: verificationCode.trim()}
            );

            if (success?.enable2fa) {
                showToast("2단계 인증(2FA)이 성공적으로 활성화되었습니다!", "success");
                onStatusChanged?.();
                onClose();
            } else {
                showToast("인증 코드가 올바르지 않습니다. 다시 시도해주세요.", "error");
            }
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLoading(false);
        }
    };

    const handleDisable2fa = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!disableCode.trim()) return;

        setLoading(true);
        try {
            const success = await fetchGraphQL<{ disable2fa: boolean }>(
                MUTATIONS.DISABLE_2FA,
                {code: disableCode.trim()}
            );

            if (success?.disable2fa) {
                showToast("2단계 인증(2FA)이 비활성화되었습니다.", "info");
                onStatusChanged?.();
                onClose();
            } else {
                showToast("인증 코드가 올바르지 않습니다. 다시 시도해주세요.", "error");
            }
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLoading(false);
        }
    };

    const handleCopySecret = () => {
        if (typeof window !== "undefined") {
            navigator.clipboard.writeText(totpSecret);
            setCopied(true);
            showToast("보안 비밀키가 클립보드에 복사되었습니다.", "success");
            setTimeout(() => setCopied(false), 2000);
        }
    };

    return (
        <div className="modal-overlay" onClick={onClose}>
            <div
                className="modal-content"
                onClick={(e) => e.stopPropagation()}
                style={{maxWidth: "480px"}}
            >
                <div className="modal-header">
                    <div style={{display: "flex", alignItems: "center", gap: "8px"}}>
                        <ShieldCheck size={20} color="var(--accent-primary)"/>
                        <h3 className="modal-title">2단계 인증 (TOTP 2FA) 설정</h3>
                    </div>
                    <button onClick={onClose} className="modal-close-btn">
                        <X size={18}/>
                    </button>
                </div>

                <div style={{padding: "20px"}}>
                    {step === "status" ? (
                        <div>
                            {user?.is2faEnabled ? (
                                <div>
                                    <div
                                        style={{
                                            padding: "16px",
                                            borderRadius: "var(--radius-md)",
                                            backgroundColor: "rgba(16, 185, 129, 0.1)",
                                            border: "1px solid rgba(16, 185, 129, 0.3)",
                                            display: "flex",
                                            alignItems: "center",
                                            gap: "12px",
                                            marginBottom: "20px",
                                        }}
                                    >
                                        <Lock size={24} color="#10b981"/>
                                        <div>
                                            <div
                                                style={{
                                                    fontWeight: 700,
                                                    fontSize: "14px",
                                                    color: "#10b981",
                                                }}
                                            >
                                                2단계 인증이 활성화되어 있습니다
                                            </div>
                                            <div
                                                style={{
                                                    fontSize: "12px",
                                                    color: "var(--text-secondary)",
                                                    marginTop: "2px",
                                                }}
                                            >
                                                계정이 안전하게 보호되고 있습니다.
                                            </div>
                                        </div>
                                    </div>

                                    <form onSubmit={handleDisable2fa}>
                                        <label
                                            style={{
                                                fontSize: "13px",
                                                fontWeight: 700,
                                                color: "var(--text-primary)",
                                                marginBottom: "6px",
                                                display: "block",
                                            }}
                                        >
                                            2FA를 해제하려면 OTP 앱의 6자리 인증 코드를 입력하세요
                                        </label>
                                        <input
                                            type="text"
                                            placeholder="000000"
                                            maxLength={6}
                                            value={disableCode}
                                            onChange={(e) => setDisableCode(e.target.value.replace(/\D/g, ""))}
                                            className="composer-textarea"
                                            style={{
                                                width: "100%",
                                                letterSpacing: "6px",
                                                textAlign: "center",
                                                fontSize: "20px",
                                                fontWeight: 800,
                                                padding: "10px",
                                                borderRadius: "var(--radius-md)",
                                                border: "1px solid var(--border-subtle)",
                                                marginBottom: "16px",
                                            }}
                                        />
                                        <div
                                            style={{
                                                display: "flex",
                                                justifyContent: "flex-end",
                                                gap: "8px",
                                            }}
                                        >
                                            <button
                                                type="button"
                                                onClick={onClose}
                                                className="btn-secondary"
                                            >
                                                닫기
                                            </button>
                                            <button
                                                type="submit"
                                                disabled={disableCode.length !== 6 || loading}
                                                className="btn-primary"
                                                style={{
                                                    backgroundColor: "var(--accent-secondary)",
                                                    borderColor: "var(--accent-secondary)",
                                                }}
                                            >
                                                <Unlock size={14}/>
                                                {loading ? "해제 중..." : "2FA 비활성화"}
                                            </button>
                                        </div>
                                    </form>
                                </div>
                            ) : (
                                <div>
                                    <div
                                        style={{
                                            padding: "16px",
                                            borderRadius: "var(--radius-md)",
                                            backgroundColor: "var(--bg-surface)",
                                            border: "1px solid var(--border-subtle)",
                                            marginBottom: "20px",
                                        }}
                                    >
                                        <p
                                            style={{
                                                fontSize: "13px",
                                                color: "var(--text-secondary)",
                                                lineHeight: "1.6",
                                            }}
                                        >
                                            Google Authenticator, Authy 등의 OTP 앱을 연동하여 로그인
                                            시 6자리 일회용 비밀번호를 요구함으로써 계정 보안을 한층 더
                                            강화할 수 있습니다.
                                        </p>
                                    </div>

                                    <div
                                        style={{
                                            display: "flex",
                                            justifyContent: "flex-end",
                                            gap: "8px",
                                        }}
                                    >
                                        <button
                                            type="button"
                                            onClick={onClose}
                                            className="btn-secondary"
                                        >
                                            닫기
                                        </button>
                                        <button
                                            type="button"
                                            onClick={handleStartSetup}
                                            disabled={loading}
                                            className="btn-primary"
                                        >
                                            <Key size={14}/>
                                            {loading ? "키 생성 중..." : "2FA 설정 시작하기"}
                                        </button>
                                    </div>
                                </div>
                            )}
                        </div>
                    ) : (
                        <form onSubmit={handleEnable2fa}>
                            <div
                                style={{
                                    fontSize: "13px",
                                    color: "var(--text-secondary)",
                                    lineHeight: "1.5",
                                    marginBottom: "16px",
                                }}
                            >
                                1. 인증 앱(Google Authenticator, 1Password 등)에 아래 보안 키를
                                직접 입력하거나 URL을 등록하세요.
                            </div>

                            {/* Secret Key Copy Box */}
                            <div
                                style={{
                                    display: "flex",
                                    alignItems: "center",
                                    justifyContent: "space-between",
                                    padding: "10px 14px",
                                    borderRadius: "var(--radius-md)",
                                    backgroundColor: "var(--bg-input)",
                                    border: "1px solid var(--border-subtle)",
                                    marginBottom: "16px",
                                }}
                            >
                                <code
                                    style={{
                                        fontSize: "14px",
                                        fontWeight: 700,
                                        color: "var(--accent-primary)",
                                        letterSpacing: "1px",
                                        wordBreak: "break-all",
                                    }}
                                >
                                    {totpSecret}
                                </code>
                                <button
                                    type="button"
                                    onClick={handleCopySecret}
                                    className="btn-secondary"
                                    style={{padding: "6px 10px", fontSize: "11px", flexShrink: 0}}
                                >
                                    {copied ? <Check size={13} color="#10b981"/> : <Copy size={13}/>}
                                    {copied ? "복사됨" : "복사"}
                                </button>
                            </div>

                            <div
                                style={{
                                    fontSize: "13px",
                                    color: "var(--text-secondary)",
                                    marginBottom: "8px",
                                }}
                            >
                                2. 앱에 표시되는 6자리 인증 코드를 입력하여 설정을 완료하세요:
                            </div>

                            <input
                                type="text"
                                placeholder="000000"
                                maxLength={6}
                                value={verificationCode}
                                onChange={(e) =>
                                    setVerificationCode(e.target.value.replace(/\D/g, ""))
                                }
                                className="composer-textarea"
                                style={{
                                    width: "100%",
                                    letterSpacing: "6px",
                                    textAlign: "center",
                                    fontSize: "22px",
                                    fontWeight: 800,
                                    padding: "10px",
                                    borderRadius: "var(--radius-md)",
                                    border: "1px solid var(--border-subtle)",
                                    marginBottom: "20px",
                                }}
                                autoFocus
                            />

                            <div
                                style={{
                                    display: "flex",
                                    justifyContent: "flex-end",
                                    gap: "8px",
                                }}
                            >
                                <button
                                    type="button"
                                    onClick={() => setStep("status")}
                                    className="btn-secondary"
                                >
                                    이전
                                </button>
                                <button
                                    type="submit"
                                    disabled={verificationCode.length !== 6 || loading}
                                    className="btn-primary"
                                >
                                    <Check size={14}/>
                                    {loading ? "인증 중..." : "인증 및 활성화"}
                                </button>
                            </div>
                        </form>
                    )}
                </div>
            </div>
        </div>
    );
}
