\set a (:client_id % 64) + 1
\set n random(1, 200)
BEGIN;
SELECT stable_id, account_id, mailbox_id FROM messages WHERE stable_id IN (SELECT value FROM json_array_elements_text(('["m' || :a || '-' || :n || '"]')::json));
SELECT pg_advisory_xact_lock(hashtextextended('acct' || :a, 719304));
SELECT id FROM accounts WHERE id = 'acct' || :a FOR KEY SHARE;
SELECT id FROM mailboxes WHERE account_id = 'acct' || :a AND id = 'mbox' || :a FOR UPDATE;
SELECT stable_id, account_id, mailbox_id, flags_json FROM messages WHERE stable_id IN (SELECT value FROM json_array_elements_text(('["m' || :a || '-' || :n || '"]')::json)) ORDER BY stable_id FOR UPDATE;
UPDATE messages SET flags_json = CASE WHEN flags_json = '[]' THEN '["Seen"]' ELSE '[]' END WHERE stable_id = 'm' || :a || '-' || :n;
UPDATE mailboxes SET unread = CASE WHEN unread - 1 < 0 THEN 0 ELSE unread - 1 END WHERE id = 'mbox' || :a;
COMMIT;
BEGIN;
SELECT pg_advisory_xact_lock(hashtextextended('acct' || :a, 719304));
INSERT INTO changes(account_id, type, state, stable_id, op, at) VALUES ('acct' || :a, 'Email', (SELECT COALESCE(MAX(state), 0) + 1 FROM changes WHERE account_id = 'acct' || :a AND type = 'Email'), 'm' || :a || '-' || :n, 'updated', '2026-11-24T00:00:00Z');
INSERT INTO changes(account_id, type, state, stable_id, op, at) VALUES ('acct' || :a, 'Mailbox', (SELECT COALESCE(MAX(state), 0) + 1 FROM changes WHERE account_id = 'acct' || :a AND type = 'Mailbox'), 'mbox' || :a, 'updated', '2026-11-24T00:00:00Z');
COMMIT;
