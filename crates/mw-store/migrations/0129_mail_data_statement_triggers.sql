-- PostgreSQL only: its 0129 replaces 0113's row-level messages triggers, whose
-- cost grew with the number of rows in one statement (every row updated the
-- account's shared mail_data_accounts/mail_data_outbox row, and those row
-- versions accumulate until the transaction ends). SQLite updates such a row in
-- place; measured flat (8,000 and 32,000 rows per statement), so its 0113
-- triggers stay. This file keeps the two migration directories numbered alike.
SELECT 1;
