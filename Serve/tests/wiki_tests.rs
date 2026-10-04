use async_graphql::Request;
use chrono::Utc;
use serve::domain::models::{WikiArticle, WikiRevision};
use serve::domain::WikiRepository;
use serve::graphql::build_schema;
use serve::infrastructure::{
    auth::AuthUser,
    config::{AuthConfig, DatabaseConfig},
    db::postgres::Database,
    pubsub::MessageBroker,
};
use uuid::Uuid;

#[tokio::test]
async fn test_wiki_graphql_crud_and_slug_generation() {
    let test_file = format!("./data/test_wiki_db_{}.db", Uuid::new_v4());
    let _ = std::fs::remove_file(&test_file);
    let _ = std::fs::remove_file(format!("{}-shm", &test_file));
    let _ = std::fs::remove_file(format!("{}-wal", &test_file));

    let config = DatabaseConfig::sqlite_mode(&test_file);
    let db = Database::connect(&config)
        .await
        .expect("Failed to initialize SQLite database");

    let auth_config = AuthConfig::default();
    let broker = MessageBroker::default();
    let schema = build_schema(db.clone(), auth_config, broker);

    // 1. Create Article via GraphQL Mutation (Anonymous)
    let create_mutation = r##"
        mutation {
            createArticle(input: {
                title: "경복궁 나들이 (Gyeongbokgung Trip)",
                content: "# 경복궁 나들이\n\n조선 제일의 궁궐을 산책해 봅시다.",
                latitude: 37.5796,
                longitude: 126.9770,
                zoom: 16.0,
                tags: ["궁궐", "산책", "조선"],
                author: "익명탐험가"
            }) {
                id
                title
                slug
                summary
                category
                tags
                author
                views
                revisions {
                    id
                    title
                    editSummary
                }
            }
        }
    "##;

    let res = schema.execute(Request::new(create_mutation)).await;
    assert!(res.errors.is_empty(), "Create article error: {:?}", res.errors);

    let data = res.data.into_json().unwrap();
    let article_data = &data["createArticle"];
    let first_slug = article_data["slug"].as_str().unwrap().to_string();
    assert_eq!(article_data["title"], "경복궁 나들이 (Gyeongbokgung Trip)");
    assert_eq!(article_data["category"], "궁궐");
    assert_eq!(article_data["author"], "익명탐험가");
    assert_eq!(article_data["views"], 0);
    assert_eq!(article_data["revisions"].as_array().unwrap().len(), 1);

    // 2. Slug Collision Resolution: create another article with same title
    let res2 = schema.execute(Request::new(create_mutation)).await;
    assert!(res2.errors.is_empty());
    let data2 = res2.data.into_json().unwrap();
    let second_slug = data2["createArticle"]["slug"].as_str().unwrap().to_string();
    assert_ne!(first_slug, second_slug);
    assert!(second_slug.starts_with(&first_slug));
    assert!(second_slug.ends_with("-1"));

    // 3. Query Article by Slug with Revisions DataLoader
    let query_by_slug = format!(
        r##"
        query {{
            article(slug: "{}") {{
                id
                title
                slug
                category
                tags
                revisions {{
                    id
                    title
                    editSummary
                }}
            }}
        }}
    "##,
        first_slug
    );

    let res_query = schema.execute(Request::new(&query_by_slug)).await;
    assert!(res_query.errors.is_empty());
    let query_data = res_query.data.into_json().unwrap();
    let queried_article = &query_data["article"];
    assert_eq!(queried_article["slug"], first_slug);
    assert_eq!(queried_article["revisions"].as_array().unwrap().len(), 1);

    // 4. Update Article with New Revision
    let update_mutation = format!(
        r##"
        mutation {{
            updateArticle(
                slug: "{}"
                input: {{
                    title: "경복궁 역사 나들이"
                    content: "# 수정된 경복궁\n\n역사와 전통이 살아 숨쉬는 곳입니다."
                    tags: ["궁궐", "역사", "문화재"]
                    editSummary: "역사 정보 추가 및 태그 변경"
                }}
            ) {{
                id
                title
                category
                tags
                revisions {{
                    id
                    title
                    editSummary
                }}
            }}
        }}
    "##,
        first_slug
    );

    let res_update = schema.execute(Request::new(&update_mutation)).await;
    assert!(res_update.errors.is_empty(), "Update error: {:?}", res_update.errors);
    let update_data = res_update.data.into_json().unwrap();
    let updated_article = &update_data["updateArticle"];
    assert_eq!(updated_article["title"], "경복궁 역사 나들이");
    assert_eq!(updated_article["category"], "궁궐");
    // Should now have 2 revisions
    assert_eq!(updated_article["revisions"].as_array().unwrap().len(), 2);

    // 5. Query Articles with Filters (Tag & Bounding Box)
    let tag_query = r##"
        query {
            articles(filter: { tag: "역사" }) {
                id
                slug
                title
            }
        }
    "##;
    let res_tag = schema.execute(Request::new(tag_query)).await;
    let tag_data = res_tag.data.into_json().unwrap();
    assert_eq!(tag_data["articles"].as_array().unwrap().len(), 1);

    let bbox_query = r##"
        query {
            articles(filter: {
                minLat: 37.0,
                maxLat: 38.0,
                minLng: 126.0,
                maxLng: 127.0
            }) {
                id
                slug
                title
            }
        }
    "##;
    let res_bbox = schema.execute(Request::new(bbox_query)).await;
    let bbox_data = res_bbox.data.into_json().unwrap();
    assert_eq!(bbox_data["articles"].as_array().unwrap().len(), 2);

    // 6. Record View and Check Trends
    let first_article_id = updated_article["id"].as_str().unwrap();
    let record_view_mutation = format!(
        r##"
        mutation {{
            recordArticleView(articleId: "{}")
        }}
    "##,
        first_article_id
    );
    let res_view = schema.execute(Request::new(&record_view_mutation)).await;
    assert!(res_view.errors.is_empty());

    let trends_query = r##"
        query {
            wikiTrends(forceRefresh: true) {
                totalArticles
                articles {
                    rank
                    title
                    slug
                    score
                    change
                }
                tags {
                    rank
                    tag
                    count
                }
            }
        }
    "##;
    let res_trends = schema.execute(Request::new(trends_query)).await;
    assert!(res_trends.errors.is_empty(), "Trends error: {:?}", res_trends.errors);
    let trends_data = res_trends.data.into_json().unwrap();
    let wiki_trends = &trends_data["wikiTrends"];
    assert_eq!(wiki_trends["totalArticles"], 2);
    assert!(!wiki_trends["articles"].as_array().unwrap().is_empty());
    assert!(!wiki_trends["tags"].as_array().unwrap().is_empty());

    // 7. Delete Article
    let delete_mutation = format!(
        r##"
        mutation {{
            deleteArticle(slug: "{}")
        }}
    "##,
        first_slug
    );
    let res_delete = schema.execute(Request::new(&delete_mutation)).await;
    assert!(res_delete.errors.is_empty());
    assert_eq!(res_delete.data.into_json().unwrap()["deleteArticle"], true);

    // Verify deletion
    let res_verify = schema.execute(Request::new(&query_by_slug)).await;
    assert!(res_verify.errors.is_empty());
    assert!(res_verify.data.into_json().unwrap()["article"].is_null());

    let _ = std::fs::remove_file(&test_file);
    let _ = std::fs::remove_file(format!("{}-shm", &test_file));
    let _ = std::fs::remove_file(format!("{}-wal", &test_file));
}

#[tokio::test]
async fn test_wiki_authenticated_creation_and_seeding() {
    let test_file = format!("./data/test_wiki_seed_db_{}.db", Uuid::new_v4());
    let _ = std::fs::remove_file(&test_file);
    let _ = std::fs::remove_file(format!("{}-shm", &test_file));
    let _ = std::fs::remove_file(format!("{}-wal", &test_file));

    let config = DatabaseConfig::sqlite_mode(&test_file);
    let db = Database::connect(&config)
        .await
        .expect("Failed to initialize SQLite database");

    // 1. Run seed data (which includes 6 Seoul landmarks)
    db.seed_wiki_data(chrono::Utc::now())
        .await
        .expect("Failed to seed wiki data");

    let auth_config = AuthConfig::default();
    let broker = MessageBroker::default();
    let schema = build_schema(db.clone(), auth_config, broker);

    // 2. Query seeded articles
    let query_all = r##"
        query {
            articles(filter: { limit: 100 }) {
                id
                slug
                title
                category
                tags
                views
            }
        }
    "##;
    let res = schema.execute(Request::new(query_all)).await;
    assert!(res.errors.is_empty());
    let data = res.data.into_json().unwrap();
    let articles = data["articles"].as_array().unwrap();
    assert_eq!(articles.len(), 6, "Seeded articles count should be 6");

    // 3. Test Authenticated User Context Creation
    use serve::domain::repositories::UserRepository;
    let user = db
        .register_user(
            "ferro_explorer".to_string(),
            "explorer@ferro.dev".to_string(),
            "hashed_pw".to_string(),
            "Ferro Explorer".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to register test user");

    let logged_in_user_id = user.id;
    let auth_user = AuthUser {
        user_id: logged_in_user_id,
        username: "ferro_explorer".to_string(),
    };

    let create_authenticated = r##"
        mutation {
            createArticle(input: {
                title: "남산골 한옥마을 (Namsangol Hanok Village)",
                content: "# 남산골 한옥마을\n\n서울 도심 속 전통 한옥 정취를 느낄 수 있는 문화 공간.",
                latitude: 37.5591,
                longitude: 126.9942,
                tags: ["한옥", "전통", "산책", "문화체험"]
            }) {
                id
                title
                slug
                author
                userId
            }
        }
    "##;

    let req = Request::new(create_authenticated).data(auth_user);
    let res_auth = schema.execute(req).await;
    assert!(res_auth.errors.is_empty());
    let auth_data = res_auth.data.into_json().unwrap();
    let created = &auth_data["createArticle"];
    assert_eq!(created["author"], "ferro_explorer");
    assert_eq!(created["userId"], logged_in_user_id.to_string());

    // Clean up
    let _ = std::fs::remove_file(&test_file);
    let _ = std::fs::remove_file(format!("{}-shm", &test_file));
    let _ = std::fs::remove_file(format!("{}-wal", &test_file));
}

#[tokio::test]
async fn test_wiki_article_transaction_atomicity() {
    let test_file = format!("./data/test_wiki_tx_{}.db", Uuid::new_v4());
    let _ = std::fs::remove_file(&test_file);
    let _ = std::fs::remove_file(format!("{}-shm", &test_file));
    let _ = std::fs::remove_file(format!("{}-wal", &test_file));

    let config = DatabaseConfig::sqlite_mode(&test_file);
    let db = Database::connect(&config).await.expect("Failed to initialize SQLite");

    let article_id = Uuid::new_v4();
    let rev_id = Uuid::new_v4();

    let article = WikiArticle {
        id: article_id,
        user_id: None,
        title: "Atomic Test Landmark".to_string(),
        slug: "atomic-test-landmark".to_string(),
        summary: Some("Atomic summary".to_string()),
        content: "Atomic content".to_string(),
        latitude: 37.5,
        longitude: 127.0,
        zoom: 15.0,
        category: "Test".to_string(),
        tags: vec!["atomic".to_string()],
        geojson: None,
        author: "Tester".to_string(),
        views: 0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let revision = WikiRevision {
        id: rev_id,
        article_id,
        user_id: None,
        title: article.title.clone(),
        content: article.content.clone(),
        latitude: article.latitude,
        longitude: article.longitude,
        edit_summary: Some("Initial revision".to_string()),
        author: article.author.clone(),
        created_at: Utc::now(),
    };

    // 1. Success case: both article and revision exist
    let created = db
        .create_article(&article, &revision)
        .await
        .expect("Article creation should succeed");
    assert_eq!(created.id, article_id);

    let fetched_article = db.find_article_by_id(article_id).await.unwrap();
    assert!(fetched_article.is_some());
    let fetched_revs = db.get_revisions(article_id, 10).await.unwrap();
    assert_eq!(fetched_revs.len(), 1);

    // 2. Conflict / rollback case: attempting to insert same article_id or duplicate revision_id
    // Here we pass a new article but with the EXISTING revision.id (PRIMARY KEY conflict on wiki_revisions)
    let bad_article_id = Uuid::new_v4();
    let bad_article = WikiArticle {
        id: bad_article_id,
        slug: "bad-atomic-test".to_string(),
        ..article.clone()
    };
    let duplicate_rev = WikiRevision {
        id: rev_id, // Already exists! Primary key violation in wiki_revisions
        article_id: bad_article_id,
        ..revision.clone()
    };

    let res = db.create_article(&bad_article, &duplicate_rev).await;
    assert!(
        res.is_err(),
        "Duplicate revision ID must cause transaction rollback"
    );

    // Verify rollback: bad_article MUST NOT exist in wiki_articles!
    let rolled_back_article = db.find_article_by_id(bad_article_id).await.unwrap();
    assert!(
        rolled_back_article.is_none(),
        "Article insert should have been rolled back!"
    );

    let _ = std::fs::remove_file(&test_file);
    let _ = std::fs::remove_file(format!("{}-shm", &test_file));
    let _ = std::fs::remove_file(format!("{}-wal", &test_file));
}
