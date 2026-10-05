const GRAPHQL_ENDPOINT =
    process.env.NEXT_PUBLIC_GRAPHQL_ENDPOINT || "http://127.0.0.1:8080/graphql";

export const API_BASE_URL =
    process.env.NEXT_PUBLIC_API_BASE_URL ||
    GRAPHQL_ENDPOINT.replace(/\/graphql\/?$/, "");

export interface UploadResponse {
    url: string;
    key: string;
    size: number;
    content_type: string;
}

/**
 * Uploads an image or media asset to Serve's /api/upload endpoint.
 * Validates file size (max 5MB) and authentic magic-byte file signature.
 * Returns the fully qualified public URL for the uploaded asset.
 */
export async function uploadMedia(file: File, token?: string | null): Promise<string> {
    if (!file) {
        throw new Error("업로드할 파일이 지정되지 않았습니다.");
    }

    if (file.size > 5 * 1024 * 1024) {
        throw new Error("파일 크기는 최대 5MB까지 업로드할 수 있습니다.");
    }

    const authToken =
        token ||
        (typeof window !== "undefined"
            ? localStorage.getItem("ferro_token")
            : null);

    if (!authToken) {
        throw new Error("미디어를 업로드하려면 로그인이 필요합니다.");
    }

    const formData = new FormData();
    formData.append("file", file);

    const uploadUrl = `${API_BASE_URL}/api/upload`;
    const res = await fetch(uploadUrl, {
        method: "POST",
        headers: {
            Authorization: `Bearer ${authToken}`,
        },
        body: formData,
    });

    if (!res.ok) {
        let errMessage = "파일 업로드에 실패했습니다.";
        try {
            const errJson = await res.json();
            if (errJson.message) {
                errMessage = errJson.message;
            } else if (errJson.error) {
                errMessage = errJson.error;
            }
        } catch {
            // fallback
        }
        throw new Error(errMessage);
    }

    const data: UploadResponse = await res.json();
    if (!data.url) {
        throw new Error("업로드 결과 URL이 올바르지 않습니다.");
    }

    if (data.url.startsWith("http://") || data.url.startsWith("https://")) {
        return data.url;
    }

    return `${API_BASE_URL}${data.url.startsWith("/") ? "" : "/"}${data.url}`;
}
