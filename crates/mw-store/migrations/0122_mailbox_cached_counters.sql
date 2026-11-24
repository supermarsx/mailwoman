-- t50hp-e26: mailboxes.total / mailboxes.unread count the rows cached in the
-- folder (unread = flags without the JSON string "Seen"), maintained by every
-- row write in its own transaction. A sync used to overwrite them with the
-- server's STATUS MESSAGES / UNSEEN and then apply the same upstream change
-- again as a row delta, so a read, unread or expunge made on the server was
-- counted twice, and a local ingest was not counted at all. The server's
-- figures move to their own columns; the cached counters are recomputed once.
ALTER TABLE mailboxes ADD COLUMN upstream_total INTEGER NOT NULL DEFAULT 0;
ALTER TABLE mailboxes ADD COLUMN upstream_unseen INTEGER NOT NULL DEFAULT 0;
UPDATE mailboxes SET upstream_total = total, upstream_unseen = unread;
-- recount
-- Same rule as mw-store `flags_json_is_unread`: read iff the value is a JSON
-- array holding the string "Seen"; anything unparseable counts as unread.
UPDATE mailboxes SET
    total = (SELECT COUNT(*) FROM messages m WHERE m.mailbox_id = mailboxes.id),
    unread = (SELECT COUNT(*) FROM messages m WHERE m.mailbox_id = mailboxes.id
        AND NOT (json_valid(m.flags_json) AND json_type(m.flags_json) = 'array'
            AND EXISTS (SELECT 1 FROM json_each(m.flags_json) e
                WHERE e.type = 'text' AND e.value = 'Seen')));
