-- Data subject requests: the request queue, a per-request event trail and the
-- parts of a built export. A request row keeps the filterable columns; the rest
-- (details, decisions, hold, completion record, download token hash) is in
-- body_json, updated under the rev counter. Export parts are stored one file per
-- row, in chunks, and are deleted when the export expires.
CREATE TABLE dsr_request (
    id TEXT PRIMARY KEY NOT NULL,
    address TEXT NOT NULL,
    domain TEXT NOT NULL,
    account_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    due_at BIGINT NOT NULL,
    assignee TEXT,
    rev BIGINT NOT NULL,
    body_json TEXT NOT NULL
);
CREATE INDEX dsr_request_address ON dsr_request(address, created_at);
CREATE INDEX dsr_request_status ON dsr_request(status, due_at);
CREATE INDEX dsr_request_domain ON dsr_request(domain, status);

CREATE TABLE dsr_event (
    id TEXT PRIMARY KEY NOT NULL,
    request_id TEXT NOT NULL,
    ts BIGINT NOT NULL,
    actor TEXT NOT NULL,
    kind TEXT NOT NULL,
    detail_json TEXT NOT NULL
);
CREATE INDEX dsr_event_request ON dsr_event(request_id, ts);

CREATE TABLE dsr_part (
    request_id TEXT NOT NULL,
    name TEXT NOT NULL,
    seq BIGINT NOT NULL,
    sealed BIGINT NOT NULL,
    data BLOB NOT NULL,
    PRIMARY KEY(request_id, name, seq)
);
