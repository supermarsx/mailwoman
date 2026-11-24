-- t62-n16 changes PostgreSQL storage parameters only (fillfactor and per-table
-- autovacuum thresholds for the tables a flag change updates). SQLite has no
-- equivalent. This file keeps the two migration directories numbered alike.
SELECT 1;
