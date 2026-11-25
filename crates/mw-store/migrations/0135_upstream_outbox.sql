-- t62-n6a: durable per-account upstream outbox (plan t62-n6-upstream-outbox.md §2).
-- Used only with MW_UPSTREAM_ACK=local: a flag change, a move or a draft copy is
-- acknowledged after the local commit and applied to the mail server later from
-- these rows. With the default mode (upstream) nothing writes them.
--
-- upstream_outbox holds INTENTS, one row per (account, message, field) with the
-- wanted value, not an operation log: repeated changes overwrite the row and a
-- change back to the server's value deletes it. No account foreign key (same
-- convention as 0113/0116); erasure removes the rows through data_map.rs.
-- Rows hold ids, UIDs, mailbox ids and flag names, which `messages` already
-- stores unsealed; only error_detail (server text) is sealed.
CREATE TABLE upstream_outbox_head (
  account_id   TEXT PRIMARY KEY NOT NULL,
  next_seq     INTEGER NOT NULL DEFAULT 1,
  backlog      INTEGER NOT NULL DEFAULT 0,
  upstream_gen INTEGER NOT NULL DEFAULT 0,
  paused       TEXT CHECK (paused IS NULL OR paused IN ('auth','operator')),
  paused_at    TEXT
);
CREATE TABLE upstream_outbox (
  account_id      TEXT NOT NULL,
  stable_id       TEXT NOT NULL,
  field           TEXT NOT NULL,
  seq             INTEGER NOT NULL,
  first_at        TEXT NOT NULL,
  updated_at      TEXT NOT NULL,
  want            TEXT,
  sent            TEXT,
  base            TEXT,
  shadow          TEXT,
  base_mailbox_id TEXT,
  base_uidvalidity INTEGER,
  base_uid        INTEGER,
  state           TEXT NOT NULL CHECK (state IN ('pending','inflight','unresolved')),
  claim           TEXT,
  attempts        INTEGER NOT NULL DEFAULT 0,
  next_at         TEXT,
  error_class     TEXT,
  error_detail    BLOB,
  origin          TEXT NOT NULL CHECK (origin IN ('user','rule','sieve','spam','system')),
  upstream_gen    INTEGER NOT NULL,
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
  count      INTEGER NOT NULL DEFAULT 1,
  seen_at    TEXT
);
CREATE INDEX upstream_notice_account ON upstream_notice(account_id, at);
