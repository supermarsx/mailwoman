-- One bounded scheduler position, not per-run history. No scan/backfill of changes.
-- No account FK: deleted groups remain useful lexicographic seek boundaries.
CREATE TABLE change_log_maintenance_cursor (
 id BIGINT NOT NULL PRIMARY KEY CHECK (id=1),
 account_id TEXT NOT NULL,
 type TEXT NOT NULL,
 updated_at BIGINT NOT NULL
);
INSERT INTO change_log_maintenance_cursor(id,account_id,type,updated_at) VALUES(1,'','',0);
