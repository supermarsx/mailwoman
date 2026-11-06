-- Pseudonym keys for the per-person scrub, and the anchor records of
-- chain-preserving pseudonymisation of the audit log.
--
-- scrub_key holds one random 32-byte secret per generation, sealed with the
-- server key. Rotating adds a generation; retired generations stay until
-- destroy_after, after which the secret is deleted and pseudonyms made with it
-- can no longer be recomputed.
CREATE TABLE scrub_key (
    gen BIGINT PRIMARY KEY NOT NULL,
    sealed_key BLOB NOT NULL,
    created_at BIGINT NOT NULL,
    retired_at BIGINT,
    destroy_after BIGINT
);

-- A re-seal of the audit hash chain after pseudonymisation. old_head_hash is the
-- chain head before the re-seal and new_head_hash the head after it, so an
-- externally held copy of the old head can be mapped to the new one.
CREATE TABLE audit_rechain (
    id TEXT PRIMARY KEY NOT NULL,
    ts BIGINT NOT NULL,
    scrub_id TEXT NOT NULL,
    from_seq BIGINT NOT NULL,
    to_seq BIGINT NOT NULL,
    old_head_hash TEXT,
    new_head_hash TEXT,
    rows_changed BIGINT NOT NULL,
    algorithm TEXT NOT NULL,
    key_gen BIGINT NOT NULL
);
