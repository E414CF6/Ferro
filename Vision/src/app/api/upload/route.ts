import { NextRequest, NextResponse } from "next/server";

const BACKEND_URL =
    process.env.BACKEND_URL ||
    process.env.NEXT_PUBLIC_API_BASE_URL ||
    "http://127.0.0.1:8080";

export async function POST(req: NextRequest) {
    try {
        const authHeader = req.headers.get("authorization");
        const formData = await req.formData();

        const headers: Record<string, string> = {};
        if (authHeader) {
            headers["Authorization"] = authHeader;
        }

        const backendTarget = `${BACKEND_URL.replace(/\/+$/, "")}/api/upload`;

        const response = await fetch(backendTarget, {
            method: "POST",
            headers,
            body: formData,
        });

        const data = await response.json();
        return NextResponse.json(data, { status: response.status });
    } catch (error: any) {
        console.error("Upload Proxy Error:", error);
        return NextResponse.json(
            { error: error?.message || "Failed to proxy upload to backend" },
            { status: 502 }
        );
    }
}
