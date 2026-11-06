-- Erasure tombstones and the backup register.
--
-- A tombstone is written for every erasure and every per-person scrub. It holds a
-- keyed hash of the subject (HMAC of the normalised address under the pseudonym
-- key of generation key_gen) and no raw identifier. A restore replays the
-- tombstones that are newer than the backup it restores, so a person erased after
-- the backup was taken is erased again.
--
-- seq is a gapless-enough, strictly increasing counter used as the watermark:
-- a backup records the highest seq at the time it was made.
CREATE TABLE erasure_tombstone (
    seq BIGINT PRIMARY KEY NOT NULL,
    id TEXT NOT NULL UNIQUE,
    subject_hash TEXT NOT NULL,
    key_gen BIGINT NOT NULL,
    scope TEXT NOT NULL,
    request_id TEXT,
    erased_at BIGINT NOT NULL
);
CREATE INDEX erasure_tombstone_subject ON erasure_tombstone(subject_hash);
CREATE INDEX erasure_tombstone_time ON erasure_tombstone(erased_at);

-- One row per backup the operator registers (label, where it lives, the
-- tombstone watermark at creation). manifest_json holds the full manifest.
CREATE TABLE backup_register (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    label TEXT NOT NULL,
    location TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    tombstone_watermark BIGINT NOT NULL,
    manifest_json TEXT NOT NULL
);
CREATE INDEX backup_register_time ON backup_register(created_at);
