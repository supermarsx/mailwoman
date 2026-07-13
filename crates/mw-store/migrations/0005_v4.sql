-- V4 (Crypto & Security) schema (plan §2.4). Additive over 0001_init +
-- 0002_v1_cache + 0003_v2 + 0004_v3; never edit an earlier migration. This is the
-- keyring / verdict-cache / DLP-audit / sender-control / crypto-change storage
-- behind the Mailwoman crypto + security surface.
--
-- Scaffolder note (e0): this is the skeleton. e6 fills the typed repo methods
-- (crypto_keys / key_associations / security_verdicts / dlp_audit /
-- sender_controls CRUD) in `crates/mw-store/src/v4.rs`; e0 ships the working
-- `crypto_changes` state-token triple (mirrors the V3 `pim_changes` log).
--
-- PRIVACY INVARIANTS (plan §1.2/§1.8/risk #4):
--   * `crypto_keys.encrypted_private_backup` is an OPAQUE client-encrypted blob —
--     the server NEVER decrypts it. There is deliberately NO plaintext-private
--     column: private-key material lives only in the browser worker + client vault.
--   * `dlp_audit` is REDACTED: it records the matched detector + rule, NEVER the
--     matched content (via `mw-store` redact.rs).
--   * `security_verdicts` caches a computed verdict keyed by email + raw_hash;
--     invalidated when the raw message hash changes.

-- Keys/certs: own keys (with an opaque client-encrypted backup) + harvested /
-- contact / looked-up PUBLIC keys. `kind`/`trust`/`source` are opaque strings the
-- engine owns (frozen §2.1 token sets); the store never interprets them.
CREATE TABLE IF NOT EXISTS crypto_keys (
    id                       TEXT PRIMARY KEY NOT NULL,
    account_id               TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    kind                     TEXT NOT NULL,                 -- 'pgp' | 'smime'
    is_own                   INTEGER NOT NULL DEFAULT 0,
    addresses_json           TEXT NOT NULL DEFAULT '[]',
    fingerprint              TEXT NOT NULL DEFAULT '',
    key_id                   TEXT NOT NULL DEFAULT '',
    algorithm                TEXT NOT NULL DEFAULT '',
    created_at               TEXT NOT NULL,
    expires_at               TEXT,
    public_key               TEXT,                          -- armored PGP public key
    cert_pem                 TEXT,                          -- S/MIME certificate PEM
    trust                    TEXT NOT NULL DEFAULT 'unverified',
    autocrypt                INTEGER NOT NULL DEFAULT 0,
    source                   TEXT NOT NULL DEFAULT 'imported',
    encrypted_private_backup BLOB,                          -- OPAQUE, never decrypted
    verified_at              TEXT,
    key_history_json         TEXT NOT NULL DEFAULT '[]'
);
CREATE INDEX IF NOT EXISTS idx_crypto_keys_addr ON crypto_keys (account_id, fingerprint);

-- Per-address → key association with a first-seen timestamp (TOFU history).
CREATE TABLE IF NOT EXISTS key_associations (
    account_id    TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    address       TEXT NOT NULL,
    crypto_key_id TEXT NOT NULL REFERENCES crypto_keys(id) ON DELETE CASCADE,
    seen_at       TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_key_assoc ON key_associations (account_id, address);

-- Lazy verdict cache (plan §2.2): a computed SecurityVerdict keyed by email_id +
-- raw_hash. Invalidated when raw_hash changes (a re-fetched/edited raw message).
CREATE TABLE IF NOT EXISTS security_verdicts (
    email_id     TEXT PRIMARY KEY NOT NULL,
    account_id   TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    raw_hash     TEXT NOT NULL,
    verdict_json BLOB NOT NULL,
    computed_at  TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_security_verdicts_acct ON security_verdicts (account_id);

-- REDACTED DLP audit trail (plan §1.8): matched detector + rule + action, NEVER
-- the matched content. One row per evaluation that matched a rule.
CREATE TABLE IF NOT EXISTS dlp_audit (
    id                    TEXT PRIMARY KEY NOT NULL,
    account_id            TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    at                    TEXT NOT NULL,
    rule_id               TEXT NOT NULL,
    rule_name             TEXT NOT NULL,
    action                TEXT NOT NULL,                    -- warn|block|require-encryption|notify-admin
    matched_detectors_json TEXT NOT NULL DEFAULT '[]',
    blocked               INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_dlp_audit_at ON dlp_audit (account_id, at);

-- Sender controls (plan §1.9): block/silence/ignore-conversation/report scoped to
-- an address and/or thread, linked to the real MailRule they materialized (if any).
CREATE TABLE IF NOT EXISTS sender_controls (
    account_id   TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    address      TEXT,
    thread_id    TEXT,
    action       TEXT NOT NULL,                             -- block|silence|ignore-conversation|report-*
    mail_rule_id TEXT,
    at           TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_sender_controls ON sender_controls (account_id, address);

-- Per-account crypto/security change log (plan §2.4, mirrors the V3 `pim_changes`
-- table): the raw material for `CryptoKey`/`MailRule` state tokens + `*/changes`
-- + the push `StateChange.changed` map. `type` is a ChangeType name
-- ('CryptoKey' | 'MailRule').
CREATE TABLE IF NOT EXISTS crypto_changes (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    type       TEXT NOT NULL,
    state      INTEGER NOT NULL,
    object_id  TEXT NOT NULL,
    op         TEXT NOT NULL,
    at         TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_crypto_changes
    ON crypto_changes (account_id, type, state);

-- The PQC-hybrid-wrapped mw-store seal master key at rest (plan §1.7/§2.4). The
-- seal key was previously env/derived only (no DB row); V4 gives it an additive
-- home so it can be wrapped with the hybrid X25519+ML-KEM-768 suite. `suite` is
-- the crypto-agility algorithm-suite tag (e.g. 'x25519-ml-kem-768-v1', matching
-- `mw_crypto::STORE_KEY_WRAP_SUITE'). Store-key scope only — NOT a user-facing
-- claim (ml-kem is unaudited, plan §6#8). e6 populates + reads this.
CREATE TABLE IF NOT EXISTS store_key_material (
    id                TEXT PRIMARY KEY NOT NULL,
    wrapped_seal_key  BLOB NOT NULL,
    suite             TEXT NOT NULL,
    created_at        TEXT NOT NULL
);
