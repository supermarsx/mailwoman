-- 0011 (26.12 t12 conformance): per-account EWS credential binding — POSTGRES variant
-- (SPEC §6.5/§27, t12 plan §2 e-ews / §5 flag 1). Behaviourally identical to the
-- SQLite `migrations/0011_ews_account_cred.sql`; dialect differences: BLOB → BYTEA,
-- INTEGER → BIGINT (the boolean-as-0/1 column stays BIGINT so the i64 bind/read path
-- is UNIFORM, matching 0001..0010 — NOT native BOOLEAN, which caused the V6 0007
-- bool-bind bug). TEXT stays TEXT. ADDITIVE — NEVER edit an earlier migration.
--
-- INVARIANTS: identical to the SQLite variant — the cleartext EWS credential is
-- stored SEALED (XChaCha20-Poly1305) in `sealed_cred`, never plaintext; `endpoint` /
-- `endpoint_host` are non-secret plaintext; the auth scheme is derived from the
-- sealed credential (empty NT domain ⇒ Basic, non-empty ⇒ NTLMv2), not stored.
CREATE TABLE IF NOT EXISTS ews_account_cred (
    account_id     TEXT PRIMARY KEY NOT NULL,
    endpoint       TEXT NOT NULL,
    endpoint_host  TEXT NOT NULL,
    sealed_cred    BYTEA NOT NULL,
    enabled        BIGINT NOT NULL DEFAULT 1,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
);
