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

use async_graphql::Request;
use futures_util::StreamExt;
use serve::domain::repositories::UserRepository;
use serve::graphql::build_schema;
use serve::infrastructure::{
    auth::AuthUser,
    config::{AuthConfig, DatabaseConfig},
    db::postgres::Database,
};

#[tokio::test]
async fn test_subscription_dm_security_authorization() {
    let test_file = format!("./data/test_sub_dm_{}.db", Uuid::new_v4());
    let _ = std::fs::remove_file(&test_file);
    let config = DatabaseConfig::sqlite_mode(&test_file);
    let db = Database::connect(&config).await.expect("Failed to connect SQLite");
    let broker = MessageBroker::new(16);
    let schema = build_schema(db, AuthConfig::default(), broker.clone());

    let alice_id = Uuid::new_v4();
    let bob_id = Uuid::new_v4();
    let auth_alice = AuthUser {
        user_id: alice_id,
        username: "alice".to_string(),
    };

    // 1. Unauthenticated subscription must fail with AUTH_TOKEN_REQUIRED
    let anon_sub = format!(
        r#"subscription {{ directMessageReceived(userId: "{}") {{ id content }} }}"#,
        alice_id
    );
    let mut stream = schema.execute_stream(Request::new(anon_sub));
    let resp = stream.next().await.expect("Expected response");
    assert!(!resp.errors.is_empty());
    let json_resp = serde_json::to_value(&resp).unwrap();
    assert_eq!(
        json_resp["errors"][0]["extensions"]["code"].as_str().unwrap(),
        "AUTH_TOKEN_REQUIRED"
    );

    // 2. Alice subscribing to Bob's DMs must fail with FORBIDDEN (eavesdropping blocked)
    let eavesdrop_sub = format!(
        r#"subscription {{ directMessageReceived(userId: "{}") {{ id content }} }}"#,
        bob_id
    );
    let mut stream = schema.execute_stream(Request::new(eavesdrop_sub).data(auth_alice.clone()));
    let resp = stream.next().await.expect("Expected response");
    assert!(!resp.errors.is_empty());
    let json_resp = serde_json::to_value(&resp).unwrap();
    assert_eq!(
        json_resp["errors"][0]["extensions"]["code"].as_str().unwrap(),
        "FORBIDDEN"
    );

    // 3. Alice subscribing to Alice's DMs succeeds and receives message
    let alice_sub = format!(
        r#"subscription {{ directMessageReceived(userId: "{}") {{ id content }} }}"#,
        alice_id
    );
    let mut stream = schema.execute_stream(Request::new(alice_sub).data(auth_alice.clone()));

    // Publish a DM addressed to Alice asynchronously after subscriber begins listening
    let msg_for_alice = DirectMessage {
        id: Uuid::new_v4(),
        sender_id: bob_id,
        recipient_id: alice_id,
        conversation_id: None,
        content: "Hello Alice!".to_string(),
        is_read: false,
        is_edited: false,
        created_at: Utc::now(),
    };
    let broker_dm = broker.clone();
    let msg_clone = msg_for_alice.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        broker_dm.publish(msg_clone);
    });

    let resp = stream.next().await.expect("Expected subscription item");
    assert!(resp.errors.is_empty(), "Expected no errors: {:?}", resp.errors);
    let json_resp = serde_json::to_value(&resp.data).unwrap();
    assert_eq!(
        json_resp["directMessageReceived"]["id"].as_str().unwrap(),
        msg_for_alice.id.to_string()
    );
    assert_eq!(
        json_resp["directMessageReceived"]["content"].as_str().unwrap(),
        "Hello Alice!"
    );

    let _ = std::fs::remove_file(&test_file);
}

#[tokio::test]
async fn test_subscription_notification_security_authorization() {
    let test_file = format!("./data/test_sub_notif_{}.db", Uuid::new_v4());
    let _ = std::fs::remove_file(&test_file);
    let config = DatabaseConfig::sqlite_mode(&test_file);
    let db = Database::connect(&config).await.expect("Failed to connect SQLite");
    let broker = MessageBroker::new(16);
    let schema = build_schema(db, AuthConfig::default(), broker.clone());

    let alice_id = Uuid::new_v4();
    let bob_id = Uuid::new_v4();
    let auth_alice = AuthUser {
        user_id: alice_id,
        username: "alice".to_string(),
    };

    // 1. Unauthenticated subscription fails with AUTH_TOKEN_REQUIRED
    let anon_sub = format!(
        r#"subscription {{ notificationReceived(userId: "{}") {{ id }} }}"#,
        alice_id
    );
    let mut stream = schema.execute_stream(Request::new(anon_sub));
    let resp = stream.next().await.expect("Expected response");
    assert!(!resp.errors.is_empty());
    let json_resp = serde_json::to_value(&resp).unwrap();
    assert_eq!(
        json_resp["errors"][0]["extensions"]["code"].as_str().unwrap(),
        "AUTH_TOKEN_REQUIRED"
    );

    // 2. Alice subscribing to Bob's notifications fails with FORBIDDEN
    let eavesdrop_sub = format!(
        r#"subscription {{ notificationReceived(userId: "{}") {{ id }} }}"#,
        bob_id
    );
    let mut stream = schema.execute_stream(Request::new(eavesdrop_sub).data(auth_alice.clone()));
    let resp = stream.next().await.expect("Expected response");
    assert!(!resp.errors.is_empty());
    let json_resp = serde_json::to_value(&resp).unwrap();
    assert_eq!(
        json_resp["errors"][0]["extensions"]["code"].as_str().unwrap(),
        "FORBIDDEN"
    );

    // 3. Alice subscribing to Alice's notifications succeeds
    let alice_sub = format!(
        r#"subscription {{ notificationReceived(userId: "{}") {{ id }} }}"#,
        alice_id
    );
    let mut stream = schema.execute_stream(Request::new(alice_sub).data(auth_alice.clone()));

    let notif = Notification {
        id: Uuid::new_v4(),
        recipient_id: alice_id,
        actor_id: bob_id,
        notification_type: NotificationType::LikePost,
        entity_id: Some(Uuid::new_v4()),
        is_read: false,
        created_at: Utc::now(),
    };
    let broker_notif = broker.clone();
    let notif_clone = notif.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        broker_notif.publish_notification(notif_clone);
    });

    let resp = stream.next().await.expect("Expected notification");
    assert!(resp.errors.is_empty(), "Expected no errors: {:?}", resp.errors);
    let json_resp = serde_json::to_value(&resp.data).unwrap();
    assert_eq!(
        json_resp["notificationReceived"]["id"].as_str().unwrap(),
        notif.id.to_string()
    );

    let _ = std::fs::remove_file(&test_file);
}

#[tokio::test]
async fn test_subscription_typing_status_security() {
    let test_file = format!("./data/test_sub_typing_{}.db", Uuid::new_v4());
    let _ = std::fs::remove_file(&test_file);
    let config = DatabaseConfig::sqlite_mode(&test_file);
    let db = Database::connect(&config).await.expect("Failed to connect SQLite");
    let broker = MessageBroker::new(16);
    let schema = build_schema(db, AuthConfig::default(), broker.clone());

    let alice_id = Uuid::new_v4();
    let bob_id = Uuid::new_v4();
    let auth_alice = AuthUser {
        user_id: alice_id,
        username: "alice".to_string(),
    };

    // 1. Unauthenticated subscription fails with AUTH_TOKEN_REQUIRED
    let anon_sub = format!(
        r#"subscription {{ typingStatus(recipientId: "{}") {{ isTyping }} }}"#,
        alice_id
    );
    let mut stream = schema.execute_stream(Request::new(anon_sub));
    let resp = stream.next().await.expect("Expected response");
    assert!(!resp.errors.is_empty());
    let json_resp = serde_json::to_value(&resp).unwrap();
    assert_eq!(
        json_resp["errors"][0]["extensions"]["code"].as_str().unwrap(),
        "AUTH_TOKEN_REQUIRED"
    );

    // 2. Omitting both conversationId and recipientId fails with BAD_REQUEST
    let bad_sub = r#"subscription { typingStatus { isTyping } }"#;
    let mut stream = schema.execute_stream(Request::new(bad_sub).data(auth_alice.clone()));
    let resp = stream.next().await.expect("Expected response");
    assert!(!resp.errors.is_empty());
    let json_resp = serde_json::to_value(&resp).unwrap();
    assert_eq!(
        json_resp["errors"][0]["extensions"]["code"].as_str().unwrap(),
        "BAD_REQUEST"
    );

    // 3. Specifying another user as recipientId fails with FORBIDDEN
    let forbidden_sub = format!(
        r#"subscription {{ typingStatus(recipientId: "{}") {{ isTyping }} }}"#,
        bob_id
    );
    let mut stream = schema.execute_stream(Request::new(forbidden_sub).data(auth_alice.clone()));
    let resp = stream.next().await.expect("Expected response");
    assert!(!resp.errors.is_empty());
    let json_resp = serde_json::to_value(&resp).unwrap();
    assert_eq!(
        json_resp["errors"][0]["extensions"]["code"].as_str().unwrap(),
        "FORBIDDEN"
    );

    let _ = std::fs::remove_file(&test_file);
}

#[tokio::test]
async fn test_resolve_user_id_idor_prevention() {
    let test_file = format!("./data/test_idor_{}.db", Uuid::new_v4());
    let _ = std::fs::remove_file(&test_file);
    let config = DatabaseConfig::sqlite_mode(&test_file);
    let db = Database::connect(&config).await.expect("Failed to connect SQLite");

    // Register Alice & Bob in DB
    let alice = db
        .register_user(
            "alice_idor".to_string(),
            "alice_idor@example.com".to_string(),
            "hashed_pw".to_string(),
            "Alice IDOR Test".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

    let bob = db
        .register_user(
            "bob_idor".to_string(),
            "bob_idor@example.com".to_string(),
            "hashed_pw".to_string(),
            "Bob IDOR Test".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

    let broker = MessageBroker::new(16);
    let schema = build_schema(db, AuthConfig::default(), broker);

    let auth_alice = AuthUser {
        user_id: alice.id,
        username: alice.username.clone(),
    };

    // 1. Alice attempts to create a post specifying authorId: Bob (IDOR attack)
    let idor_mutation = format!(
        r#"
        mutation {{
            createPost(
                content: "Malicious post impersonating Bob"
                authorId: "{}"
            ) {{
                id
            }}
        }}
        "#,
        bob.id
    );
    let req = Request::new(idor_mutation).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(!res.errors.is_empty(), "Expected IDOR attempt to be rejected");
    let json_res = serde_json::to_value(&res).unwrap();
    assert_eq!(
        json_res["errors"][0]["extensions"]["code"].as_str().unwrap(),
        "FORBIDDEN"
    );

    // 2. Alice creates a post specifying authorId: Alice (Explicit matching ID) -> Success
    let legit_explicit_mutation = format!(
        r#"
        mutation {{
            createPost(
                content: "Legitimate post by Alice"
                authorId: "{}"
            ) {{
                id
                content
            }}
        }}
        "#,
        alice.id
    );
    let req = Request::new(legit_explicit_mutation).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty(), "Legitimate explicit ID should succeed: {:?}", res.errors);

    // 3. Alice creates a post without specifying authorId (Context resolution) -> Success
    let legit_implicit_mutation = r#"
        mutation {
            createPost(
                content: "Legitimate post by Alice implicitly"
            ) {
                id
                content
            }
        }
    "#;
    let req = Request::new(legit_implicit_mutation).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty(), "Legitimate implicit ID should succeed: {:?}", res.errors);

    let _ = std::fs::remove_file(&test_file);
}
