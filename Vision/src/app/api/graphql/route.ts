import { NextRequest, NextResponse } from "next/server";

const BACKEND_URL =
    process.env.BACKEND_URL ||
    process.env.NEXT_PUBLIC_API_BASE_URL ||
    "http://127.0.0.1:8080";

export async function POST(req: NextRequest) {
    try {
        const body = await req.json();
        const authHeader = req.headers.get("authorization");
        const clientIp = req.headers.get("x-forwarded-for") || req.headers.get("x-real-ip");

        const headers: Record<string, string> = {
            "Content-Type": "application/json",
        };

        if (authHeader) {
            headers["Authorization"] = authHeader;
        }
        if (clientIp) {
            headers["X-Forwarded-For"] = clientIp;
        }

        const backendTarget = BACKEND_URL.endsWith("/graphql")
            ? BACKEND_URL
            : `${BACKEND_URL.replace(/\/+$/, "")}/graphql`;

        const response = await fetch(backendTarget, {
            method: "POST",
            headers,
            body: JSON.stringify(body),
            cache: "no-store",
        });

        const data = await response.json();
        return NextResponse.json(data, {
            status: response.status,
            headers: {
                "Cache-Control": "no-store, no-cache, must-revalidate",
            },
        });
    } catch (error: any) {
        console.error("GraphQL Proxy Error:", error);
        return NextResponse.json(
            {
                errors: [
                    {
                        message: error?.message || "Failed to proxy GraphQL request to backend",
                        extensions: { code: "BACKEND_UNAVAILABLE" },
                    },
                ],
            },
            { status: 502 }
        );
    }
}
