"use client";

import React, { useEffect, useState } from "react";
import { usePathname, useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth-context";
import Sidebar from "./Sidebar";
import RightSidebar from "./RightSidebar";
import SplashScreen from "./SplashScreen";

export default function AppShell({ children }: { children: React.ReactNode }) {
    const pathname = usePathname();
    const router = useRouter();
    const { user, loading } = useAuth();

    const [hasSplashed, setHasSplashed] = useState(false);

    const isPublicPage =
        pathname === "/login" ||
        pathname === "/signup" ||
        pathname === "/welcome";

    // Authentication Guard: require an account to view internal content
    useEffect(() => {
        if (!loading && !user && !isPublicPage) {
            router.replace("/login");
        }
    }, [loading, user, isPublicPage, router]);

    return (
        <>
            {/* Splash screen shown on initial load */}
            {!hasSplashed && (
                <SplashScreen
                    onFinished={() => setHasSplashed(true)}
                    minDurationMs={800}
                />
            )}

            {isPublicPage ? (
                <div className="standalone-container">{children}</div>
            ) : (
                /* Only render internal app shell if user is authenticated */
                user ? (
                    <div className="app-container">
                        <Sidebar />
                        <main className="main-content">{children}</main>
                        <RightSidebar />
                    </div>
                ) : (
                    /* Show clean loading placeholder while redirecting to /login */
                    <div
                        style={{
                            minHeight: "100vh",
                            backgroundColor: "var(--bg-main, #070a12)",
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "center",
                        }}
                    />
                )
            )}
        </>
    );
}
