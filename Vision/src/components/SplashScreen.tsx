"use client";

import React, {useEffect, useState} from "react";

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
                backgroundColor: "#000000",
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "center",
                opacity: fade ? 0 : 1,
                transform: fade ? "scale(1.08)" : "scale(1)",
                transition: "opacity 0.35s ease-out, transform 0.35s ease-out",
                pointerEvents: fade ? "none" : "all",
            }}
        >
            {/* Center Geometric Wireframe 'F' Logo (from Login Page) */}
            <div
                style={{
                    width: "96px",
                    height: "96px",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    animation: "splashPulse 1.6s ease-in-out infinite",
                    filter: "drop-shadow(0 0 24px rgba(255, 255, 255, 0.15))",
                }}
            >
                <svg
                    viewBox="0 0 600 600"
                    width="100%"
                    height="100%"
                    fill="none"
                    xmlns="http://www.w3.org/2000/svg"
                    aria-hidden="true"
                >
                    <defs>
                        {/* Metallic Chrome Stroke Gradients */}
                        <linearGradient id="splashChromeOuter" x1="0%" y1="0%" x2="100%" y2="100%">
                            <stop offset="0%" stopColor="#ffffff" stopOpacity="0.95"/>
                            <stop offset="25%" stopColor="#71767b" stopOpacity="0.8"/>
                            <stop offset="45%" stopColor="#1e2229" stopOpacity="0.85"/>
                            <stop offset="70%" stopColor="#cbd5e1" stopOpacity="0.9"/>
                            <stop offset="100%" stopColor="#ffffff" stopOpacity="0.95"/>
                        </linearGradient>
                        <linearGradient id="splashChromeInner" x1="100%" y1="0%" x2="0%" y2="100%">
                            <stop offset="0%" stopColor="#ffffff" stopOpacity="0.85"/>
                            <stop offset="30%" stopColor="#475569" stopOpacity="0.7"/>
                            <stop offset="55%" stopColor="#0f172a" stopOpacity="0.9"/>
                            <stop offset="85%" stopColor="#94a3b8" stopOpacity="0.8"/>
                            <stop offset="100%" stopColor="#ffffff" stopOpacity="0.85"/>
                        </linearGradient>
                    </defs>

                    {/* Outer Geometric Wireframe F Outline */}
                    <path
                        d="M120 70 L480 70 L460 160 L240 160 L230 250 L410 250 L390 330 L220 330 L195 530 L105 530 Z"
                        stroke="url(#splashChromeOuter)"
                        strokeWidth="6"
                        strokeLinejoin="miter"
                        strokeMiterlimit="4"
                        opacity="0.95"
                    />

                    {/* Inner Parallel Precision Wireframe F Contour */}
                    <path
                        d="M145 95 L445 95 L433 140 L260 140 L250 270 L380 270 L368 310 L240 310 L215 505 L135 505 Z"
                        stroke="url(#splashChromeInner)"
                        strokeWidth="3"
                        strokeLinejoin="miter"
                        strokeMiterlimit="4"
                        opacity="0.75"
                    />

                    {/* Bevel Chamfer Connectors */}
                    <line x1="120" y1="70" x2="145" y2="95" stroke="url(#splashChromeOuter)" strokeWidth="2.5"
                          opacity="0.6"/>
                    <line x1="480" y1="70" x2="445" y2="95" stroke="url(#splashChromeOuter)" strokeWidth="2.5"
                          opacity="0.6"/>
                    <line x1="460" y1="160" x2="433" y2="140" stroke="url(#splashChromeOuter)" strokeWidth="2.5"
                          opacity="0.6"/>
                    <line x1="240" y1="160" x2="260" y2="140" stroke="url(#splashChromeOuter)" strokeWidth="2.5"
                          opacity="0.6"/>
                    <line x1="230" y1="250" x2="250" y2="270" stroke="url(#splashChromeOuter)" strokeWidth="2.5"
                          opacity="0.6"/>
                    <line x1="410" y1="250" x2="380" y2="270" stroke="url(#splashChromeOuter)" strokeWidth="2.5"
                          opacity="0.6"/>
                    <line x1="390" y1="330" x2="368" y2="310" stroke="url(#splashChromeOuter)" strokeWidth="2.5"
                          opacity="0.6"/>
                    <line x1="220" y1="330" x2="240" y2="310" stroke="url(#splashChromeOuter)" strokeWidth="2.5"
                          opacity="0.6"/>
                    <line x1="195" y1="530" x2="215" y2="505" stroke="url(#splashChromeOuter)" strokeWidth="2.5"
                          opacity="0.6"/>
                    <line x1="105" y1="530" x2="135" y2="505" stroke="url(#splashChromeOuter)" strokeWidth="2.5"
                          opacity="0.6"/>
                </svg>
            </div>

            <style jsx>{`
                @keyframes splashPulse {
                    0%, 100% {
                        transform: scale(1);
                        opacity: 0.95;
                    }
                    50% {
                        transform: scale(1.05);
                        opacity: 1;
                    }
                }
            `}</style>
        </div>
    );
}
