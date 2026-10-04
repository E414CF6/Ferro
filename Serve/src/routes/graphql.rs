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
    Extension(config): Extension<AppConfig>,
    headers: HeaderMap,
    protocol: GraphQLProtocol,
    upgrade: WebSocketUpgrade,
) -> Response {
    let mut initial_data = async_graphql::Data::default();
    if let Some(auth_header) = headers.get("Authorization").and_then(|h| h.to_str().ok()) {
        let auth_str = auth_header.trim();
        if let Some(token) = auth_str
            .strip_prefix("Bearer ")
            .or_else(|| auth_str.strip_prefix("bearer "))
        {
            if let Ok(auth_user) = verify_jwt(token.trim(), &config.auth.jwt_secret) {
                initial_data.insert(auth_user);
            }
        }
    }

    let jwt_secret = config.auth.jwt_secret.clone();

    upgrade.on_upgrade(move |socket| {
        GraphQLWebSocket::new(socket, schema, protocol)
            .with_data(initial_data)
            .on_connection_init(move |value| {
                let secret = jwt_secret.clone();
                async move {
                    let mut data = async_graphql::Data::default();
                    if let Some(token_val) = value
                        .get("Authorization")
                        .or_else(|| value.get("authorization"))
                        .or_else(|| value.get("token"))
                        .and_then(|v| v.as_str())
                    {
                        let token = token_val.trim();
                        let token = token
                            .strip_prefix("Bearer ")
                            .or_else(|| token.strip_prefix("bearer "))
                            .unwrap_or(token);
                        match verify_jwt(token.trim(), &secret) {
                            Ok(auth_user) => {
                                data.insert(auth_user);
                            }
                            Err(e) => {
                                return Err(async_graphql::Error::new(e.to_string()));
                            }
                        }
                    }
                    Ok(data)
                }
            })
            .serve()
    })
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
