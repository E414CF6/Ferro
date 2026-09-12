# Ferro

Ferro is a high-performance, full-stack social networking platform built with a modular Rust backend, a native Android client using Jetpack Compose, and a modern Next.js web application.

---

## Key Capabilities

- **Ephemeral Stories**: 24-hour auto-expiring media stories with real-time viewer analytics, author-restricted viewer lists, and unread indicator rings.
- **Direct Messaging (DM)**: Real-time 1:1 private conversations featuring unread counters, message snippets, and read receipt checkmarks.
- **Interactive Feed and Posts**: Dual feed modes (Following and Explore), live voting polls with real-time percentages, quote posts, multi-media attachments, nested comments, and optimistic bookmarking and liking.
- **Discovery and Search**: Full-text post content search, trending hashtags, and creator recommendations with instant follow actions.
- **Security and User Profiles**: Two-factor authentication (2FA TOTP), handle-based user queries, customizable banners, avatars, bios, and follower statistics.

---

## Technical Stack

| Component          | Layer              | Technologies                                                                                 |
|--------------------|--------------------|----------------------------------------------------------------------------------------------|
| **Serve**          | Backend API        | Rust 2024, Axum 0.8, async-graphql 7.2, SQLx 0.9, Tokio, Tower-HTTP, Argon2, JWT, DataLoader |
| **Vision**         | Web Client         | Next.js 15 (App Router), React 19, TypeScript 5, Custom CSS Design System, Lucide Icons      |
| **Infrastructure** | Database & Runtime | SQLite (Default for local development), PostgreSQL 16 Alpine, Podman / Docker Compose        |

---

## Repository Structure

```
Ferro/
├── Serve/                         # Rust GraphQL Backend API
│   ├── Cargo.toml
│   ├── podman-compose.yml         # Containerized PostgreSQL service
│   ├── compose.prod.yml           # Production Compose setup
│   ├── migrations/                # SQLx database schema migrations
│   ├── tests/                     # Integration tests
│   └── src/
│       ├── application/           # Application and business services
│       ├── domain/                # Entities, domain models, and error definitions
│       ├── graphql/               # GraphQL schema, queries, mutations, and resolvers
│       ├── infrastructure/        # Database pools, DataLoaders, security, and configuration
│       └── routes/                # Axum route handlers and health endpoints
│
└── Vision/                        # Next.js 15 Web Application
    ├── package.json
    ├── next.config.ts
    └── src/
        ├── app/                   # Next.js App Router pages and global design system
        ├── components/            # UI components and modals
        └── lib/                   # Auth context, Toast context, GraphQL client, types
```

---

## Getting Started

### Prerequisites

- **Rust**: 1.80+ (Rust 2024 edition)
- **Node.js**: 20.x or later with npm
- **Android Studio**: Ladybug / Meerkat or compatible with Android SDK 35
- **Container Engine** (Optional): Podman or Docker

---

### 1. Running the Backend (`Serve`)

From the `Serve` directory:

```bash
# Start backend using SQLite (default)
cargo run --bin serve
```

- GraphQL Endpoint: `http://localhost:8080/graphql`
- GraphiQL Playground: `http://localhost:8080/graphiql`

To populate the database with seed data for testing:

```bash
cargo run --bin seed
```

---

### 2. Running the Web Client (`Vision`)

From the `Vision` directory:

```bash
# Install dependencies
npm install

# Start development server
npm run dev
```

Open `http://localhost:3000` in your web browser.

---

### 3. Native Android Client (`Vision/Android` - Roadmap)

A native Android client using Jetpack Compose and MVI / Clean Architecture is planned. When available:
1. Open `Vision/Android` in Android Studio.
2. Sync the project with Gradle files.
3. Run on an Android Emulator (connected via `http://10.0.2.2:8080/graphql`).

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
