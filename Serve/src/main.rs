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

    // Check for CLI subcommands (e.g. `serve migrate` or `serve --migrate`)
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("Ferro Serve - High-Performance Backend Service");
        println!();
        println!("USAGE:");
        println!("    serve [COMMAND]");
        println!();
        println!("COMMANDS:");
        println!("    migrate       Run database migrations and exit");
        println!("    (no command)  Start HTTP/WebSocket API server");
        println!();
        println!("OPTIONS:");
        println!("    -h, --help    Print help information");
        return;
    }

    if args
        .iter()
        .any(|arg| arg == "--migrate" || arg == "migrate")
    {
        info!(target: "serve::cli", "Standalone migration command triggered. Running database migrations...");
        let mut migrate_config = config.database.clone();
        migrate_config.auto_migrate = true;
        let db = Database::connect(&migrate_config)
            .await
            .expect("Failed to connect and execute migrations");
        db.init_tables()
            .await
            .expect("Failed to execute migrations");
        info!(target: "serve::cli", "Database migrations completed successfully. Exiting.");
        return;
    }

    // 3. Connect & Initialize Database with configuration (pool limits, timeouts, auto-migration, seeding)
    let db = Database::connect(&config.database)
        .await
        .expect("Failed to connect to PostgreSQL database");

    // 4. Initialize real-time message broker for subscriptions (Redis PubSub or In-Memory)
    let broker = if config.pubsub_driver == infrastructure::config::PubSubDriver::Redis
        || config.redis.enabled
    {
        info!(target: "serve::init", redis_url = %config.redis.url, "Clustered Redis PubSub broker initialized");
        MessageBroker::with_redis(1024, &config.redis.url)
    } else {
        MessageBroker::default()
    };

    // 5. Build GraphQL Schema with Tracing, DataLoaders, Security Limits & Subscriptions
    let schema = build_schema_with_options(
        db.clone(),
        config.auth.clone(),
        broker,
        config.server.enable_introspection,
    );

    // 6. Spawn Background Sweeper: Purges stories expired > 7 days periodically
    let db_sweeper = db.clone();
    tokio::spawn(async move {
        use domain::repositories::StoryRepository;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            let cutoff = chrono::Utc::now() - chrono::Duration::days(7);
            if let Ok(count) = db_sweeper.prune_expired_stories(cutoff).await {
                if count > 0 {
                    tracing::info!(target: "serve::sweeper", pruned_count = count, "Purged expired story records older than 7 days");
                }
            }
        }
    });

    // 7. Create Axum Router with HTTP Tracing, Security Headers, CORS, WebSocket, and Health check routes
    let app = create_router(schema, db, config.clone());

    // 8. Bind Socket Address and Start Axum Server with Graceful Shutdown
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

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
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
