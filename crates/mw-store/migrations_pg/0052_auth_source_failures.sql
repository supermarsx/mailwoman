-- Credential-failure state is separate from manual global source bans.
CREATE TABLE auth_source_failure (
    surface TEXT NOT NULL CHECK(surface IN ('admin','webmail')),
    source_ip TEXT NOT NULL,
    attempts_json TEXT NOT NULL,
    locked_until BIGINT NOT NULL DEFAULT 0,
    expires_at BIGINT NOT NULL,
    revision TEXT NOT NULL,
    PRIMARY KEY(surface,source_ip)
);
CREATE INDEX auth_source_failure_expiry ON auth_source_failure(expires_at);
