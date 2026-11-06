-- Browser-side errors reported by the web app, grouped by fingerprint.
-- Pseudonymous diagnostics: a group holds a scrubbed name, message, boundary and
-- route template; an event holds scrubbed stack frames, a coarse browser, OS and
-- viewport class, the build and locale. No user id, address, token, URL query,
-- message or file name is stored. account_hash is a salted hash and is NULL unless
-- the administrator enabled observability.client.includeAccountHash.
CREATE TABLE client_error_group (
    id TEXT PRIMARY KEY NOT NULL,
    fingerprint TEXT NOT NULL UNIQUE,
    kind TEXT NOT NULL,
    name TEXT NOT NULL,
    message TEXT NOT NULL,
    boundary TEXT NOT NULL DEFAULT '',
    route_template TEXT NOT NULL DEFAULT '',
    first_seen BIGINT NOT NULL,
    last_seen BIGINT NOT NULL,
    count BIGINT NOT NULL DEFAULT 0 CHECK(count >= 0),
    last_build TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open' CHECK(status IN ('open', 'resolved', 'ignored')),
    resolved_build TEXT,
    resolved_at BIGINT,
    reopened_count BIGINT NOT NULL DEFAULT 0
);
CREATE INDEX client_error_group_seen ON client_error_group(last_seen);
CREATE INDEX client_error_group_status ON client_error_group(status, last_seen);
CREATE TABLE client_error_event (
    id TEXT PRIMARY KEY NOT NULL,
    group_id TEXT NOT NULL,
    time BIGINT NOT NULL,
    build TEXT NOT NULL DEFAULT '',
    occurrences BIGINT NOT NULL DEFAULT 1,
    visit TEXT NOT NULL DEFAULT '',
    stack_raw TEXT NOT NULL DEFAULT '',
    stack_symbolicated TEXT,
    ua_family TEXT NOT NULL DEFAULT '',
    os_family TEXT NOT NULL DEFAULT '',
    viewport TEXT NOT NULL DEFAULT '',
    locale TEXT NOT NULL DEFAULT '',
    account_hash TEXT,
    extra TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX client_error_event_group ON client_error_event(group_id, time);
CREATE INDEX client_error_event_time ON client_error_event(time);
