import Foundation

public struct GraphQLErrorExtension: Codable, Sendable {
    public let code: String?
    public let detail: String?
}

public struct GraphQLErrorItem: Codable, Sendable {
    public let message: String
    public let extensions: GraphQLErrorExtension?
}

public struct GraphQLResponse<T: Decodable & Sendable>: Decodable, Sendable {
    public let data: T?
    public let errors: [GraphQLErrorItem]?
}

public enum FerroAPIError: LocalizedError, Sendable {
    case invalidResponse
    case httpError(statusCode: Int, message: String)
    case graphQLError(code: String, detail: String)
    case networkError(String)
    case decodingError(String)
    case unauthorized

    public var errorDescription: String? {
        switch self {
        case .invalidResponse:
            return "Invalid server response"
        case .httpError(let code, let msg):
            return "HTTP Error \(code): \(msg)"
        case .graphQLError(let code, let detail):
            return "\(detail) [\(code)]"
        case .networkError(let msg):
            return "Network error: \(msg)"
        case .decodingError(let msg):
            return "Data format error: \(msg)"
        case .unauthorized:
            return "Authentication required. Please log in again."
        }
    }
}

public actor GraphQLClient {
    public static let shared = GraphQLClient()

    private let session: URLSession
    private let jsonDecoder: JSONDecoder
    private var isRefreshing = false

    public init(session: URLSession = .shared) {
        self.session = session
        let decoder = JSONDecoder()
        self.jsonDecoder = decoder
    }

    public func execute<T: Decodable & Sendable>(
        query: String,
        variables: [String: Any]? = nil,
        requiresAuth: Bool = true
    ) async throws -> T {
        var request = try buildRequest(query: query, variables: variables, requiresAuth: requiresAuth)

        let (data, response) = try await session.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse else {
            throw FerroAPIError.invalidResponse
        }

        // Check if token expired and needs rotation
        if httpResponse.statusCode == 401 || httpResponse.statusCode == 403 {
            if requiresAuth {
                let refreshed = try await attemptTokenRefresh()
                if refreshed {
                    request = try buildRequest(query: query, variables: variables, requiresAuth: true)
                    let (retryData, retryResponse) = try await session.data(for: request)
                    return try parseResponse(data: retryData, response: retryResponse)
                }
            }
            throw FerroAPIError.unauthorized
        }

        do {
            return try parseResponse(data: data, response: httpResponse)
        } catch let FerroAPIError.graphQLError(code, _) where code == "AUTH_TOKEN_INVALID" && requiresAuth {
            let refreshed = try await attemptTokenRefresh()
            if refreshed {
                let retryReq = try buildRequest(query: query, variables: variables, requiresAuth: true)
                let (retryData, retryResponse) = try await session.data(for: retryReq)
                return try parseResponse(data: retryData, response: retryResponse)
            }
            throw FerroAPIError.unauthorized
        }
    }

    private func buildRequest(
        query: String,
        variables: [String: Any]?,
        requiresAuth: Bool
    ) throws -> URLRequest {
        let endpoint = APIConfiguration.shared.graphqlURL
        var request = URLRequest(url: endpoint)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("application/json", forHTTPHeaderField: "Accept")
        request.setValue(UUID().uuidString, forHTTPHeaderField: "x-request-id")

        if requiresAuth, let token = KeychainHelper.shared.getAccessToken() {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }

        var payload: [String: Any] = ["query": query]
        if let variables = variables, !variables.isEmpty {
            payload["variables"] = variables
        }

        request.httpBody = try JSONSerialization.data(withJSONObject: payload, options: [])
        return request
    }

    private func parseResponse<T: Decodable & Sendable>(
        data: Data,
        response: URLResponse
    ) throws -> T {
        guard let httpResponse = response as? HTTPURLResponse else {
            throw FerroAPIError.invalidResponse
        }

        if httpResponse.statusCode >= 400 && httpResponse.statusCode != 401 {
            let msg = String(data: data, encoding: .utf8) ?? "Unknown error"
            throw FerroAPIError.httpError(statusCode: httpResponse.statusCode, message: msg)
        }

        let gqlResponse: GraphQLResponse<T>
        do {
            gqlResponse = try jsonDecoder.decode(GraphQLResponse<T>.self, from: data)
        } catch {
            throw FerroAPIError.decodingError(error.localizedDescription)
        }

        if let firstError = gqlResponse.errors?.first {
            let code = firstError.extensions?.code ?? "UNKNOWN_ERROR"
            let detail = firstError.extensions?.detail ?? firstError.message
            throw FerroAPIError.graphQLError(code: code, detail: detail)
        }

        guard let responseData = gqlResponse.data else {
            throw FerroAPIError.invalidResponse
        }

        return responseData
    }

    private func attemptTokenRefresh() async throws -> Bool {
        guard !isRefreshing else { return false }
        guard let currentRefreshToken = KeychainHelper.shared.getRefreshToken() else {
            return false
        }

        isRefreshing = true
        defer { isRefreshing = false }

        let mutation = """
        mutation RefreshToken($refreshToken: String!) {
            refreshToken(refreshToken: $refreshToken) {
                token
                refreshToken
                expiresIn
                tokenType
                user {
                    id
                    username
                    email
                    displayName
                    avatarUrl
                }
            }
        }
        """

        struct RefreshResult: Decodable, Sendable {
            let refreshToken: AuthPayload
        }

        do {
            let result: RefreshResult = try await execute(
                query: mutation,
                variables: ["refreshToken": currentRefreshToken],
                requiresAuth: false
            )
            KeychainHelper.shared.saveAccessToken(result.refreshToken.token)
            KeychainHelper.shared.saveRefreshToken(result.refreshToken.refreshToken)
            return true
        } catch {
            KeychainHelper.shared.clearTokens()
            return false
        }
    }
}
