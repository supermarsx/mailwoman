-- SQLite cross-store copy already uses PRAGMA defer_foreign_keys for its transaction.
-- PostgreSQL's matching migration makes the three integration constraints deferrable.
SELECT 1;
