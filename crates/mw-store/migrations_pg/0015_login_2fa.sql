-- 0015 (26.16 t16): login second-factor (TOTP / WebAuthn / recovery codes /
-- require-2FA policy) — POSTGRES variant. Behaviourally identical to the SQLite
-- `migrations/0015_login_2fa.sql`; dialect differences: BLOB → BYTEA, INTEGER →
-- BIGINT (every boolean-as-0/1 column stays BIGINT so the i64 bind/read path is
-- UNIFORM, matching 0001..0014 — NOT native BOOLEAN, which caused the V6 0007
-- bool-bind bug). TEXT stays TEXT. ADDITIVE — NEVER edit an earlier migration.
--
-- INVARIANTS: identical to the SQLite variant — TOTP `sealed_secret` is stored
-- SEALED (XChaCha20-Poly1305), never plaintext; `cose_public_key` is a public
-- verification key (not sealed); `recovery_codes.code_hash` is an argon2 hash,
-- single-use via `used`; `sign_count` regression is a cloned-authenticator signal
-- rejected app-side.
CREATE TABLE IF NOT EXISTS totp_secrets (
    account_id     TEXT PRIMARY KEY NOT NULL,
    sealed_secret  BYTEA NOT NULL,
    confirmed      BIGINT NOT NULL DEFAULT 0,
    created_at     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS webauthn_credentials (
    credential_id    TEXT PRIMARY KEY NOT NULL,
    account_id       TEXT NOT NULL,
    cose_public_key  BYTEA NOT NULL,
    sign_count       BIGINT NOT NULL DEFAULT 0,
    transports       TEXT NOT NULL DEFAULT '',
    label            TEXT NOT NULL DEFAULT '',
    created_at       TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_webauthn_credentials_account ON webauthn_credentials (account_id);

CREATE TABLE IF NOT EXISTS recovery_codes (
    account_id   TEXT NOT NULL,
    code_hash    TEXT NOT NULL,
    used         BIGINT NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL,
    PRIMARY KEY (account_id, code_hash)
);
CREATE INDEX IF NOT EXISTS idx_recovery_codes_account ON recovery_codes (account_id);

CREATE TABLE IF NOT EXISTS twofa_policy (
    scope_kind   TEXT NOT NULL,
    scope_value  TEXT NOT NULL DEFAULT '',
    require_2fa  BIGINT NOT NULL DEFAULT 0,
    updated_by   TEXT NOT NULL DEFAULT '',
    updated_at   TEXT NOT NULL,
    PRIMARY KEY (scope_kind, scope_value)
);
