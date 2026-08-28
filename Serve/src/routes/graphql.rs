use crate::graphql::SnsSchema;
use crate::infrastructure::auth::verify_jwt;
use crate::infrastructure::config::AppConfig;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
use axum::{
    Extension,
    extract::ws::WebSocketUpgrade,
    http::HeaderMap,
    response::{Html, IntoResponse, Response},
};
use tracing::{error, info, warn};

pub async fn graphql_handler(
    Extension(schema): Extension<SnsSchema>,
    Extension(config): Extension<AppConfig>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();

    // 1. Resolve authentication credentials from Bearer token
    let mut current_user = None;
    if let Some(auth_header) = headers.get("Authorization").and_then(|h| h.to_str().ok()) {
        let auth_str = auth_header.trim();
        if let Some(token) = auth_str
            .strip_prefix("Bearer ")
            .or_else(|| auth_str.strip_prefix("bearer "))
        {
            let token = token.trim();
            match verify_jwt(token, &config.auth.jwt_secret) {
                Ok(auth_user) => {
                    current_user = Some((auth_user.user_id, auth_user.username.clone()));
                    req = req.data(auth_user);
                }
                Err(err) => {
                    warn!(target: "serve::graphql", error = %err, "Authorization token rejected");
                }
            }
        }
    }

    let op_name = req
        .operation_name
        .clone()
        .unwrap_or_else(|| "<anonymous>".to_string());
    let start = std::time::Instant::now();

    info!(
        target: "serve::graphql",
        operation = %op_name,
        user = ?current_user.as_ref().map(|(id, name)| format!("{}:{}", id, name)),
        "Executing GraphQL operation"
    );

    let res = schema.execute(req).await;
    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;

    if !res.errors.is_empty() {
        for err in &res.errors {
            error!(
                target: "serve::graphql",
                operation = %op_name,
                elapsed_ms = format_args!("{:.2}ms", elapsed_ms),
                path = ?err.path,
                locations = ?err.locations,
                message = %err.message,
                extensions = ?err.extensions,
                "GraphQL operation produced error"
            );
        }
    } else {
        info!(
            target: "serve::graphql",
            operation = %op_name,
            elapsed_ms = format_args!("{:.2}ms", elapsed_ms),
            "GraphQL operation executed successfully"
        );
    }

    res.into()
}

pub async fn graphql_ws_handler(
    Extension(schema): Extension<SnsSchema>,
    protocol: GraphQLProtocol,
    upgrade: WebSocketUpgrade,
) -> Response {
    upgrade.on_upgrade(move |socket| GraphQLWebSocket::new(socket, schema, protocol).serve())
}

pub async fn graphiql_handler(Extension(config): Extension<AppConfig>) -> impl IntoResponse {
    let ws_endpoint = format!("{}/ws", config.server.graphql_path);
    Html(
        GraphiQLSource::build()
            .endpoint(&config.server.graphql_path)
            .subscription_endpoint(&ws_endpoint)
            .title("Serve GraphQL IDE")
            .finish(),
    )
}
