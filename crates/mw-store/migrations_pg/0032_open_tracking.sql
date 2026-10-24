-- 0032 (26.20 t29-e8): opt-in, per-message open tracking — POSTGRES variant.
-- Behaviourally identical to the SQLite `migrations/0032_open_tracking.sql`, whose
-- header carries the full rationale. Dialect differences: INTEGER → BIGINT (the
-- uniform i64 bind/read path) and BLOB → BYTEA. ADDITIVE over 0001..0031 — NEVER
-- edit an earlier migration.
--
-- `open_tracking`: one row per minted token, keyed by the token's SHA-256 (the
-- token is never stored); subject and recipients sealed; totals that keep moving
-- after `open_events` stops at 200 rows per token. `open_events`: the time of each
-- recorded load, nothing else.
CREATE TABLE IF NOT EXISTS open_tracking (
    token_hash        TEXT PRIMARY KEY,
    account_id        TEXT NOT NULL,
    email_id          TEXT,
    subject_sealed    BYTEA NOT NULL,
    recipients_sealed BYTEA NOT NULL,
    created_at        TEXT NOT NULL,
    open_count        BIGINT NOT NULL DEFAULT 0,
    first_open_at     TEXT,
    last_open_at      TEXT
);
CREATE INDEX IF NOT EXISTS idx_open_tracking_email ON open_tracking (account_id, email_id);
CREATE INDEX IF NOT EXISTS idx_open_tracking_created ON open_tracking (created_at);

CREATE TABLE IF NOT EXISTS open_events (
    token_hash TEXT NOT NULL,
    at         TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_open_events_token ON open_events (token_hash);
