use crate::domain::events::EventPublisher;
use crate::domain::models::{DirectMessage, Notification, TypingEvent};
use tokio::sync::broadcast;

/// In-memory pub/sub message broker for real-time GraphQL subscriptions.
/// Single-process broadcast implementation. For distributed deployments,
/// this can be swapped with a Redis/NATS implementation implementing `EventPublisher`.
#[derive(Clone)]
pub struct MessageBroker {
    dm_sender: broadcast::Sender<DirectMessage>,
    notif_sender: broadcast::Sender<Notification>,
    typing_sender: broadcast::Sender<TypingEvent>,
}

impl MessageBroker {
    pub fn new(capacity: usize) -> Self {
        let (dm_sender, _) = broadcast::channel(capacity);
        let (notif_sender, _) = broadcast::channel(capacity);
        let (typing_sender, _) = broadcast::channel(capacity);
        Self {
            dm_sender,
            notif_sender,
            typing_sender,
        }
    }

    pub fn publish(&self, message: DirectMessage) {
        let _ = self.dm_sender.send(message);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DirectMessage> {
        self.dm_sender.subscribe()
    }

    pub fn publish_notification(&self, notification: Notification) {
        let _ = self.notif_sender.send(notification);
    }

    pub fn subscribe_notification(&self) -> broadcast::Receiver<Notification> {
        self.notif_sender.subscribe()
    }

    pub fn publish_typing(&self, event: TypingEvent) {
        let _ = self.typing_sender.send(event);
    }

    pub fn subscribe_typing(&self) -> broadcast::Receiver<TypingEvent> {
        self.typing_sender.subscribe()
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
