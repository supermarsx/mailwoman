-- t34 review finding S2 changes PostgreSQL only (its 0118 drops the 0115
-- change-log triggers there). SQLite keeps them: the bounded guard snapshot
-- reads the log, and SQLite admits one writer at a time, so the per-insert
-- prune cannot make two writers wait on each other. This file keeps the two
-- migration directories numbered alike.
SELECT 1;
