CREATE TABLE admin_webauthn_credentials (
 credential_id TEXT PRIMARY KEY,
 account_id TEXT NOT NULL REFERENCES admin_account(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
 cose_public_key BLOB NOT NULL,
 sign_count INTEGER NOT NULL CHECK(sign_count >= 0),
 transports TEXT NOT NULL,
 label TEXT NOT NULL,
 created_at TEXT NOT NULL,
 backup_eligible INTEGER NOT NULL CHECK(backup_eligible IN (0,1)),
 backup_state INTEGER NOT NULL CHECK(backup_state IN (0,1)),
 uv_initialized INTEGER NOT NULL CHECK(uv_initialized = 1),
 state_revision INTEGER NOT NULL DEFAULT 0,
 CHECK(backup_state = 0 OR backup_eligible = 1)
);
CREATE INDEX admin_webauthn_account ON admin_webauthn_credentials(account_id);
CREATE TABLE admin_recovery_codes (
 account_id TEXT NOT NULL REFERENCES admin_account(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
 code_hash TEXT NOT NULL,
 used INTEGER NOT NULL DEFAULT 0 CHECK(used IN (0,1)),
 PRIMARY KEY(account_id, code_hash)
);
