-- t62-n6a: durable per-account upstream outbox (plan t62-n6-upstream-outbox.md §2).
-- Same tables as the SQLite migration (INTEGER -> BIGINT, BLOB -> BYTEA). Used
-- only with MW_UPSTREAM_ACK=local. No triggers. Every writer takes the
-- account's advisory lock (719304) first, then the head row: commit order of an
-- account's intents equals their seq order.
CREATE TABLE upstream_outbox_head (
  account_id   TEXT PRIMARY KEY NOT NULL,
  next_seq     BIGINT NOT NULL DEFAULT 1,
  backlog      BIGINT NOT NULL DEFAULT 0,
  upstream_gen BIGINT NOT NULL DEFAULT 0,
  paused       TEXT CHECK (paused IS NULL OR paused IN ('auth','operator')),
  paused_at    TEXT
);
CREATE TABLE upstream_outbox (
  account_id      TEXT NOT NULL,
  stable_id       TEXT NOT NULL,
  field           TEXT NOT NULL,
  seq             BIGINT NOT NULL,
  first_at        TEXT NOT NULL,
  updated_at      TEXT NOT NULL,
  want            TEXT,
  sent            TEXT,
  base            TEXT,
  shadow          TEXT,
  base_mailbox_id TEXT,
  base_uidvalidity BIGINT,
  base_uid        BIGINT,
  state           TEXT NOT NULL CHECK (state IN ('pending','inflight','unresolved')),
  claim           TEXT,
  attempts        BIGINT NOT NULL DEFAULT 0,
  next_at         TEXT,
  error_class     TEXT,
  error_detail    BYTEA,
  origin          TEXT NOT NULL CHECK (origin IN ('user','rule','sieve','spam','system')),
  upstream_gen    BIGINT NOT NULL,
  PRIMARY KEY (account_id, stable_id, field)
);
CREATE INDEX upstream_outbox_due ON upstream_outbox(account_id, state, next_at, seq);
CREATE UNIQUE INDEX upstream_outbox_base
  ON upstream_outbox(account_id, base_mailbox_id, base_uidvalidity, base_uid)
  WHERE base_uid IS NOT NULL;
CREATE TABLE upstream_notice (
  id         TEXT PRIMARY KEY NOT NULL,
  account_id TEXT NOT NULL,
  at         TEXT NOT NULL,
  stable_id  TEXT,
  op         TEXT NOT NULL,
  reason     TEXT NOT NULL,
  count      BIGINT NOT NULL DEFAULT 1,
  seen_at    TEXT
);
CREATE INDEX upstream_notice_account ON upstream_notice(account_id, at);
