use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serve::graphql::{build_schema, build_schema_with_options};
use serve::infrastructure::config::AppConfig;
use serve::infrastructure::db::postgres::Database;
use serve::routes::create_router;
use tower::ServiceExt;

#[tokio::test]
async fn test_graphql_standard_error_codes_and_extensions() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://serve:password@localhost:5432/serve".to_string());

    let db = match Database::connect_url(&database_url).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test: PostgreSQL connection failed: {}", e);
            return;
        }
    };

    let config = AppConfig::default();
    let schema = build_schema(
        db.clone(),
        config.auth.clone(),
        serve::infrastructure::pubsub::MessageBroker::default(),
    );
    let app = create_router(schema, db, config);

    // 1. Error on invalid login
    let login_query = r#"{"query": "mutation { login(usernameOrEmail: \"nonexistent_user\", password: \"wrongpw\") { token } }"}"#;

    let req_login = Request::builder()
        .uri("/graphql")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(login_query))
        .unwrap();

    let res_login = app.clone().oneshot(req_login).await.unwrap();
    assert_eq!(res_login.status(), StatusCode::OK);

    let body_login = axum::body::to_bytes(res_login.into_body(), usize::MAX)
        .await
        .unwrap();
    let json_login: serde_json::Value = serde_json::from_slice(&body_login).unwrap();
    let errors_login = json_login["errors"]
        .as_array()
        .expect("Expected GraphQL errors");
    assert!(!errors_login.is_empty());
    assert_eq!(
        errors_login[0]["extensions"]["code"].as_str().unwrap(),
        "AUTH_INVALID_CREDENTIALS"
    );
    assert_eq!(
        errors_login[0]["message"].as_str().unwrap(),
        "AUTH_INVALID_CREDENTIALS"
    );

    // 2. Error on unauthenticated query
    let me_query = r#"{"query": "query { me { id username } }"}"#;
    let req_me = Request::builder()
        .uri("/graphql")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(me_query))
        .unwrap();

    let res_me = app.oneshot(req_me).await.unwrap();
    let body_me = axum::body::to_bytes(res_me.into_body(), usize::MAX)
        .await
        .unwrap();
    let json_me: serde_json::Value = serde_json::from_slice(&body_me).unwrap();
    let errors_me = json_me["errors"].as_array().expect("Expected errors");
    assert_eq!(
        errors_me[0]["extensions"]["code"].as_str().unwrap(),
        "AUTH_USER_ID_OR_TOKEN_REQUIRED"
    );
    assert_eq!(
        errors_me[0]["message"].as_str().unwrap(),
        "AUTH_USER_ID_OR_TOKEN_REQUIRED"
    );
}

#[tokio::test]
async fn test_graphql_introspection_disabled_in_production() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://serve:password@localhost:5432/serve".to_string());

    let db = match Database::connect_url(&database_url).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test: PostgreSQL connection failed: {}", e);
            return;
        }
    };

    let config = AppConfig::default();
    // Build schema with enable_introspection = false
    let schema = build_schema_with_options(
        db.clone(),
        config.auth.clone(),
        serve::infrastructure::pubsub::MessageBroker::default(),
        false,
    );
    let app = create_router(schema, db, config);

    let introspection_query = r#"{"query": "query { __schema { types { name } } }"}"#;
    let req = Request::builder()
        .uri("/graphql")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(introspection_query))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let errors = json["errors"]
        .as_array()
        .expect("Expected errors when introspection disabled");
    assert!(!errors.is_empty());
}
