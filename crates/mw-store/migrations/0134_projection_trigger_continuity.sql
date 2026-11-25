-- The 0125 projection triggers can no longer be replaced silently (review finding
-- F4): a stamp of their definitions (sqlite_master.sql). A trigger recreated with
-- another body changes the live stamp; Store::message_projection_installed then
-- answers false (the metadata planner refuses) until
-- Store::restamp_message_projection has audited both tables clean and stored the new
-- stamp. Limitation: a trigger dropped and recreated with the identical text leaves
-- the same stamp (SQLite keeps no row version for it); writes in between are only
-- found by the audit. A migration that recreates these triggers stores the new stamp
-- itself.
CREATE TABLE message_projection_stamp (
 id INTEGER NOT NULL PRIMARY KEY CHECK (id = 1),
 stamp TEXT NOT NULL
);
INSERT INTO message_projection_stamp(id,stamp)
SELECT 1, group_concat(name||':'||sql, ',') FROM (SELECT name, sql FROM sqlite_master
 WHERE type='trigger' AND tbl_name='messages'
 AND name IN ('message_projection_insert','message_projection_update','message_projection_delete') ORDER BY name);
