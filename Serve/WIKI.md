# Wiki

## 🚀 Project Overview

Serve is a modern, high-performance SNS (Social Networking Service) backend API built with Rust. It provides a robust
foundation for social interactions, including user profiles, real-time direct messaging, and ephemeral stories.

## 🛠 Tech Stack

- **Language**: [Rust](https://www.rust-lang.org/)
- **Web Framework**: [Axum](https://github.com/tokio-rs/axum)
- **GraphQL**: [async-graphql](https://github.com/async-graphql/async-graphql)
- **Database**: [PostgreSQL 16](https://www.postgresql.org/) (Running via Podman)
- **ORM/Query Builder**: [SQLx](https://github.com/launchbadge/sqlx) (Asynchronous SQL)
- **Authentication**: Argon2id (Password Hashing) & JWT (JSON Web Tokens)
- **Containerization**: Podman

## 🏗 Architecture

The project follows a layered architecture to ensure separation of concerns and testability:

### 1. Application Layer (`src/application/`)

Contains high-level business logic services that orchestrate the flow of data between the domain and the presentation
layer. *Examples:* `auth_service.rs`, `story_service.rs`, `dm_service.rs`.

### 2. Domain Layer (`src/domain/`)

The core of the application, containing business rules, models, and repository abstractions. It is independent of
external frameworks. *Includes:* `models.rs`, `errors.rs`, and repository traits in `repositories/`.

### 3. Infrastructure Layer (`src/infrastructure/`)

Contains implementations of the domain's requirements, such as database access, authentication mechanisms, and external
storage. *Includes:* `db/` (PostgreSQL/SQLx implementations), `auth/` (JWT/Argon2), `config/`, and `logging/`.

### 4. Presentation/Route Layer (`src/routes/` & `src/graphql/`)

Handles incoming HTTP requests and maps them to application services. *Includes:* Axum routes for health checks and
uploads, and the GraphQL engine for all core social features.

## ✨ Core Features

### 📸 Instagram-style Stories

Ephemeral media posts that automatically expire after 24 hours.

- **Lifecycle**: Created via `createStory`, viewable via `storiesFeed`.
- **Engagement**: Users can view stories, and creators can see who viewed their content.
- **Expiration**: Automatically marked as expired after 24 hours.

### 💬 Direct Messages (DM)

Real-time, private 1:1 conversations.

- **Conversations**: Maintain a list of active chats with snippet previews.
- **Read Receipts**: Track whether messages have been read by the recipient.

### 👤 User Profiles

Rich, customizable user identities.

- **Details**: Includes avatar, header images, bio, and more.
- **Social Stats**: Dynamically calculated fields like post counts and follower status.

## 🛠 Development & Setup

### Prerequisites

- [Rust Toolchain](https://rustup.rs/)
- [Podman](https://podman.io/)

### Running the Application

1. **Start Database**:

   ```bash
   podman-compose up -d
   ```

2. **Run the Server**:

   ```bash
   cargo run
   ```

### Testing

The project includes comprehensive test suites:

- **Unit Tests**: Found in `src/tests/` covering errors, config, and logging.
- **Integration Tests**: Covering GraphQL resolvers and end-to-end API flows.
