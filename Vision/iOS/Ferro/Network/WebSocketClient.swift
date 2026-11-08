import Foundation

public protocol WebSocketClientDelegate: AnyObject, Sendable {
    func webSocketDidReceive(message: String)
    func webSocketDidChangeState(isConnected: Bool)
}

public actor WebSocketClient {
    public static let shared = WebSocketClient()

    private var webSocketTask: URLSessionWebSocketTask?
    private var isConnected = false
    private var isConnecting = false
    private var messageHandlers: [String: @Sendable (String) -> Void] = [:]

    public init() {}

    public func connect() {
        guard !isConnected && !isConnecting else { return }
        isConnecting = true

        let url = APIConfiguration.shared.websocketURL
        var request = URLRequest(url: url)
        if let token = KeychainHelper.shared.getAccessToken() {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }

        let session = URLSession(configuration: .default)
        let task = session.webSocketTask(with: request)
        self.webSocketTask = task
        task.resume()

        self.isConnected = true
        self.isConnecting = false

        // Send connection_init for graphql-transport-ws
        let initMessage = "{\"type\":\"connection_init\"}"
        sendRaw(message: initMessage)

        listenNext()
    }

    public func disconnect() {
        webSocketTask?.cancel(with: .goingAway, reason: nil)
        webSocketTask = nil
        isConnected = false
        isConnecting = false
    }

    public func sendRaw(message: String) {
        let msg = URLSessionWebSocketTask.Message.string(message)
        webSocketTask?.send(msg) { error in
            if let error = error {
                print("WebSocket send error: \(error.localizedDescription)")
            }
        }
    }

    public func subscribe(id: String, query: String, variables: [String: Any]? = nil, onMessage: @escaping @Sendable (String) -> Void) {
        messageHandlers[id] = onMessage

        var payload: [String: Any] = ["query": query]
        if let variables = variables {
            payload["variables"] = variables
        }

        let subscribeDict: [String: Any] = [
            "id": id,
            "type": "subscribe",
            "payload": payload
        ]

        if let data = try? JSONSerialization.data(withJSONObject: subscribeDict),
           let string = String(data: data, encoding: .utf8) {
            sendRaw(message: string)
        }
    }

    public func unsubscribe(id: String) {
        messageHandlers.removeValue(forKey: id)
        let stopDict: [String: Any] = [
            "id": id,
            "type": "complete"
        ]
        if let data = try? JSONSerialization.data(withJSONObject: stopDict),
           let string = String(data: data, encoding: .utf8) {
            sendRaw(message: string)
        }
    }

    private func listenNext() {
        guard isConnected else { return }
        webSocketTask?.receive { [weak self] result in
            guard let self = self else { return }
            Task {
                await self.handleReceiveResult(result)
            }
        }
    }

    private func handleReceiveResult(_ result: Result<URLSessionWebSocketTask.Message, Error>) {
        switch result {
        case .success(let message):
            switch message {
            case .string(let text):
                notifyHandlers(text)
            case .data(let data):
                if let text = String(data: data, encoding: .utf8) {
                    notifyHandlers(text)
                }
            @unknown default:
                break
            }
            listenNext()
        case .failure(let error):
            print("WebSocket receive error: \(error.localizedDescription)")
            isConnected = false
        }
    }

    private func notifyHandlers(_ text: String) {
        for handler in messageHandlers.values {
            handler(text)
        }
    }
}
