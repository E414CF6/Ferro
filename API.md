# Ferro API

> Complete technical documentation for the **Serve** API service powering the Ferro Social Platform.
> The full primary specification is maintained at [Serve/API.md](file:///Users/kernelily/Project/Ferro/Serve/API.md).

---

## Quick Overview

* **Protocol**: HTTP/1.1, HTTP/2, WebSocket (`graphql-transport-ws`, `graphql-ws`)
* **Base URL**: `http://localhost:8080`
* **GraphQL HTTP Endpoint**: `POST /graphql`
* **GraphQL WebSocket Endpoint**: `GET /graphql/ws`
* **Health & Probes**: `GET /health`, `GET /health/live`, `GET /health/ready`
* **Prometheus Metrics**: `GET /metrics`
* **Media Upload**: `POST /api/upload` (Supports JPEG, PNG, GIF, WEBP, HEIC, HEIF, AVIF with automatic EXIF GPS
  stripping)
* **Authentication**: Dual Token Architecture (Access Token 15m + Rotating Refresh Token 7d)
* **Rate Limiting**: Distributed Redis Lua Sliding Window with in-memory fallback
* **Pub/Sub Broker**: Clustered Redis Pub/Sub for real-time GraphQL Subscriptions

For the detailed GraphQL Schema, Queries, Mutations, Subscriptions, and Error Registry, please refer to the complete
specification in:

* **[Serve/API.md](file:///Users/kernelily/Project/Ferro/Serve/API.md)**
