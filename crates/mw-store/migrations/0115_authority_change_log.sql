-- t34 bounded guard snapshot. A broadcast, append-only record of every
-- authority_epoch change and every settings_revision change, readable by any
-- number of processes (the 0111 outbox is claim-based and has one consumer).
-- A row carries the epoch incarnation, never the scope identifier: an epoch row
-- keeps one incarnation for its whole life (authority_epoch_monotonic), so
-- (scope_kind, incarnation) names the scope a cached fence observed, while
-- account ids, subjects and session ids stay out of this table.
-- AUTOINCREMENT never reuses a seq. SQLite admits one writer at a time, so seq
-- order is commit order. No backfill. The insert trigger keeps the newest
-- 65536 seq values; a reader whose cursor is older than max(seq)-65536 has
-- missed rows and must treat every cached observation as stale.
CREATE TABLE authority_change_log (
 seq INTEGER PRIMARY KEY AUTOINCREMENT,
 scope_kind TEXT NOT NULL,
 incarnation TEXT NOT NULL,
 at BIGINT NOT NULL
);
CREATE TRIGGER authority_change_log_epoch_insert AFTER INSERT ON authority_epoch BEGIN
 INSERT INTO authority_change_log(scope_kind,incarnation,at) VALUES(NEW.scope_kind,NEW.incarnation,CAST((julianday('now')-2440587.5)*86400000.0 AS INTEGER));
END;
CREATE TRIGGER authority_change_log_epoch_update AFTER UPDATE ON authority_epoch BEGIN
 INSERT INTO authority_change_log(scope_kind,incarnation,at) VALUES(OLD.scope_kind,OLD.incarnation,CAST((julianday('now')-2440587.5)*86400000.0 AS INTEGER));
END;
CREATE TRIGGER authority_change_log_epoch_delete AFTER DELETE ON authority_epoch BEGIN
 INSERT INTO authority_change_log(scope_kind,incarnation,at) VALUES(OLD.scope_kind,OLD.incarnation,CAST((julianday('now')-2440587.5)*86400000.0 AS INTEGER));
END;
CREATE TRIGGER authority_change_log_revision_insert AFTER INSERT ON settings_revision BEGIN
 INSERT INTO authority_change_log(scope_kind,incarnation,at) VALUES('settings-revision','',CAST((julianday('now')-2440587.5)*86400000.0 AS INTEGER));
END;
CREATE TRIGGER authority_change_log_revision_update AFTER UPDATE ON settings_revision BEGIN
 INSERT INTO authority_change_log(scope_kind,incarnation,at) VALUES('settings-revision','',CAST((julianday('now')-2440587.5)*86400000.0 AS INTEGER));
END;
CREATE TRIGGER authority_change_log_revision_delete AFTER DELETE ON settings_revision BEGIN
 INSERT INTO authority_change_log(scope_kind,incarnation,at) VALUES('settings-revision','',CAST((julianday('now')-2440587.5)*86400000.0 AS INTEGER));
END;
CREATE TRIGGER authority_change_log_prune AFTER INSERT ON authority_change_log BEGIN
 DELETE FROM authority_change_log WHERE seq<=NEW.seq-65536;
END;
