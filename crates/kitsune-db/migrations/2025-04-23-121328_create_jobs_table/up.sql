CREATE TYPE job_state as ENUM (
    'queued',
    'running',
    'failed',
    'completed'
);

CREATE TABLE jobs (
    id UUID PRIMARY KEY,
    meta JSONB NOT NULL,

    state job_state NOT NULL,
    fail_count INTEGER NOT NULL DEFAULT 0,
    run_at TIMESTAMPTZ NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    FOREIGN KEY (id) REFERENCES job_context (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE INDEX "idx-jobs-run_at" ON jobs (run_at);
CREATE INDEX "idx-jobs-state" ON jobs USING HASH (state);
CREATE INDEX "idx-jobs-updated_at" ON jobs (updated_at);

SELECT diesel_manage_updated_at('jobs');
