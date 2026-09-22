-- 0028 (26.20 t24-e6): one account row per mail identity — POSTGRES variant.
-- Behaviourally identical to the SQLite `migrations/0028_account_identity.sql`, which
-- carries the full reasoning and the per-table uniqueness inventory. ADDITIVE — NEVER
-- edit an earlier migration. (0024 remains retired.)
--
-- THE CONSTRAINT: a unique index on
--
--     (kind, rtrim(translate(host, 'A..Z', 'a..z'), '.'), port, translate(username, 'A..Z', 'a..z'))
--
-- the same expression `Store::account_id_by_identity` compares on for Postgres.
--
-- DIALECT DIFFERENCES, AND WHY
--   * Case folding. Postgres' `lower()` folds by the database locale (`lower('Ä')` is
--     `'ä'`); SQLite's folds only A-Z. `translate` with the 26 ASCII letters is the
--     Postgres spelling of SQLite's `lower()`, so both backends treat exactly the same
--     identities as equal. `translate` and `rtrim` are IMMUTABLE, as an index
--     expression requires.
--   * Ordering. Picking the survivor sorts on `created_at` and `id`, which are text;
--     `COLLATE "C"` makes that byte order, which is SQLite's default, so the same
--     survivor is chosen on both backends whatever the database collation.
--   * Audit rows. The id is `gen_random_uuid()` (core since Postgres 13) where SQLite
--     assembles the same shape from `randomblob`; the timestamp is RFC 3339 UTC from
--     `to_char`; detail_json is built with `json_build_object` rather than
--     `json_object`. The fields and their meaning are identical.
--
-- Survivor choice, the second-factor rule, the audit rows and what is not rewritten:
-- see the SQLite variant.
-- ── 1. Identity groups and survivors ─────────────────────────────────────────
-- A factor is what the login gate counts (a confirmed TOTP, a passkey) plus the
-- recovery codes issued with them.
CREATE TEMP TABLE mw0028_factor AS
    SELECT account_id, created_at FROM totp_secrets WHERE confirmed = 1
    UNION ALL SELECT account_id, created_at FROM webauthn_credentials
    UNION ALL SELECT account_id, created_at FROM recovery_codes;

CREATE TEMP TABLE mw0028_acct AS
SELECT id, grp, rn FROM (
    SELECT a.id,
           DENSE_RANK() OVER (ORDER BY a.kind, rtrim(translate(a.host, 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz'), '.'), a.port, translate(a.username, 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz')) AS grp,
           ROW_NUMBER() OVER (
               PARTITION BY a.kind, rtrim(translate(a.host, 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz'), '.'), a.port, translate(a.username, 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz')
               ORDER BY CASE WHEN f.first_at IS NULL THEN 1 ELSE 0 END,
                        f.first_at COLLATE "C", a.id COLLATE "C"
           ) AS rn
      FROM accounts a
      LEFT JOIN (SELECT account_id, MIN(created_at) AS first_at
                   FROM mw0028_factor GROUP BY account_id) f
        ON f.account_id = a.id
) ranked;
DELETE FROM mw0028_acct
 WHERE grp IN (SELECT grp FROM mw0028_acct GROUP BY grp HAVING COUNT(*) = 1);
CREATE INDEX mw0028_acct_id ON mw0028_acct (id);

CREATE TEMP TABLE mw0028_map AS
SELECT l.id AS loser, s.id AS survivor
  FROM mw0028_acct l, mw0028_acct s
 WHERE s.grp = l.grp AND s.rn = 1 AND l.rn > 1;
CREATE INDEX mw0028_map_loser ON mw0028_map (loser);

-- ── 2. Mailboxes: UNIQUE (account_id, name, uidvalidity) ─────────────────────
CREATE TEMP TABLE mw0028_mbox AS
SELECT d.id AS old_id, k.id AS new_id
  FROM mailboxes d, mw0028_acct rd, mailboxes k, mw0028_acct rk
 WHERE rd.id = d.account_id AND rk.id = k.account_id AND rk.grp = rd.grp
   AND k.name = d.name AND k.uidvalidity = d.uidvalidity AND k.id <> d.id
   AND rk.rn = (SELECT MIN(r2.rn) FROM mailboxes m2, mw0028_acct r2
                 WHERE r2.id = m2.account_id AND r2.grp = rd.grp
                   AND m2.name = d.name AND m2.uidvalidity = d.uidvalidity);
CREATE INDEX mw0028_mbox_old ON mw0028_mbox (old_id);
UPDATE messages SET mailbox_id = (SELECT new_id FROM mw0028_mbox WHERE old_id = messages.mailbox_id)
 WHERE mailbox_id IN (SELECT old_id FROM mw0028_mbox);
UPDATE sync_state SET mailbox_id = (SELECT new_id FROM mw0028_mbox WHERE old_id = sync_state.mailbox_id)
 WHERE mailbox_id IN (SELECT old_id FROM mw0028_mbox);
UPDATE mailboxes SET parent_id = (SELECT new_id FROM mw0028_mbox WHERE old_id = mailboxes.parent_id)
 WHERE parent_id IN (SELECT old_id FROM mw0028_mbox);
UPDATE identities SET sent_mailbox_id = (SELECT new_id FROM mw0028_mbox WHERE old_id = identities.sent_mailbox_id)
 WHERE sent_mailbox_id IN (SELECT old_id FROM mw0028_mbox);
UPDATE changes SET stable_id = (SELECT new_id FROM mw0028_mbox WHERE old_id = changes.stable_id)
 WHERE stable_id IN (SELECT old_id FROM mw0028_mbox)
   AND type = 'Mailbox';
DELETE FROM mailboxes WHERE id IN (SELECT old_id FROM mw0028_mbox);
UPDATE mailboxes SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = mailboxes.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);

-- ── 3. Threads: UNIQUE (account_id, root_message_id) ─────────────────────────
CREATE TEMP TABLE mw0028_thr AS
SELECT d.thread_id AS old_id, k.thread_id AS new_id
  FROM threads d, mw0028_acct rd, threads k, mw0028_acct rk
 WHERE rd.id = d.account_id AND rk.id = k.account_id AND rk.grp = rd.grp
   AND k.root_message_id = d.root_message_id AND k.thread_id <> d.thread_id
   AND rk.rn = (SELECT MIN(r2.rn) FROM threads t2, mw0028_acct r2
                 WHERE r2.id = t2.account_id AND r2.grp = rd.grp
                   AND t2.root_message_id = d.root_message_id);
CREATE INDEX mw0028_thr_old ON mw0028_thr (old_id);
UPDATE messages SET thread_id = (SELECT new_id FROM mw0028_thr WHERE old_id = messages.thread_id)
 WHERE thread_id IN (SELECT old_id FROM mw0028_thr);
UPDATE sender_controls SET thread_id = (SELECT new_id FROM mw0028_thr WHERE old_id = sender_controls.thread_id)
 WHERE thread_id IN (SELECT old_id FROM mw0028_thr);
UPDATE changes SET stable_id = (SELECT new_id FROM mw0028_thr WHERE old_id = changes.stable_id)
 WHERE stable_id IN (SELECT old_id FROM mw0028_thr)
   AND type = 'Thread';
DELETE FROM threads WHERE thread_id IN (SELECT old_id FROM mw0028_thr);
UPDATE threads SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = threads.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);

-- ── 4. Messages: UNIQUE (account_id, mailbox_id, uidvalidity, uid) ──────────
-- Mailbox ids are already merged, so two copies of one upstream message now share
-- (mailbox_id, uidvalidity, uid). The copy on the lowest-ranked account is kept.
CREATE TEMP TABLE mw0028_msg AS
SELECT d.stable_id AS old_id, k.stable_id AS new_id, d.blob_ref AS old_blob, rd.rn AS old_rn
  FROM messages d, mw0028_acct rd, messages k, mw0028_acct rk
 WHERE rd.id = d.account_id AND rk.id = k.account_id AND rk.grp = rd.grp
   AND k.mailbox_id = d.mailbox_id AND k.uidvalidity = d.uidvalidity AND k.uid = d.uid
   AND k.stable_id <> d.stable_id
   AND rk.rn = (SELECT MIN(r2.rn) FROM messages m2, mw0028_acct r2
                 WHERE r2.id = m2.account_id AND r2.grp = rd.grp
                   AND m2.mailbox_id = d.mailbox_id AND m2.uidvalidity = d.uidvalidity
                   AND m2.uid = d.uid);
CREATE INDEX mw0028_msg_old ON mw0028_msg (old_id);
CREATE INDEX mw0028_msg_new ON mw0028_msg (new_id);
CREATE INDEX mw0028_msg_blob ON mw0028_msg (old_blob);
UPDATE message_meta SET stable_id = (SELECT new_id FROM mw0028_msg WHERE old_id = message_meta.stable_id)
 WHERE stable_id IN (SELECT old_id FROM mw0028_msg)
   AND NOT EXISTS (SELECT 1 FROM message_meta k, mw0028_msg m
                    WHERE m.old_id = message_meta.stable_id AND k.stable_id = m.new_id)
   AND NOT EXISTS (SELECT 1 FROM message_meta p, mw0028_msg m, mw0028_msg pm
                    WHERE m.old_id = message_meta.stable_id AND pm.new_id = m.new_id
                      AND pm.old_id = p.stable_id AND pm.old_rn < m.old_rn);
UPDATE message_embeddings SET stable_id = (SELECT new_id FROM mw0028_msg WHERE old_id = message_embeddings.stable_id)
 WHERE stable_id IN (SELECT old_id FROM mw0028_msg)
   AND NOT EXISTS (SELECT 1 FROM message_embeddings k, mw0028_msg m
                    WHERE m.old_id = message_embeddings.stable_id AND k.stable_id = m.new_id)
   AND NOT EXISTS (SELECT 1 FROM message_embeddings p, mw0028_msg m, mw0028_msg pm
                    WHERE m.old_id = message_embeddings.stable_id AND pm.new_id = m.new_id
                      AND pm.old_id = p.stable_id AND pm.old_rn < m.old_rn);
UPDATE security_verdicts SET email_id = (SELECT new_id FROM mw0028_msg WHERE old_id = security_verdicts.email_id)
 WHERE email_id IN (SELECT old_id FROM mw0028_msg)
   AND NOT EXISTS (SELECT 1 FROM security_verdicts k, mw0028_msg m
                    WHERE m.old_id = security_verdicts.email_id AND k.email_id = m.new_id)
   AND NOT EXISTS (SELECT 1 FROM security_verdicts p, mw0028_msg m, mw0028_msg pm
                    WHERE m.old_id = security_verdicts.email_id AND pm.new_id = m.new_id
                      AND pm.old_id = p.email_id AND pm.old_rn < m.old_rn);
DELETE FROM message_embeddings WHERE stable_id IN (SELECT old_id FROM mw0028_msg);
DELETE FROM security_verdicts WHERE email_id IN (SELECT old_id FROM mw0028_msg);
UPDATE pop3_uidl SET stable_id = (SELECT new_id FROM mw0028_msg WHERE old_id = pop3_uidl.stable_id)
 WHERE stable_id IN (SELECT old_id FROM mw0028_msg);
UPDATE submissions SET email_id = (SELECT new_id FROM mw0028_msg WHERE old_id = submissions.email_id)
 WHERE email_id IN (SELECT old_id FROM mw0028_msg);
UPDATE changes SET stable_id = (SELECT new_id FROM mw0028_msg WHERE old_id = changes.stable_id)
 WHERE stable_id IN (SELECT old_id FROM mw0028_msg)
   AND type = 'Email';
UPDATE remote_image_grants SET scope_value = (SELECT new_id FROM mw0028_msg WHERE old_id = remote_image_grants.scope_value)
 WHERE scope_value IN (SELECT old_id FROM mw0028_msg)
   AND scope_kind = 'single';
-- A kept copy with no cached body inherits the dropped copy's body.
UPDATE messages SET blob_ref = (SELECT m.old_blob FROM mw0028_msg m
                                 WHERE m.new_id = messages.stable_id AND m.old_blob IS NOT NULL
                                 ORDER BY m.old_rn LIMIT 1)
 WHERE blob_ref IS NULL
   AND stable_id IN (SELECT new_id FROM mw0028_msg WHERE old_blob IS NOT NULL);
DELETE FROM messages WHERE stable_id IN (SELECT old_id FROM mw0028_msg);
-- A dropped copy's body that no message references any more is that copy's body.
DELETE FROM bodies
 WHERE blob_ref IN (SELECT old_blob FROM mw0028_msg)
   AND blob_ref NOT IN (SELECT blob_ref FROM messages
                         WHERE blob_ref IN (SELECT old_blob FROM mw0028_msg));
UPDATE messages SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = messages.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);

-- ── 5. Second factors: only the survivor's own are kept ─────────────────────
-- A duplicate's TOTP secret, passkeys and recovery codes are never moved onto the
-- survivor. Each dropped factor is first recorded in audit_log, content-free: the
-- factor type, both account ids, when it was created, and for a passkey its public
-- credential id. No secret, key bytes or code hash is copied. Recovery codes are
-- recorded as one row per account (they are issued, and used, as one set).
INSERT INTO audit_log (id, ts, actor, actor_kind, action, target, detail_json, ip)
SELECT gen_random_uuid()::text, to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"+00:00"'), 'migration-0028', 'system',
       'twofa-factor-dropped-on-merge', m.survivor,
       json_build_object('factor', 'totp', 'confirmed', (t.confirmed = 1), 'duplicateAccountId', m.loser, 'survivorAccountId', m.survivor, 'createdAt', t.created_at)::text,
       NULL
  FROM totp_secrets t, mw0028_map m
 WHERE t.account_id = m.loser;
INSERT INTO audit_log (id, ts, actor, actor_kind, action, target, detail_json, ip)
SELECT gen_random_uuid()::text, to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"+00:00"'), 'migration-0028', 'system',
       'twofa-factor-dropped-on-merge', m.survivor,
       json_build_object('factor', 'passkey', 'credentialId', c.credential_id, 'duplicateAccountId', m.loser, 'survivorAccountId', m.survivor, 'createdAt', c.created_at)::text,
       NULL
  FROM webauthn_credentials c, mw0028_map m
 WHERE c.account_id = m.loser;
INSERT INTO audit_log (id, ts, actor, actor_kind, action, target, detail_json, ip)
SELECT gen_random_uuid()::text, to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"+00:00"'), 'migration-0028', 'system',
       'twofa-factor-dropped-on-merge', m.survivor,
       json_build_object('factor', 'recovery-codes', 'count', COUNT(*), 'unused', SUM(CASE WHEN r.used = 0 THEN 1 ELSE 0 END), 'duplicateAccountId', m.loser, 'survivorAccountId', m.survivor, 'createdAt', MIN(r.created_at))::text,
       NULL
  FROM recovery_codes r, mw0028_map m
 WHERE r.account_id = m.loser
 GROUP BY m.loser, m.survivor;
DELETE FROM totp_secrets WHERE account_id IN (SELECT loser FROM mw0028_map);
DELETE FROM webauthn_credentials WHERE account_id IN (SELECT loser FROM mw0028_map);
DELETE FROM recovery_codes WHERE account_id IN (SELECT loser FROM mw0028_map);

-- ── 6. Every other account-keyed table ───────────────────────────────────────
DELETE FROM pop3_uidl
 WHERE account_id IN (SELECT loser FROM mw0028_map)
   AND EXISTS (SELECT 1 FROM pop3_uidl o, mw0028_acct ro, mw0028_acct rt
              WHERE ro.id = o.account_id AND rt.id = pop3_uidl.account_id
                AND ro.grp = rt.grp AND ro.rn < rt.rn
                AND o.uidl = pop3_uidl.uidl);
UPDATE pop3_uidl SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = pop3_uidl.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
DELETE FROM sync_state
 WHERE account_id IN (SELECT loser FROM mw0028_map)
   AND EXISTS (SELECT 1 FROM sync_state o, mw0028_acct ro, mw0028_acct rt
              WHERE ro.id = o.account_id AND rt.id = sync_state.account_id
                AND ro.grp = rt.grp AND ro.rn < rt.rn
                AND o.mailbox_id = sync_state.mailbox_id);
UPDATE sync_state SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = sync_state.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
DELETE FROM quotas
 WHERE account_id IN (SELECT loser FROM mw0028_map)
   AND EXISTS (SELECT 1 FROM quotas o, mw0028_acct ro, mw0028_acct rt
              WHERE ro.id = o.account_id AND rt.id = quotas.account_id
                AND ro.grp = rt.grp AND ro.rn < rt.rn);
UPDATE quotas SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = quotas.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
DELETE FROM zeroaccess_accounts
 WHERE account_id IN (SELECT loser FROM mw0028_map)
   AND EXISTS (SELECT 1 FROM zeroaccess_accounts o, mw0028_acct ro, mw0028_acct rt
              WHERE ro.id = o.account_id AND rt.id = zeroaccess_accounts.account_id
                AND ro.grp = rt.grp AND ro.rn < rt.rn);
UPDATE zeroaccess_accounts SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = zeroaccess_accounts.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
DELETE FROM passwd_config
 WHERE account_id IN (SELECT loser FROM mw0028_map)
   AND EXISTS (SELECT 1 FROM passwd_config o, mw0028_acct ro, mw0028_acct rt
              WHERE ro.id = o.account_id AND rt.id = passwd_config.account_id
                AND ro.grp = rt.grp AND ro.rn < rt.rn);
UPDATE passwd_config SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = passwd_config.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
DELETE FROM bridge_accounts
 WHERE account_id IN (SELECT loser FROM mw0028_map)
   AND EXISTS (SELECT 1 FROM bridge_accounts o, mw0028_acct ro, mw0028_acct rt
              WHERE ro.id = o.account_id AND rt.id = bridge_accounts.account_id
                AND ro.grp = rt.grp AND ro.rn < rt.rn);
UPDATE bridge_accounts SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = bridge_accounts.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
DELETE FROM ews_account_cred
 WHERE account_id IN (SELECT loser FROM mw0028_map)
   AND EXISTS (SELECT 1 FROM ews_account_cred o, mw0028_acct ro, mw0028_acct rt
              WHERE ro.id = o.account_id AND rt.id = ews_account_cred.account_id
                AND ro.grp = rt.grp AND ro.rn < rt.rn);
UPDATE ews_account_cred SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = ews_account_cred.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
DELETE FROM notification_rules
 WHERE account_id IN (SELECT loser FROM mw0028_map)
   AND EXISTS (SELECT 1 FROM notification_rules o, mw0028_acct ro, mw0028_acct rt
              WHERE ro.id = o.account_id AND rt.id = notification_rules.account_id
                AND ro.grp = rt.grp AND ro.rn < rt.rn);
UPDATE notification_rules SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = notification_rules.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
DELETE FROM bridge_oauth_tokens
 WHERE bridge_account_id IN (SELECT loser FROM mw0028_map)
   AND EXISTS (SELECT 1 FROM bridge_oauth_tokens o, mw0028_acct ro, mw0028_acct rt
              WHERE ro.id = o.bridge_account_id AND rt.id = bridge_oauth_tokens.bridge_account_id
                AND ro.grp = rt.grp AND ro.rn < rt.rn);
UPDATE bridge_oauth_tokens SET bridge_account_id = (SELECT survivor FROM mw0028_map WHERE loser = bridge_oauth_tokens.bridge_account_id)
 WHERE bridge_account_id IN (SELECT loser FROM mw0028_map);
DELETE FROM plugin_grants
 WHERE account_id IN (SELECT loser FROM mw0028_map)
   AND EXISTS (SELECT 1 FROM plugin_grants o, mw0028_acct ro, mw0028_acct rt
              WHERE ro.id = o.account_id AND rt.id = plugin_grants.account_id
                AND ro.grp = rt.grp AND ro.rn < rt.rn
                AND o.plugin_id = plugin_grants.plugin_id
                AND o.capability = plugin_grants.capability);
UPDATE plugin_grants SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = plugin_grants.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
DELETE FROM plugin_kv
 WHERE account_id IN (SELECT loser FROM mw0028_map)
   AND EXISTS (SELECT 1 FROM plugin_kv o, mw0028_acct ro, mw0028_acct rt
              WHERE ro.id = o.account_id AND rt.id = plugin_kv.account_id
                AND ro.grp = rt.grp AND ro.rn < rt.rn
                AND o.plugin_id = plugin_kv.plugin_id
                AND o.key = plugin_kv.key);
UPDATE plugin_kv SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = plugin_kv.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
DELETE FROM remote_image_grants
 WHERE account_id IN (SELECT loser FROM mw0028_map)
   AND EXISTS (SELECT 1 FROM remote_image_grants o, mw0028_acct ro, mw0028_acct rt
              WHERE ro.id = o.account_id AND rt.id = remote_image_grants.account_id
                AND ro.grp = rt.grp AND ro.rn < rt.rn
                AND o.scope_kind = remote_image_grants.scope_kind
                AND o.scope_value = remote_image_grants.scope_value);
UPDATE remote_image_grants SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = remote_image_grants.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
DELETE FROM signatures
 WHERE account_id IN (SELECT loser FROM mw0028_map)
   AND EXISTS (SELECT 1 FROM signatures o, mw0028_acct ro, mw0028_acct rt
              WHERE ro.id = o.account_id AND rt.id = signatures.account_id
                AND ro.grp = rt.grp AND ro.rn < rt.rn
                AND o.name = signatures.name);
UPDATE signatures SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = signatures.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE sessions SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = sessions.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE bodies SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = bodies.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE submissions SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = submissions.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE identities SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = identities.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE changes SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = changes.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE calendars SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = calendars.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE notebooks SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = notebooks.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE notes SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = notes.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE address_books SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = address_books.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE pim_changes SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = pim_changes.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE crypto_keys SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = crypto_keys.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE key_associations SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = key_associations.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE security_verdicts SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = security_verdicts.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE dlp_audit SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = dlp_audit.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE sender_controls SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = sender_controls.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE crypto_changes SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = crypto_changes.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE push_subscriptions SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = push_subscriptions.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE native_sessions SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = native_sessions.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE api_keys SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = api_keys.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE oauth_tokens SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = oauth_tokens.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE webhooks SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = webhooks.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE password_change_audit SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = password_change_audit.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE masked_email SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = masked_email.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE uploaded_blobs SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = uploaded_blobs.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE message_embeddings SET account_id = (SELECT survivor FROM mw0028_map WHERE loser = message_embeddings.account_id)
 WHERE account_id IN (SELECT loser FROM mw0028_map);
UPDATE tags SET "user" = (SELECT survivor FROM mw0028_map WHERE loser = tags."user")
 WHERE "user" IN (SELECT loser FROM mw0028_map);
UPDATE saved_searches SET "user" = (SELECT survivor FROM mw0028_map WHERE loser = saved_searches."user")
 WHERE "user" IN (SELECT loser FROM mw0028_map);
-- assist_config is keyed by the scope string 'user:<account id>'.
DELETE FROM assist_config
 WHERE scope IN (SELECT 'user:' || loser FROM mw0028_map)
   AND EXISTS (SELECT 1 FROM assist_config o, mw0028_acct ro, mw0028_acct rt
                WHERE o.scope = 'user:' || ro.id AND assist_config.scope = 'user:' || rt.id
                  AND ro.grp = rt.grp AND ro.rn < rt.rn);
UPDATE assist_config
   SET scope = (SELECT 'user:' || survivor FROM mw0028_map WHERE 'user:' || loser = assist_config.scope)
 WHERE scope IN (SELECT 'user:' || loser FROM mw0028_map);

-- ── 7. Drop the emptied duplicates, then constrain ────────────────────────────
DELETE FROM accounts WHERE id IN (SELECT loser FROM mw0028_map);

CREATE UNIQUE INDEX IF NOT EXISTS idx_accounts_identity
    ON accounts (kind, (rtrim(translate(host, 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz'), '.')), port, (translate(username, 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz')));

DROP TABLE mw0028_msg;
DROP TABLE mw0028_thr;
DROP TABLE mw0028_mbox;
DROP TABLE mw0028_map;
DROP TABLE mw0028_acct;
DROP TABLE mw0028_factor;
