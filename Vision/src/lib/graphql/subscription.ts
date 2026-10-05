import {DirectMessage, Notification, TypingEvent} from "../types";

const GRAPHQL_ENDPOINT =
    process.env.NEXT_PUBLIC_GRAPHQL_ENDPOINT || "http://127.0.0.1:8080/graphql";

const WS_ENDPOINT =
    process.env.NEXT_PUBLIC_GRAPHQL_WS_ENDPOINT ||
    GRAPHQL_ENDPOINT.replace(/^http/, "ws").replace(/\/graphql\/?$/, "/graphql/ws");

type SubscriptionCallback<T = any> = (data: T) => void;

interface ActiveSubscription {
    id: string;
    query: string;
    variables?: Record<string, any>;
    callback: SubscriptionCallback;
}

class GraphQLSubscriptionClient {
    private socket: WebSocket | null = null;
    private subscriptions = new Map<string, ActiveSubscription>();
    private subIdCounter = 0;
    private isConnected = false;
    private reconnectTimeout: any = null;
    private pingInterval: any = null;

    private getAuthToken(): string | null {
        if (typeof window !== "undefined") {
            return localStorage.getItem("ferro_token");
        }
        return null;
    }

    public connect() {
        if (typeof window === "undefined") return;
        if (this.socket && (this.socket.readyState === WebSocket.OPEN || this.socket.readyState === WebSocket.CONNECTING)) {
            return;
        }

        try {
            this.socket = new WebSocket(WS_ENDPOINT, "graphql-transport-ws");

            this.socket.onopen = () => {
                const token = this.getAuthToken();
                const initPayload = token ? { Authorization: `Bearer ${token}` } : {};
                this.send({
                    type: "connection_init",
                    payload: initPayload,
                });
            };

            this.socket.onmessage = (event) => {
                try {
                    const msg = JSON.parse(event.data);
                    if (msg.type === "connection_ack") {
                        this.isConnected = true;
                        this.startPing();
                        // Re-subscribe all active subscriptions
                        this.subscriptions.forEach((sub) => {
                            this.sendSubscribe(sub);
                        });
                    } else if (msg.type === "next" && msg.id) {
                        const sub = this.subscriptions.get(msg.id);
                        if (sub && msg.payload?.data) {
                            sub.callback(msg.payload.data);
                        }
                    } else if (msg.type === "ping") {
                        this.send({ type: "pong" });
                    }
                } catch (e) {
                    console.error("Error parsing WS message:", e);
                }
            };

            this.socket.onerror = (err) => {
                console.warn("GraphQL WebSocket error:", err);
            };

            this.socket.onclose = () => {
                this.isConnected = false;
                this.stopPing();
                this.scheduleReconnect();
            };
        } catch (e) {
            console.warn("Failed to initialize WebSocket:", e);
            this.scheduleReconnect();
        }
    }

    private startPing() {
        this.stopPing();
        this.pingInterval = setInterval(() => {
            if (this.socket && this.socket.readyState === WebSocket.OPEN) {
                this.send({ type: "ping" });
            }
        }, 20000);
    }

    private stopPing() {
        if (this.pingInterval) {
            clearInterval(this.pingInterval);
            this.pingInterval = null;
        }
    }

    private scheduleReconnect() {
        if (this.reconnectTimeout) return;
        this.reconnectTimeout = setTimeout(() => {
            this.reconnectTimeout = null;
            if (this.subscriptions.size > 0) {
                this.connect();
            }
        }, 3000);
    }

    private send(data: any) {
        if (this.socket && this.socket.readyState === WebSocket.OPEN) {
            this.socket.send(JSON.stringify(data));
        }
    }

    private sendSubscribe(sub: ActiveSubscription) {
        this.send({
            id: sub.id,
            type: "subscribe",
            payload: {
                query: sub.query,
                variables: sub.variables || {},
            },
        });
    }

    public subscribe<T = any>(
        query: string,
        variables: Record<string, any> | undefined,
        callback: SubscriptionCallback<T>
    ): () => void {
        const id = `sub_${++this.subIdCounter}`;
        const sub: ActiveSubscription = { id, query, variables, callback };
        this.subscriptions.set(id, sub);

        if (this.isConnected) {
            this.sendSubscribe(sub);
        } else {
            this.connect();
        }

        return () => {
            this.subscriptions.delete(id);
            if (this.isConnected) {
                this.send({ id, type: "complete" });
            }
            if (this.subscriptions.size === 0 && this.socket) {
                this.socket.close();
                this.socket = null;
            }
        };
    }
}

export const wsClient = new GraphQLSubscriptionClient();

/* Convenience Subscription Helpers */

export function subscribeToDirectMessages(
    userId: string,
    onMessage: (msg: DirectMessage) => void
): () => void {
    const query = `
    subscription OnDirectMessage($userId: ID!) {
      directMessageReceived(userId: $userId) {
        id
        conversationId
        senderId
        recipientId
        content
        isRead
        createdAt
        sender {
          id
          username
          displayName
          avatarUrl
        }
        recipient {
          id
          username
          displayName
          avatarUrl
        }
      }
    }
  `;

    return wsClient.subscribe<{ directMessageReceived: DirectMessage }>(
        query,
        { userId },
        (data) => {
            if (data?.directMessageReceived) {
                onMessage(data.directMessageReceived);
            }
        }
    );
}

export function subscribeToNotifications(
    userId: string,
    onNotification: (notif: Notification) => void
): () => void {
    const query = `
    subscription OnNotification($userId: ID!) {
      notificationReceived(userId: $userId) {
        id
        recipientId
        senderId
        actorId
        notificationType
        targetId
        entityId
        isRead
        createdAt
        sender {
          id
          username
          displayName
          avatarUrl
        }
        actor {
          id
          username
          displayName
          avatarUrl
        }
        targetPost {
          id
          content
        }
        targetComment {
          id
          content
        }
      }
    }
  `;

    return wsClient.subscribe<{ notificationReceived: Notification }>(
        query,
        { userId },
        (data) => {
            if (data?.notificationReceived) {
                onNotification(data.notificationReceived);
            }
        }
    );
}

export function subscribeToTypingStatus(
    params: { conversationId?: string; recipientId?: string },
    onTyping: (event: TypingEvent) => void
): () => void {
    const query = `
    subscription OnTyping($conversationId: ID, $recipientId: ID) {
      typingStatus(conversationId: $conversationId, recipientId: $recipientId) {
        userId
        conversationId
        recipientId
        isTyping
      }
    }
  `;

    return wsClient.subscribe<{ typingStatus: TypingEvent }>(
        query,
        params,
        (data) => {
            if (data?.typingStatus) {
                onTyping(data.typingStatus);
            }
        }
    );
}
