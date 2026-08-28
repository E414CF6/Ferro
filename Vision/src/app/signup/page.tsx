"use client";

import {useEffect} from "react";
import {useRouter} from "next/navigation";

export default function SignupPage() {
    const router = useRouter();

    useEffect(() => {
        router.replace("/welcome?tab=signup");
    }, [router]);

    return <div style={{padding: "40px", textAlign: "center", color: "var(--text-muted)"}}>회원가입 페이지로 이동 중...</div>;
}
