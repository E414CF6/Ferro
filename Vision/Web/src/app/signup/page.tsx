"use client";

import {useEffect} from "react";
import {useRouter} from "next/navigation";

export default function SignupRedirectPage() {
    const router = useRouter();

    useEffect(() => {
        router.replace("/login");
    }, [router]);

    return (
        <div
            style={{
                minHeight: "100vh",
                backgroundColor: "#000000",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
            }}
        />
    );
}
