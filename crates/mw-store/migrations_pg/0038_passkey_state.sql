-- Legacy backup eligibility is unknown until a successful signed assertion.
ALTER TABLE webauthn_credentials ADD COLUMN backup_eligible BIGINT CHECK (backup_eligible IN (0, 1));
ALTER TABLE webauthn_credentials ADD COLUMN backup_state BIGINT NOT NULL DEFAULT 0 CHECK (backup_state IN (0, 1));
ALTER TABLE webauthn_credentials ADD COLUMN uv_initialized BIGINT NOT NULL DEFAULT 0 CHECK (uv_initialized IN (0, 1));
ALTER TABLE webauthn_credentials ADD COLUMN state_revision BIGINT NOT NULL DEFAULT 0;
