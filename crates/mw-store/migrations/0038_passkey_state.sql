-- Legacy backup eligibility is unknown until a successful signed assertion.
ALTER TABLE webauthn_credentials ADD COLUMN backup_eligible INTEGER CHECK (backup_eligible IN (0, 1));
ALTER TABLE webauthn_credentials ADD COLUMN backup_state INTEGER NOT NULL DEFAULT 0 CHECK (backup_state IN (0, 1));
ALTER TABLE webauthn_credentials ADD COLUMN uv_initialized INTEGER NOT NULL DEFAULT 0 CHECK (uv_initialized IN (0, 1));
ALTER TABLE webauthn_credentials ADD COLUMN state_revision INTEGER NOT NULL DEFAULT 0;
