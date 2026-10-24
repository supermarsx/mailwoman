-- 0032 (26.20 t29-e8): opt-in, per-message open tracking.
-- ADDITIVE over 0001..0031 — NEVER edit an earlier migration. SQLite variant; the
-- behaviourally-identical Postgres variant is `migrations_pg/0032_open_tracking.sql`.
-- (0024 remains retired and must never be filled — see `tests/t22_migration_tombstone.rs`.)
--
-- WHAT
-- A sender who ticks "record when this message is opened" gets a token; the sent
-- HTML carries a 1x1 image at `<MW_PUBLIC_URL>/t/<token>.gif`. When a mail program
-- loads that image, the server records the time. Nothing else about the request is
-- stored: no address, no user agent, no headers.
--
-- `open_tracking` — one row per minted token.
-- * `token_hash` — hex SHA-256 of the token. The token itself is never stored, so
--   a copy of the database cannot be turned into working image URLs.
-- * `account_id` — the sender's account; every read is scoped by it.
-- * `email_id` — the sent message, bound after the send (`PUT /api/track/open/{token}`).
--   NULL until then; unbound rows older than 24 h are deleted on the next mint.
-- * `subject_sealed`, `recipients_sealed` — sealed under the server key like other
--   message metadata (notes, 0019), so they are not readable at rest.
-- * `open_count`, `first_open_at`, `last_open_at` — the totals. They keep moving
--   after `open_events` stops growing (200 rows per token).
-- Every row older than 90 days is deleted on the next mint.
--
-- `open_events` — the time of each recorded load, at most 200 per token.
CREATE TABLE IF NOT EXISTS open_tracking (
    token_hash        TEXT PRIMARY KEY,
    account_id        TEXT NOT NULL,
    email_id          TEXT,
    subject_sealed    BLOB NOT NULL,
    recipients_sealed BLOB NOT NULL,
    created_at        TEXT NOT NULL,
    open_count        INTEGER NOT NULL DEFAULT 0,
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
