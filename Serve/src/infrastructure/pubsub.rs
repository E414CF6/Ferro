use crate::domain::events::EventPublisher;
use crate::domain::models::{DirectMessage, Notification, TypingEvent};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tracing::{error, info, warn};

const CHANNEL_DM: &str = "ferro:pubsub:dm";
const CHANNEL_NOTIF: &str = "ferro:pubsub:notification";
const CHANNEL_TYPING: &str = "ferro:pubsub:typing";

/// Multi-node distributed / in-memory PubSub message broker.
/// Supports both local tokio broadcast channels and Redis PubSub clustering.
#[derive(Clone)]
pub struct MessageBroker {
    dm_sender: broadcast::Sender<DirectMessage>,
    notif_sender: broadcast::Sender<Notification>,
    typing_sender: broadcast::Sender<TypingEvent>,
    redis_client: Option<redis::Client>,
}

impl MessageBroker {
    /// Create in-memory single-process broker
    pub fn new(capacity: usize) -> Self {
        let (dm_sender, _) = broadcast::channel(capacity);
        let (notif_sender, _) = broadcast::channel(capacity);
        let (typing_sender, _) = broadcast::channel(capacity);
        Self {
            dm_sender,
            notif_sender,
            typing_sender,
            redis_client: None,
        }
    }

    /// Create broker with Redis PubSub synchronization across cluster nodes
    pub fn with_redis(capacity: usize, redis_url: &str) -> Self {
        let client = match redis::Client::open(redis_url) {
            Ok(c) => Some(c),
            Err(e) => {
                warn!(target: "serve::pubsub", error = %e, "Invalid Redis URL, falling back to in-memory pubsub");
                None
            }
        };

        let broker = Self {
            dm_sender: broadcast::channel(capacity).0,
            notif_sender: broadcast::channel(capacity).0,
            typing_sender: broadcast::channel(capacity).0,
            redis_client: client.clone(),
        };

        if let Some(client) = client {
            info!(target: "serve::pubsub", "Starting Redis PubSub background listener task...");
            let dm_sender = broker.dm_sender.clone();
            let notif_sender = broker.notif_sender.clone();
            let typing_sender = broker.typing_sender.clone();
            let url_string = redis_url.to_string();

            tokio::spawn(async move {
                loop {
                    match client.get_async_pubsub().await {
                        Ok(mut pubsub) => {
                            if let Err(e) = pubsub
                                .subscribe(&[CHANNEL_DM, CHANNEL_NOTIF, CHANNEL_TYPING])
                                .await
                            {
                                error!(target: "serve::pubsub", error = %e, "Failed to subscribe to Redis channels");
                                tokio::time::sleep(Duration::from_secs(3)).await;
                                continue;
                            }

                            info!(target: "serve::pubsub", "Connected to Redis PubSub cluster listener");
                            let mut stream = pubsub.on_message();

                            while let Some(msg) = stream.next().await {
                                let channel = msg.get_channel_name().to_string();
                                let payload: String = match msg.get_payload() {
                                    Ok(p) => p,
                                    Err(e) => {
                                        warn!(target: "serve::pubsub", error = %e, "Failed to decode Redis pubsub payload");
                                        continue;
                                    }
                                };

                                match channel.as_str() {
                                    CHANNEL_DM => {
                                        if let Ok(dm) =
                                            serde_json::from_str::<DirectMessage>(&payload)
                                        {
                                            let _ = dm_sender.send(dm);
                                        }
                                    }
                                    CHANNEL_NOTIF => {
                                        if let Ok(n) =
                                            serde_json::from_str::<Notification>(&payload)
                                        {
                                            let _ = notif_sender.send(n);
                                        }
                                    }
                                    CHANNEL_TYPING => {
                                        if let Ok(t) = serde_json::from_str::<TypingEvent>(&payload)
                                        {
                                            let _ = typing_sender.send(t);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            warn!(target: "serve::pubsub", "Redis PubSub stream closed, reconnecting in 2s...");
                        }
                        Err(e) => {
                            warn!(target: "serve::pubsub", error = %e, redis_url = %url_string, "Redis connection failed, retrying in 3s...");
                        }
                    }
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            });
        }

        broker
    }

    pub fn publish(&self, message: DirectMessage) {
        if let Some(ref client) = self.redis_client {
            let client = client.clone();
            let payload = serde_json::to_string(&message).unwrap_or_default();
            tokio::spawn(async move {
                if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                    let _: Result<(), redis::RedisError> = redis::cmd("PUBLISH")
                        .arg(CHANNEL_DM)
                        .arg(&payload)
                        .query_async(&mut conn)
                        .await;
                }
            });
        } else {
            let _ = self.dm_sender.send(message);
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DirectMessage> {
        self.dm_sender.subscribe()
    }

    pub fn publish_notification(&self, notification: Notification) {
        if let Some(ref client) = self.redis_client {
            let client = client.clone();
            let payload = serde_json::to_string(&notification).unwrap_or_default();
            tokio::spawn(async move {
                if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                    let _: Result<(), redis::RedisError> = redis::cmd("PUBLISH")
                        .arg(CHANNEL_NOTIF)
                        .arg(&payload)
                        .query_async(&mut conn)
                        .await;
                }
            });
        } else {
            let _ = self.notif_sender.send(notification);
        }
    }

    pub fn subscribe_notification(&self) -> broadcast::Receiver<Notification> {
        self.notif_sender.subscribe()
    }

    pub fn publish_typing(&self, event: TypingEvent) {
        if let Some(ref client) = self.redis_client {
            let client = client.clone();
            let payload = serde_json::to_string(&event).unwrap_or_default();
            tokio::spawn(async move {
                if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                    let _: Result<(), redis::RedisError> = redis::cmd("PUBLISH")
                        .arg(CHANNEL_TYPING)
                        .arg(&payload)
                        .query_async(&mut conn)
                        .await;
                }
            });
        } else {
            let _ = self.typing_sender.send(event);
        }
    }

    pub fn subscribe_typing(&self) -> broadcast::Receiver<TypingEvent> {
        self.typing_sender.subscribe()
    }

    #[allow(dead_code)]
    pub fn is_redis_connected(&self) -> bool {
        self.redis_client.is_some()
    }
}

impl EventPublisher for MessageBroker {
    fn publish_dm(&self, message: DirectMessage) {
        self.publish(message);
    }

    fn subscribe_dm(&self) -> broadcast::Receiver<DirectMessage> {
        self.subscribe()
    }

    fn publish_notification(&self, notification: Notification) {
        self.publish_notification(notification);
    }

    fn subscribe_notification(&self) -> broadcast::Receiver<Notification> {
        self.subscribe_notification()
    }

    fn publish_typing(&self, event: TypingEvent) {
        self.publish_typing(event);
    }

    fn subscribe_typing(&self) -> broadcast::Receiver<TypingEvent> {
        self.subscribe_typing()
    }
}

impl Default for MessageBroker {
    fn default() -> Self {
        Self::new(1024)
    }
}
