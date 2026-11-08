import Foundation

public final class APIConfiguration: @unchecked Sendable {
    public static let shared = APIConfiguration()

    private let baseURLKey = "ferro_api_base_url"
    private let defaultBaseURL = "http://127.0.0.1:8080"

    private init() {}

    public var baseURLString: String {
        get {
            UserDefaults.standard.string(forKey: baseURLKey) ?? defaultBaseURL
        }
        set {
            UserDefaults.standard.set(newValue, forKey: baseURLKey)
        }
    }

    public var baseURL: URL {
        URL(string: baseURLString) ?? URL(string: defaultBaseURL)!
    }

    public var graphqlURL: URL {
        baseURL.appendingPathComponent("graphql")
    }

    public var websocketURL: URL {
        var components = URLComponents(url: baseURL, resolvingAgainstBaseURL: false)
        if components?.scheme == "https" {
            components?.scheme = "wss"
        } else {
            components?.scheme = "ws"
        }
        let wsPath = "/graphql/ws"
        components?.path = wsPath
        return components?.url ?? URL(string: "ws://127.0.0.1:8080/graphql/ws")!
    }

    public var healthLiveURL: URL {
        baseURL.appendingPathComponent("health")
    }

    public var healthReadyURL: URL {
        baseURL.appendingPathComponent("health/ready")
    }

    public var uploadURL: URL {
        baseURL.appendingPathComponent("api/upload")
    }

    public func staticMediaURL(for filename: String) -> URL {
        baseURL.appendingPathComponent("uploads").appendingPathComponent(filename)
    }

    public func resetToDefault() {
        baseURLString = defaultBaseURL
    }
}
