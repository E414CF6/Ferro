export type SupportedLocale = "ko" | "en";

export type ErrorCode =
    | "AUTH_INVALID_CREDENTIALS"
    | "AUTH_TOKEN_REQUIRED"
    | "AUTH_TOKEN_INVALID"
    | "AUTH_USER_ID_INVALID"
    | "AUTH_USER_ID_OR_TOKEN_REQUIRED"
    | "AUTH_HASHING_FAILED"
    | "AUTH_TOKEN_GENERATION_FAILED"
    | "AUTH_USER_ALREADY_EXISTS"
    | "AUTH_INVALID_USERNAME"
    | "AUTH_INVALID_EMAIL"
    | "AUTH_PASSWORD_TOO_SHORT"
    | "USER_NOT_FOUND"
    | "USER_CANNOT_FOLLOW_SELF"
    | "USER_PROFILE_UPDATE_FAILED"
    | "USER_FOLLOW_FAILED"
    | "USER_UNFOLLOW_FAILED"
    | "STORY_NOT_FOUND"
    | "STORY_AUTHOR_NOT_FOUND"
    | "STORY_VIEWERS_AUTH_REQUIRED"
    | "STORY_VIEWERS_UNAUTHORIZED"
    | "STORY_CREATE_FAILED"
    | "STORY_DELETE_FAILED"
    | "STORY_VIEW_FAILED"
    | "POST_NOT_FOUND"
    | "POST_AUTHOR_NOT_FOUND"
    | "POST_CREATE_FAILED"
    | "POST_DELETE_FAILED"
    | "POST_LIKE_FAILED"
    | "POST_UNLIKE_FAILED"
    | "POST_CONTENT_INVALID"
    | "COMMENT_NOT_FOUND"
    | "COMMENT_AUTHOR_NOT_FOUND"
    | "COMMENT_CREATE_FAILED"
    | "COMMENT_CONTENT_INVALID"
    | "DM_CONVERSATIONS_AUTH_REQUIRED"
    | "DM_MESSAGES_AUTH_REQUIRED"
    | "DM_CANNOT_SEND_TO_SELF"
    | "DM_SEND_FAILED"
    | "DM_MARK_READ_FAILED"
    | "DM_SENDER_NOT_FOUND"
    | "DM_RECIPIENT_NOT_FOUND"
    | "DM_CONTENT_INVALID"
    | "ERROR_NOT_FOUND"
    | "ERROR_BAD_REQUEST"
    | "ERROR_UNAUTHENTICATED"
    | "ERROR_FORBIDDEN"
    | "ERROR_CONFLICT"
    | "ERROR_INTERNAL";

export class AppGraphQLError extends Error {
    code: ErrorCode;
    params: Record<string, string>;

    constructor(message: string, code: ErrorCode = "ERROR_INTERNAL", params: Record<string, string> = {}) {
        super(message);
        this.name = "AppGraphQLError";
        this.code = code;
        this.params = params;
    }
}

export const ERROR_MESSAGES: Record<SupportedLocale, Record<string, string>> = {
    ko: {
        AUTH_INVALID_CREDENTIALS: "아이디(이메일) 또는 비밀번호가 올바르지 않습니다.",
        AUTH_TOKEN_REQUIRED: "인증이 필요합니다. 로그인 후 다시 시도해주세요.",
        AUTH_TOKEN_INVALID: "인증 토큰이 만료되었거나 유효하지 않습니다.",
        AUTH_USER_ID_INVALID: "토큰의 사용자 정보가 올바르지 않습니다.",
        AUTH_USER_ID_OR_TOKEN_REQUIRED: "인증 정보 또는 사용자 ID가 필요합니다.",
        AUTH_HASHING_FAILED: "비밀번호 암호화에 실패했습니다.",
        AUTH_TOKEN_GENERATION_FAILED: "인증 토큰 발급에 실패했습니다.",
        AUTH_USER_ALREADY_EXISTS: "이미 등록된 사용자 이름({username}) 또는 이메일({email})입니다.",
        AUTH_INVALID_USERNAME: "사용자 아이디는 3~50자의 영문, 숫자, 밑줄(_)만 사용할 수 있습니다.",
        AUTH_INVALID_EMAIL: "유효한 이메일 주소 형식을 입력해주세요.",
        AUTH_PASSWORD_TOO_SHORT: "비밀번호는 최소 8자 이상이어야 합니다.",

        USER_NOT_FOUND: "사용자를 찾을 수 없습니다.",
        USER_CANNOT_FOLLOW_SELF: "자기 자신을 팔로우할 수 없습니다.",
        USER_PROFILE_UPDATE_FAILED: "프로필 수정에 실패했습니다: {detail}",
        USER_FOLLOW_FAILED: "팔로우 처리에 실패했습니다: {detail}",
        USER_UNFOLLOW_FAILED: "언팔로우 처리에 실패했습니다: {detail}",

        STORY_NOT_FOUND: "해당 스토리를 찾을 수 없습니다.",
        STORY_AUTHOR_NOT_FOUND: "스토리 작성자를 찾을 수 없습니다.",
        STORY_VIEWERS_AUTH_REQUIRED: "스토리 조회자 목록을 보려면 로그인이 필요합니다.",
        STORY_VIEWERS_UNAUTHORIZED: "스토리 작성자만 조회자 목록을 확인할 수 있습니다.",
        STORY_CREATE_FAILED: "스토리 생성에 실패했습니다: {detail}",
        STORY_DELETE_FAILED: "스토리 삭제에 실패했습니다: {detail}",
        STORY_VIEW_FAILED: "스토리 조회 기록에 실패했습니다: {detail}",

        POST_NOT_FOUND: "게시물을 찾을 수 없습니다.",
        POST_AUTHOR_NOT_FOUND: "게시물 작성자를 찾을 수 없습니다.",
        POST_CREATE_FAILED: "게시물 작성에 실패했습니다: {detail}",
        POST_DELETE_FAILED: "게시물 삭제에 실패했습니다: {detail}",
        POST_LIKE_FAILED: "좋아요 처리에 실패했습니다: {detail}",
        POST_UNLIKE_FAILED: "좋아요 취소에 실패했습니다: {detail}",
        POST_CONTENT_INVALID: "게시물 내용을 입력해주세요. (최대 2,000자)",

        COMMENT_NOT_FOUND: "댓글을 찾을 수 없습니다.",
        COMMENT_AUTHOR_NOT_FOUND: "댓글 작성자를 찾을 수 없습니다.",
        COMMENT_CREATE_FAILED: "댓글 작성에 실패했습니다: {detail}",
        COMMENT_CONTENT_INVALID: "댓글 내용을 입력해주세요. (최대 500자)",

        DM_CONVERSATIONS_AUTH_REQUIRED: "메시지 대화 목록을 불러오려면 로그인이 필요합니다.",
        DM_MESSAGES_AUTH_REQUIRED: "메시지 목록을 보려면 로그인이 필요합니다.",
        DM_CANNOT_SEND_TO_SELF: "자기 자신에게는 다이렉트 메시지를 보낼 수 없습니다.",
        DM_SEND_FAILED: "메시지 전송에 실패했습니다: {detail}",
        DM_MARK_READ_FAILED: "메시지 읽음 처리에 실패했습니다: {detail}",
        DM_SENDER_NOT_FOUND: "메시지 발신자를 찾을 수 없습니다.",
        DM_RECIPIENT_NOT_FOUND: "메시지 수신자를 찾을 수 없습니다.",
        DM_CONTENT_INVALID: "메시지 내용을 입력해주세요. (최대 2,000자)",

        ERROR_NOT_FOUND: "요청하신 리소스를 찾을 수 없습니다.",
        ERROR_BAD_REQUEST: "잘못된 요청입니다: {detail}",
        ERROR_UNAUTHENTICATED: "로그인이 필요한 요청입니다.",
        ERROR_FORBIDDEN: "접근 권한이 없습니다.",
        ERROR_CONFLICT: "데이터 충돌이 발생했습니다.",
        ERROR_INTERNAL: "서버 내부 오류가 발생했습니다. 잠시 후 다시 시도해주세요.",
    }, en: {
        AUTH_INVALID_CREDENTIALS: "Invalid username (or email) or password.",
        AUTH_TOKEN_REQUIRED: "Authentication required. Please sign in.",
        AUTH_TOKEN_INVALID: "Your session has expired or is invalid.",
        AUTH_USER_ID_INVALID: "User ID in the session token is invalid.",
        AUTH_USER_ID_OR_TOKEN_REQUIRED: "Authentication token or User ID is required.",
        AUTH_HASHING_FAILED: "Failed to securely hash password.",
        AUTH_TOKEN_GENERATION_FAILED: "Failed to generate authorization token.",
        AUTH_USER_ALREADY_EXISTS: "A user with username ({username}) or email ({email}) already exists.",
        AUTH_INVALID_USERNAME: "Username must be 3-50 alphanumeric characters or underscores.",
        AUTH_INVALID_EMAIL: "Please enter a valid email address.",
        AUTH_PASSWORD_TOO_SHORT: "Password must be at least 8 characters long.",

        USER_NOT_FOUND: "User not found.",
        USER_CANNOT_FOLLOW_SELF: "You cannot follow yourself.",
        USER_PROFILE_UPDATE_FAILED: "Failed to update profile: {detail}",
        USER_FOLLOW_FAILED: "Failed to follow user: {detail}",
        USER_UNFOLLOW_FAILED: "Failed to unfollow user: {detail}",

        STORY_NOT_FOUND: "Story not found.",
        STORY_AUTHOR_NOT_FOUND: "Story author not found.",
        STORY_VIEWERS_AUTH_REQUIRED: "Authentication required to view story viewers.",
        STORY_VIEWERS_UNAUTHORIZED: "Only the story author can inspect the viewer list.",
        STORY_CREATE_FAILED: "Failed to create story: {detail}",
        STORY_DELETE_FAILED: "Failed to delete story: {detail}",
        STORY_VIEW_FAILED: "Failed to record story view: {detail}",

        POST_NOT_FOUND: "Post not found.",
        POST_AUTHOR_NOT_FOUND: "Post author not found.",
        POST_CREATE_FAILED: "Failed to publish post: {detail}",
        POST_DELETE_FAILED: "Failed to delete post: {detail}",
        POST_LIKE_FAILED: "Failed to like post: {detail}",
        POST_UNLIKE_FAILED: "Failed to unlike post: {detail}",
        POST_CONTENT_INVALID: "Post content cannot be empty (max 2,000 characters).",

        COMMENT_NOT_FOUND: "Comment not found.",
        COMMENT_AUTHOR_NOT_FOUND: "Comment author not found.",
        COMMENT_CREATE_FAILED: "Failed to post comment: {detail}",
        COMMENT_CONTENT_INVALID: "Comment content cannot be empty (max 500 characters).",

        DM_CONVERSATIONS_AUTH_REQUIRED: "Sign in required to view direct conversations.",
        DM_MESSAGES_AUTH_REQUIRED: "Sign in required to view messages.",
        DM_CANNOT_SEND_TO_SELF: "You cannot send direct messages to yourself.",
        DM_SEND_FAILED: "Failed to send message: {detail}",
        DM_MARK_READ_FAILED: "Failed to mark messages as read: {detail}",
        DM_SENDER_NOT_FOUND: "Message sender not found.",
        DM_RECIPIENT_NOT_FOUND: "Message recipient not found.",
        DM_CONTENT_INVALID: "Direct message cannot be empty (max 2,000 characters).",

        ERROR_NOT_FOUND: "Requested resource was not found.",
        ERROR_BAD_REQUEST: "Invalid request: {detail}",
        ERROR_UNAUTHENTICATED: "Authentication is required for this action.",
        ERROR_FORBIDDEN: "You do not have permission to access this resource.",
        ERROR_CONFLICT: "A data conflict occurred.",
        ERROR_INTERNAL: "An internal server error occurred. Please try again later.",
    },
};

/**
 * Formats a localized error message given an error code and dynamic interpolation parameters.
 */
export function formatErrorMessage(error: any, locale: SupportedLocale = "ko"): string {
    if (!error) return "알 수 없는 오류가 발생했습니다.";

    let code: string = error.code || error.extensions?.code || "";
    let params: Record<string, string> = error.params || error.extensions?.params || {};
    let fallbackMessage: string = error.message || "";

    // If code exists in translation dictionary
    if (code && ERROR_MESSAGES[locale]?.[code]) {
        let template = ERROR_MESSAGES[locale][code];
        for (const [key, value] of Object.entries(params)) {
            template = template.replace(new RegExp(`\\{${key}\\}`, "g"), String(value));
        }
        return template.replace(/\{[a-zA-Z0-9_]+\}/g, "");
    }

    if (fallbackMessage) {
        return fallbackMessage;
    }

    return ERROR_MESSAGES[locale]?.ERROR_INTERNAL || "오류가 발생했습니다.";
}
