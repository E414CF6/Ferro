import {AppGraphQLError, formatErrorMessage} from "../i18n";

export {AppGraphQLError, formatErrorMessage};

const GRAPHQL_ENDPOINT =
    process.env.NEXT_PUBLIC_GRAPHQL_ENDPOINT || "http://127.0.0.1:8080/graphql";

export async function fetchGraphQL<T = any>(
    query: string,
    variables: Record<string, any> = {},
    token?: string | null
): Promise<T> {
    const headers: Record<string, string> = {
        "Content-Type": "application/json",
    };

    const authToken =
        token ||
        (typeof window !== "undefined"
            ? localStorage.getItem("ferro_token")
            : null);
    if (authToken) {
        headers["Authorization"] = `Bearer ${authToken}`;
    }

    const res = await fetch(GRAPHQL_ENDPOINT, {
        method: "POST",
        headers,
        body: JSON.stringify({query, variables}),
        cache: "no-store",
    });

    const json = await res.json();

    if (json.errors && json.errors.length > 0) {
        const firstErr = json.errors[0];
        const code = firstErr.extensions?.code;
        const params = firstErr.extensions?.params;
        const localizedMsg = formatErrorMessage(firstErr, "ko");
        const error = new AppGraphQLError(localizedMsg, code, params);
        throw error;
    }

    return json.data;
}
