use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serve::graphql::build_schema;
use serve::infrastructure::security::RateLimiter;
use serve::infrastructure::{config::AppConfig, db::postgres::Database, pubsub::MessageBroker};
use serve::routes::create_router;
use std::time::Duration;
use tower::ServiceExt;

#[tokio::test]
async fn test_rate_limiter_sliding_window() {
    // 3 requests allowed per 1 second window
    let limiter = RateLimiter::new(3, Duration::from_secs(1));
    let client_ip = "192.168.1.100";

    // 1st request -> allowed (remaining: 2)
    let (allowed, remaining, _) = limiter.check(client_ip).await;
    assert!(allowed);
    assert_eq!(remaining, 2);

    // 2nd request -> allowed (remaining: 1)
    let (allowed, remaining, _) = limiter.check(client_ip).await;
    assert!(allowed);
    assert_eq!(remaining, 1);

    // 3rd request -> allowed (remaining: 0)
    let (allowed, remaining, _) = limiter.check(client_ip).await;
    assert!(allowed);
    assert_eq!(remaining, 0);

    // 4th request -> blocked! (remaining: 0)
    let (allowed, remaining, _) = limiter.check(client_ip).await;
    assert!(!allowed);
    assert_eq!(remaining, 0);

    // Another client IP -> allowed independently
    let (other_allowed, other_remaining, _) = limiter.check("10.0.0.1").await;
    assert!(other_allowed);
    assert_eq!(other_remaining, 2);

    // After window duration expires -> allowed again
    tokio::time::sleep(Duration::from_millis(1100)).await;
    let (allowed_after, _, _) = limiter.check(client_ip).await;
    assert!(allowed_after);
}

#[tokio::test]
async fn test_rate_limiter_http_middleware_integration() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://serve:password@localhost:5432/serve".to_string());

    let db = match Database::connect_url(&database_url).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test: PostgreSQL connection failed: {}", e);
            return;
        }
    };

    let mut config = AppConfig::default();
    config.rate_limit.max_requests = 2; // Only 2 requests allowed
    config.rate_limit.window_secs = 2;

    let schema = build_schema(db.clone(), config.auth.clone(), MessageBroker::default());
    let app = create_router(schema, db, config);

    let client_ip = "203.0.113.42";

    // Health endpoint should bypass rate limiter
    for _ in 0..5 {
        let health_req = Request::builder()
            .method("GET")
            .uri("/health")
            .header("x-forwarded-for", client_ip)
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(health_req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    // 1st request to /graphql -> OK
    let req1 = Request::builder()
        .method("POST")
        .uri("/graphql")
        .header("content-type", "application/json")
        .header("x-forwarded-for", client_ip)
        .body(Body::from(r#"{"query": "{ __typename }"}"#))
        .unwrap();
    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);
    assert_eq!(res1.headers().get("x-ratelimit-limit").unwrap(), "2");
    assert_eq!(res1.headers().get("x-ratelimit-remaining").unwrap(), "1");

    // 2nd request to /graphql -> OK
    let req2 = Request::builder()
        .method("POST")
        .uri("/graphql")
        .header("content-type", "application/json")
        .header("x-forwarded-for", client_ip)
        .body(Body::from(r#"{"query": "{ __typename }"}"#))
        .unwrap();
    let res2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);
    assert_eq!(res2.headers().get("x-ratelimit-remaining").unwrap(), "0");

    // 3rd request to /graphql -> 429 TOO_MANY_REQUESTS
    let req3 = Request::builder()
        .method("POST")
        .uri("/graphql")
        .header("content-type", "application/json")
        .header("x-forwarded-for", client_ip)
        .body(Body::from(r#"{"query": "{ __typename }"}"#))
        .unwrap();
    let res3 = app.clone().oneshot(req3).await.unwrap();
    assert_eq!(res3.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(res3.headers().contains_key("retry-after"));
    assert_eq!(res3.headers().get("x-ratelimit-remaining").unwrap(), "0");
}
