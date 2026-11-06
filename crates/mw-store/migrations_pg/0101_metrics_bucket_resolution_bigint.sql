-- The resolution column was created as INTEGER, which Postgres types as 32 bits while the store
-- reads every integer as 64 bits (migrate-store and the store tests failed to decode it).
-- Widen it; the CHECK and the primary key carry over.
ALTER TABLE metrics_bucket ALTER COLUMN resolution_seconds TYPE BIGINT;
