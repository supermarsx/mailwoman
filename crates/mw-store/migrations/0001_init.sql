CREATE TABLE IF NOT EXISTS sessions (
    id            TEXT PRIMARY KEY NOT NULL,
    account_id    TEXT NOT NULL,
    username      TEXT NOT NULL,
    jmap_url      TEXT NOT NULL,
    api_url       TEXT NOT NULL,
    sealed_creds  BLOB NOT NULL,
    created_at    TEXT NOT NULL,
    last_seen     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key    TEXT PRIMARY KEY NOT NULL,
    value  TEXT NOT NULL
);
