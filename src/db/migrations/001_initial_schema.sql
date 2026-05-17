-- Users table
CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email         TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Jobs table
CREATE TABLE jobs (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status       TEXT NOT NULL DEFAULT 'pending',
    context      TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    error        TEXT
);

CREATE INDEX jobs_user_id_idx    ON jobs(user_id);
CREATE INDEX jobs_status_idx     ON jobs(status);
CREATE INDEX jobs_created_at_idx ON jobs(created_at DESC);

-- Documents table (input + output)
CREATE TABLE documents (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id     UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    filename   TEXT NOT NULL,
    content    TEXT NOT NULL,
    doc_type   TEXT NOT NULL DEFAULT 'input',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX documents_job_id_idx      ON documents(job_id);
CREATE INDEX documents_doc_type_idx    ON documents(doc_type);
