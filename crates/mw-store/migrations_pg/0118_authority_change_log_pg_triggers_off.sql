-- t34 review finding S2. Nothing reads authority_change_log on PostgreSQL (the
-- bounded guard snapshot is SQLite-only), but the 0115 per-insert prune made
-- concurrent authority writes wait on each other's uncommitted delete of the
-- same oldest row once the log passed 65536 rows, and two transactions that
-- also bump each other's epochs then failed with 40P01. Drop the 0115 triggers
-- and functions so authority writes on PostgreSQL lock exactly as before 0115.
-- The empty table stays: both dialects keep the same table set, which complete
-- migration (migrate_complete_from_sqlite) requires. A future PostgreSQL reader
-- must add its own commit-ordered design and a pruning job that does not run per
-- insert. IF EXISTS: an operator may already have removed some of these.
DROP TRIGGER IF EXISTS authority_change_log_prune ON authority_change_log;
DROP TRIGGER IF EXISTS authority_change_log_revision ON settings_revision;
DROP TRIGGER IF EXISTS authority_change_log_epoch ON authority_epoch;
DROP FUNCTION IF EXISTS mw_authority_change_log_prune();
DROP FUNCTION IF EXISTS mw_authority_change_log_revision();
DROP FUNCTION IF EXISTS mw_authority_change_log_epoch();
TRUNCATE authority_change_log;
