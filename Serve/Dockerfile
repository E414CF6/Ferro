# ==============================================================================
# Multi-Stage Production Dockerfile for Serve (Rust Axum + GraphQL + SQLx)
# ==============================================================================

# ------------------------------------------------------------------------------
# 1. Chef Stage: Install cargo-chef for dependency caching
# ------------------------------------------------------------------------------
FROM rust:1.85-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

# ------------------------------------------------------------------------------
# 2. Planner Stage: Compute recipe.json lockfile snapshot
# ------------------------------------------------------------------------------
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo chef prepare --recipe-path recipe.json

# ------------------------------------------------------------------------------
# 3. Builder Stage: Build external dependencies first, then application binary
# ------------------------------------------------------------------------------
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies only (cached across code edits)
RUN cargo chef cook --release --recipe-path recipe.json

# Copy full application source code and migrations
COPY Cargo.toml Cargo.lock ./
COPY migrations/ migrations/
COPY src/ src/

# Build release binary
RUN cargo build --release --bin serve --bin seed

# ------------------------------------------------------------------------------
# 4. Runtime Stage: Minimal, secure Debian-slim container
# ------------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Install CA certificates and SQLite runtime libraries
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libsqlite3-0 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root application user
RUN groupadd -g 10001 appgroup && \
    useradd -u 10001 -g appgroup -s /bin/sh -m appuser

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /app/target/release/serve /app/serve
COPY --from=builder /app/target/release/seed /app/seed
COPY migrations/ /app/migrations/

# Create persistent storage directories
RUN mkdir -p /app/data /app/uploads && \
    chown -R appuser:appgroup /app

USER appuser

# Expose default HTTP port
EXPOSE 8080

# Environment defaults for container
ENV APP_ENV=production \
    HOST=0.0.0.0 \
    PORT=8080 \
    DATABASE_DRIVER=postgres \
    DB_AUTO_MIGRATE=true \
    RUST_LOG=info,serve=info,tower_http=info,sqlx=warn

# Health check probe against liveness endpoint
HEALTHCHECK --interval=15s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://127.0.0.1:8080/health || exit 1

ENTRYPOINT ["/app/serve"]
