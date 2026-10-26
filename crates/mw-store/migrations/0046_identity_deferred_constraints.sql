-- PostgreSQL requires deferred identity foreign keys during migrate-store.
-- SQLite already defers foreign keys for the copy transaction; keep migration
-- numbering aligned without rewriting the applied identity schema.
SELECT 1;
