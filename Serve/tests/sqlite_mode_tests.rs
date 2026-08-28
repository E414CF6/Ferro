use async_graphql::Request;
use serve::domain::models::*;
use serve::domain::repositories::*;
use serve::graphql::build_schema;
use serve::infrastructure::{
    auth::AuthUser,
    config::{AuthConfig, DatabaseConfig},
    db::postgres::Database,
    pubsub::MessageBroker,
};
use uuid::Uuid;

#[tokio::test]
async fn test_sqlite_database_mode_crud_and_persistence() {
    let test_file = format!("./data/test_sqlite_db_{}.db", Uuid::new_v4());
    let _ = std::fs::remove_file(&test_file);
    let _ = std::fs::remove_file(format!("{}-shm", &test_file));
    let _ = std::fs::remove_file(format!("{}-wal", &test_file));

    // 1. Connect to SQLite Database
    let config = DatabaseConfig::sqlite_mode(&test_file);
    let db = Database::connect(&config)
        .await
        .expect("Failed to initialize SQLite database");

    assert!(db.is_sqlite(), "Database should be in SQLite mode");

    // 2. User Registration & Retrieval with unique emails
    let user1 = db
        .register_user(
            "alice_sqlite_unique".to_string(),
            "alice_sqlite_unique@example.com".to_string(),
            "hashed_pw_1".to_string(),
            "Alice in SQLiteland".to_string(),
            Some("Testing SQLite mode".to_string()),
            None,
            None,
            Some("Seoul".to_string()),
            Some("https://alice.dev".to_string()),
        )
        .await
        .expect("Failed to register user1");

    let user2 = db
        .register_user(
            "bob_sqlite_unique".to_string(),
            "bob_sqlite_unique@example.com".to_string(),
            "hashed_pw_2".to_string(),
            "Bob The Builder".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to register user2");

    let fetched_user = db
        .get_user_by_username("alice_sqlite_unique")
        .await
        .expect("User should be found by username");
    assert_eq!(fetched_user.id, user1.id);
    assert_eq!(fetched_user.display_name, "Alice in SQLiteland");

    // 3. Follow & Social Graph
    db.follow_user(user1.id, user2.id)
        .await
        .expect("Alice follows Bob");
    assert!(db.is_following(user1.id, user2.id).await);
    assert_eq!(db.get_following_count(user1.id).await, 1);
    assert_eq!(db.get_followers_count(user2.id).await, 1);

    // 4. Posts, Likes, Reposts, Bookmarks
    let post = db
        .create_post(
            user1.id,
            "Hello from SQLite database mode! #rust #ferro #sqlite".to_string(),
            Some(PostAudience::Public),
        )
        .await
        .expect("Failed to create post");

    assert_eq!(post.author_id, user1.id);
    assert_eq!(db.get_user_posts_count(user1.id).await, 1);

    // Like Post
    db.like_post(user2.id, post.id)
        .await
        .expect("Bob likes Alice's post");
    assert!(db.is_post_liked_by(post.id, user2.id).await);
    assert_eq!(db.get_likes_count(post.id).await, 1);

    // Repost Post
    db.repost_post(user2.id, post.id)
        .await
        .expect("Bob reposts Alice's post");
    assert!(db.is_post_reposted_by(post.id, user2.id).await);
    assert_eq!(db.get_reposts_count(post.id).await, 1);

    // Save Post (Bookmark)
    db.save_post(user2.id, post.id)
        .await
        .expect("Bob saves post");
    assert!(db.is_post_saved_by(post.id, user2.id).await);
    let saved_posts = db.get_saved_posts(user2.id, None, None).await;
    assert_eq!(saved_posts.len(), 1);
    assert_eq!(saved_posts[0].id, post.id);

    // 5. Comments & Nested Threads
    let comment = db
        .create_comment(
            post.id,
            user2.id,
            "SQLite mode is so blazingly fast!".to_string(),
            None,
        )
        .await
        .expect("Bob comments on post");

    let reply = db
        .create_comment(
            post.id,
            user1.id,
            "Glad you like it, Bob!".to_string(),
            Some(comment.id),
        )
        .await
        .expect("Alice replies to Bob");

    assert_eq!(db.get_replies_count(comment.id).await, 1);
    let replies = db.get_replies_for_comment(comment.id).await;
    assert_eq!(replies.len(), 1);
    assert_eq!(replies[0].id, reply.id);

    // 6. Direct Messages
    let _dm = db
        .send_direct_message(
            user1.id,
            user2.id,
            "Hey Bob, did you see the new SQLite backend?".to_string(),
        )
        .await
        .expect("Alice sends DM to Bob");

    let unread_dm_count = db.get_unread_dm_count(user2.id).await;
    assert_eq!(unread_dm_count, 1);

    db.mark_direct_messages_as_read(user2.id, user1.id)
        .await
        .expect("Bob marks messages as read");
    assert_eq!(db.get_unread_dm_count(user2.id).await, 0);

    // 7. Stories
    let story = db
        .create_story(
            user1.id,
            "https://images.unsplash.com/photo-rust".to_string(),
            Some("Coding late night in SQLite mode!".to_string()),
        )
        .await
        .expect("Alice creates story");

    assert!(db.has_active_stories(user1.id).await);
    db.view_story(story.id, user2.id)
        .await
        .expect("Bob views story");
    assert!(db.is_story_viewed_by(story.id, user2.id).await);
    assert_eq!(db.get_story_views_count(story.id).await, 1);

    // 8. Polls & Voting
    let poll = db
        .create_poll(
            post.id,
            "Do you like zero-setup SQLite database mode?".to_string(),
            vec!["Yes, absolutely!".to_string(), "It's awesome!".to_string()],
            86400,
        )
        .await
        .expect("Create poll");

    let options = db.get_poll_options(poll.id).await;
    assert_eq!(options.len(), 2);

    let vote = db
        .vote_poll(poll.id, options[0].id, user2.id)
        .await
        .expect("Bob votes");
    assert_eq!(vote.option_id, options[0].id);
    assert_eq!(db.get_poll_total_votes(poll.id).await, 1);

    // 9. Collections & User Lists
    let col = db
        .create_bookmark_collection(
            user1.id,
            "Rust Gems".to_string(),
            Some("Awesome posts".to_string()),
            false,
        )
        .await
        .expect("Create collection");

    db.add_post_to_collection(col.id, user1.id, post.id)
        .await
        .expect("Add post to collection");

    let cols = db.get_user_collections(user1.id).await;
    assert_eq!(cols.len(), 1);

    // 10. Persistence Check: Drop db connection and reload from the same SQLite file
    drop(db);

    let reloaded_db = Database::connect(&config)
        .await
        .expect("Failed to reload SQLite database from file");

    let reloaded_user1 = reloaded_db
        .get_user_by_username("alice_sqlite_unique")
        .await
        .expect("User should persist in SQLite file");
    assert_eq!(reloaded_user1.id, user1.id);

    let reloaded_post = reloaded_db
        .get_post_by_id(post.id)
        .await
        .expect("Post should persist in SQLite file");
    assert_eq!(reloaded_post.content, post.content);

    // Clean up
    let _ = std::fs::remove_file(&test_file);
    let _ = std::fs::remove_file(format!("{}-shm", &test_file));
    let _ = std::fs::remove_file(format!("{}-wal", &test_file));
}

#[tokio::test]
async fn test_sqlite_database_graphql_queries_and_dataloaders() {
    let test_file = format!("./data/test_gql_sqlite_db_{}.db", Uuid::new_v4());
    let _ = std::fs::remove_file(&test_file);
    let _ = std::fs::remove_file(format!("{}-shm", &test_file));
    let _ = std::fs::remove_file(format!("{}-wal", &test_file));

    let config = DatabaseConfig::sqlite_mode(&test_file);
    let db = Database::connect(&config)
        .await
        .expect("Failed to initialize SQLite database");

    let schema = build_schema(db, AuthConfig::default(), MessageBroker::default());

    // 1. GraphQL Mutation: Signup
    let signup_mutation = r#"
        mutation {
            signup(
                username: "ferro_sqlite_user_unique"
                email: "ferro_sqlite_unique@example.com"
                password: "password123"
                displayName: "Ferro SQLite Dev"
            ) {
                token
                user {
                    id
                    username
                    displayName
                }
            }
        }
    "#;

    let res = schema.execute(Request::new(signup_mutation)).await;
    assert!(
        res.errors.is_empty(),
        "Signup failed with errors: {:?}",
        res.errors
    );

    let res_data = res.data.into_json().unwrap();
    let _token = res_data["signup"]["token"].as_str().unwrap().to_string();
    let user_id_str = res_data["signup"]["user"]["id"].as_str().unwrap();
    let user_id = Uuid::parse_str(user_id_str).unwrap();

    let auth_user = AuthUser {
        user_id,
        username: "ferro_sqlite_user_unique".to_string(),
    };

    // 2. GraphQL Mutation: Create Post
    let create_post_mutation = r#"
        mutation {
            createPost(
                content: "Testing GraphQL and DataLoader with embedded SQLite! #zero_config"
            ) {
                id
                content
                author {
                    username
                    displayName
                }
            }
        }
    "#;

    let req = Request::new(create_post_mutation).data(auth_user.clone());
    let post_res = schema.execute(req).await;
    assert!(
        post_res.errors.is_empty(),
        "CreatePost failed with errors: {:?}",
        post_res.errors
    );

    let post_data = post_res.data.into_json().unwrap();
    let post_id = post_data["createPost"]["id"].as_str().unwrap();

    // 3. GraphQL Query: Feed and User with DataLoaders
    let query = format!(
        r#"
        query {{
            post(id: "{}") {{
                id
                content
                likesCount
                repostsCount
                                author {{
                    username
                    postsCount
                    followersCount
                }}
            }}
        }}
    "#,
        post_id
    );

    let req = Request::new(query).data(auth_user);
    let query_res = schema.execute(req).await;
    assert!(
        query_res.errors.is_empty(),
        "Query failed with errors: {:?}",
        query_res.errors
    );

    let query_data = query_res.data.into_json().unwrap();
    assert_eq!(
        query_data["post"]["author"]["username"],
        "ferro_sqlite_user_unique"
    );
    assert_eq!(query_data["post"]["author"]["postsCount"], 1);

    // Clean up
    let _ = std::fs::remove_file(&test_file);
    let _ = std::fs::remove_file(format!("{}-shm", &test_file));
    let _ = std::fs::remove_file(format!("{}-wal", &test_file));
}
