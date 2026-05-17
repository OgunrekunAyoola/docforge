# API_SPEC.md — Docforge REST API

Base URL: `http://localhost:3000`

All protected endpoints require: `Authorization: Bearer <jwt_token>`

---

## Error Response Format

All errors return:
```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Job with id abc-123 not found"
  }
}
```

| HTTP Status | Code |
|---|---|
| 400 | `BAD_REQUEST` |
| 401 | `UNAUTHORIZED` |
| 403 | `FORBIDDEN` |
| 404 | `NOT_FOUND` |
| 409 | `CONFLICT` |
| 422 | `VALIDATION_ERROR` |
| 429 | `RATE_LIMIT_EXCEEDED` |
| 500 | `INTERNAL_ERROR` |

---

## Health

### GET /health
Liveness check. No auth required.

**Response 200:**
```json
{ "status": "ok" }
```

### GET /ready
Readiness check. Verifies DB + Redis are reachable.

**Response 200:**
```json
{ "status": "ready" }
```

**Response 400:** DB or Redis not ready.

### GET /metrics
Prometheus metrics scrape endpoint.

**Response 200:** `text/plain` Prometheus format.

---

## Auth

### POST /api/auth/register
Create a new account. Rate limited to 10 req/s per IP.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "securepassword"
}
```
Constraints: valid email, password ≥ 8 characters.

**Response 200:**
```json
{
  "token": "eyJ...",
  "user_id": "uuid",
  "email": "user@example.com"
}
```

**Errors:** 409 (email taken), 422 (validation failed)

```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"mypassword"}'
```

---

### POST /api/auth/login
Authenticate and get a JWT token. Rate limited to 10 req/s per IP.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "securepassword"
}
```

**Response 200:**
```json
{
  "token": "eyJ...",
  "user_id": "uuid",
  "email": "user@example.com"
}
```

**Errors:** 401 (invalid credentials)

```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"mypassword"}'
```

---

## Jobs

### POST /api/jobs
Create a new transformation job.

**Request:**
```json
{
  "context": "Project B is a fintech SaaS API built with FastAPI and PostgreSQL"
}
```
Constraints: context 1–2000 characters.

**Response 201:**
```json
{
  "id": "uuid",
  "status": "pending",
  "context": "Project B is...",
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-01T00:00:00Z",
  "completed_at": null,
  "error": null
}
```

```bash
curl -X POST http://localhost:3000/api/jobs \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"context":"Project B is a fintech API"}'
```

---

### GET /api/jobs
List the authenticated user's jobs, newest first.

**Query params:** `limit` (default 20, max 100), `offset` (default 0)

**Response 200:** Array of job objects.

```bash
curl http://localhost:3000/api/jobs?limit=10 \
  -H "Authorization: Bearer $TOKEN"
```

---

### GET /api/jobs/:id
Get a single job's details and status.

**Response 200:** Job object.

**Errors:** 404 (not found or not yours)

```bash
curl http://localhost:3000/api/jobs/$JOB_ID \
  -H "Authorization: Bearer $TOKEN"
```

---

### DELETE /api/jobs/:id
Delete a job and all its documents.

**Response 204:** No content.

**Errors:** 404 (not found or not yours)

```bash
curl -X DELETE http://localhost:3000/api/jobs/$JOB_ID \
  -H "Authorization: Bearer $TOKEN"
```

---

### POST /api/jobs/:id/documents
Upload source documents for transformation. Triggers the worker.

**Request:** `multipart/form-data` with one or more files.

Constraints:
- File types: `.md`, `.txt`, `.rst`, `.adoc` only
- Max file size: 10MB per file (configurable)
- Max 20 files per job

**Response 202:** Accepted (job enqueued).

**Errors:** 400 (invalid file type, too large, no files), 404 (job not found)

```bash
curl -X POST http://localhost:3000/api/jobs/$JOB_ID/documents \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@ARCHITECTURE.md" \
  -F "file=@README.md"
```

---

### GET /api/jobs/:id/documents
List input documents for a job.

**Response 200:** Array of document objects.

```json
[
  {
    "id": "uuid",
    "job_id": "uuid",
    "filename": "ARCHITECTURE.md",
    "content": "...",
    "doc_type": "input",
    "created_at": "2025-01-01T00:00:00Z"
  }
]
```

---

### GET /api/jobs/:id/result
Download the transformed output documents. Job must be `completed`.

**Response 200:** Array of output document objects (same shape, `doc_type: "output"`).

**Errors:** 400 (job not yet completed), 404 (not found)

```bash
curl http://localhost:3000/api/jobs/$JOB_ID/result \
  -H "Authorization: Bearer $TOKEN"
```

---

## Job Status Values

| Status | Meaning |
|---|---|
| `pending` | Created, not yet picked up by worker |
| `processing` | Worker is transforming documents |
| `completed` | All documents transformed |
| `failed` | Transformation failed after 3 retries |
| `cancelled` | Deleted by user |
