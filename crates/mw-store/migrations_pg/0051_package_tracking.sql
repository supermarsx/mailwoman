CREATE TABLE package_provider (
    id TEXT PRIMARY KEY NOT NULL CHECK (id = 'aftership'),
    enabled BIGINT NOT NULL DEFAULT 0,
    refresh_secs BIGINT NOT NULL DEFAULT 900,
    sealed_key BYTEA NOT NULL,
    revision TEXT NOT NULL
);
CREATE TABLE package_account (
    account_id TEXT PRIMARY KEY NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    state TEXT NOT NULL,
    revision TEXT NOT NULL
);
