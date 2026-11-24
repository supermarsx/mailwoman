-- t49pg changes PostgreSQL only: its 0120 adds the xid column and the
-- change-log triggers that its 0118 dropped. SQLite keeps the 0115 triggers and
-- pages by seq (one writer at a time, so seq follows commit order). This file
-- keeps the two migration directories numbered alike.
SELECT 1;
