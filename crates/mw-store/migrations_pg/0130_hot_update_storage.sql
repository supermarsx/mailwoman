-- t62-n16: storage parameters for the tables every message flag change
-- updates. A PostgreSQL UPDATE is HOT (heap-only: no index entry written, old
-- version pruned on the page) only when no indexed column changes AND the new
-- row version fits on the same heap page. None of these updates changes an
-- indexed column; with the default fillfactor of 100 a bulk-loaded or
-- vacuumed page is full, so the next update of each row on it writes a new
-- index entry in every index of the table instead.
--
-- fillfactor applies to pages written from now on (inserts, and pages that
-- VACUUM FULL / CLUSTER / pg_repack rewrite); existing pages keep their
-- current fill. Nothing here rewrites a table. ALTER TABLE ... SET for these
-- parameters takes SHARE UPDATE EXCLUSIVE: reads and writes continue.
--
-- The 0125 flag-state table is not listed: its partial indexes test seen and
-- flagged, so a flag change is never HOT there, whatever the fillfactor.
--
-- Rows: messages ~0.8 KB -> 85 leaves room for one new version per page of
-- about nine rows; mail_data_rows one small row per message, rewritten on
-- every change; mail_data_outbox / mail_data_accounts one row per account,
-- rewritten up to twice per changed message; mailboxes one row per folder,
-- its counters rewritten by every flag change that alters unread.
ALTER TABLE messages SET (fillfactor = 85, autovacuum_vacuum_scale_factor = 0.02, autovacuum_analyze_scale_factor = 0.02);
ALTER TABLE mail_data_rows SET (fillfactor = 85, autovacuum_vacuum_scale_factor = 0.02);
ALTER TABLE mail_data_outbox SET (fillfactor = 50, autovacuum_vacuum_scale_factor = 0.0, autovacuum_vacuum_threshold = 1000);
ALTER TABLE mail_data_accounts SET (fillfactor = 50, autovacuum_vacuum_scale_factor = 0.0, autovacuum_vacuum_threshold = 1000);
ALTER TABLE mailboxes SET (fillfactor = 70, autovacuum_vacuum_scale_factor = 0.0, autovacuum_vacuum_threshold = 1000);
