-- PostgreSQL only: its 0124 makes the 14 foreign keys created by 0102-0113
-- DEFERRABLE, so migrate-store's SET CONSTRAINTS ALL DEFERRED covers them.
-- SQLite needs no change (the copy runs with deferred foreign keys there).
-- This file keeps the two migration directories numbered alike.
SELECT 1;
