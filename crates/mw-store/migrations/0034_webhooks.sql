-- 0034 (26.20 f34 / t28-e18): inbound webhooks as stored objects, their replay
-- cache, and who registered an outbound webhook.
-- ADDITIVE over 0001..0033 — NEVER edit an earlier migration. SQLite variant; the
-- behaviourally-identical Postgres variant is `migrations_pg/0034_webhooks.sql`.
-- (0024 remains retired and must never be filled — see `tests/t22_migration_tombstone.rs`.)
--
-- WHY
-- Before 0034 every inbound webhook call was checked against one process-wide
-- secret (`MW_WEBHOOK_INBOUND_SECRET`), named its account in the body, could ask for
-- any action, and could be replayed. 0034 makes each inbound webhook a row with its
-- own sealed secret, its bound account and the actions it may run; a signed call
-- carries a timestamp and a nonce, and a nonce is accepted once.
--
-- `inbound_webhooks`
-- * `secret_sealed` — the HMAC-SHA256 secret, sealed under the server key like
--   `webhooks.secret_sealed` (0007). Shown once when the webhook is created.
-- * `account_id` — the only account a call through this webhook acts on; a body
--   naming another account is refused.
-- * `actions_json` — JSON array of the action kinds it may run (`run_rules`,
--   `sieve`). An action outside the list is refused.
-- * timestamps are INTEGER unix seconds (the 0033 convention for new tables).
--
-- `webhook_nonce` — one row per accepted (webhook, nonce). The primary key makes
-- the second insert of a pair fail, which is the replay check, on every process
-- sharing the database. Rows older than the signature window are deleted.
--
-- `webhooks` (0007, outbound) gains `label` and `created_by` so the admin list can
-- say what an endpoint is for and who registered it.
CREATE TABLE IF NOT EXISTS inbound_webhooks (
    id             TEXT PRIMARY KEY NOT NULL,
    account_id     TEXT NOT NULL,
    secret_sealed  BLOB NOT NULL,
    actions_json   TEXT NOT NULL DEFAULT '[]',
    label          TEXT NOT NULL DEFAULT '',
    created_by     TEXT NOT NULL,
    created_at     INTEGER NOT NULL,
    last_used_at   INTEGER
);
CREATE INDEX IF NOT EXISTS idx_inbound_webhooks_acct ON inbound_webhooks (account_id);

CREATE TABLE IF NOT EXISTS webhook_nonce (
    hook_id   TEXT NOT NULL,
    nonce     TEXT NOT NULL,
    seen_at   INTEGER NOT NULL,
    PRIMARY KEY (hook_id, nonce)
);
CREATE INDEX IF NOT EXISTS idx_webhook_nonce_seen ON webhook_nonce (seen_at);

ALTER TABLE webhooks ADD COLUMN label TEXT NOT NULL DEFAULT '';
ALTER TABLE webhooks ADD COLUMN created_by TEXT NOT NULL DEFAULT '';
