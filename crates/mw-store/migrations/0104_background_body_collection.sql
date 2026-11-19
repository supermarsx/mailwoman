-- Two-pass body collection: legacy orphans receive a grace period on first observation.
CREATE INDEX IF NOT EXISTS messages_blob_ref_idx ON messages(blob_ref) WHERE blob_ref IS NOT NULL;
CREATE TABLE body_gc_candidate (
 blob_ref TEXT PRIMARY KEY REFERENCES bodies(blob_ref) ON DELETE CASCADE,
 first_seen BIGINT NOT NULL
);
CREATE INDEX body_gc_candidate_age_idx ON body_gc_candidate(first_seen,blob_ref);
