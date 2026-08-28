use async_graphql::Request;
use serve::graphql::build_schema;
use serve::infrastructure::{auth::AuthUser, config::AuthConfig, db::postgres::Database};
use uuid::Uuid;

#[tokio::test]
async fn test_new_features_flow() {
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

    // 1. Sign up a test user
    let user_name = format!("trend_user_{}", Uuid::new_v4().simple());
    let signup_mutation = format!(
        r#"
        mutation {{
            signup(
                username: "{}"
                email: "{}@example.com"
                password: "password123"
                displayName: "Trending Tester"
            ) {{
                user {{
                    id
                    username
                    displayName
                }}
            }}
        }}
        "#,
        user_name, user_name
    );

    let res = schema.execute(signup_mutation.as_str()).await;
    assert!(res.errors.is_empty(), "Signup failed: {:?}", res.errors);
    let json_data = serde_json::to_value(&res.data).unwrap();
    let user_id_str = json_data
        .get("signup")
        .unwrap()
        .get("user")
        .unwrap()
        .get("id")
        .unwrap()
        .as_str()
        .unwrap();
    let user_id = Uuid::parse_str(user_id_str).unwrap();
    let auth_user = AuthUser {
        user_id,
        username: user_name.clone(),
    };

    // 2. Test searchUsers query
    let search_query = format!(
        r#"
        query {{
            searchUsers(query: "{}", limit: 5) {{
                id
                username
                displayName
            }}
        }}
        "#,
        &user_name[0..10]
    );
    let res = schema.execute(search_query.as_str()).await;
    assert!(
        res.errors.is_empty(),
        "searchUsers failed: {:?}",
        res.errors
    );
    let search_data = serde_json::to_value(&res.data).unwrap();
    let users_list = search_data.get("searchUsers").unwrap().as_array().unwrap();
    assert!(!users_list.is_empty());
    assert_eq!(
        users_list[0].get("username").unwrap().as_str().unwrap(),
        user_name
    );

    // 3. Create a post with hashtags (#rust #axum #blazinglyFast)
    let create_post_query = r#"
        mutation {
            createPost(content: "Learning #rust with #axum and #ferro! It is #blazinglyFast.") {
                id
                content
                isSavedByMe
            }
        }
    "#;
    let req = Request::new(create_post_query).data(auth_user.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty(), "createPost failed: {:?}", res.errors);
    let post_data = serde_json::to_value(&res.data).unwrap();
    let post_id = post_data
        .get("createPost")
        .unwrap()
        .get("id")
        .unwrap()
        .as_str()
        .unwrap();
    let is_saved_initially = post_data
        .get("createPost")
        .unwrap()
        .get("isSavedByMe")
        .unwrap()
        .as_bool()
        .unwrap();
    assert!(!is_saved_initially);

    // 4. Test trendingHashtags query
    let trending_query = r#"
        query {
            trendingHashtags(limit: 5) {
                tag
                count
            }
        }
    "#;
    let res = schema.execute(trending_query).await;
    assert!(
        res.errors.is_empty(),
        "trendingHashtags failed: {:?}",
        res.errors
    );
    let trend_data = serde_json::to_value(&res.data).unwrap();
    let trends = trend_data
        .get("trendingHashtags")
        .unwrap()
        .as_array()
        .unwrap();
    assert!(!trends.is_empty());
    let has_rust = trends
        .iter()
        .any(|t| t.get("tag").unwrap().as_str().unwrap() == "rust");
    assert!(has_rust, "Expected #rust in trending hashtags");

    // 5. Test updatePost mutation
    let update_post_query = format!(
        r#"
        mutation {{
            updatePost(postId: "{}", content: "Updated content for #rust #ferro.") {{
                id
                content
            }}
        }}
        "#,
        post_id
    );
    let req = Request::new(update_post_query.as_str()).data(auth_user.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty(), "updatePost failed: {:?}", res.errors);
    let updated_post = serde_json::to_value(&res.data).unwrap();
    assert_eq!(
        updated_post
            .get("updatePost")
            .unwrap()
            .get("content")
            .unwrap()
            .as_str()
            .unwrap(),
        "Updated content for #rust #ferro."
    );

    // 6. Test savePost / unsavePost / savedPosts
    let save_mutation = format!(
        r#"
        mutation {{
            savePost(postId: "{}") {{
                id
                isSavedByMe
            }}
        }}
        "#,
        post_id
    );
    let req = Request::new(save_mutation.as_str()).data(auth_user.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty(), "savePost failed: {:?}", res.errors);
    let saved_data = serde_json::to_value(&res.data).unwrap();
    assert!(
        saved_data
            .get("savePost")
            .unwrap()
            .get("isSavedByMe")
            .unwrap()
            .as_bool()
            .unwrap()
    );

    // Verify savedPosts query returns the post
    let saved_posts_query = r#"
        query {
            savedPosts {
                id
                content
                isSavedByMe
            }
        }
    "#;
    let req = Request::new(saved_posts_query).data(auth_user.clone());
    let res = schema.execute(req).await;
    assert!(
        res.errors.is_empty(),
        "savedPosts query failed: {:?}",
        res.errors
    );
    let list_data = serde_json::to_value(&res.data).unwrap();
    let saved_list = list_data.get("savedPosts").unwrap().as_array().unwrap();
    assert!(
        saved_list
            .iter()
            .any(|p| p.get("id").unwrap().as_str().unwrap() == post_id)
    );

    // Unsave post
    let unsave_mutation = format!(
        r#"
        mutation {{
            unsavePost(postId: "{}") {{
                id
                isSavedByMe
            }}
        }}
        "#,
        post_id
    );
    let req = Request::new(unsave_mutation.as_str()).data(auth_user.clone());
    let res = schema.execute(req).await;
    assert!(res.errors.is_empty(), "unsavePost failed: {:?}", res.errors);
    let unsaved_data = serde_json::to_value(&res.data).unwrap();
    assert!(
        !unsaved_data
            .get("unsavePost")
            .unwrap()
            .get("isSavedByMe")
            .unwrap()
            .as_bool()
            .unwrap()
    );

    // 7. Test createComment and deleteComment
    let create_comment_query = format!(
        r#"
        mutation {{
            createComment(postId: "{}", content: "Great test post!") {{
                id
                content
            }}
        }}
        "#,
        post_id
    );
    let req = Request::new(create_comment_query.as_str()).data(auth_user.clone());
    let res = schema.execute(req).await;
    assert!(
        res.errors.is_empty(),
        "createComment failed: {:?}",
        res.errors
    );
    let comment_data = serde_json::to_value(&res.data).unwrap();
    let comment_id = comment_data
        .get("createComment")
        .unwrap()
        .get("id")
        .unwrap()
        .as_str()
        .unwrap();

    let delete_comment_query = format!(
        r#"
        mutation {{
            deleteComment(commentId: "{}")
        }}
        "#,
        comment_id
    );
    let req = Request::new(delete_comment_query.as_str()).data(auth_user.clone());
    let res = schema.execute(req).await;
    assert!(
        res.errors.is_empty(),
        "deleteComment failed: {:?}",
        res.errors
    );
    let del_data = serde_json::to_value(&res.data).unwrap();
    assert!(del_data.get("deleteComment").unwrap().as_bool().unwrap());
}
