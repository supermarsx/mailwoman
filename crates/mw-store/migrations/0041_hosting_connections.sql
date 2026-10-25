-- Explicitly configured hosting-panel authority; deployment-local and sealed.
CREATE TABLE hosting_connection (
    id TEXT PRIMARY KEY NOT NULL,
    config_json TEXT NOT NULL,
    sealed_credential BLOB,
    revision TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);
