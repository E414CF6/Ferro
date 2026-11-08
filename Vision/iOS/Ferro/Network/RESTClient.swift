import Foundation

public struct HealthStatus: Codable, Sendable {
    public let status: String
    public let service: String?
    public let version: String?
}

public struct ReadinessStatus: Codable, Sendable {
    public let status: String
    public let database: String?
    public let driver: String?
    public let redis: String?
}

public struct UploadResult: Codable, Sendable {
    public let url: String
    public let key: String
    public let size: Int
    public let contentType: String

    enum CodingKeys: String, CodingKey {
        case url
        case key
        case size
        case contentType = "content_type"
    }
}

public actor RESTClient {
    public static let shared = RESTClient()

    private let session: URLSession

    public init(session: URLSession = .shared) {
        self.session = session
    }

    public func checkLiveness() async throws -> HealthStatus {
        let url = APIConfiguration.shared.healthLiveURL
        let (data, response) = try await session.data(from: url)
        guard let http = response as? HTTPURLResponse, http.statusCode == 200 else {
            throw FerroAPIError.invalidResponse
        }
        return try JSONDecoder().decode(HealthStatus.self, from: data)
    }

    public func checkReadiness() async throws -> ReadinessStatus {
        let url = APIConfiguration.shared.healthReadyURL
        let (data, response) = try await session.data(from: url)
        guard let http = response as? HTTPURLResponse else {
            throw FerroAPIError.invalidResponse
        }
        if http.statusCode != 200 {
            let msg = String(data: data, encoding: .utf8) ?? "Unhealthy"
            throw FerroAPIError.httpError(statusCode: http.statusCode, message: msg)
        }
        return try JSONDecoder().decode(ReadinessStatus.self, from: data)
    }

    public func uploadMedia(
        fileData: Data,
        fileName: String,
        mimeType: String
    ) async throws -> UploadResult {
        let url = APIConfiguration.shared.uploadURL
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue(UUID().uuidString, forHTTPHeaderField: "x-request-id")

        if let token = KeychainHelper.shared.getAccessToken() {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }

        let boundary = "Boundary-\(UUID().uuidString)"
        request.setValue("multipart/form-data; boundary=\(boundary)", forHTTPHeaderField: "Content-Type")

        var body = Data()
        let boundaryPrefix = "--\(boundary)\r\n"

        body.append(Data(boundaryPrefix.utf8))
        body.append(Data("Content-Disposition: form-data; name=\"file\"; filename=\"\(fileName)\"\r\n".utf8))
        body.append(Data("Content-Type: \(mimeType)\r\n\r\n".utf8))
        body.append(fileData)
        body.append(Data("\r\n".utf8))
        body.append(Data("--\(boundary)--\r\n".utf8))

        request.httpBody = body

        let (data, response) = try await session.data(for: request)
        guard let http = response as? HTTPURLResponse else {
            throw FerroAPIError.invalidResponse
        }

        if http.statusCode == 201 || http.statusCode == 200 {
            return try JSONDecoder().decode(UploadResult.self, from: data)
        } else {
            let errorText = String(data: data, encoding: .utf8) ?? "Upload failed"
            throw FerroAPIError.httpError(statusCode: http.statusCode, message: errorText)
        }
    }
}
