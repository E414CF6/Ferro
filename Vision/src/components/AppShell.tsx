"use client";

import React from "react";
import {usePathname} from "next/navigation";
import Sidebar from "./Sidebar";
import RightSidebar from "./RightSidebar";

export default function AppShell({children}: { children: React.ReactNode }) {
    const pathname = usePathname();

    // Standalone full-width routes (Landing / Login / Signup)
    const isStandalonePage =
        pathname === "/welcome" ||
        pathname === "/login" ||
        pathname === "/signup" ||
        pathname === "/landing";

    if (isStandalonePage) {
        return <div className="standalone-container">{children}</div>;
    }

    return (
        <div className="app-container">
            <Sidebar/>
            <main className="main-content">{children}</main>
            <RightSidebar/>
        </div>
    );
}
