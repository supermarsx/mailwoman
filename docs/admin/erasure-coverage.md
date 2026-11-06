# Erasure and export coverage

This page lists where the server keeps data about one person, what an erasure
does with each table, which export section carries it, and what the per-person
scrub and the backup markings do. It describes controls. It does not state that
a deployment meets any legal requirement.

## One registry

`crates/mw-store/src/data_map.rs` holds a single registry. Each entry names a
table and says how its rows are found for one person, what happens at erasure
(deleted, removed with a parent row, or kept with a reason), which export
section carries it, which retention control limits it and which columns the
scrub replaces. The same registry drives:

- the erasure statements,
- the verify pass after an erasure, which counts the rows each entry still finds
  (a table that is only counted, because its rows go with a parent row, is
  checked too),
- the table-driven export sections.

A test (`crates/mw-store/tests/data_map_coverage.rs`) migrates an empty database,
lists every table and column and fails when a table with a per-person column
(an account, owner, user, subject, address, sender, recipient or principal
column, or a foreign key to `accounts`) has no entry. A new migration therefore
cannot add personal data that erasure and export skip. The table below is
generated from the registry and a second test fails when it is out of date
(`BLESS=1 cargo test -p mw-store --test data_map_coverage` regenerates it).

## Export sections

An export holds these sections. Rows come from the registry; sealed secrets
(authenticator secrets, recovery codes, stored passwords, sealed tokens, key
material) appear as a marker saying the item is present and is not exported.

| Section | Holds |
|---|---|
| `profile`, `settings`, `identities`, `contacts`, `calendars`, `notes`, `rules` | the directory row, user settings, identities, address books, calendars, notes, mail rules |
| `mailstate` | pins and snoozes, tags, saved searches, trusted senders, read-receipt rules and status, remote-image grants, masked addresses, queued submissions |
| `tasks` | tasks |
| `security` | key metadata and associations, API key prefixes, passkey labels, OAuth token metadata, authorization consents, password change history, markers for sealed secrets |
| `integrations` | external mail and DAV connections, linked mailboxes, bridges, webhooks, markers for sealed credentials |
| `delegation` | delegations granted and received, shared mailboxes, group membership |
| `usage` | usage counters, storage measurements, quotas, last activity |
| `records` | consent ledger, restrictions, retention rules, tracking pixels and opens, uploads, plug-in data keys, DLP decisions |
| `sessions`, `consent`, `audit`, `requests`, `mail` | sessions and sign-in history, cookie consent, audit entries about the person, data requests, mail |

## What stays after an erasure

The completion record lists these. Each is an entry marked "kept" in the table
below with its reason:

- the directory row, closed (status deleted) so the address cannot be enrolled
  again without an administrator;
- audit history, for the audit retention period (see the scrub below);
- the request and its trail, for the request retention period;
- an opaque identity principal id with no identifier once its bindings are gone;
- mail held by the upstream mail server, which this server does not delete;
- sealed archive segments written by retention archiving; the person's index
  entries are removed and the segments go when they expire.

## Scrubbing one person from history

The Scrub person action (Admin console, Erasure and scrubbing) and the
DSR-driven scrub replace the person's identifiers in audit and log history with
a stable keyed pseudonym. A dry run reports counts without changing anything.

- The pseudonym is `HMAC-SHA256(key, "person" || address)` cut to 16 or 32 hex
  characters (`scrub.pseudonymAlgorithm`), prefixed with its key generation. The
  same person gets the same pseudonym under one generation.
- Keys are random per generation and stored sealed with the server key. Rotating
  adds a generation. A retired key is deleted `scrub.keyRetentionDays` after
  rotation (0 keeps it). After deletion, old pseudonyms can no longer be
  recomputed.
- Scrubbed: `audit_log`, `sso_login_audit` (the stored hash of the IdP subject),
  `alert_event`, `assist_audit`, `signin_event` (attempted user name of failed
  attempts), `breach_event`, `breach_delivery`, and the rotated log files
  `mailwoman*.log` and `mailwoman*.log.gz` when `scrub.includeLogs` is on.
- `dlp_audit` rows belong to the account and are deleted with it; they cannot be
  re-keyed because of the foreign key.
- `request_protection_*` rows are keyed by a one-way hash of a network source,
  not by person, and are bounded by their own retention.
- The audit log is hash chained. Rewriting a row changes its hash and the hash of
  every row after it, so the scrub re-seals the chain from the first changed row
  and writes an anchor record (`audit_rechain`) with the old head hash, the new
  head hash, the range and the number of rows changed. Chain verification passes
  after a scrub. A copy of the old head that was stored outside the database
  (for example by an external audit sink) does not match the new head; the
  anchor maps one to the other. Rows written by the server that have no chain
  position yet are sealed first.
- With `scrub.autoRunOnErasure` a completed erasure runs the scrub for the same
  person.

The same operation is available offline for log files:

```sh
mailwoman erasure scrub-logs --dir /var/log/mailwoman --address person@example.org --dry-run
mailwoman erasure scrub-logs --dir /var/log/mailwoman --address person@example.org
```

## Backups and erasure

A backup is a copy of the database as it was. Erasing a person afterwards does
not change the copy, so a restore would bring the person back. Three things deal
with that:

1. **Tombstones.** Every erasure and every scrub writes a tombstone to
   `erasure_tombstone`: a keyed hash of the address, the key generation, the
   scope (`erasure` or `scrub`), the request id and the time. No raw identifier
   is stored. Tombstones are kept `backup.tombstoneRetentionDays` days (3650 by
   default, 0 keeps them forever).
2. **Watermark.** A backup records the highest tombstone sequence at the time it
   was made, in its manifest (`tombstoneWatermark`).
3. **Replay.** With `backup.propagateErasure` on (the default), a restore replays
   the tombstones newer than the backup's watermark: it finds every person in the
   restored data whose address hashes to a tombstone and erases them again with
   the same registry.

The admin view lists registered backups with a "contains erased subjects" marker
and the number of people erased or scrubbed since the backup was made.

SQLite stores are backed up and restored with the command line:

```sh
# back up (consistent copy plus <file>.manifest.json)
mailwoman erasure backup --out /backups/2026-11-05.sqlite --label nightly

# restore to a new file and replay erasures made since, using the live database
mailwoman erasure restore --backup /backups/2026-11-05.sqlite \
    --to /data/restored.sqlite --live-db /data/mailwoman.db

# or hand the tombstones over as a file (for example from another host)
mailwoman erasure export-tombstones --since 41 --out tombstones.json
mailwoman erasure restore --backup /backups/2026-11-05.sqlite \
    --to /data/restored.sqlite --tombstones tombstones.json
```

`MW_SERVER_KEY` must be the key the databases were written with. Replay needs the
pseudonym key generations the tombstones were made with; they are carried with
the hand-over file and with `--live-db`. A key deleted by
`scrub.keyRetentionDays` makes its tombstones unusable, and the restore reports
them as errors.

Postgres stores and volume snapshots made with `deploy.sh backup` are not written
by this command. Register them in the admin view with a label, a location and the
tombstone watermark at the time of the backup (the current one is the default) so
the marker works. Replay after a restore of such a backup is a manual step: run
`erasure export-tombstones` on the live database and replay on the restored
copy.

Limits: a backup older than the tombstone retention can contain people whose
tombstone no longer exists; backups held outside this server (off-site copies)
are not touched; the replay erases by registry and does not reach the search
index of a restored install, which is rebuilt from the restored cache.

## Registry

<!-- registry:start -->
| Table | Found by | At erasure | In the export | Limited by or scrubbed |
|---|---|---|---|---|
| `account_ban` | - | kept: abuse-prevention record keyed by a value (address or network); expires on its own and is lifted by an administrator | - | - |
| `account_draft` | `account_id=id` | deleted | `mailstate` (marker: present, not exported) | - |
| `account_draft` | `owner=id` | deleted | - | - |
| `account_oauth` | `account_id=id` | deleted | `integrations` (marker: present, not exported) | - |
| `account_provider` | `account_id=id` | deleted | `integrations` | - |
| `account_restriction` | `account_id=id` | deleted | `records` | - |
| `accounts` | `id=id` | deleted | - | - |
| `address_books` | `account_id=id` | deleted | `contacts` (built by code) | - |
| `admin_account` | - | kept: operator account of the admin console; removed with the administrator account, not by a mail user's erasure | - | - |
| `admin_account_role` | - | kept: operator account of the admin console; removed with the administrator account, not by a mail user's erasure | - | - |
| `admin_capability_override` | - | kept: operator account of the admin console; removed with the administrator account, not by a mail user's erasure | - | - |
| `admin_recovery_codes` | - | kept: operator account of the admin console; removed with the administrator account, not by a mail user's erasure | - | - |
| `admin_saved_query` | - | kept: operator account of the admin console; removed with the administrator account, not by a mail user's erasure | - | - |
| `admin_sessions` | - | kept: operator account of the admin console; removed with the administrator account, not by a mail user's erasure | - | - |
| `admin_totp` | - | kept: operator account of the admin console; removed with the administrator account, not by a mail user's erasure | - | - |
| `admin_users` | - | kept: operator account of the admin console; removed with the administrator account, not by a mail user's erasure | - | - |
| `admin_webauthn_credentials` | - | kept: operator account of the admin console; removed with the administrator account, not by a mail user's erasure | - | - |
| `alert_event` | - | kept: alert history kept for the audit retention period; identifiers in the summary are pseudonymised by the scrub | - | retention: audit.retention; scrub: summary (identifiers inside the summary are replaced) |
| `api_keys` | `account_id=id` | deleted | `security` | - |
| `assist_audit` | - | kept: capability audit kept for the audit retention period; the actor is pseudonymised by the scrub | - | retention: audit.retention; scrub: actor (the actor is replaced with the keyed pseudonym) |
| `audit_log` | - | kept: security and administrative audit trail kept for the audit retention period; identifiers are pseudonymised by the scrub | `audit` (built by code) | retention: audit.retention; scrub: actor,target,detail_json,ip (chain-preserving: rows are re-sealed and an audit_rechain anchor is written) |
| `audit_rechain` | - | kept: anchor records of chain-preserving pseudonymisation; hold hashes only | - | - |
| `auth_failure` | `key_kind='account' AND key=id` | deleted | - | retention: window expiry |
| `auth_source_failure` | - | kept: keyed by source address and surface, not by person; expires on its own (expires_at) | - | - |
| `backup_register` | - | kept: backup manifests: label, time and tombstone watermark, no identifiers | - | - |
| `bodies` | `account_id=id` | removed with its parent row | `mail` (built by code) | - |
| `brand_asset` | `owner_account=id` | deleted | - | - |
| `breach_delivery` | - | kept: delivery log of breach notices kept as part of the breach documentation; the recipient is pseudonymised by the scrub | - | retention: breach.retention; scrub: recipient,detail (the notified address is replaced by the keyed pseudonym) |
| `breach_event` | - | kept: trail of a breach incident kept as part of the breach documentation; identifiers are pseudonymised by the scrub | - | retention: breach.retention; scrub: actor,detail_json (identifiers in the trail are replaced by the keyed pseudonym) |
| `breach_incident` | - | kept: breach documentation kept for the regulatory record; holds no per-person key | - | - |
| `bridge_accounts` | `account_id=id` | deleted | `integrations` (marker: present, not exported) | - |
| `bridge_oauth_tokens` | `bridge_account_id IN (SELECT bridge_id FROM bridge_accounts WHERE account_id=id) OR bridge_account_id=id` | deleted | `integrations` (marker: present, not exported) | - |
| `calendar_shares` | `calendar_id IN (SELECT id FROM calendars WHERE account_id=id)` | removed with its parent row | - | - |
| `calendars` | `account_id=id` | deleted | `calendars` (built by code) | - |
| `changes` | `account_id=id` | deleted | - | - |
| `child_account` | `owner_account_id=id` | deleted | `integrations` | - |
| `client_error_event` | - | kept: pseudonymous diagnostics: scrubbed stack frames and a coarse browser, OS and viewport class per occurrence; account_hash is a salted one-way hash, empty unless the administrator enabled it, and events expire with retention.clientErrors.eventDays | - | - |
| `client_error_group` | - | kept: browser error groups: a scrubbed error name, message, boundary and route template with counts; no person identifiers | - | - |
| `consent_record` | `account_id=id` | deleted | `records` | - |
| `contact_groups` | `address_book_id IN (SELECT id FROM address_books WHERE account_id=id)` | removed with its parent row | - | - |
| `contacts` | `address_book_id IN (SELECT id FROM address_books WHERE account_id=id)` | removed with its parent row | `contacts` (built by code) | - |
| `cookie_consent` | `account_id=id` | deleted | `consent` (built by code) | - |
| `crypto_changes` | `account_id=id` | deleted | - | - |
| `crypto_keys` | `account_id=id` | deleted | `security` | - |
| `data_export` | - | kept: legacy export log; rows are removed with the request retention | - | - |
| `dav_account` | `account_id=id` | deleted | `integrations` | - |
| `delegation_grant` | `owner_account_id=id` | deleted | `delegation` | - |
| `delegation_grant` | `delegate_owner_account_id=id` | deleted | `delegation` | - |
| `diag_probe_account` | - | kept: operator-configured probe mailbox, not a person's account | - | - |
| `dlp_audit` | `account_id=id` | deleted | `records` | retention: audit.retention; scrub: account_id (rows are removed with the account; a foreign key cannot be re-keyed, so a scrub-only run leaves them) |
| `domain_signature_template` | - | kept: domain-wide template; the matched column is an editing flag | - | - |
| `dsr_event` | - | kept: the trail of a request, kept and removed with it | - | - |
| `dsr_part` | - | kept: stored export parts, removed when the export expires or the person is erased | - | - |
| `dsr_request` | - | kept: the request and its trail are kept for the request retention period as proof of handling | `requests` (built by code) | retention: dsr.retention |
| `egress_proxy` | - | kept: operator-configured proxy; the username is a proxy credential | - | - |
| `erasure_tombstone` | - | kept: holds only a keyed hash of the erased subject, written so a restore can erase the person again | - | - |
| `event_instances` | `event_id IN (SELECT id FROM events WHERE calendar_id IN (SELECT id FROM calendars WHERE account_id=id))` | removed with its parent row | - | - |
| `events` | `calendar_id IN (SELECT id FROM calendars WHERE account_id=id)` | removed with its parent row | `calendars` (built by code) | - |
| `ews_account_cred` | `account_id=id` | deleted | `integrations` | - |
| `feature_user_subject` | `account_id=id` | deleted | - | - |
| `focused_sender` | `owner_subject=id` | deleted | `mailstate` | - |
| `identities` | `account_id=id` | deleted | - | - |
| `identity_binding` | `principal_id=id` | deleted | - | - |
| `identity_principal` | - | kept: an opaque random id with no identifier once its bindings are deleted | - | - |
| `identity_verification` | - | kept: one-time identity verification tokens (hash only); expire on their own | - | - |
| `inbound_webhooks` | `account_id=id` | deleted | `integrations` | - |
| `ip_block` | - | kept: abuse-prevention block keyed by network, expires on its own | - | - |
| `key_associations` | `account_id=id` | deleted | `security` | - |
| `linked_account` | `owner_principal_id=id` | deleted | `integrations` | - |
| `linked_account` | `child_principal_id=id` | deleted | - | - |
| `linked_challenge` | - | kept: short-lived sign-in challenge bound to a session; removed with the session | - | - |
| `linked_context` | `link_id IN (SELECT id FROM linked_account WHERE owner_principal_id=id)` | deleted | - | - |
| `linked_credential` | `owner_principal_id=id` | deleted | `integrations` (marker: present, not exported) | - |
| `linked_grant` | `link_id IN (SELECT id FROM linked_account WHERE owner_principal_id=id)` | deleted | - | - |
| `linked_layout` | `owner_principal_id=id` | deleted | - | - |
| `linked_primary` | `owner_principal_id=id` | deleted | - | - |
| `mail_user` | - | kept: the directory row is closed (status deleted), not removed, so the address cannot be re-enrolled without an administrator | `profile` (built by code) | - |
| `mail_user_account` | `account_id=id` | deleted | - | - |
| `mailboxes` | `account_id=id` | removed with its parent row | - | - |
| `masked_email` | `account_id=id` | deleted | `mailstate` | - |
| `mdn_sender_rule` | `account_id=id` | deleted | `mailstate` | - |
| `message_embeddings` | `account_id=id` | deleted | - | - |
| `message_meta` | `stable_id IN (SELECT stable_id FROM messages WHERE account_id=id)` | removed with its parent row | `mailstate` | - |
| `messages` | `account_id=id` | removed with its parent row | `mail` (built by code) | - |
| `metrics_bucket` | - | kept: aggregate metrics, no person identifiers: counts and quantile sketches per instance, metric and time bucket, with fixed role-class dimensions only | - | - |
| `native_sessions` | `account_id=id` | deleted | `sessions` (built by code) | - |
| `nextcloud_account_link` | `account_id=id` | deleted | `integrations` (marker: present, not exported) | - |
| `nextcloud_pim_binding` | `account_id=id` | deleted | - | - |
| `notebooks` | `account_id=id` | deleted | `notes` (built by code) | - |
| `notes` | `account_id=id` | deleted | `notes` (built by code) | - |
| `notification_rules` | `account_id=id` | deleted | - | - |
| `oauth_tokens` | `account_id=id` | deleted | `security` | - |
| `oidc_authorization` | - | kept: short-lived authorization request; expires on its own and is removed with its session | - | - |
| `oidc_code` | - | kept: short-lived authorization code; expires on its own | - | - |
| `oidc_consent` | `principal_id=id` | deleted | `security` | - |
| `oidc_grant` | `principal_id=id` | deleted | - | - |
| `oidc_token` | - | kept: tokens expire on their own and are removed with their grant | - | - |
| `open_events` | `token_hash IN (SELECT token_hash FROM open_tracking WHERE account_id=id)` | deleted | `records` | - |
| `open_tracking` | `account_id=id` | deleted | `records` | - |
| `org_view` | - | kept: organisation-wide saved views, not a person's data | - | - |
| `package_account` | `account_id=id` | deleted | - | - |
| `passwd_config` | `account_id=id` | deleted | - | - |
| `password_change_audit` | `account_id=id` | deleted | `security` | - |
| `personal_identity` | `owner_subject=id` | deleted | - | - |
| `personal_notification_rules` | `owner_subject=id` | deleted | - | - |
| `personal_push_subscription` | `owner_subject=id` | deleted | - | - |
| `personal_saved_search` | `owner_subject=id` | deleted | `mailstate` | - |
| `personal_setting_value` | `owner_subject=id` | deleted | - | - |
| `personal_signature` | `owner_subject=id` | deleted | - | - |
| `pim_changes` | `account_id=id` | deleted | - | - |
| `plugin_grants` | `account_id=id` | deleted | - | - |
| `plugin_kv` | `account_id=id` | deleted | `records` | - |
| `pop3_uidl` | `account_id=id` | deleted | - | - |
| `push_config` | - | kept: the server's own web-push key pair, not a person's data | - | - |
| `push_subscriptions` | `account_id=id` | deleted | - | - |
| `quotas` | `account_id=id` | deleted | `usage` | - |
| `receipt_status` | `account_id=id` | deleted | `mailstate` | - |
| `recovery_codes` | `account_id=id` | deleted | `security` (marker: present, not exported) | - |
| `remembered_device` | `account_id=id` | deleted | `sessions` (built by code) | - |
| `remote_image_grants` | `account_id=id` | deleted | `mailstate` | - |
| `request_protection_event` | - | kept: keyed by a one-way hash of a network source, not by person; bounded by its retention and not linkable to an address, so a per-person scrub has nothing to match | - | retention: requestProtection.retention |
| `request_protection_subject` | - | kept: keyed by a one-way hash of a network source, not by person; expires on its own | - | retention: requestProtection.retention |
| `retention_archive_index` | `account_id=id` | deleted | - | - |
| `retention_archive_segment` | - | kept: append-only sealed archive files hold rows of many people; an erased person's index entries are removed, and the segment content goes when its segment expires | - | - |
| `saved_searches` | `"user"=id` | deleted | `mailstate` | - |
| `scheduled_task` | - | kept: the lease owner is a server node name | - | - |
| `scrub_key` | - | kept: the keyed-pseudonym secrets; destroyed after the configured retention | - | - |
| `security_verdicts` | `account_id=id` | deleted | - | - |
| `sender_controls` | `account_id=id` | deleted | `mailstate` | - |
| `session_principal` | `principal_id=id` | deleted | - | - |
| `sessions` | `account_id=id` | deleted | `sessions` (built by code) | - |
| `setting_policy` | - | kept: the matched column is an override flag | - | - |
| `setting_value` | `scope_kind='user' AND scope_id=id` | deleted | `settings` (built by code) | - |
| `shared_mailbox` | `account_id=id` | deleted | `delegation` | - |
| `sieve_script` | `account_id=id` | deleted | `rules` (built by code) | - |
| `signatures` | `account_id=id` | deleted | - | - |
| `signin_allow` | - | kept: operator allow-list entries for sign-in patterns | - | - |
| `signin_event` | `account_id=id` | deleted | `sessions` (built by code) | - |
| `signin_identity` | `account_id=id` | deleted | `security` | - |
| `ssh_tunnel` | - | kept: operator-configured tunnel; the username is a tunnel credential | - | - |
| `sso_config` | - | kept: server-wide configuration with no per-person key | - | - |
| `sso_login_audit` | - | kept: sign-in audit kept for the audit retention period; the subject is pseudonymised by the scrub | - | retention: audit.retention; scrub: subject_hash (the subject is already a hash; the scrub re-keys it to the pseudonym) |
| `submissions` | `account_id=id` | deleted | `mailstate` | - |
| `sync_state` | `account_id=id` | deleted | - | - |
| `tags` | `"user"=id` | deleted | `mailstate` | - |
| `tasks` | `list_id IN (SELECT id FROM calendars WHERE account_id=id)` | removed with its parent row | `tasks` | - |
| `template` | - | kept: templates are owned by a scope (global, domain or user) and are managed by the template editor | - | - |
| `template_version` | - | kept: template content (a mail subject) owned by the template scope | - | - |
| `threads` | `account_id=id` | deleted | - | - |
| `totp_secrets` | `account_id=id` | deleted | `security` (marker: present, not exported) | - |
| `uploaded_blobs` | `account_id=id` | deleted | `records` | - |
| `usage_counter` | `address=id` | deleted | `usage` | - |
| `usage_storage` | `address=id` | deleted | `usage` | - |
| `user_activity` | `account_id=id` | deleted | `usage` | - |
| `user_group` | - | kept: group definitions belong to the directory, not to one member | - | - |
| `user_group_member` | `account_id=id` | deleted | `delegation` | - |
| `user_pim_retention` | `account_id=id` | deleted | `records` | - |
| `user_retention_account` | `account_id=id` | deleted | `records` | - |
| `user_retention_rule` | `account_id=id` | deleted | `records` | - |
| `user_retention_run` | `account_id=id` | deleted | `records` | - |
| `webauthn_credentials` | `account_id=id` | deleted | `security` | - |
| `webhooks` | `account_id=id` | deleted | `integrations` | - |
| `zeroaccess_accounts` | `account_id=id` | deleted | `security` (marker: present, not exported) | - |
<!-- registry:end -->
