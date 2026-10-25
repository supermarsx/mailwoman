-- Explicitly configured hosting-panel authority; deployment-local and sealed.
CREATE TABLE hosting_connection (
    id TEXT PRIMARY KEY NOT NULL,
    config_json TEXT NOT NULL,
    sealed_credential BYTEA,
    revision TEXT NOT NULL,
    updated_at BIGINT NOT NULL
);
