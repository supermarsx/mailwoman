-- PostgreSQL only: its 0132 gives two row triggers column lists so a flag change
-- does not call them. The SQLite triggers already carry WHEN conditions. This
-- file keeps the two migration directories numbered alike.
SELECT 1;
