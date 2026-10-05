"use client";

import React, { useEffect, useState } from "react";

interface SplashScreenProps {
    onFinished?: () => void;
    minDurationMs?: number;
}

export default function SplashScreen({
    onFinished,
    minDurationMs = 800,
}: SplashScreenProps) {
    const [fade, setFade] = useState(false);
    const [removed, setRemoved] = useState(false);

    useEffect(() => {
        const timer = setTimeout(() => {
            setFade(true);
            const removeTimer = setTimeout(() => {
                setRemoved(true);
                onFinished?.();
            }, 350);
            return () => clearTimeout(removeTimer);
        }, minDurationMs);

        return () => clearTimeout(timer);
    }, [minDurationMs, onFinished]);

    if (removed) return null;

    return (
        <div
            style={{
                position: "fixed",
                inset: 0,
                zIndex: 99999,
                backgroundColor: "var(--bg-main, #070a12)",
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "center",
                opacity: fade ? 0 : 1,
                transition: "opacity 0.35s ease-out",
                pointerEvents: fade ? "none" : "all",
            }}
        >
            {/* Background Radial Glow */}
            <div
                style={{
                    position: "absolute",
                    width: "360px",
                    height: "360px",
                    borderRadius: "50%",
                    background: "radial-gradient(circle, rgba(56, 189, 248, 0.15) 0%, rgba(168, 85, 247, 0.08) 50%, transparent 70%)",
                    filter: "blur(40px)",
                    pointerEvents: "none",
                }}
            />

            {/* Logo Badge */}
            <div
                style={{
                    width: "80px",
                    height: "80px",
                    borderRadius: "22px",
                    background: "linear-gradient(135deg, rgba(56, 189, 248, 0.2) 0%, rgba(168, 85, 247, 0.2) 100%)",
                    border: "1px solid rgba(255, 255, 255, 0.15)",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    fontSize: "40px",
                    boxShadow: "0 0 32px rgba(56, 189, 248, 0.3)",
                    marginBottom: "20px",
                    animation: "splashFloat 2s ease-in-out infinite",
                }}
            >
                🦀
            </div>

            {/* Brand Title */}
            <h1
                style={{
                    fontSize: "32px",
                    fontWeight: 800,
                    color: "var(--text-primary, #f8fafc)",
                    letterSpacing: "-0.8px",
                    margin: 0,
                }}
            >
                Ferro
            </h1>

            {/* Minimal Loading Indicator */}
            <div
                style={{
                    marginTop: "32px",
                    width: "120px",
                    height: "3px",
                    borderRadius: "9999px",
                    backgroundColor: "rgba(255, 255, 255, 0.08)",
                    overflow: "hidden",
                    position: "relative",
                }}
            >
                <div
                    style={{
                        position: "absolute",
                        top: 0,
                        bottom: 0,
                        width: "45%",
                        background: "linear-gradient(90deg, #38bdf8, #a855f7)",
                        borderRadius: "9999px",
                        animation: "splashProgress 1.2s ease-in-out infinite",
                    }}
                />
            </div>

            <style jsx>{`
                @keyframes splashFloat {
                    0%, 100% {
                        transform: translateY(0px) scale(1);
                    }
                    50% {
                        transform: translateY(-6px) scale(1.02);
                    }
                }
                @keyframes splashProgress {
                    0% {
                        left: -45%;
                    }
                    50% {
                        left: 45%;
                    }
                    100% {
                        left: 100%;
                    }
                }
            `}</style>
        </div>
    );
}
