import "./globals.css";

import type {Metadata} from "next";

import {AuthProvider} from "@/lib/auth-context";
import {ToastProvider} from "@/lib/toast-context";
import AppShell from "@/components/AppShell";

export const metadata: Metadata = {
    title: "Ferro — Stay Connected & Express in Real-Time",
    description: "Modern, high-performance social networking platform built with Rust and Next.js 15.",
};

export default function RootLayout({
                                       children,
                                   }: Readonly<{
    children: React.ReactNode;
}>) {
    return (
        <html lang="ko">
        <body>
        <AuthProvider>
            <ToastProvider>
                <AppShell>{children}</AppShell>
            </ToastProvider>
        </AuthProvider>
        </body>
        </html>
    );
}
