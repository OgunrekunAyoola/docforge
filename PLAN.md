# PLAN.md — Docforge Full Development Lifecycle

**Project:** Docforge SaaS API  
**Stack:** Rust · Axum · PostgreSQL · Redis · Ollama · Docker  
**Goal:** Production-grade portfolio project demonstrating senior Rust backend engineering  
**Status:** [ ] In Progress

---

## Development Lifecycle Overview

```
Phase 0  → Environment & Tooling Setup
Phase 1  → Project Scaffolding & Architecture
Phase 2  → Database Layer (PostgreSQL + sqlx)
Phase 3  → Core API (Axum handlers, auth, job CRUD)
Phase 4  → Redis Queue (producer + consumer)
Phase 5  → Background Worker (concurrent Ollama processing)
Phase 6  → Observability (tracing, metrics, structured logs)
Phase 7  → Security Hardening
Phase 8  → Docker & Deployment
Phase 9  → Testing & Quality Audit
Phase 10 → Documentation & Demo Polish
```

---

## Phase 0 — Environment & Tooling Setup
**Status:** [ ]

### Goal
Get a fully working local development environment before writing a single line of application code.

### Tasks
- [ ] Install Rust via `rustup` (stable toolchain)
- [ ] Install `cargo-watch` for auto-recompile on save
- [ ] Install `sqlx-cli` for database migrations (`cargo install sqlx-cli`)
- [ ] Install Docker Desktop
- [ ] Install Ollama and pull a model (`ollama pull llama3`)
- [ ] Install a PostgreSQL GUI (TablePlus or pgAdmin) for inspection
- [ ] Install RedisInsight for Redis queue inspection
- [ ] Create `.env` file from `.env.example`
- [ ] Verify Ollama responds: `curl http://localhost:11434/api/tags`
- [ ] Verify Docker is running: `docker info`

### Rust Concepts Introduced
- `rustup`, toolchains, `cargo` basics
- `Cargo.toml` — dependencies, features, workspace

### Done When
- [ ] `rustc --version` shows stable
- [ ] `ollama run llama3 "hello"` responds
- [ ] `docker compose up` starts postgres + redis without errors

---

## Phase 1 — Project Scaffolding & Architecture
**Status:** [ ]

### Goal
Create the full project skeleton — every module file, no logic yet. Establish patterns all future code will follow.

### Tasks
- [ ] `cargo new docforge --bin`
- [ ] Add all dependencies to `Cargo.toml`
- [ ] Create full module tree (see ARCHITECTURE.md)
- [ ] Implement `Config` struct loading from environment
- [ ] Implement global `AppError` enum with `thiserror`
- [ ] Implement `AppError` → JSON response mapping
- [ ] Create `docker-compose.yml` (postgres, redis, ollama)
- [ ] Create `Dockerfile` for the Rust app (multi-stage build)
- [ ] Create `.env.example` with all variables documented
- [ ] Create `.gitignore`
- [ ] Wire `main.rs` to boot: load config → connect DB → connect Redis → start server
- [ ] Add `/health` and `/ready` endpoints
- [ ] Confirm `cargo build` succeeds with skeleton

### Rust Concepts Introduced
- `struct`, `enum`, `impl`, `pub`, `mod`, `use`
- `Result<T, E>` and the `?` operator
- Trait implementation (`impl IntoResponse for AppError`)
- Environment variable reading with `std::env`
- `tokio::main` async entry point

### Done When
- [ ] All audit checklist items pass
- [ ] `GET /health` returns `200 OK`
- [ ] `GET /ready` returns `200 OK` when DB + Redis connected, `503` otherwise

---

## Phase 2 — Database Layer
**Status:** [ ]

### Goal
Design and implement the full data model. Every table, every query, compile-time verified.

### Schema

```sql
-- Users table
CREATE TABLE users (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email       TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Jobs table
CREATE TABLE jobs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status      TEXT NOT NULL DEFAULT 'pending',
    context     TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    error       TEXT
);

-- Documents table (input)
CREATE TABLE documents (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id      UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    filename    TEXT NOT NULL,
    content     TEXT NOT NULL,
    doc_type    TEXT NOT NULL DEFAULT 'input',  -- 'input' | 'output'
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Tasks
- [ ] Write migration `001_initial_schema.sql`
- [ ] Run migration: `sqlx migrate run`
- [ ] Implement `User` model struct with `sqlx::FromRow`
- [ ] Implement `Job` model struct + `JobStatus` enum
- [ ] Implement `Document` model struct
- [ ] Write `db::users` query functions (create, find_by_email)
- [ ] Write `db::jobs` query functions (create, find_by_id, update_status, list_by_user)
- [ ] Write `db::documents` query functions (create_batch, find_by_job)
- [ ] Add `sqlx::PgPool` to shared app state
- [ ] Write DB integration tests using a test database

### Rust Concepts Introduced
- `sqlx::query!` macro (compile-time SQL checking)
- `#[derive(sqlx::FromRow)]`
- `uuid::Uuid` type
- `chrono::DateTime<Utc>` for timestamps
- Database connection pooling with `PgPool`
- Async database queries

### Done When
- [ ] All audit checklist items pass
- [ ] All queries verified against real database
- [ ] Test database setup documented

---

## Phase 3 — Core API (Auth + Job CRUD)
**Status:** [ ]

### Goal
Build the full REST API surface. Users can register, log in, create jobs, upload documents, and poll job status.

### API Endpoints

```
POST   /api/auth/register        — create account
POST   /api/auth/login           — get JWT token

POST   /api/jobs                 — create new transformation job
GET    /api/jobs                 — list user's jobs (paginated)
GET    /api/jobs/:id             — get job + status
DELETE /api/jobs/:id             — cancel/delete job

POST   /api/jobs/:id/documents   — upload source documents (multipart)
GET    /api/jobs/:id/documents   — list documents for job
GET    /api/jobs/:id/result      — download transformed documents as zip
```

### Tasks
- [ ] Implement `POST /api/auth/register` with password hashing (`argon2`)
- [ ] Implement `POST /api/auth/login` returning signed JWT
- [ ] Implement JWT middleware — extract user from token on protected routes
- [ ] Implement `POST /api/jobs` — create job, return job ID
- [ ] Implement `GET /api/jobs` — list with pagination
- [ ] Implement `GET /api/jobs/:id` — fetch job details + status
- [ ] Implement `DELETE /api/jobs/:id` — cancel job, cascade delete documents
- [ ] Implement `POST /api/jobs/:id/documents` — multipart file upload, store in DB
- [ ] Implement `GET /api/jobs/:id/result` — return output documents
- [ ] Add request validation with `validator` crate
- [ ] Add rate limiting middleware (per-IP, per-user)
- [ ] Enforce max file size (10MB default)
- [ ] Write integration tests for all endpoints

### Rust Concepts Introduced
- Axum routing, `Router`, `State`, `Extension`
- Middleware with `tower` and `axum::middleware`
- Multipart form handling
- JWT encoding/decoding
- Password hashing (argon2)
- Request/response structs with `serde`
- Input validation with `validator`
- Extractors: `Json<T>`, `Path<T>`, `Query<T>`, `Multipart`

### Done When
- [ ] All audit checklist items pass
- [ ] Full auth flow works (register → login → use token)
- [ ] Job lifecycle works via API (create → upload docs → poll status)
- [ ] Unauthenticated requests rejected with `401`

---

## Phase 4 — Redis Queue
**Status:** [ ]

### Goal
Decouple job creation from job processing. HTTP handler creates the job and enqueues it — never processes it directly.

### Queue Design
```
Queue name: "docforge:jobs"
Message:    JSON { job_id: UUID, user_id: UUID, enqueued_at: timestamp }
Pattern:    RPUSH to enqueue, BLPOP to dequeue (blocking pop)
Dead letter: "docforge:jobs:failed" for jobs that error after max retries
```

### Tasks
- [ ] Create `RedisPool` connection in app state
- [ ] Implement `queue::producer::enqueue_job(job_id)` — RPUSH to Redis
- [ ] Implement `queue::consumer::dequeue_job()` — BLPOP from Redis (blocking)
- [ ] Add job enqueueing to `POST /api/jobs` handler (after DB insert)
- [ ] Implement dead letter queue for failed jobs
- [ ] Implement job retry logic (max 3 attempts, exponential backoff)
- [ ] Add `GET /api/jobs/:id` status to reflect queue position
- [ ] Write queue producer/consumer unit tests with mocked Redis

### Rust Concepts Introduced
- Redis client with `redis` crate
- Connection pooling with `deadpool-redis`
- JSON serialization for queue messages
- `Arc<T>` for sharing Redis pool across threads
- Blocking operations in async context (`spawn_blocking`)

### Done When
- [ ] All audit checklist items pass
- [ ] Job created via API appears in Redis queue (visible in RedisInsight)
- [ ] Failed jobs appear in dead letter queue after 3 retries

---

## Phase 5 — Background Worker
**Status:** [ ]

### Goal
A concurrent worker that dequeues jobs, reads their documents, sends to Ollama, saves results, and updates job status.

### Worker Flow
```
Loop forever:
  1. BLPOP job_id from Redis queue
  2. Load job + documents from PostgreSQL
  3. For each document: spawn async task → call Ollama → get transformed doc
  4. Wait for all tasks to complete (join_all)
  5. Save all output documents to PostgreSQL
  6. Update job status to "completed" (or "failed")
  7. Emit metrics for job duration, doc count, success/failure
```

### Ollama Prompt Template
```
You are an expert software architect.

I will give you a document from Project A. Your task is to rewrite it 
for Project B, preserving all structural patterns and section headings, 
but adapting all project-specific content to the new context.

Project B context: {context}

Document filename: {filename}
Document content:
{content}

Output only the rewritten document. Do not explain your changes.
```

### Tasks
- [ ] Implement `worker::run()` — infinite loop with BLPOP
- [ ] Implement `worker::transformer::transform_document()` — single doc transformation
- [ ] Implement concurrent document processing with `tokio::spawn` + `join_all`
- [ ] Implement `ollama::client::OllamaClient` with `reqwest`
- [ ] Implement `ollama::prompt::build_prompt()` — prompt template rendering
- [ ] Handle Ollama timeout (30s default, configurable)
- [ ] Handle partial failure — if one doc fails, others still complete
- [ ] Update job status throughout: pending → processing → completed/failed
- [ ] Boot worker in `main.rs` as a separate `tokio::spawn` task alongside the server
- [ ] Implement worker concurrency limit (max N jobs simultaneously)
- [ ] Write worker unit tests with mocked Ollama

### Rust Concepts Introduced
- `tokio::spawn` for concurrent tasks
- `futures::future::join_all` — wait for all concurrent results
- `Arc<OllamaClient>` — shared HTTP client
- `tokio::time::timeout` — timeout wrapper
- `FuturesUnordered` for dynamic task sets
- Graceful shutdown with `tokio::signal`

### Done When
- [ ] All audit checklist items pass
- [ ] Full flow works: POST job → upload docs → worker picks up → Ollama transforms → results in DB
- [ ] Multiple documents processed concurrently (verify via logs)
- [ ] Job status transitions visible via `GET /api/jobs/:id`

---

## Phase 6 — Observability
**Status:** [ ]

### Goal
Full production observability. Every request traced, every job measured, errors surfaced, system health visible.

### Layers
1. **Structured Logging** — JSON logs with trace IDs, job IDs, user IDs
2. **Distributed Tracing** — spans for every handler and worker operation
3. **Metrics** — counters and histograms for Prometheus scraping
4. **Health Checks** — `/health` (alive) + `/ready` (dependencies up)

### Metrics to Track
```
docforge_jobs_total{status="completed|failed|pending"}  — counter
docforge_job_duration_seconds                           — histogram
docforge_documents_processed_total                      — counter
docforge_ollama_request_duration_seconds               — histogram
docforge_ollama_errors_total                           — counter
http_requests_total{method, path, status}              — counter
http_request_duration_seconds{method, path}            — histogram
```

### Tasks
- [ ] Set up `tracing-subscriber` with JSON formatter
- [ ] Add `TraceLayer` from `tower-http` for automatic HTTP tracing
- [ ] Add `#[tracing::instrument]` to all handlers and worker functions
- [ ] Inject `job_id` and `user_id` into tracing spans for correlation
- [ ] Set up `metrics` + `metrics-exporter-prometheus`
- [ ] Add `GET /metrics` endpoint for Prometheus scraping
- [ ] Emit all metrics defined above
- [ ] Add request ID middleware (inject `X-Request-ID` header)
- [ ] Log job state transitions at INFO with structured fields
- [ ] Log all errors at ERROR with full context chain
- [ ] Update `/ready` to check DB ping + Redis ping

### Rust Concepts Introduced
- `tracing` macros: `info!`, `error!`, `debug!`, `warn!`, `span!`
- `#[tracing::instrument]` proc macro
- Tower middleware layers
- Prometheus metrics format

### Done When
- [ ] All audit checklist items pass
- [ ] `GET /metrics` returns Prometheus-formatted metrics
- [ ] Logs are valid JSON with trace IDs
- [ ] A full job run produces correlated log lines from API → queue → worker

---

## Phase 7 — Security Hardening
**Status:** [ ]

### Goal
Close every obvious attack surface before deployment.

### Tasks
- [ ] Add CORS middleware with explicit origin allowlist
- [ ] Add security headers (`X-Content-Type-Options`, `X-Frame-Options`, `Strict-Transport-Security`)
- [ ] Enforce HTTPS in production (redirect HTTP → HTTPS)
- [ ] Rate limit: 10 requests/second per IP on auth endpoints
- [ ] Rate limit: 100 requests/minute per user on job endpoints
- [ ] Validate file types on upload (only `.md`, `.txt`, `.rst`, `.adoc`)
- [ ] Sanitize document content before sending to Ollama (strip null bytes, enforce UTF-8)
- [ ] Enforce max document count per job (20 files)
- [ ] Enforce max job count per user (50 jobs)
- [ ] Add request body size limit (10MB)
- [ ] Ensure no sensitive fields in API responses (password_hash, etc.)
- [ ] Add audit log for auth events (register, login, failed login)
- [ ] Dependency audit: `cargo audit`

### Done When
- [ ] All audit checklist items pass
- [ ] `cargo audit` reports zero known vulnerabilities
- [ ] Auth brute force is rate limited
- [ ] Malformed uploads are rejected cleanly

---

## Phase 8 — Docker & Deployment
**Status:** [ ]

### Goal
The entire system boots with one command. Anyone can run this on any machine.

### docker-compose.yml Services
```
services:
  app       — Rust API (multi-stage build, minimal image)
  worker    — Same binary, different entrypoint (--worker flag)
  postgres  — PostgreSQL 16
  redis     — Redis 7
  ollama    — Ollama server with model pre-pulled
  prometheus — Prometheus scraping /metrics
  grafana   — Grafana dashboards
```

### Dockerfile Strategy (Multi-Stage)
```dockerfile
# Stage 1: Builder
FROM rust:1.78 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Runtime (minimal)
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/docforge /usr/local/bin/
CMD ["docforge"]
```

### Tasks
- [ ] Write multi-stage `Dockerfile`
- [ ] Write `docker-compose.yml` with all services
- [ ] Write `docker-compose.override.yml` for local dev (volume mounts)
- [ ] Add `--worker` CLI flag to run in worker mode instead of API mode
- [ ] Add database migration step to Docker entrypoint
- [ ] Add Ollama model pre-pull to Docker setup
- [ ] Configure Prometheus to scrape `app:3000/metrics`
- [ ] Import Grafana dashboard JSON for key metrics
- [ ] Write `docker-compose.test.yml` for CI testing
- [ ] Document all environment variables in `.env.example`
- [ ] Test cold start: `docker compose up` on a clean machine

### Done When
- [ ] All audit checklist items pass
- [ ] `docker compose up` starts all 7 services
- [ ] Full job flow works inside Docker
- [ ] Grafana dashboard shows live metrics
- [ ] Image size under 100MB

---

## Phase 9 — Testing & Quality Audit
**Status:** [ ]

### Goal
Comprehensive test coverage. Confidence the system works before anyone sees it.

### Test Pyramid

```
Unit Tests (fast, no I/O):
  - Config loading
  - Error type mapping
  - Prompt template building
  - Queue message serialization
  - Document validation logic

Integration Tests (real DB, mocked Ollama/Redis):
  - All API endpoint happy paths
  - All API endpoint error paths
  - Auth flow (register → login → use token → expire)
  - Job lifecycle (create → upload → complete)
  - Concurrent document processing

End-to-End Tests (full Docker stack):
  - Full flow from API call to completed job result
  - Worker picks up queued job
  - Metrics appear after activity
```

### Tasks
- [ ] Set up test database (separate from dev)
- [ ] Write test helpers: `create_test_user()`, `create_test_job()`, `upload_fixture_docs()`
- [ ] Write unit tests for all pure functions
- [ ] Write integration tests for all API endpoints
- [ ] Write integration test for full job lifecycle
- [ ] Write worker unit test with mocked Ollama
- [ ] Run `cargo tarpaulin` for code coverage (target: 70%+)
- [ ] Run `cargo audit` for dependency vulnerabilities
- [ ] Run `cargo clippy -- -D warnings`
- [ ] Run `cargo fmt --check`
- [ ] Performance test: 10 concurrent job submissions
- [ ] Load test: measure requests/second at baseline

### Done When
- [ ] All tests pass
- [ ] Coverage above 70%
- [ ] Zero clippy warnings
- [ ] Zero audit vulnerabilities
- [ ] Load test results documented

---

## Phase 10 — Documentation & Demo Polish
**Status:** [ ]

### Goal
Make the project look and feel professional. A recruiter or engineer should be able to understand and run it in under 10 minutes.

### Tasks
- [ ] Write `README.md` with: what it does, demo GIF/screenshot, quick start, architecture diagram
- [ ] Write `API_SPEC.md` with every endpoint documented (method, path, request, response, errors)
- [ ] Add example request/response for every endpoint using `curl`
- [ ] Create fixture document set (5 realistic architecture docs) for demo
- [ ] Record a terminal demo: `docker compose up` → create job → upload docs → watch worker → fetch results
- [ ] Write a `LEARNINGS.md` — what Rust concepts you learned and where they appear in the code
- [ ] Add GitHub Actions CI pipeline:
  - `cargo build`
  - `cargo test`
  - `cargo clippy`
  - `cargo fmt --check`
  - `cargo audit`
- [ ] Tag `v1.0.0` release

### Done When
- [ ] README tells the full story
- [ ] CI pipeline green
- [ ] Demo works cleanly end-to-end
- [ ] Project ready to share with recruiter

---

## Rust Concepts Mastered By End

| Concept | Where You'll Learn It |
|---|---|
| Ownership & Borrowing | Phase 1-2, everywhere |
| Structs & Enums | Phase 1 |
| Traits & impl | Phase 1-3 |
| Result & Option | Phase 1-2 |
| `?` operator | Phase 2-3 |
| async/await | Phase 3-5 |
| tokio::spawn | Phase 4-5 |
| Arc, Mutex, RwLock | Phase 4-5 |
| Lifetimes | Phase 4-5 |
| Iterators & Closures | Phase 5 |
| Generics | Phase 3, 5 |
| Trait objects | Phase 6 |
| Proc macros (derive, instrument) | Phase 2, 6 |
| Testing patterns | Phase 9 |
| Cargo workspace & features | Phase 1 |

---

## Risk Register

| Risk | Likelihood | Mitigation |
|---|---|---|
| Ollama too slow for demo | Medium | Use smaller model (phi3, mistral) |
| sqlx compile-time checks fail on CI | Low | Provide `SQLX_OFFLINE=true` with `.sqlx/` cache |
| Redis connection drops under load | Low | Use connection pool with retry |
| Ollama returns malformed output | Medium | Validate output before saving, retry prompt |
| Docker image too large | Low | Multi-stage build, debian-slim base |

---

## Timeline Estimate

| Phase | Estimated Sessions |
|---|---|
| 0 — Environment | 1 session |
| 1 — Scaffolding | 1-2 sessions |
| 2 — Database | 2 sessions |
| 3 — Core API | 3 sessions |
| 4 — Redis Queue | 1-2 sessions |
| 5 — Worker | 2-3 sessions |
| 6 — Observability | 1-2 sessions |
| 7 — Security | 1 session |
| 8 — Docker | 1-2 sessions |
| 9 — Testing | 2 sessions |
| 10 — Docs & Polish | 1 session |
| **Total** | **~16-20 sessions** |
