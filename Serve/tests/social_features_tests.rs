use async_graphql::Request;
use chrono::Utc;
use serve::graphql::build_schema;
use serve::graphql::types::{decode_cursor, encode_cursor};
use serve::infrastructure::{auth::AuthUser, config::AuthConfig, db::postgres::Database};
use uuid::Uuid;

#[test]
fn test_cursor_encoding_and_decoding() {
    let now = Utc::now();
    let id = Uuid::new_v4();

    let cursor = encode_cursor(now, id);
    assert!(!cursor.is_empty());

    let decoded = decode_cursor(&cursor).expect("Failed to decode cursor");
    assert_eq!(decoded.1, id);
    assert_eq!(decoded.0.timestamp(), now.timestamp());
}

#[tokio::test]
async fn test_social_features_flow() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://serve:password@localhost:5432/serve".to_string());

    let db = match Database::connect_url(&database_url).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!(
                "Skipping integration test: PostgreSQL connection failed: {}",
                e
            );
            return;
        }
    };

    let broker = serve::infrastructure::pubsub::MessageBroker::default();
    let schema = build_schema(db, AuthConfig::default(), broker);

    // 1. Sign up Alice (Author)
    let alice_name = format!("alice_{}", Uuid::new_v4().simple());
    let signup_alice = format!(
        r#"
        mutation {{
            signup(
                username: "{}"
                email: "{}@example.com"
                password: "password123"
                displayName: "Alice Dev"
            ) {{
                user {{ id username displayName isPrivate }}
            }}
        }}
        "#,
        alice_name, alice_name
    );
    let res = schema.execute(signup_alice.as_str()).await;
    assert!(
        res.errors.is_empty(),
        "Alice signup failed: {:?}",
        res.errors
    );
    let json_data = serde_json::to_value(&res.data).unwrap();
    let alice_id_str = json_data["signup"]["user"]["id"].as_str().unwrap();
    let alice_id = Uuid::parse_str(alice_id_str).unwrap();
    let auth_alice = AuthUser {
        user_id: alice_id,
        username: alice_name.clone(),
    };

    // 2. Sign up Bob (Follower / Commenter)
    let bob_name = format!("bob_{}", Uuid::new_v4().simple());
    let signup_bob = format!(
        r#"
        mutation {{
            signup(
                username: "{}"
                email: "{}@example.com"
                password: "password123"
                displayName: "Bob Contributor"
            ) {{
                user {{ id username displayName }}
            }}
        }}
        "#,
        bob_name, bob_name
    );
    let res = schema.execute(signup_bob.as_str()).await;
    assert!(res.errors.is_empty(), "Bob signup failed: {:?}", res.errors);
    let json_data = serde_json::to_value(&res.data).unwrap();
    let bob_id_str = json_data["signup"]["user"]["id"].as_str().unwrap();
    let bob_id = Uuid::parse_str(bob_id_str).unwrap();
    let auth_bob = AuthUser {
        user_id: bob_id,
        username: bob_name.clone(),
    };

    // 3. Bob follows Alice -> triggers Follow notification for Alice
    let follow_mutation = format!(
        r#"
        mutation {{
            followUser(followeeId: "{}") {{
                id
                followersCount
            }}
        }}
        "#,
        alice_id
    );
    let req = Request::new(follow_mutation).data(auth_bob.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty(), "Follow failed: {:?}", res.errors);

    // Verify Alice received 1 unread notification (FOLLOW)
    let notifs_query = r#"
        query {
            unreadNotificationsCount
            notifications {
                id
                type
                isRead
                actor { username }
            }
        }
    "#;
    let req = Request::new(notifs_query).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(
        res.errors.is_empty(),
        "Get notifications failed: {:?}",
        res.errors
    );
    let json_data = serde_json::to_value(&res.data).unwrap();
    assert_eq!(json_data["unreadNotificationsCount"].as_u64().unwrap(), 1);
    let notifs = json_data["notifications"].as_array().unwrap();
    assert!(!notifs.is_empty());
    assert_eq!(notifs[0]["type"].as_str().unwrap(), "FOLLOW");
    assert_eq!(notifs[0]["actor"]["username"].as_str().unwrap(), bob_name);

    // 4. Alice creates posts to test Cursor Pagination and Hashtag / Mentions
    let mut post_ids = Vec::new();
    for i in 1..=3 {
        let create_post = format!(
            r#"
            mutation {{
                createPost(content: "Post #{} by Alice about #ferrorocks and @{}") {{
                    id
                    content
                    hashtags
                }}
            }}
            "#,
            i, bob_name
        );
        let req = Request::new(create_post).data(auth_alice.clone());
        let res = schema.execute(req).await;
        assert!(
            res.errors.is_empty(),
            "Create post failed: {:?}",
            res.errors
        );
        let p_data = serde_json::to_value(&res.data).unwrap();
        post_ids.push(p_data["createPost"]["id"].as_str().unwrap().to_string());
    }

    // 5. Test postsConnection cursor pagination (first: 2)
    let conn_query = r#"
        query {
            postsConnection(first: 2) {
                pageInfo {
                    hasNextPage
                    hasPreviousPage
                    startCursor
                    endCursor
                }
                edges {
                    cursor
                    node {
                        id
                        content
                    }
                }
            }
        }
    "#;
    let res = schema.execute(conn_query).await;
    assert!(
        res.errors.is_empty(),
        "postsConnection failed: {:?}",
        res.errors
    );
    let json_data = serde_json::to_value(&res.data).unwrap();
    let page_info = &json_data["postsConnection"]["pageInfo"];
    assert!(page_info["hasNextPage"].as_bool().unwrap());
    let end_cursor = page_info["endCursor"].as_str().unwrap();

    // Fetch next page using after cursor
    let next_page_query = format!(
        r#"
        query {{
            postsConnection(first: 2, after: "{}") {{
                pageInfo {{
                    hasPreviousPage
                }}
                edges {{
                    node {{ id content }}
                }}
            }}
        }}
        "#,
        end_cursor
    );
    let res = schema.execute(next_page_query.as_str()).await;
    assert!(
        res.errors.is_empty(),
        "postsConnection page 2 failed: {:?}",
        res.errors
    );

    // 6. Test Repost & Quote Post: Bob reposts Alice's post
    let target_post_id = &post_ids[0];
    let repost_mutation = format!(
        r#"
        mutation {{
            repostPost(postId: "{}") {{
                id
                repostsCount
                isRepostedByMe
            }}
        }}
        "#,
        target_post_id
    );
    let req = Request::new(repost_mutation).data(auth_bob.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty(), "Repost failed: {:?}", res.errors);
    let json_data = serde_json::to_value(&res.data).unwrap();
    assert_eq!(json_data["repostPost"]["repostsCount"].as_u64().unwrap(), 1);
    assert!(json_data["repostPost"]["isRepostedByMe"].as_bool().unwrap());

    // Bob quotes Alice's post
    let quote_mutation = format!(
        r#"
        mutation {{
            quotePost(postId: "{}", content: "Check out this awesome post from Alice!") {{
                id
                content
                quotePostId
                quotePost {{
                    id
                }}
            }}
        }}
        "#,
        target_post_id
    );
    let req = Request::new(quote_mutation).data(auth_bob.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty(), "Quote post failed: {:?}", res.errors);
    let json_data = serde_json::to_value(&res.data).unwrap();
    assert_eq!(
        json_data["quotePost"]["quotePostId"].as_str().unwrap(),
        target_post_id
    );

    // 7. Test Search & Discovery: Posts by hashtag
    let hashtag_query = r#"
        query {
            postsByHashtag(hashtag: "ferrorocks", first: 5) {
                edges {
                    node {
                        id
                        content
                    }
                }
            }
        }
    "#;
    let res = schema.execute(hashtag_query).await;
    assert!(
        res.errors.is_empty(),
        "postsByHashtag failed: {:?}",
        res.errors
    );
    let json_data = serde_json::to_value(&res.data).unwrap();
    let edges = json_data["postsByHashtag"]["edges"].as_array().unwrap();
    assert!(!edges.is_empty());

    // 8. Test Trust & Safety: Privacy & Follow Request flow
    // Alice turns private mode ON
    let update_privacy = r#"
        mutation {
            updateUserPrivacy(isPrivate: true) {
                id
                isPrivate
            }
        }
    "#;
    let req = Request::new(update_privacy).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    assert!(
        res.data.into_json().unwrap()["updateUserPrivacy"]["isPrivate"]
            .as_bool()
            .unwrap()
    );

    // Charlie signs up and follows private Alice -> creates FollowRequest
    let charlie_name = format!("charlie_{}", Uuid::new_v4().simple());
    let signup_charlie = format!(
        r#"
        mutation {{
            signup(
                username: "{}"
                email: "{}@example.com"
                password: "password123"
                displayName: "Charlie Reviewer"
            ) {{
                user {{ id username }}
            }}
        }}
        "#,
        charlie_name, charlie_name
    );
    let res = schema.execute(signup_charlie.as_str()).await;
    let json_data = serde_json::to_value(&res.data).unwrap();
    let charlie_id_str = json_data["signup"]["user"]["id"].as_str().unwrap();
    let charlie_id = Uuid::parse_str(charlie_id_str).unwrap();
    let auth_charlie = AuthUser {
        user_id: charlie_id,
        username: charlie_name.clone(),
    };

    let charlie_follow_req = format!(
        r#"
        mutation {{
            followUser(followeeId: "{}") {{
                id
                hasPendingFollowRequest
            }}
        }}
        "#,
        alice_id
    );
    let req = Request::new(charlie_follow_req).data(auth_charlie.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    assert!(
        res.data.into_json().unwrap()["followUser"]["hasPendingFollowRequest"]
            .as_bool()
            .unwrap()
    );

    // Alice accepts Charlie's follow request
    let accept_req = format!(
        r#"
        mutation {{
            acceptFollowRequest(requesterId: "{}") {{
                id
                username
            }}
        }}
        "#,
        charlie_id
    );
    let req = Request::new(accept_req).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(
        res.errors.is_empty(),
        "Accept follow request failed: {:?}",
        res.errors
    );

    // 9. Test Trust & Safety: Blocks
    // Bob blocks Charlie
    let block_mutation = format!(
        r#"
        mutation {{
            blockUser(userId: "{}")
        }}
        "#,
        charlie_id
    );
    let req = Request::new(block_mutation).data(auth_bob.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    assert!(
        res.data.into_json().unwrap()["blockUser"]
            .as_bool()
            .unwrap()
    );

    // Charlie tries to DM Bob -> Blocked error
    let blocked_dm_mutation = format!(
        r#"
        mutation {{
            sendDirectMessage(recipientId: "{}", content: "Hello Bob!") {{
                id
            }}
        }}
        "#,
        bob_id
    );
    let req = Request::new(blocked_dm_mutation).data(auth_charlie.clone());
    let res = schema.execute(req).await;
    assert!(!res.errors.is_empty(), "Blocked DM should return error");

    // 10. Test Advanced DMs: Group Chats & Message Edit
    let group_chat_mutation = format!(
        r#"
        mutation {{
            createGroupConversation(title: "Dev Team", participantIds: ["{}", "{}"]) {{
                id
                title
                isGroup
                participants {{
                    id
                }}
            }}
        }}
        "#,
        bob_id, charlie_id
    );
    let req = Request::new(group_chat_mutation).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(
        res.errors.is_empty(),
        "Group chat creation failed: {:?}",
        res.errors
    );
    let json_data = serde_json::to_value(&res.data).unwrap();
    let group_id = json_data["createGroupConversation"]["id"].as_str().unwrap();

    // Alice sends a message to the group
    let group_msg_mutation = format!(
        r#"
        mutation {{
            sendGroupMessage(conversationId: "{}", content: "Hello Team!") {{
                id
                content
                conversationId
            }}
        }}
        "#,
        group_id
    );
    let req = Request::new(group_msg_mutation).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(
        res.errors.is_empty(),
        "Send group message failed: {:?}",
        res.errors
    );

    // Alice sends typing indicator
    let typing_mutation = format!(
        r#"
        mutation {{
            sendTypingIndicator(conversationId: "{}", isTyping: true)
        }}
        "#,
        group_id
    );
    let req = Request::new(typing_mutation).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    assert!(
        res.data.into_json().unwrap()["sendTypingIndicator"]
            .as_bool()
            .unwrap()
    );

    // 11. Test Threaded Comments Interactions: Likes, Edits, Pinned Comments
    let comment_post_id = &post_ids[0];
    let create_comm_mutation = format!(
        r#"
        mutation {{
            createComment(postId: "{}", content: "Top comment for interaction testing") {{
                id
                content
                likesCount
                isEdited
                depth
            }}
        }}
        "#,
        comment_post_id
    );
    let req = Request::new(create_comm_mutation).data(auth_bob.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    let comm_data = serde_json::to_value(&res.data).unwrap();
    let comm_id = comm_data["createComment"]["id"].as_str().unwrap();
    assert_eq!(
        comm_data["createComment"]["likesCount"].as_u64().unwrap(),
        0
    );
    assert_eq!(comm_data["createComment"]["depth"].as_u64().unwrap(), 0);

    // Alice likes Bob's comment
    let like_comm_mutation = format!(
        r#"
        mutation {{
            likeComment(commentId: "{}") {{
                id
                likesCount
                isLikedByMe
            }}
        }}
        "#,
        comm_id
    );
    let req = Request::new(like_comm_mutation).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    let like_comm_data = serde_json::to_value(&res.data).unwrap();
    assert_eq!(
        like_comm_data["likeComment"]["likesCount"]
            .as_u64()
            .unwrap(),
        1
    );
    assert!(
        like_comm_data["likeComment"]["isLikedByMe"]
            .as_bool()
            .unwrap()
    );

    // Bob edits his comment
    let edit_comm_mutation = format!(
        r#"
        mutation {{
            editComment(commentId: "{}", content: "Edited top comment content") {{
                id
                content
                isEdited
            }}
        }}
        "#,
        comm_id
    );
    let req = Request::new(edit_comm_mutation).data(auth_bob.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    let edit_data = serde_json::to_value(&res.data).unwrap();
    assert_eq!(
        edit_data["editComment"]["content"].as_str().unwrap(),
        "Edited top comment content"
    );
    assert!(edit_data["editComment"]["isEdited"].as_bool().unwrap());

    // Alice pins Bob's comment to the post
    let pin_comm_mutation = format!(
        r#"
        mutation {{
            pinComment(postId: "{}", commentId: "{}") {{
                id
                pinnedComment {{
                    id
                    content
                }}
            }}
        }}
        "#,
        comment_post_id, comm_id
    );
    let req = Request::new(pin_comm_mutation).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    let pin_data = serde_json::to_value(&res.data).unwrap();
    assert_eq!(
        pin_data["pinComment"]["pinnedComment"]["id"]
            .as_str()
            .unwrap(),
        comm_id
    );

    // 12. Test Post with Media Gallery and Interactive Poll
    let create_post_with_media_poll = r#"
        mutation {
            createPost(
                content: "Exploring Rust with media and polls!"
                audience: PUBLIC
                media: [
                    {
                        mediaUrl: "https://example.com/rust1.jpg"
                        mediaType: IMAGE
                        altText: "Rust Crab"
                    },
                    {
                        mediaUrl: "https://example.com/ferro.mp4"
                        mediaType: VIDEO
                        altText: "Ferro Demo Video"
                    }
                ]
                poll: {
                    question: "What is your favorite Rust async runtime?"
                    options: ["Tokio", "async-std", "smol"]
                    durationSeconds: 3600
                }
            ) {
                id
                audience
                media {
                    id
                    mediaUrl
                    mediaType
                    altText
                }
                poll {
                    id
                    question
                    totalVotes
                    options {
                        id
                        optionText
                        votesCount
                    }
                }
            }
        }
    "#;
    let req = Request::new(create_post_with_media_poll).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(
        res.errors.is_empty(),
        "Create post with media & poll failed: {:?}",
        res.errors
    );
    let post_media_data = serde_json::to_value(&res.data).unwrap();
    let poll_post = &post_media_data["createPost"];
    let poll_post_id = poll_post["id"].as_str().unwrap();
    assert_eq!(poll_post["media"].as_array().unwrap().len(), 2);
    let poll_id = poll_post["poll"]["id"].as_str().unwrap();
    let tokio_opt_id = poll_post["poll"]["options"][0]["id"].as_str().unwrap();

    // Bob votes on Tokio in Alice's poll
    let vote_mutation = format!(
        r#"
        mutation {{
            votePoll(pollId: "{}", optionId: "{}") {{
                id
                optionText
                votesCount
                percentage
                isVotedByMe
            }}
        }}
        "#,
        poll_id, tokio_opt_id
    );
    let req = Request::new(vote_mutation).data(auth_bob.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty(), "Vote failed: {:?}", res.errors);
    let vote_data = serde_json::to_value(&res.data).unwrap();
    assert_eq!(vote_data["votePoll"]["votesCount"].as_u64().unwrap(), 1);
    assert_eq!(vote_data["votePoll"]["percentage"].as_f64().unwrap(), 100.0);
    assert!(vote_data["votePoll"]["isVotedByMe"].as_bool().unwrap());

    // 13. Test Post Views & Analytics
    let view_mutation = format!(
        r#"
        mutation {{
            recordPostView(postId: "{}")
        }}
        "#,
        poll_post_id
    );
    let req = Request::new(view_mutation).data(auth_bob.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    assert!(
        res.data.into_json().unwrap()["recordPostView"]
            .as_bool()
            .unwrap()
    );

    let analytics_query = format!(
        r#"
        query {{
            postAnalytics(postId: "{}") {{
                postId
                viewsCount
                likesCount
                engagementRate
            }}
        }}
        "#,
        poll_post_id
    );
    let res = schema.execute(analytics_query.as_str()).await;
    assert!(res.errors.is_empty());
    let an_data = serde_json::to_value(&res.data).unwrap();
    assert!(an_data["postAnalytics"]["viewsCount"].as_i64().unwrap() >= 1);

    // 14. Test Custom User Lists
    let create_list_mutation = format!(
        r#"
        mutation {{
            createUserList(
                name: "Rust Core Contributors"
                description: "Key people in Rust ecosystem"
                memberIds: ["{}"]
            ) {{
                id
                name
                membersCount
                members {{ id username }}
            }}
        }}
        "#,
        bob_id
    );
    let req = Request::new(create_list_mutation).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    let list_data = serde_json::to_value(&res.data).unwrap();
    let list_id = list_data["createUserList"]["id"].as_str().unwrap();
    assert_eq!(
        list_data["createUserList"]["membersCount"]
            .as_u64()
            .unwrap(),
        1
    );

    // Query list feed
    let list_feed_query = format!(
        r#"
        query {{
            listFeedConnection(listId: "{}", first: 5) {{
                edges {{
                    node {{ id content }}
                }}
            }}
        }}
        "#,
        list_id
    );
    let res = schema.execute(list_feed_query.as_str()).await;
    assert!(res.errors.is_empty());

    // 15. Test Bookmark Collections
    let create_coll_mutation = r#"
        mutation {
            createBookmarkCollection(
                name: "Rust Learning Resources"
                description: "Curated Rust articles and tips"
                isPrivate: true
            ) {
                id
                name
                isPrivate
            }
        }
    "#;
    let req = Request::new(create_coll_mutation).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    let coll_data = serde_json::to_value(&res.data).unwrap();
    let coll_id = coll_data["createBookmarkCollection"]["id"]
        .as_str()
        .unwrap();

    // Alice adds post to collection
    let add_to_coll_mutation = format!(
        r#"
        mutation {{
            addPostToCollection(collectionId: "{}", postId: "{}")
        }}
        "#,
        coll_id, poll_post_id
    );
    let req = Request::new(add_to_coll_mutation).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    assert!(
        res.data.into_json().unwrap()["addPostToCollection"]
            .as_bool()
            .unwrap()
    );

    // Query collection posts
    let coll_posts_query = format!(
        r#"
        query {{
            bookmarkCollection(id: "{}") {{
                id
                name
                postsConnection(first: 5) {{
                    edges {{
                        node {{ id content }}
                    }}
                }}
            }}
        }}
        "#,
        coll_id
    );
    let res = schema.execute(coll_posts_query.as_str()).await;
    assert!(res.errors.is_empty());
    let cp_data = serde_json::to_value(&res.data).unwrap();
    assert_eq!(
        cp_data["bookmarkCollection"]["postsConnection"]["edges"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    // 16. Test Content Reporting & Moderation
    let report_mutation = format!(
        r#"
        mutation {{
            reportContent(
                targetType: POST
                targetId: "{}"
                reason: SPAM
                details: "Suspicious spam link"
            ) {{
                id
                targetType
                reason
                status
            }}
        }}
        "#,
        poll_post_id
    );
    let req = Request::new(report_mutation).data(auth_bob.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    let report_data = serde_json::to_value(&res.data).unwrap();
    let report_id = report_data["reportContent"]["id"].as_str().unwrap();
    assert_eq!(
        report_data["reportContent"]["status"].as_str().unwrap(),
        "PENDING"
    );

    // Resolve report
    let resolve_mutation = format!(
        r#"
        mutation {{
            resolveReport(reportId: "{}", status: RESOLVED) {{
                id
                status
            }}
        }}
        "#,
        report_id
    );
    let req = Request::new(resolve_mutation).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty());
    assert_eq!(
        res.data.into_json().unwrap()["resolveReport"]["status"]
            .as_str()
            .unwrap(),
        "RESOLVED"
    );

    // 17. Test 2FA TOTP Setup & Enable / Disable
    let setup_2fa_mutation = r#"
        mutation {
            setup2fa {
                secret
                otpauthUri
            }
        }
    "#;
    let req = Request::new(setup_2fa_mutation).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty(), "setup2fa failed: {:?}", res.errors);
    let setup_data = serde_json::to_value(&res.data).unwrap();
    let secret = setup_data["setup2fa"]["secret"].as_str().unwrap();
    assert!(!secret.is_empty());

    // Generate valid TOTP code from secret
    let secret_bytes = totp_rs::Secret::Encoded(secret.to_string())
        .to_bytes()
        .unwrap();
    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Serve".to_string()),
        alice_name.to_string(),
    )
    .unwrap();
    let valid_code = totp.generate_current().unwrap();

    // Enable 2FA with valid code
    let enable_2fa_mutation = format!(
        r#"
        mutation {{
            enable2fa(code: "{}")
        }}
        "#,
        valid_code
    );
    let req = Request::new(enable_2fa_mutation).data(auth_alice.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty(), "enable2fa failed: {:?}", res.errors);
    assert!(
        res.data.into_json().unwrap()["enable2fa"]
            .as_bool()
            .unwrap()
    );
}
