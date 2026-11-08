"use client";

import React, {useState} from "react";
import {ReportReason, ReportTargetType} from "@/lib/types";
import {fetchGraphQL, MUTATIONS} from "@/lib/graphql";
import {useToast} from "@/lib/toast-context";
import {formatErrorMessage} from "@/lib/i18n";
import {Send, ShieldAlert, X} from "lucide-react";

interface ReportModalProps {
    targetType: ReportTargetType;
    targetId: string;
    targetTitle?: string;
    onClose: () => void;
}

const REASONS: { value: ReportReason; label: string; desc: string }[] = [
    {
        value: "SPAM",
        label: "스팸 및 광고",
        desc: "원치 않는 상업적 광고, 사기, 도배 게시물",
    },
    {
        value: "HARASSMENT",
        label: "괴롭힘 및 사이버 불링",
        desc: "타인을 비방하거나 지속적으로 괴롭히는 행위",
    },
    {
        value: "HATE_SPEECH",
        label: "혐오 발언",
        desc: "특정 집단에 대한 차별, 혐오 또는 폭력 조장",
    },
    {
        value: "INAPPROPRIATE",
        label: "부적절한 콘텐츠",
        desc: "선정적이거나 폭력적인 콘텐츠",
    },
    {
        value: "COPYRIGHT",
        label: "저작권 침해",
        desc: "무단 도용 또는 지식재산권 침해",
    },
    {
        value: "OTHER",
        label: "기타 사유",
        desc: "커뮤니티 가이드라인을 위반하는 기타 사유",
    },
];

export default function ReportModal({
                                        targetType,
                                        targetId,
                                        targetTitle,
                                        onClose,
                                    }: ReportModalProps) {
    const {showToast} = useToast();
    const [selectedReason, setSelectedReason] = useState<ReportReason>("SPAM");
    const [details, setDetails] = useState("");
    const [loading, setLoading] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setLoading(true);

        try {
            await fetchGraphQL(MUTATIONS.REPORT_CONTENT, {
                targetType,
                targetId,
                reason: selectedReason,
                details: details.trim() || undefined,
            });

            showToast(
                "신고가 접수되었습니다. 커뮤니티 가이드라인에 따라 신속히 검토하겠습니다.",
                "success"
            );
            onClose();
        } catch (err: any) {
            showToast(formatErrorMessage(err, "ko"), "error");
        } finally {
            setLoading(false);
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
                        <ShieldAlert size={20} color="var(--accent-secondary)"/>
                        <h3 className="modal-title">콘텐츠 신고하기</h3>
                    </div>
                    <button onClick={onClose} className="modal-close-btn">
                        <X size={18}/>
                    </button>
                </div>

                <form onSubmit={handleSubmit} style={{padding: "20px"}}>
                    {targetTitle && (
                        <div
                            style={{
                                padding: "10px 14px",
                                backgroundColor: "var(--bg-input)",
                                borderRadius: "var(--radius-md)",
                                fontSize: "12px",
                                color: "var(--text-secondary)",
                                marginBottom: "16px",
                                borderLeft: "3px solid var(--accent-secondary)",
                            }}
                        >
                            신고 대상: <strong>{targetTitle}</strong>
                        </div>
                    )}

                    <label
                        style={{
                            fontSize: "13px",
                            fontWeight: 700,
                            color: "var(--text-primary)",
                            marginBottom: "8px",
                            display: "block",
                        }}
                    >
                        신고 사유를 선택해주세요
                    </label>

                    <div
                        style={{
                            display: "flex",
                            flexDirection: "column",
                            gap: "8px",
                            marginBottom: "16px",
                        }}
                    >
                        {REASONS.map((r) => (
                            <label
                                key={r.value}
                                style={{
                                    display: "flex",
                                    alignItems: "flex-start",
                                    gap: "10px",
                                    padding: "10px 12px",
                                    borderRadius: "var(--radius-md)",
                                    border: `1px solid ${
                                        selectedReason === r.value
                                            ? "var(--accent-secondary)"
                                            : "var(--border-subtle)"
                                    }`,
                                    backgroundColor:
                                        selectedReason === r.value
                                            ? "rgba(244, 63, 94, 0.08)"
                                            : "var(--bg-surface)",
                                    cursor: "pointer",
                                    transition: "all var(--transition-fast)",
                                }}
                            >
                                <input
                                    type="radio"
                                    name="report_reason"
                                    value={r.value}
                                    checked={selectedReason === r.value}
                                    onChange={() => setSelectedReason(r.value)}
                                    style={{marginTop: "3px", accentColor: "var(--accent-secondary)"}}
                                />
                                <div>
                                    <div
                                        style={{
                                            fontSize: "13px",
                                            fontWeight: 700,
                                            color: "var(--text-primary)",
                                        }}
                                    >
                                        {r.label}
                                    </div>
                                    <div style={{fontSize: "11px", color: "var(--text-muted)"}}>
                                        {r.desc}
                                    </div>
                                </div>
                            </label>
                        ))}
                    </div>

                    <label
                        style={{
                            fontSize: "13px",
                            fontWeight: 700,
                            color: "var(--text-primary)",
                            marginBottom: "6px",
                            display: "block",
                        }}
                    >
                        추가 설명 (선택사항)
                    </label>
                    <textarea
                        value={details}
                        onChange={(e) => setDetails(e.target.value)}
                        placeholder="검토에 도움이 될 상세 내용을 입력해주세요..."
                        rows={3}
                        className="composer-textarea"
                        style={{
                            width: "100%",
                            borderRadius: "var(--radius-md)",
                            marginBottom: "16px",
                            border: "1px solid var(--border-subtle)",
                            padding: "10px",
                            fontSize: "13px",
                        }}
                    />

                    <div style={{display: "flex", justifyContent: "flex-end", gap: "8px"}}>
                        <button type="button" onClick={onClose} className="btn-secondary">
                            취소
                        </button>
                        <button
                            type="submit"
                            disabled={loading}
                            className="btn-primary"
                            style={{
                                backgroundColor: "var(--accent-secondary)",
                                borderColor: "var(--accent-secondary)",
                            }}
                        >
                            <Send size={13}/>
                            {loading ? "신고 접수 중..." : "신고 제출"}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
}
