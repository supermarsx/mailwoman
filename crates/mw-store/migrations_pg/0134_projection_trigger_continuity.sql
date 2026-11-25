-- The 0125 projection triggers can no longer be skipped silently (review finding F4).
-- 1. ENABLE ALWAYS: they also fire under session_replication_role=replica and in
--    logical-replication apply, which skip ordinary (origin) triggers with no
--    catalog change.
-- 2. A stamp of the triggers and their functions (pg_trigger oid/xmin/tgenabled,
--    pg_proc oid/xmin). Disabling and re-enabling a trigger, or replacing it or a
--    function, changes the live stamp; Store::message_projection_installed then
--    answers false (the metadata planner refuses) until
--    Store::restamp_message_projection has audited both tables clean and stored the
--    new stamp. A migration that recreates these triggers or functions stores the new
--    stamp itself, in its own transaction.
ALTER TABLE messages ENABLE ALWAYS TRIGGER message_projection_insert;
ALTER TABLE messages ENABLE ALWAYS TRIGGER message_projection_update;
ALTER TABLE messages ENABLE ALWAYS TRIGGER message_projection_delete;
CREATE TABLE message_projection_stamp (
 id BIGINT NOT NULL PRIMARY KEY CHECK (id = 1),
 stamp TEXT NOT NULL
);
INSERT INTO message_projection_stamp(id,stamp)
SELECT 1, string_agg(t.tgname||':'||t.oid::text||':'||t.xmin::text||':'||t.tgenabled::text||':'||p.oid::text||':'||p.xmin::text, ',' ORDER BY t.tgname)
 || ',' || (SELECT f.oid::text||':'||f.xmin::text FROM pg_catalog.pg_proc f JOIN pg_catalog.pg_namespace fn ON fn.oid=f.pronamespace WHERE fn.nspname=current_schema() AND f.proname='mw_message_flags')
FROM pg_catalog.pg_trigger t
JOIN pg_catalog.pg_class c ON c.oid=t.tgrelid
JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
JOIN pg_catalog.pg_proc p ON p.oid=t.tgfoid
WHERE n.nspname=current_schema() AND c.relname='messages' AND NOT t.tgisinternal
 AND t.tgname IN ('message_projection_insert','message_projection_update','message_projection_delete');
