# Serve

Serve is a high-performance backend API service for the Ferro social network platform. It is built in Rust using Axum,
`async-graphql`, and SQLx, with support for both SQLite and PostgreSQL.

---

## Architectural Highlights

- **Framework & Runtime**: Asynchronous I/O powered by Tokio and Axum 0.8.
- **GraphQL Schema**: Strongly typed GraphQL schema and resolvers built with `async-graphql 7.2`.
- **N+1 Optimization**: Batched data fetching through `async-graphql` DataLoaders for users, posts, and interaction
  metrics.
- **Database Support**: Abstracted storage layer supporting SQLite for streamlined local development and PostgreSQL 16
  for production deployments.
- **Object & Blob Storage**: Multi-driver storage engine supporting Local filesystem and S3 / Cloudflare R2 / MinIO
  cloud storage.
- **Security & Hardening**:
    - Sliding window rate limiting middleware with proxy/client IP extraction (`CF-Connecting-IP`, `X-Forwarded-For`,
      `X-Real-IP`).
    - Production-aware GraphQL Introspection protection and query depth/complexity bounding.
    - Multi-signature Magic Byte validation for file uploads and mandatory JWT token authentication.
    - Password hashing with Argon2, JWT token authorization, and TOTP 2FA.
- **Containerization**: Production-ready Multi-Stage `Dockerfile` (with `cargo-chef` cache acceleration) and
  `compose.prod.yml`.
- **Modular Design**: Clean separation across domain entities, application services, GraphQL resolvers, and
  infrastructure adapters.

---

## Key Features

1. **Ephemeral Stories (24h Expiration)**
    - Automatic 24-hour expiration calculation (`expires_at`, `is_expired`).
    - View tracking with unread detection (`isViewedByMe`, `viewStory`).
    - Author-restricted access controls ensuring only story creators can inspect the viewer list (`viewers`).
    - Active story feed aggregation (`storiesFeed`, `hasActiveStories`).

2. **Direct Messaging (DM)**
    - 1:1 private conversations with persistent messaging (`sendDirectMessage`, `directMessages`, `conversations`).
    - Real-time unread counts and conversation summaries (`unreadDmCount`, `markMessagesAsRead`).
    - Delivery tracking and read receipts (`isRead`).

3. **Interactive Posts, Polls, and Feeds**
    - Multi-option voting polls with live percentage calculations and expiration handling.
    - Rich media attachments, quote posts, and nested reply comments.
    - Dual-feed querying for following-only and global explore feeds.
    - Bookmark management and optimistic interaction metrics.

4. **Profiles, Follow Graph, and Discovery**
    - Profile customization (avatar, header cover, biography, location, and website).
    - Handle-based resolution (`profile(username: "...")`).
    - Full-text post searching and trending hashtag analytics.

---

## Getting Started

### Prerequisites

- [Rust Toolchain](https://rustup.rs/) (latest stable release supporting the 2024 edition)
- SQLite (included by default) or Podman/Docker for PostgreSQL

### Configuration

Copy the example environment configuration file:

```bash
cp .env.example .env
```

Review and update the settings in `.env` as needed.

### Running the Server

To launch the backend API server with automated migrations:

```bash
cargo run --bin serve
```

- Server Address: `http://127.0.0.1:8080`
- GraphQL API: `http://127.0.0.1:8080/graphql`
- GraphiQL Interactive IDE: `http://127.0.0.1:8080/graphiql`

### Running in Production with Docker / Podman Compose

To build and launch the production stack (Serve API + PostgreSQL):

```bash
docker compose -f compose.prod.yml up -d --build
# or: podman-compose -f compose.prod.yml up -d
```

### Seeding Test Data (Optional)

By default, the server starts with an empty database (`DB_SEED_DATA=false`). To seed initial test users, posts, stories,
and social graph connections:

```bash
cargo run --bin seed
```

Pre-configured seed accounts (Password for all: `password123`):

- `ferro_dev` (`ferro@example.com`)
- `alex_coder` (`alex@example.com`)
- `design_guru` (`sophia@example.com`)

---

## Environment Variables Reference

| Variable                  | Default Value         | Description                                                     |
|---------------------------|-----------------------|-----------------------------------------------------------------|
| `APP_ENV`                 | `development`         | Runtime environment mode (`development`, `production`, `test`). |
| `HOST`                    | `127.0.0.1`           | Network interface address for binding the server.               |
| `PORT`                    | `8080`                | Port on which the HTTP server listens.                          |
| `DATABASE_DRIVER`         | `sqlite`              | Database driver selection (`sqlite` or `postgres`).             |
| `DATABASE_URL`            | `sqlite://serve.db`   | Connection string for the target database.                      |
| `SQLITE_PATH`             | `./serve.db`          | Filesystem path for SQLite database file storage.               |
| `DB_MAX_CONNECTIONS`      | `20`                  | Maximum number of connections in the database pool.             |
| `DB_MIN_CONNECTIONS`      | `2`                   | Minimum number of idle connections maintained in the pool.      |
| `DB_AUTO_MIGRATE`         | `true`                | Automatically run pending SQLx migrations on startup.           |
| `DB_SEED_DATA`            | `false`               | Automatically insert seed data during server startup if true.   |
| `STORAGE_DRIVER`          | `local`               | Storage driver selection (`local` or `s3` / `r2` / `minio`).    |
| `STORAGE_LOCAL_PATH`      | `./uploads`           | Local directory path for file uploads.                          |
| `STORAGE_LOCAL_BASE_URL`  | `/uploads`            | Public URL prefix for locally hosted media files.               |
| `S3_BUCKET`               | `serve-media`         | S3 / R2 Bucket name.                                            |
| `S3_REGION`               | `auto`                | AWS / Cloudflare R2 Region.                                     |
| `S3_ENDPOINT`             | `None`                | Custom S3 / MinIO API endpoint URL.                             |
| `S3_PUBLIC_URL`           | `None`                | Public CDN / Custom Domain base URL for uploaded media.         |
| `JWT_SECRET`              | `...`                 | Secret key used for signing and verifying JSON Web Tokens.      |
| `JWT_EXPIRATION_DAYS`     | `30`                  | Duration in days before issued JWT tokens expire.               |
| `RATE_LIMIT_ENABLED`      | `true`                | Enables IP-based sliding window rate limiting.                  |
| `RATE_LIMIT_MAX_REQUESTS` | `120`                 | Maximum requests permitted within the sliding window per IP.    |
| `RATE_LIMIT_WINDOW_SECS`  | `60`                  | Duration in seconds for the rate limiting window.               |
| `ENABLE_GRAPHIQL`         | `true`                | Enables the GraphiQL interactive web playground.                |
| `ENABLE_INTROSPECTION`    | `true`                | Enables GraphQL schema introspection (auto-disabled in prod).   |
| `GRAPHQL_PATH`            | `/graphql`            | Endpoint path for GraphQL POST queries and mutations.           |
| `GRAPHIQL_PATH`           | `/graphiql`           | Endpoint path for the GraphiQL UI.                              |
| `LOG_FORMAT`              | `pretty`              | Output format for application logs (`pretty` or `json`).        |
| `LOG_FILTER`              | `info,serve=debug...` | Log level directives for `tracing-subscriber`.                  |

---

## Directory Structure

```
Serve/
├── Cargo.toml                 # Rust dependencies and package configuration
├── Dockerfile                 # Multi-stage production container build
├── compose.prod.yml           # Production stack orchestration (Serve + PostgreSQL)
├── podman-compose.yml         # Container configuration for PostgreSQL
├── migrations/                # Database migration scripts (SQLx)
└── src/
    ├── main.rs                # Server binary entry point
    ├── bin/
    │   └── seed.rs            # Database seeding utility binary
    ├── application/           # Application use cases and business services
    ├── domain/                # Entities, models, value objects, and error types
    ├── graphql/               # GraphQL schema definitions, queries, and mutations
    │   ├── context.rs         # Execution context and DataLoader registry
    │   ├── mutation/          # Mutation resolvers (auth, post, story, DM, etc.)
    │   ├── query/             # Query resolvers
    │   └── types/             # GraphQL object and input types
    ├── infrastructure/        # Database pools, authentication, crypto, rate limit, and storage
    └── routes/                # Axum HTTP routes, GraphQL handlers, upload, and health probes
```
