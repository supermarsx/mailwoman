-- Archive tier for the long retention class. A segment is one sealed, compressed
-- file of aged rows from one store; segments of a store form a hash chain
-- (seal = HMAC over the previous seal, the content hash and the segment's
-- counters). The index maps a row key to its segment so a lookup does not open
-- every file. Row contents are only in the segment file.
CREATE TABLE retention_archive_segment (
    id TEXT PRIMARY KEY NOT NULL,
    store_id TEXT NOT NULL,
    seq BIGINT NOT NULL,
    row_count BIGINT NOT NULL,
    first_ts BIGINT NOT NULL,
    last_ts BIGINT NOT NULL,
    bytes BIGINT NOT NULL,
    content_hash TEXT NOT NULL,
    prev_seal TEXT NOT NULL,
    seal TEXT NOT NULL,
    key_gen BIGINT NOT NULL,
    location TEXT NOT NULL,
    state TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    verified_at BIGINT NOT NULL,
    verify_status TEXT NOT NULL
);
CREATE UNIQUE INDEX retention_archive_segment_seq ON retention_archive_segment(store_id, seq);

CREATE TABLE retention_archive_index (
    id TEXT PRIMARY KEY NOT NULL,
    segment_id TEXT NOT NULL,
    store_id TEXT NOT NULL,
    row_key TEXT NOT NULL,
    ts BIGINT NOT NULL,
    account_id TEXT
);
CREATE INDEX retention_archive_index_key ON retention_archive_index(store_id, row_key);
CREATE INDEX retention_archive_index_segment ON retention_archive_index(segment_id);
CREATE INDEX retention_archive_index_account ON retention_archive_index(account_id);
