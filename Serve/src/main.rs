#![allow(
    clippy::collapsible_if,
    clippy::collapsible_else_if,
    clippy::unnecessary_map_or,
    clippy::too_many_arguments
)]

mod application;
mod domain;
mod graphql;
mod infrastructure;
mod routes;

use infrastructure::config::AppConfig;
use infrastructure::db::postgres::Database;
use infrastructure::logging::init_logging;
use infrastructure::pubsub::MessageBroker;

use graphql::build_schema_with_options;
use routes::create_router;
use tracing::info;

#[tokio::main]
async fn main() {
    // 1. Initialize structured logging system
    init_logging();

    // 2. Load and validate unified configuration from environment variables / .env
    let config = AppConfig::from_env();
    config.print_summary();

    // 3. Connect & Initialize Database with configuration (pool limits, timeouts, auto-migration, seeding)
    let db = Database::connect(&config.database)
        .await
        .expect("Failed to connect to PostgreSQL database");

    // 4. Initialize real-time message broker for subscriptions
    let broker = MessageBroker::default();

    // 5. Build GraphQL Schema with Tracing, DataLoaders, Security Limits & Subscriptions
    let schema = build_schema_with_options(
        db.clone(),
        config.auth.clone(),
        broker,
        config.server.enable_introspection,
    );

    // 6. Create Axum Router with HTTP Tracing, Security Headers, CORS, WebSocket, and Health check routes
    let app = create_router(schema, db, config.clone());

    // 7. Bind Socket Address and Start Axum Server with Graceful Shutdown
    let addr = config.server.socket_addr();
    info!(
        target: "serve::init",
        address = %addr,
        "Serve running at http://{}",
        addr
    );
    if config.server.enable_graphiql {
        info!(
            target: "serve::init",
            graphiql_url = %format!("http://{}{}", addr, config.server.graphiql_path),
            "GraphiQL IDE available at http://{}{}",
            addr,
            config.server.graphiql_path
        );
    }
    info!(
        target: "serve::init",
        health_url = %format!("http://{}/health", addr),
        ready_url = %format!("http://{}/health/ready", addr),
        metrics_url = %format!("http://{}/metrics", addr),
        "Health, metrics & readiness probes ready"
    );

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to address {}: {}", addr, e));

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!(target: "serve::shutdown", "Received Ctrl+C (SIGINT) signal. Initiating graceful shutdown...");
        },
        _ = terminate => {
            info!(target: "serve::shutdown", "Received SIGTERM signal. Initiating graceful shutdown...");
        },
    }
}
