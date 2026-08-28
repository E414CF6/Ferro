use crate::domain::models::{DirectMessage, Notification, TypingEvent};
use tokio::sync::broadcast;

/// Trait for event publishing in the application.
/// The default implementation uses an in-memory broadcast channel.
/// Future implementations can use Redis, NATS, Kafka, etc. for distributed systems.
#[allow(dead_code)]
pub trait EventPublisher: Send + Sync {
    /// Publish a new direct message event
    fn publish_dm(&self, message: DirectMessage);
    /// Subscribe to direct message events, returns a broadcast receiver
    fn subscribe_dm(&self) -> broadcast::Receiver<DirectMessage>;

    /// Publish a real-time notification event
    fn publish_notification(&self, notification: Notification);
    /// Subscribe to real-time notification events
    fn subscribe_notification(&self) -> broadcast::Receiver<Notification>;

    /// Publish a real-time typing indicator event
    fn publish_typing(&self, event: TypingEvent);
    /// Subscribe to real-time typing indicator events
    fn subscribe_typing(&self) -> broadcast::Receiver<TypingEvent>;
}
