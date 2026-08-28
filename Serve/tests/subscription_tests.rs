use chrono::Utc;
use serve::domain::models::{DirectMessage, Notification, NotificationType, TypingEvent, User};
use serve::infrastructure::pubsub::MessageBroker;
use uuid::Uuid;

#[tokio::test]
async fn test_message_broker_pubsub_channel() {
    let broker = MessageBroker::new(16);
    let mut rx = broker.subscribe();

    let u1 = User {
        id: Uuid::new_v4(),
        username: "user_one".to_string(),
        email: "u1@example.com".to_string(),
        password_hash: "hash".to_string(),
        display_name: "User One".to_string(),
        bio: None,
        avatar_url: None,
        header_image_url: None,
        location: None,
        website: None,
        is_private: false,
        is_2fa_enabled: false,
        totp_secret: None,
        created_at: Utc::now(),
    };

    let u2 = User {
        id: Uuid::new_v4(),
        username: "user_two".to_string(),
        email: "u2@example.com".to_string(),
        password_hash: "hash".to_string(),
        display_name: "User Two".to_string(),
        bio: None,
        avatar_url: None,
        header_image_url: None,
        location: None,
        website: None,
        is_private: false,
        is_2fa_enabled: false,
        totp_secret: None,
        created_at: Utc::now(),
    };

    let msg = DirectMessage {
        id: Uuid::new_v4(),
        sender_id: u1.id,
        recipient_id: u2.id,
        conversation_id: None,
        content: "Real-time subscription test".to_string(),
        is_read: false,
        is_edited: false,
        created_at: Utc::now(),
    };

    broker.publish(msg.clone());

    let received = rx
        .recv()
        .await
        .expect("Failed to receive message from broker");
    assert_eq!(received.id, msg.id);
    assert_eq!(received.content, "Real-time subscription test");
    assert_eq!(received.sender_id, u1.id);
    assert_eq!(received.recipient_id, u2.id);
}

#[tokio::test]
async fn test_message_broker_notification_pubsub_channel() {
    let broker = MessageBroker::new(16);
    let mut rx = broker.subscribe_notification();

    let notif = Notification {
        id: Uuid::new_v4(),
        recipient_id: Uuid::new_v4(),
        actor_id: Uuid::new_v4(),
        notification_type: NotificationType::LikePost,
        entity_id: Some(Uuid::new_v4()),
        is_read: false,
        created_at: Utc::now(),
    };

    broker.publish_notification(notif.clone());

    let received = rx
        .recv()
        .await
        .expect("Failed to receive notification from broker");
    assert_eq!(received.id, notif.id);
    assert_eq!(received.recipient_id, notif.recipient_id);
    assert_eq!(received.actor_id, notif.actor_id);
    assert_eq!(received.notification_type, NotificationType::LikePost);
}

#[tokio::test]
async fn test_message_broker_typing_pubsub_channel() {
    let broker = MessageBroker::new(16);
    let mut rx = broker.subscribe_typing();

    let uid = Uuid::new_v4();
    let cid = Uuid::new_v4();
    let event = TypingEvent {
        user_id: uid,
        conversation_id: Some(cid),
        recipient_id: None,
        is_typing: true,
    };

    broker.publish_typing(event.clone());

    let received = rx
        .recv()
        .await
        .expect("Failed to receive typing event from broker");
    assert_eq!(received.user_id, uid);
    assert_eq!(received.conversation_id, Some(cid));
    assert!(received.is_typing);
}
