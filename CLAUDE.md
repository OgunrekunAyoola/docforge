# CLAUDE.md — Docforge Agentic Build Rules

This file governs all AI-assisted development on the Docforge project.
Every agent session must read this file before writing any code.

---

## Project Identity

**Name:** Docforge  
**Type:** SaaS REST API  
**Purpose:** Accepts architecture/design documents from a source project and generates equivalent documents adapted to a new project context — using a local Ollama LLM as the transformation engine.  
**Target:** Rust startup portfolio project demonstrating production-grade backend engineering.

---

## Tech Stack (Non-Negotiable)

| Layer | Technology | Crate |
|---|---|---|
| Web Framework | Axum | `axum` |
| Async Runtime | Tokio | `tokio` |
| Database | PostgreSQL | `sqlx` |
| Queue / Cache | Redis | `redis` |
| Background Worker | Tokio task | `tokio` |
| LLM | Ollama (local) | `reqwest` |
| Serialization | JSON | `serde`, `serde_json` |
| Error Handling | Structured | `anyhow`, `thiserror` |
| Observability | Tracing | `tracing`, `tracing-subscriber` |
| Metrics | Prometheus | `metrics`, `metrics-exporter-prometheus` |
| Auth | JWT | `jsonwebtoken` |
| Validation | Request validation | `validator` |
| Config | Environment vars | `config`, `dotenvy` |
| Containerization | Docker | `Dockerfile`, `docker-compose.yml` |
| Testing | Built-in + HTTP | `tokio::test`, `axum-test` |

---

## Module Structure

```
src/
├── main.rs              — entry point, boots server + worker
├── config.rs            — environment config loader
├── error.rs             — global AppError enum
├── db/
│   ├── mod.rs           — pool setup
│   └── migrations/      — SQL migration files
├── api/
│   ├── mod.rs           — router assembly
│   ├── jobs.rs          — job CRUD handlers
│   ├── auth.rs          — auth handlers
│   └── health.rs        — health + readiness endpoints
├── models/
│   ├── job.rs           — Job struct, JobStatus enum
│   ├── document.rs      — Document struct
│   └── user.rs          — User struct
├── queue/
│   ├── mod.rs           — Redis queue abstraction
│   ├── producer.rs      — enqueue jobs
│   └── consumer.rs      — dequeue + dispatch
├── worker/
│   ├── mod.rs           — worker loop
│   └── transformer.rs   — document transformation logic
├── ollama/
│   ├── mod.rs           — Ollama HTTP client
│   └── prompt.rs        — prompt templates
└── observability/
    ├── mod.rs           — tracing + metrics setup
    └── metrics.rs       — custom metric definitions
```

---

## Non-Negotiable Code Rules

### Error Handling
- NEVER use `unwrap()` or `expect()` in non-test code without a comment proving it cannot panic
- ALL errors must be propagated with `?` or explicitly matched
- ALL API handlers must return `Result<impl IntoResponse, AppError>`
- `AppError` must implement `IntoResponse` to produce structured JSON error responses

### Async
- NEVER block the async runtime — no `std::thread::sleep`, no synchronous I/O on the main thread
- Use `tokio::time::sleep` for delays
- Use `tokio::fs` for file operations, never `std::fs` in async context

### Ownership & Borrowing
- Avoid `.clone()` without a comment explaining why ownership cannot be transferred
- Prefer passing references over cloning for read-only access
- Use `Arc<T>` for shared state across tasks, never raw pointers

### Concurrency
- All shared mutable state must use `Arc<Mutex<T>>` or `Arc<RwLock<T>>`
- Prefer `RwLock` over `Mutex` when reads are frequent and writes are rare
- Never hold a lock across an `.await` point

### Database
- NEVER write raw SQL strings — use `sqlx::query!` or `sqlx::query_as!` macros (compile-time checked)
- ALL database operations must be wrapped in proper error handling
- Use database transactions for multi-step writes
- Migrations live in `db/migrations/` and run on startup

### Security
- NEVER log sensitive data (passwords, tokens, API keys, document content)
- NEVER store secrets in code — use environment variables via `.env`
- Validate ALL incoming request bodies with `validator`
- Sanitize file content before sending to Ollama
- Rate limit all public endpoints

### Observability
- Every handler must emit a tracing span: `#[tracing::instrument]`
- Log job state transitions at `INFO` level
- Log errors at `ERROR` level with full context
- Emit a metric counter for every job state change
- HTTP request duration must be tracked as a histogram

### Testing
- Every handler must have at least one integration test
- Every business logic function must have a unit test
- Tests MUST NOT hit real external services — mock Ollama and Redis in tests
- Use `#[tokio::test]` for async tests

---

## Guardrails — What Claude Must Never Do

- Never delete or overwrite source documents provided by the user
- Never hardcode secrets, API keys, URLs, or model names
- Never skip database migrations — always use `sqlx migrate run`
- Never return raw database errors to the client — map to AppError
- Never accept unbounded file uploads — enforce max file size
- Never process jobs synchronously in the HTTP handler — always queue
- Never merge a phase without running the full audit checklist

---

## Autonomous Audit Checklist

Before declaring ANY phase or feature complete, run through every item:

### Build Quality
- [ ] `cargo build` — zero errors
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo fmt --check` — zero formatting issues
- [ ] `cargo test` — all tests pass
- [ ] No `unwrap()` calls in `src/` outside of test modules

### Functionality
- [ ] Happy path works end-to-end
- [ ] All error paths return structured JSON with correct HTTP status codes
- [ ] Invalid inputs are rejected with helpful error messages
- [ ] Feature works with Docker Compose up (not just `cargo run`)

### Security
- [ ] No secrets in code or committed `.env` files
- [ ] Auth protected routes reject unauthenticated requests
- [ ] File size limits enforced
- [ ] No sensitive data in logs

### Observability
- [ ] New handlers have tracing spans
- [ ] New job state transitions emit metrics
- [ ] Errors are logged with context

### Database
- [ ] Migration file exists for any schema change
- [ ] No raw SQL strings — all queries use `sqlx` macros
- [ ] Queries tested against a real test database

### Documentation
- [ ] New endpoints documented in API_SPEC.md
- [ ] New environment variables documented in `.env.example`
- [ ] Non-obvious logic has a short inline comment explaining WHY

---

## Environment Variables Required

```env
# Server
HOST=0.0.0.0
PORT=3000

# Database
DATABASE_URL=postgres://user:password@localhost:5432/docforge

# Redis
REDIS_URL=redis://localhost:6379

# Ollama
OLLAMA_URL=http://localhost:11434
OLLAMA_MODEL=llama3

# Auth
JWT_SECRET=your-secret-here
JWT_EXPIRY_HOURS=24

# Worker
WORKER_CONCURRENCY=3
MAX_FILE_SIZE_MB=10
```

---

## Git Discipline

- Branch per phase: `phase/1-scaffolding`, `phase/2-file-api`, etc.
- Commits must be atomic — one logical change per commit
- Commit message format: `type(scope): description`
  - `feat(jobs): add job creation endpoint`
  - `fix(worker): handle Ollama timeout gracefully`
  - `test(jobs): add integration test for job status polling`
- Never commit `.env` — only `.env.example`
- Never commit `target/` directory

---

## Definition of Done (Per Phase)

A phase is DONE when:
1. All audit checklist items above pass
2. The feature is demonstrated working via `docker compose up`
3. At least one integration test covers the new feature
4. `PLAN.md` phase is marked complete
5. Any new env vars are in `.env.example`
