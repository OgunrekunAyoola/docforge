# ARCHITECTURE.md — Docforge System Design

---

## System Overview

Docforge is a SaaS API that accepts project architecture documents and uses a local LLM (Ollama) to transform them into equivalent documents for a new project. It is built entirely in Rust and designed for production deployment.

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLIENT (curl / UI)                        │
└────────────────────────────┬────────────────────────────────────┘
                             │ HTTPS
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                     AXUM API SERVER                              │
│                                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────────┐  │
│  │  /health │  │  /auth   │  │  /jobs   │  │   /metrics    │  │
│  └──────────┘  └──────────┘  └──────────┘  └───────────────┘  │
│                                                                  │
│  Middleware: Auth JWT │ Rate Limit │ Tracing │ CORS │ Headers   │
└──────┬──────────────────────────────────────────┬───────────────┘
       │ sqlx queries                              │ RPUSH
       ▼                                           ▼
┌─────────────┐                         ┌──────────────────┐
│ PostgreSQL  │                         │   Redis Queue    │
│             │                         │                  │
│  users      │                         │ docforge:jobs    │
│  jobs       │◄────────────────────────│ docforge:failed  │
│  documents  │   update status/results │                  │
└─────────────┘                         └────────┬─────────┘
                                                  │ BLPOP
                                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│                    BACKGROUND WORKER                             │
│                                                                  │
│  1. Dequeue job_id from Redis                                    │
│  2. Load documents from PostgreSQL                               │
│  3. Spawn one task per document (concurrent)                     │
│  4. Each task → Ollama API → transformed content                 │
│  5. Save all output documents to PostgreSQL                      │
│  6. Update job status → "completed" or "failed"                  │
└──────────────────────────────────┬──────────────────────────────┘
                                   │ HTTP POST /api/chat
                                   ▼
                         ┌──────────────────┐
                         │   Ollama Server  │
                         │   (llama3 local) │
                         └──────────────────┘
```

---

## Component Breakdown

### 1. Axum API Server

**Responsibility:** Accept HTTP requests, authenticate users, persist data, enqueue jobs.

**Key design decisions:**
- All handlers are async and non-blocking
- No business logic in handlers — delegates to service functions
- All handlers return `Result<impl IntoResponse, AppError>`
- Shared state injected via `axum::extract::State<Arc<AppState>>`

```rust
// AppState — shared across all handlers
pub struct AppState {
    pub db: PgPool,
    pub redis: RedisPool,
    pub config: Arc<Config>,
}
```

**Middleware stack (applied in order):**
```
Request
  → TraceLayer (generate span, log request)
  → RequestIdLayer (inject X-Request-ID)
  → CorsLayer (validate origin)
  → SecurityHeadersLayer (X-Frame-Options, etc.)
  → RateLimitLayer (per-IP, per-user)
  → AuthLayer (validate JWT on protected routes)
  → Handler
Response
```

---

### 2. PostgreSQL (Data Layer)

**Responsibility:** Source of truth for all persistent state.

**Why sqlx over Diesel or SeaORM:**
- Compile-time SQL verification without ORM complexity
- Async-native, integrates cleanly with Tokio
- Raw SQL is readable and debuggable

**Connection pooling:**
```
PgPoolOptions::new()
    .max_connections(20)
    .connect(&config.database_url)
```

**Query pattern:**
```rust
// Compile-time verified — wrong SQL = compile error
let job = sqlx::query_as!(
    Job,
    "SELECT * FROM jobs WHERE id = $1 AND user_id = $2",
    job_id,
    user_id
)
.fetch_one(&pool)
.await?;
```

---

### 3. Redis Queue

**Responsibility:** Decouple API request handling from document processing.

**Queue pattern:** Simple list-based queue (RPUSH / BLPOP)
- Producer: API server pushes job ID after DB insert
- Consumer: Worker blocks on BLPOP — zero polling overhead

**Queue message format:**
```json
{
  "job_id": "uuid",
  "enqueued_at": "2025-01-01T00:00:00Z",
  "attempt": 1
}
```

**Dead letter queue:**
- After 3 failed attempts, message moved to `docforge:jobs:failed`
- Failed jobs marked `status = "failed"` in PostgreSQL with error message

**Retry strategy:**
```
Attempt 1 → immediate
Attempt 2 → 30s delay
Attempt 3 → 5min delay
Attempt 4 → dead letter
```

---

### 4. Background Worker

**Responsibility:** Process jobs concurrently without blocking the API.

**Worker concurrency model:**
```
Main worker loop (single task):
  └─ Dequeues job_id
  └─ Loads job + N documents from DB
  └─ Spawns N async tasks (one per document)
       ├─ Task 1: transform doc1 via Ollama
       ├─ Task 2: transform doc2 via Ollama
       └─ Task N: transform docN via Ollama
  └─ join_all(tasks) — wait for all
  └─ Save results, update status
```

**Concurrency control:**
- Max 3 concurrent jobs (configurable via `WORKER_CONCURRENCY`)
- Uses `tokio::sync::Semaphore` to limit concurrent job processing
- Each job processes its documents concurrently without limit

**Why run worker in same binary:**
- Simpler deployment (one Docker image, two modes)
- Shared `AppState`, config, and connection pools
- Boot with `--worker` flag: `docforge --worker`

---

### 5. Ollama Client

**Responsibility:** Send prompts to local Ollama, receive transformed documents.

**API used:** `POST http://localhost:11434/api/chat`

**Request shape:**
```json
{
  "model": "llama3",
  "messages": [
    { "role": "system", "content": "You are an expert software architect..." },
    { "role": "user", "content": "Transform this document:\n\n{content}" }
  ],
  "stream": false
}
```

**Client design:**
```rust
pub struct OllamaClient {
    http: reqwest::Client,  // shared, connection-pooling built in
    base_url: String,
    model: String,
    timeout: Duration,
}
```

**Why share one client across tasks:**
- `reqwest::Client` is cheaply clonable (backed by `Arc` internally)
- Connection pool is reused across all concurrent requests
- No overhead from creating new clients per task

---

### 6. Observability Layer

**Responsibility:** Make the system debuggable and measurable in production.

**Tracing architecture:**
```
tracing::span! (created per request/job)
  └─ child spans per function (#[tracing::instrument])
  └─ structured fields: job_id, user_id, filename, duration
  └─ exported to: stdout (JSON format) in dev
                  OpenTelemetry collector in prod (future)
```

**Log levels:**
```
ERROR — unexpected failures, panics, data corruption
WARN  — retries, degraded behavior, near limits
INFO  — job state transitions, server start/stop
DEBUG — Ollama requests/responses, queue operations
TRACE — per-document processing, SQL queries
```

**Metrics collection:**
```
/metrics endpoint → Prometheus scrapes every 15s → Grafana dashboard
```

---

## Data Flow: Full Job Lifecycle

```
1. USER: POST /api/jobs
   Body: { context: "Project B is a fintech API" }
   
2. API: Validate request → Insert job (status=pending) → Return job_id

3. USER: POST /api/jobs/:id/documents
   Body: multipart form with .md files
   
4. API: Validate files → Store in documents table (type=input) → RPUSH job_id to Redis

5. WORKER: BLPOP job_id from Redis
   → Load job + input documents from DB
   → Update job status=processing
   → For each document: tokio::spawn(transform)

6. WORKER TASK (per document):
   → Build prompt with document content + job context
   → POST to Ollama API
   → Receive transformed content
   → Insert into documents table (type=output)

7. WORKER: join_all(tasks)
   → All documents done: Update job status=completed, completed_at=now
   → Any document failed: Update job status=failed, error=message

8. USER: GET /api/jobs/:id
   Response: { status: "completed", document_count: 5 }

9. USER: GET /api/jobs/:id/result
   Response: JSON array of transformed documents
```

---

## API Contract

### Authentication
All endpoints except `/health`, `/ready`, `/api/auth/*` require:
```
Authorization: Bearer <jwt_token>
```

### Error Response Format (all errors)
```json
{
  "error": {
    "code": "JOB_NOT_FOUND",
    "message": "Job with id abc-123 not found",
    "request_id": "req-xyz"
  }
}
```

### Job Status Values
```
pending     → created, not yet queued or in queue
processing  → worker is transforming documents
completed   → all documents transformed successfully
failed      → transformation failed after max retries
cancelled   → user deleted the job
```

---

## Security Model

```
Layer 1 — Transport: HTTPS only in production
Layer 2 — Auth: JWT, 24h expiry, signed with HS256
Layer 3 — Authorization: Users can only access their own jobs
Layer 4 — Rate limiting: Per-IP and per-user limits
Layer 5 — Input validation: File type, size, count limits
Layer 6 — Data isolation: Cascade deletes, no cross-user queries
Layer 7 — Secrets: Environment variables only, never in code
Layer 8 — Dependencies: cargo audit in CI
```

---

## Deployment Architecture (Docker Compose)

```
┌─────────────────────────────────────────┐
│            docker-compose               │
│                                         │
│  ┌────────┐   ┌────────┐               │
│  │  app   │   │ worker │               │
│  │ :3000  │   │        │               │
│  └───┬────┘   └───┬────┘               │
│      │             │                   │
│  ┌───▼─────────────▼───┐               │
│  │      postgres :5432  │               │
│  └─────────────────────┘               │
│  ┌──────────────────────┐               │
│  │      redis :6379     │               │
│  └──────────────────────┘               │
│  ┌──────────────────────┐               │
│  │     ollama :11434    │               │
│  └──────────────────────┘               │
│  ┌──────────────────────┐               │
│  │  prometheus :9090    │               │
│  └──────────────────────┘               │
│  ┌──────────────────────┐               │
│  │    grafana :3001     │               │
│  └──────────────────────┘               │
└─────────────────────────────────────────┘
```

---

## Key Rust Patterns Used Throughout

| Pattern | Where | Why |
|---|---|---|
| `Arc<AppState>` | All handlers | Share state without copying |
| `Arc<OllamaClient>` | Worker tasks | One HTTP client, many tasks |
| `Result<T, AppError>` | Every function | Typed, propagatable errors |
| `#[derive(serde::Deserialize)]` | Request bodies | Zero-boilerplate JSON parsing |
| `#[derive(sqlx::FromRow)]` | DB models | Zero-boilerplate DB mapping |
| `tokio::spawn` + `join_all` | Worker | True concurrent processing |
| `Semaphore` | Worker | Control max concurrency |
| `#[tracing::instrument]` | All functions | Automatic span creation |
| `impl IntoResponse for AppError` | Error type | Axum-native error handling |
| `thiserror::Error` derive | Error enum | Structured error variants |
