use serve::graphql::build_schema;
use serve::infrastructure::{auth::AuthUser, config::AuthConfig, db::postgres::Database};

#[tokio::test]
async fn test_postgres_graphql_stories_flow() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://serve:password@localhost:5432/serve".to_string());

    let db = match Database::connect_url(&database_url).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test: PostgreSQL connection failed: {}", e);
            return;
        }
    };

    let schema = build_schema(
        db,
        AuthConfig::default(),
        serve::infrastructure::pubsub::MessageBroker::default(),
    );

    // 1. Register Author User
    let author_username = format!("story_author_{}", uuid::Uuid::new_v4().simple());
    let author_email = format!("{}@example.com", author_username);
    let signup_mutation = format!(
        r#"
        mutation {{
            signup(
                username: "{}"
                email: "{}"
                password: "password123"
                displayName: "Story Creator"
            ) {{
                user {{
                    id
                    username
                }}
            }}
        }}
        "#,
        author_username, author_email
    );

    let res = schema.execute(signup_mutation.as_str()).await;
    assert!(
        res.errors.is_empty(),
        "Signup author failed: {:?}",
        res.errors
    );
    let json_res = serde_json::to_value(&res.data).unwrap();
    let author_id = json_res
        .get("signup")
        .unwrap()
        .get("user")
        .unwrap()
        .get("id")
        .unwrap()
        .as_str()
        .unwrap();

    // 2. Create Story Mutation
    let auth_author = AuthUser {
        user_id: uuid::Uuid::parse_str(author_id).unwrap(),
        username: author_username.clone(),
    };

    let create_story_mutation = r#"
        mutation {
            createStory(
                mediaUrl: "https://images.unsplash.com/photo-1518770660439-4636190af475"
                caption: "Testing Instagram Story Feature!"
            ) {
                id
                mediaUrl
                caption
                isExpired
                viewsCount
                author {
                    username
                }
            }
        }
    "#;

    let req = async_graphql::Request::new(create_story_mutation).data(auth_author.clone());
    let res = schema.execute(req).await;
    assert!(
        res.errors.is_empty(),
        "Create story failed: {:?}",
        res.errors
    );
    let json_res = serde_json::to_value(&res.data).unwrap();
    let story_data = json_res.get("createStory").unwrap();
    let story_id = story_data.get("id").unwrap().as_str().unwrap();
    assert_eq!(
        story_data.get("caption").unwrap().as_str().unwrap(),
        "Testing Instagram Story Feature!"
    );

    // 3. Register Viewer User and View Story
    let viewer_username = format!("story_viewer_{}", uuid::Uuid::new_v4().simple());
    let viewer_email = format!("{}@example.com", viewer_username);
    let signup_viewer = format!(
        r#"
        mutation {{
            signup(
                username: "{}"
                email: "{}"
                password: "password123"
                displayName: "Story Viewer"
            ) {{
                user {{
                    id
                }}
            }}
        }}
        "#,
        viewer_username, viewer_email
    );

    let res = schema.execute(signup_viewer.as_str()).await;
    assert!(
        res.errors.is_empty(),
        "Signup viewer failed: {:?}",
        res.errors
    );
    let json_res = serde_json::to_value(&res.data).unwrap();
    let viewer_id = json_res
        .get("signup")
        .unwrap()
        .get("user")
        .unwrap()
        .get("id")
        .unwrap()
        .as_str()
        .unwrap();

    let auth_viewer = AuthUser {
        user_id: uuid::Uuid::parse_str(viewer_id).unwrap(),
        username: viewer_username.clone(),
    };

    let view_story_mutation = format!(
        r#"
        mutation {{
            viewStory(storyId: "{}")
        }}
        "#,
        story_id
    );

    let req = async_graphql::Request::new(view_story_mutation.as_str()).data(auth_viewer.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty(), "View story failed: {:?}", res.errors);

    // 4. Query Story Viewers as Story Author
    let query_story_viewers = format!(
        r#"
        query {{
            story(id: "{}") {{
                id
                viewsCount
                viewers {{
                    username
                }}
            }}
        }}
        "#,
        story_id
    );

    let req = async_graphql::Request::new(query_story_viewers.as_str()).data(auth_author);
    let res = schema.execute(req).await;
    assert!(
        res.errors.is_empty(),
        "Query story viewers failed: {:?}",
        res.errors
    );
    let json_res = serde_json::to_value(&res.data).unwrap();
    let story_res = json_res.get("story").unwrap();
    assert_eq!(story_res.get("viewsCount").unwrap().as_u64().unwrap(), 1);
    let viewers = story_res.get("viewers").unwrap().as_array().unwrap();
    assert_eq!(
        viewers[0].get("username").unwrap().as_str().unwrap(),
        viewer_username.as_str()
    );
}

#[tokio::test]
async fn test_postgres_graphql_direct_messages_flow() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://serve:password@localhost:5432/serve".to_string());

    let db = match Database::connect_url(&database_url).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test: PostgreSQL connection failed: {}", e);
            return;
        }
    };

    let schema = build_schema(
        db,
        AuthConfig::default(),
        serve::infrastructure::pubsub::MessageBroker::default(),
    );

    // 1. Register User 1
    let u1_name = format!("dm_user1_{}", uuid::Uuid::new_v4().simple());
    let u1_email = format!("{}@example.com", u1_name);
    let signup_u1 = format!(
        r#"mutation {{ signup(username: "{}", email: "{}", password: "password123", displayName: "DM User 1") {{ user {{ id username }} }} }}"#,
        u1_name, u1_email
    );
    let res = schema.execute(signup_u1.as_str()).await;
    assert!(res.errors.is_empty(), "Signup u1 failed: {:?}", res.errors);
    let u1_id = serde_json::to_value(&res.data).unwrap()["signup"]["user"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. Register User 2
    let u2_name = format!("dm_user2_{}", uuid::Uuid::new_v4().simple());
    let u2_email = format!("{}@example.com", u2_name);
    let signup_u2 = format!(
        r#"mutation {{ signup(username: "{}", email: "{}", password: "password123", displayName: "DM User 2") {{ user {{ id username }} }} }}"#,
        u2_name, u2_email
    );
    let res = schema.execute(signup_u2.as_str()).await;
    assert!(res.errors.is_empty(), "Signup u2 failed: {:?}", res.errors);
    let u2_id = serde_json::to_value(&res.data).unwrap()["signup"]["user"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let auth_u1 = AuthUser {
        user_id: uuid::Uuid::parse_str(&u1_id).unwrap(),
        username: u1_name.clone(),
    };
    let auth_u2 = AuthUser {
        user_id: uuid::Uuid::parse_str(&u2_id).unwrap(),
        username: u2_name.clone(),
    };

    // 3. User 1 sends message to User 2
    let send_msg_mutation = format!(
        r#"mutation {{ sendDirectMessage(recipientId: "{}", content: "Hello from User 1! 🦀") {{ id content isRead sender {{ username }} recipient {{ username }} }} }}"#,
        u2_id
    );
    let req = async_graphql::Request::new(send_msg_mutation.as_str()).data(auth_u1.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty(), "Send DM failed: {:?}", res.errors);
    let msg_data = serde_json::to_value(&res.data).unwrap()["sendDirectMessage"].clone();
    assert_eq!(
        msg_data["content"].as_str().unwrap(),
        "Hello from User 1! 🦀"
    );

    // 4. User 2 checks unread count & conversations
    let unread_query = r#"query { unreadDmCount }"#;
    let req = async_graphql::Request::new(unread_query).data(auth_u2.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    let unread_count = serde_json::to_value(&res.data).unwrap()["unreadDmCount"]
        .as_u64()
        .unwrap();
    assert_eq!(unread_count, 1);

    let convos_query =
        r#"query { conversations { otherUser { username } unreadCount lastMessage { content } } }"#;
    let req = async_graphql::Request::new(convos_query).data(auth_u2.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    let convos = serde_json::to_value(&res.data).unwrap()["conversations"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(convos.len(), 1);
    assert_eq!(
        convos[0]["otherUser"]["username"].as_str().unwrap(),
        u1_name.as_str()
    );
    assert_eq!(convos[0]["unreadCount"].as_i64().unwrap(), 1);

    // 5. User 2 marks messages as read
    let mark_read_mutation = format!(
        r#"mutation {{ markMessagesAsRead(senderId: "{}") }}"#,
        u1_id
    );
    let req = async_graphql::Request::new(mark_read_mutation.as_str()).data(auth_u2.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());

    // 6. User 2 sends reply to User 1
    let reply_mutation = format!(
        r#"mutation {{ sendDirectMessage(recipientId: "{}", content: "Hey! Received loud and clear.") {{ id content }} }}"#,
        u1_id
    );
    let req = async_graphql::Request::new(reply_mutation.as_str()).data(auth_u2.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());

    // 7. User 1 checks chat history
    let history_query = format!(
        r#"query {{ directMessages(otherUserId: "{}") {{ content sender {{ username }} isMine }} }}"#,
        u2_id
    );
    let req = async_graphql::Request::new(history_query.as_str()).data(auth_u1);
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    let msgs = serde_json::to_value(&res.data).unwrap()["directMessages"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(msgs.len(), 2);
    assert!(msgs[0]["isMine"].as_bool().unwrap());
    assert!(!msgs[1]["isMine"].as_bool().unwrap());
}
