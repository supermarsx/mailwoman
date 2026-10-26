-- New typed ownership tables. Legacy raw-account rows remain untouched.
CREATE TABLE personal_signature (
    owner_subject TEXT NOT NULL,
    name TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    is_default BIGINT NOT NULL DEFAULT 0,
    rule_json TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL,
    html TEXT,
    text TEXT,
    text_edited BIGINT NOT NULL DEFAULT 0,
    images_json TEXT NOT NULL DEFAULT '[]',
    PRIMARY KEY (owner_subject, name)
);
CREATE TABLE personal_saved_search (
    id TEXT PRIMARY KEY NOT NULL,
    owner_subject TEXT NOT NULL,
    name TEXT NOT NULL,
    query_json TEXT NOT NULL,
    as_folder BIGINT NOT NULL DEFAULT 0,
    query_text TEXT,
    builder_json TEXT,
    accounts_json TEXT,
    settings_json TEXT NOT NULL DEFAULT '{}',
    position BIGINT NOT NULL DEFAULT 0,
    icon TEXT,
    accent_id TEXT,
    start_page BIGINT NOT NULL DEFAULT 0,
    source TEXT NOT NULL DEFAULT 'user',
    org_view_id TEXT
);
CREATE INDEX personal_saved_search_owner_idx ON personal_saved_search(owner_subject, name);
