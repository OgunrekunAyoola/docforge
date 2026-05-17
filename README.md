# Docforge

A production-grade document transformation API built in Rust. Upload architecture and design documents from a source project, describe both project contexts, and Docforge rewrites them for your target stack using an LLM — in seconds.

**Built to demonstrate production backend engineering with Rust.**

---

## Demo

![Docforge job tracker showing completed transformation](docs/screenshots/Screenshot%202026-05-17%20141707.png)

**Flow:** Register → Create job → Upload docs → Watch live status → View AI-transformed result

---

## Tech Stack

| Layer | Technology |
|---|---|
| Web framework | Axum 0.7 |
| Async runtime | Tokio |
| Database | PostgreSQL + sqlx |
| Queue / Cache | Redis (deadpool-redis) |
| Background worker | Tokio tasks + Semaphore |
| LLM | Groq API (OpenAI-compatible) |
| Auth | JWT + Argon2 password hashing |
| Observability | tracing + Prometheus + Grafana |
| Frontend | Next.js 16 + Tailwind CSS |
| Containerisation | Docker + Docker Compose |

---

## Architecture

```
┌─────────────────┐     ┌──────────────┐     ┌─────────────────┐
│   Next.js UI    │────▶│  Axum API    │────▶│   PostgreSQL    │
│  (port 3002)    │     │  (port 3000) │     │   (port 5433)   │
└─────────────────┘     └──────┬───────┘     └─────────────────┘
                               │
                               ▼
                        ┌──────────────┐     ┌─────────────────┐
                        │    Redis     │────▶│  Tokio Worker   │
                        │  Job Queue   │     │  (concurrent)   │
                        └──────────────┘     └────────┬────────┘
                                                       │
                               ┌───────────────────────┘
                               ▼
                        ┌──────────────┐     ┌─────────────────┐
                        │  Groq API    │     │   Prometheus    │
                        │ Llama 3.3 70B│     │  + Grafana      │
                        └──────────────┘     └─────────────────┘
```

**Request flow:**
1. Client uploads documents and posts job context via the REST API
2. API stores documents in PostgreSQL, pushes job ID to Redis queue
3. Worker pulls from Redis, calls Groq in parallel for each document
4. Transformed documents saved back to Postgres, job status updated
5. Client polls job status and fetches results when complete

---

## Features

- **JWT authentication** — register, login, all routes protected
- **Document upload** — multipart, up to 20 files, `.md .txt .rst .adoc`, 10MB max
- **Async job queue** — Redis RPUSH/BLPOP, configurable worker concurrency
- **Parallel LLM transforms** — each document transformed concurrently via `join_all`
- **Dead letter queue** — failed jobs retried 3× with exponential backoff
- **Per-IP rate limiting** — token bucket via `governor` + `dashmap`
- **Structured logging** — JSON tracing with request IDs
- **Prometheus metrics** — job counters, HTTP histograms, LLM latency
- **Grafana dashboard** — pre-provisioned, auto-loads on `docker compose up`
- **Graceful shutdown** — Ctrl+C drains in-flight requests cleanly

---

## Quick Start

### Prerequisites
- Docker + Docker Compose
- A [Groq API key](https://console.groq.com) (free tier)

### Run

```bash
git clone https://github.com/OgunrekunAyoola/docforge.git
cd docforge

cp .env.example .env
# Edit .env and set LLM_API_KEY=your-groq-key

docker compose up
```

| Service | URL |
|---------|-----|
| API | http://localhost:3000 |
| Frontend | Run separately — see below |
| Grafana | http://localhost:3001 (admin/admin) |
| Prometheus | http://localhost:9090 |

### Run the frontend

```bash
cd frontend
npm install
npm run dev -- --port 3002
```

Then open http://localhost:3002

---

## Local Development (without Docker)

### Prerequisites
- Rust 1.88+
- PostgreSQL running on port 5433 (or update `DATABASE_URL`)
- Redis on port 6379

```bash
cp .env.example .env
# Fill in your values

cargo run
```

The server runs migrations on startup automatically.

---

## API Reference

### Auth
```
POST /api/auth/register   { email, password }  → { token }
POST /api/auth/login      { email, password }  → { token }
```

### Jobs
```
POST   /api/jobs              Create job
GET    /api/jobs              List jobs
GET    /api/jobs/:id          Get job (with status)
DELETE /api/jobs/:id          Delete job
```

### Documents
```
POST /api/jobs/:id/documents  Upload files (multipart)
GET  /api/jobs/:id/documents  List source documents
GET  /api/jobs/:id/result     Get transformed output
```

### Health
```
GET /health    Liveness check
GET /ready     Readiness check (pings DB + Redis)
GET /metrics   Prometheus metrics
```

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Bind address |
| `PORT` | `3000` | Bind port |
| `DATABASE_URL` | — | Postgres connection string |
| `REDIS_URL` | — | Redis connection string |
| `LLM_BASE_URL` | Groq API | OpenAI-compatible base URL |
| `LLM_MODEL` | `llama-3.3-70b-versatile` | Model name |
| `LLM_API_KEY` | — | API key (never commit this) |
| `JWT_SECRET` | — | Secret for signing tokens |
| `JWT_EXPIRY_HOURS` | `24` | Token lifetime |
| `WORKER_CONCURRENCY` | `3` | Parallel jobs per worker |
| `MAX_FILE_SIZE_MB` | `10` | Upload size limit |

---

## Why Rust?

This project deliberately chose Rust over Go, Python, or Node for reasons that matter in production:

- **No garbage collector** — no GC pauses means flat, predictable latency at every percentile
- **Memory safety at compile time** — the borrow checker eliminates use-after-free and data races before the binary is built
- **Fearless concurrency** — the worker's parallel `join_all` is safe by construction; the compiler rejects races
- **Tiny deployment footprint** — a single 8MB binary in a slim container vs. a full language runtime

These are the same reasons AWS (Firecracker), Cloudflare (Pingora), Microsoft (Windows components), and the Linux kernel adopted Rust.

---

## Project Structure

```
src/
├── main.rs              Entry point — boots API + worker
├── config.rs            Environment config
├── error.rs             AppError enum with structured JSON responses
├── api/                 Axum handlers and middleware
│   ├── auth.rs          Register + login
│   ├── jobs.rs          Job CRUD + document upload
│   ├── health.rs        Health, readiness, metrics
│   └── middleware/      JWT auth extractor, rate limiter
├── db/                  sqlx queries + migrations
├── models/              Job, Document, User structs
├── queue/               Redis producer + consumer
├── worker/              Job processor + LLM transformer
├── ollama/              Groq HTTP client + prompt templates
└── observability/       Tracing + Prometheus init
frontend/                Next.js 16 dashboard
```

---

## License

MIT
