-- t50hp-e26: mailboxes.total / mailboxes.unread count cached rows only (see
-- the SQLite migration). The server's STATUS figures move to their own
-- columns; the cached counters are recomputed once from the rows.
ALTER TABLE mailboxes ADD COLUMN upstream_total BIGINT NOT NULL DEFAULT 0;
ALTER TABLE mailboxes ADD COLUMN upstream_unseen BIGINT NOT NULL DEFAULT 0;
UPDATE mailboxes SET upstream_total = total, upstream_unseen = unread;
-- recount
-- Same rule as mw-store `flags_json_is_unread`: read iff the value is a JSON
-- array holding the string "Seen"; anything unparseable counts as unread.
-- The exception handler runs only for a value that is not JSON.
CREATE FUNCTION mw0122_flags_unread(flags TEXT) RETURNS BOOLEAN
LANGUAGE plpgsql IMMUTABLE AS $$
BEGIN
    RETURN NOT (jsonb_typeof(flags::jsonb) = 'array' AND flags::jsonb @> '["Seen"]'::jsonb);
EXCEPTION WHEN others THEN
    RETURN TRUE;
END
$$;
UPDATE mailboxes SET total = 0, unread = 0;
UPDATE mailboxes SET total = c.total, unread = c.unread
FROM (SELECT mailbox_id, COUNT(*) AS total,
             COUNT(*) FILTER (WHERE mw0122_flags_unread(flags_json)) AS unread
      FROM messages GROUP BY mailbox_id) c
WHERE c.mailbox_id = mailboxes.id;
DROP FUNCTION mw0122_flags_unread(TEXT);
