-- Postgres widens metrics_bucket.resolution_seconds to 64 bits in its own 0101; SQLite integers
-- are already 64-bit, so there is nothing to change here. The file keeps the two sequences aligned.
SELECT 1;
