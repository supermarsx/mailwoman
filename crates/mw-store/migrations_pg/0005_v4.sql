-- V4 (Crypto & Security) schema — POSTGRES variant (t6-e1; plan §1.1, §2.1).
-- Behaviourally identical to the SQLite `migrations/0005_v4.sql`. Dialect
-- mapping: INTEGER → BIGINT (incl. boolean-as-0/1: is_own/autocrypt/blocked),
-- BLOB → BYTEA (encrypted_private_backup / verdict_json / wrapped_seal_key), FKs
-- DEFERRABLE INITIALLY IMMEDIATE. Privacy invariants unchanged:
-- `encrypted_private_backup` is opaque (never decrypted); `dlp_audit` is redacted.

CREATE TABLE IF NOT EXISTS crypto_keys (
    id                       TEXT PRIMARY KEY NOT NULL,
    account_id               TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    kind                     TEXT NOT NULL,
    is_own                   BIGINT NOT NULL DEFAULT 0,
    addresses_json           TEXT NOT NULL DEFAULT '[]',
    fingerprint              TEXT NOT NULL DEFAULT '',
    key_id                   TEXT NOT NULL DEFAULT '',
    algorithm                TEXT NOT NULL DEFAULT '',
    created_at               TEXT NOT NULL,
    expires_at               TEXT,
    public_key               TEXT,
    cert_pem                 TEXT,
    trust                    TEXT NOT NULL DEFAULT 'unverified',
    autocrypt                BIGINT NOT NULL DEFAULT 0,
    source                   TEXT NOT NULL DEFAULT 'imported',
    encrypted_private_backup BYTEA,
    verified_at              TEXT,
    key_history_json         TEXT NOT NULL DEFAULT '[]'
);
CREATE INDEX IF NOT EXISTS idx_crypto_keys_addr ON crypto_keys (account_id, fingerprint);

CREATE TABLE IF NOT EXISTS key_associations (
    account_id    TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    address       TEXT NOT NULL,
    crypto_key_id TEXT NOT NULL REFERENCES crypto_keys(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    seen_at       TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_key_assoc ON key_associations (account_id, address);

CREATE TABLE IF NOT EXISTS security_verdicts (
    email_id     TEXT PRIMARY KEY NOT NULL,
    account_id   TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    raw_hash     TEXT NOT NULL,
    verdict_json BYTEA NOT NULL,
    computed_at  TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_security_verdicts_acct ON security_verdicts (account_id);

CREATE TABLE IF NOT EXISTS dlp_audit (
    id                    TEXT PRIMARY KEY NOT NULL,
    account_id            TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    at                    TEXT NOT NULL,
    rule_id               TEXT NOT NULL,
    rule_name             TEXT NOT NULL,
    action                TEXT NOT NULL,
    matched_detectors_json TEXT NOT NULL DEFAULT '[]',
    blocked               BIGINT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_dlp_audit_at ON dlp_audit (account_id, at);

CREATE TABLE IF NOT EXISTS sender_controls (
    account_id   TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    address      TEXT,
    thread_id    TEXT,
    action       TEXT NOT NULL,
    mail_rule_id TEXT,
    at           TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_sender_controls ON sender_controls (account_id, address);

CREATE TABLE IF NOT EXISTS crypto_changes (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    type       TEXT NOT NULL,
    state      BIGINT NOT NULL,
    object_id  TEXT NOT NULL,
    op         TEXT NOT NULL,
    at         TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_crypto_changes
    ON crypto_changes (account_id, type, state);

CREATE TABLE IF NOT EXISTS store_key_material (
    id                TEXT PRIMARY KEY NOT NULL,
    wrapped_seal_key  BYTEA NOT NULL,
    suite             TEXT NOT NULL,
    created_at        TEXT NOT NULL
);
