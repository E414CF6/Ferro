"use client";

import {useEffect} from "react";
import {useRouter} from "next/navigation";

export default function LoginPage() {
    const router = useRouter();

    useEffect(() => {
        router.replace("/welcome?tab=login");
    }, [router]);

    return <div style={{padding: "40px", textAlign: "center", color: "var(--text-muted)"}}>로그인 페이지로 이동 중...</div>;
}
