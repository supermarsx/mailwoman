-- 0011 (26.12 t12 conformance): per-account EWS credential binding for the on-prem
-- Microsoft Exchange (EWS) bridge (SPEC §6.5/§27, t12 plan §2 e-ews / §5 flag 1).
-- ADDITIVE over 0001..0010 — NEVER edit an earlier migration. This is the SQLite
-- variant (run by `sqlx::migrate!("./migrations")`); the behaviourally-identical
-- Postgres variant is `migrations_pg/0011_ews_account_cred.sql`.
--
-- INVARIANTS (mirror 0007..0010):
--   * The cleartext EWS user/domain/password/workstation is a SECRET: it is stored
--     SEALED (XChaCha20-Poly1305 under the store ServerKey) in `sealed_cred` (BLOB
--     here / BYTEA in Postgres), never in plaintext columns — the same zero-access
--     posture as `accounts.sealed_creds` / `sessions.sealed_creds`. The guest never
--     holds a long-lived secret; the host unseals per call and hands it over the
--     gated `basic-credentials` import.
--   * The boolean-as-0/1 column `enabled` is INTEGER here / BIGINT in Postgres so the
--     i64 bind/read path is UNIFORM across dialects (the V6 lesson — never native
--     BOOLEAN, which caused the 0007 Postgres bool-bind bug).
--   * `endpoint` (the EWS SOAP URL) and `endpoint_host` (its host, mirrored into the
--     bridge `net_allowlist` at mount) are NOT secrets and stay plaintext TEXT.
--   * The auth SCHEME is derived from the sealed credential at unseal time (empty NT
--     domain ⇒ HTTP Basic, non-empty ⇒ NTLMv2); no scheme column is stored.
CREATE TABLE IF NOT EXISTS ews_account_cred (
    account_id     TEXT PRIMARY KEY NOT NULL,   -- the local account this binding serves
    endpoint       TEXT NOT NULL,               -- the account's EWS SOAP endpoint URL
    endpoint_host  TEXT NOT NULL,               -- host of `endpoint` (mirrored into net_allowlist)
    sealed_cred    BLOB NOT NULL,               -- SEALED {user,domain,password,workstation}
    enabled        INTEGER NOT NULL DEFAULT 1,  -- 0/1 (binding active)
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
);
