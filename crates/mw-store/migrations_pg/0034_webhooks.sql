-- 0034 (26.20 f34 / t28-e18): inbound webhooks as stored objects, their replay
-- cache, and who registered an outbound webhook — POSTGRES variant.
-- Behaviourally identical to the SQLite `migrations/0034_webhooks.sql`, whose
-- header carries the full rationale. Dialect differences: INTEGER → BIGINT (the
-- uniform i64 bind/read path) and BLOB → BYTEA. ADDITIVE over 0001..0033 — NEVER
-- edit an earlier migration.
--
-- `inbound_webhooks`: one row per inbound webhook, with its sealed secret, bound
-- account and allowed action kinds. `webhook_nonce`: accepted (webhook, nonce)
-- pairs; the primary key is the replay check. `webhooks` gains `label` and
-- `created_by`.
CREATE TABLE IF NOT EXISTS inbound_webhooks (
    id             TEXT PRIMARY KEY NOT NULL,
    account_id     TEXT NOT NULL,
    secret_sealed  BYTEA NOT NULL,
    actions_json   TEXT NOT NULL DEFAULT '[]',
    label          TEXT NOT NULL DEFAULT '',
    created_by     TEXT NOT NULL,
    created_at     BIGINT NOT NULL,
    last_used_at   BIGINT
);
CREATE INDEX IF NOT EXISTS idx_inbound_webhooks_acct ON inbound_webhooks (account_id);

CREATE TABLE IF NOT EXISTS webhook_nonce (
    hook_id   TEXT NOT NULL,
    nonce     TEXT NOT NULL,
    seen_at   BIGINT NOT NULL,
    PRIMARY KEY (hook_id, nonce)
);
CREATE INDEX IF NOT EXISTS idx_webhook_nonce_seen ON webhook_nonce (seen_at);

ALTER TABLE webhooks ADD COLUMN label TEXT NOT NULL DEFAULT '';
ALTER TABLE webhooks ADD COLUMN created_by TEXT NOT NULL DEFAULT '';
