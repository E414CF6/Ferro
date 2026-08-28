use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
};
use serve::infrastructure::logging::{
    LogFormat,
    http_trace::{
        REQUEST_ID_HEADER, create_http_trace_layer, create_propagate_request_id_layer,
        create_request_id_layer,
    },
    init_logging,
};
use tower::ServiceExt;

#[tokio::test]
async fn test_log_format_from_env() {
    unsafe {
        std::env::set_var("LOG_FORMAT", "json");
        assert_eq!(LogFormat::from_env(), LogFormat::Json);

        std::env::set_var("LOG_FORMAT", "compact");
        assert_eq!(LogFormat::from_env(), LogFormat::Compact);

        std::env::set_var("LOG_FORMAT", "pretty");
        assert_eq!(LogFormat::from_env(), LogFormat::Pretty);

        std::env::remove_var("LOG_FORMAT");
        assert_eq!(LogFormat::from_env(), LogFormat::Pretty);
    }
}

#[tokio::test]
async fn test_init_logging_execution() {
    // Should run safely without panic even if called multiple times
    init_logging();
    init_logging();
    tracing::info!("Test log message emitted successfully");
}

#[tokio::test]
async fn test_http_tracing_and_request_id_middleware() {
    async fn hello_handler() -> impl IntoResponse {
        "Hello Logging World!"
    }

    let app = Router::new()
        .route("/test", get(hello_handler))
        .layer(create_propagate_request_id_layer())
        .layer(create_http_trace_layer())
        .layer(create_request_id_layer());

    // 1. Request without existing x-request-id
    let req = Request::builder()
        .uri("/test")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Check that x-request-id was generated and propagated in response
    let req_id_header = res.headers().get(REQUEST_ID_HEADER);
    assert!(
        req_id_header.is_some(),
        "Expected x-request-id in response headers"
    );
    let req_id_str = req_id_header.unwrap().to_str().unwrap();
    assert!(!req_id_str.is_empty());

    // 2. Request with custom x-request-id
    let custom_id = "custom-trace-id-12345";
    let req_with_custom_id = Request::builder()
        .uri("/test")
        .method("GET")
        .header(REQUEST_ID_HEADER, custom_id)
        .body(Body::empty())
        .unwrap();

    let res_custom = app.oneshot(req_with_custom_id).await.unwrap();
    assert_eq!(res_custom.status(), StatusCode::OK);
    let echoed_id = res_custom
        .headers()
        .get(REQUEST_ID_HEADER)
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(echoed_id, custom_id);
}
