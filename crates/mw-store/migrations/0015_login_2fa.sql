-- 0015 (26.16 t16): login second-factor — TOTP secrets, WebAuthn credentials,
-- one-time recovery codes, and the admin require-2FA policy. ADDITIVE over
-- 0001..0014 — NEVER edit an earlier migration. This is the SQLite variant (run by
-- `sqlx::migrate!("./migrations")`); the behaviourally-identical Postgres variant is
-- `migrations_pg/0015_login_2fa.sql`.
--
-- INVARIANTS (mirror 0011..0014):
--   * TOTP `sealed_secret` (the shared HMAC key) is a SECRET: stored SEALED
--     (XChaCha20-Poly1305 under the store ServerKey) in a BLOB here / BYTEA in
--     Postgres, never plaintext — the same zero-access posture as
--     `sessions.sealed_creds`. `webauthn_credentials.cose_public_key` is a PUBLIC
--     key (verification only) so it is NOT sealed, only stored opaque.
--   * `recovery_codes.code_hash` holds an argon2 hash of a CSPRNG code, never the
--     code itself; single-use is enforced app-side by flipping `used`.
--   * Every boolean is the 0/1 column `confirmed`/`used`/`require_2fa`, INTEGER here
--     / BIGINT in Postgres, so the i64 bind/read path is UNIFORM across dialects
--     (the V6 lesson — NEVER native BOOLEAN, which caused the 0007 pg bool-bind bug).
--   * `sign_count` (WebAuthn signature counter) is INTEGER/BIGINT, i64-uniform; a
--     non-increasing counter on assertion is a cloned-authenticator signal (rejected
--     app-side).

-- One confirmed TOTP authenticator per account (enrolment replaces any pending one).
CREATE TABLE IF NOT EXISTS totp_secrets (
    account_id     TEXT PRIMARY KEY NOT NULL,
    sealed_secret  BLOB NOT NULL,               -- SEALED HMAC key (never plaintext)
    confirmed      INTEGER NOT NULL DEFAULT 0,  -- 0/1 (enrolment verified)
    created_at     TEXT NOT NULL
);

-- Zero-or-more platform/roaming authenticators per account.
CREATE TABLE IF NOT EXISTS webauthn_credentials (
    credential_id    TEXT PRIMARY KEY NOT NULL, -- base64url credential id (globally unique)
    account_id       TEXT NOT NULL,
    cose_public_key  BLOB NOT NULL,             -- COSE_Key public bytes (public; not sealed)
    sign_count       INTEGER NOT NULL DEFAULT 0,-- last accepted signature counter
    transports       TEXT NOT NULL DEFAULT '',  -- hint list (e.g. "usb,nfc"); non-secret
    label            TEXT NOT NULL DEFAULT '',   -- user-facing name
    created_at       TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_webauthn_credentials_account ON webauthn_credentials (account_id);

-- One-time break-glass recovery codes (argon2-hashed, single-use).
CREATE TABLE IF NOT EXISTS recovery_codes (
    account_id   TEXT NOT NULL,
    code_hash    TEXT NOT NULL,                 -- argon2 hash of a CSPRNG code
    used         INTEGER NOT NULL DEFAULT 0,    -- 0/1 (consumed)
    created_at   TEXT NOT NULL,
    PRIMARY KEY (account_id, code_hash)
);
CREATE INDEX IF NOT EXISTS idx_recovery_codes_account ON recovery_codes (account_id);

-- Admin require-2FA policy: global (scope_value='') or per-domain (scope_value=domain).
CREATE TABLE IF NOT EXISTS twofa_policy (
    scope_kind   TEXT NOT NULL,                 -- 'global' | 'domain'
    scope_value  TEXT NOT NULL DEFAULT '',      -- '' for global; the domain otherwise
    require_2fa  INTEGER NOT NULL DEFAULT 0,    -- 0/1 (second factor required in scope)
    updated_by   TEXT NOT NULL DEFAULT '',
    updated_at   TEXT NOT NULL,
    PRIMARY KEY (scope_kind, scope_value)
);
