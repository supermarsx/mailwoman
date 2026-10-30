# Feature profiles and verified policy subjects

Feature profiles reuse the settings authority: global and domain defaults, locks,
profiles, groups, administrator user values, and permitted personal overrides.
The effective client catalogue includes setting provenance. Profiles do not grant
mailbox access, sender aliases, credentials, or a new authentication method.

## Administration API

`GET /admin/features` retains `catalogue`, `integrationPolicyKeys`, `profiles`,
`groups`, and `assignments`. It also returns `revision`, `memberships`,
`nativeTargets`, and `domains`, with a strong `ETag` and `Cache-Control: no-store`.
All paths use the deployment's configured admin prefix and base path.

Every mutation requires that exact catalogue `If-Match`, including creation:

* `PUT`/`DELETE /features/profiles/{id}`
* `PUT`/`DELETE /features/groups/{id}`
* `PUT /features/groups/{id}/members`, body `{ "members": ["native-account-id"] }`
* `PUT`/`DELETE /features/assignments`, body containing `scopeKind`, `scopeId`,
  `profileId`, and `priority`

Missing preconditions return 428; wildcard, weak, multiple, or stale revisions
return 412. Comparison and changes are one database transaction. A successful
mutation returns `{ "ok": true, "revision": "..." }` and the next ETag. Reload
the catalogue before editing after a conflict. This CAS serializes feature-governance
edits; account/domain provisioning has separate transactions, so auxiliary target
information is not a globally serializable snapshot of all provisioning work.
Group membership reads return the
same catalogue revision. Builtin profiles and directory-managed groups cannot be
rewritten through these editing routes.

Existing unscoped `settings.read`, `settings.write`, and
`settings.user_values` permissions remain required by their respective routes.
Domain-scoped administrators cannot administer this global catalogue. User
assignments and manual group members must identify existing native accounts from
`nativeTargets`; arbitrary addresses and proxy raw IDs are not accepted as new
targets. Domain-specific group members must belong to the group's canonical
domain. Orphaned legacy assignments can be removed without authenticating or
attributing their old targets.

## Proxy identity separation

Session policy resolution uses verified principal provenance. Native sessions keep
their existing native assignments. Proxy sessions receive global/domain policy,
including domain profiles, but cannot inherit native user/profile/group rows by
matching a raw upstream account ID or address. Their personal overrides use the
separate `personal_setting_value` table, keyed by a verified service-bound owner.
Migration 0070 does not attribute or remove legacy ambiguous rows. The table is
included in SQLite-to-PostgreSQL copying.

Client `PUT`/`DELETE /api/settings/{key}` also require the current
`X-Mailwoman-Storage-Authority` fence. Missing authority returns 428 and a changed
account returns 409. This header is a mismatch check, not an account selector.

Account-scoped plugin admission requires an existing native account; session-name
lookup is not a substitute. Existing global-only plugin admission remains global.
This change does not establish a universal realm-safe audit of every older
authentication, PIM, or third-party integration table.

## How a value is chosen (t98)

One resolver decides every feature for every route guard, `GET /api/features` and
the console's effective-value preview (`settings_policy::resolve`, wrapped by
`features::resolve_features`). For each key, first match wins:

1. an environment **pin**;
2. a **locked global** value;
3. the **broadest lock** among domain, profile, group and administrator per-user
   values (a locked domain value beats every profile, group and user value);
4. the **narrowest value that is set**: the user's own choice (only for keys users
   may change), then an administrator's value for that user, a group value, a
   profile value, a domain value;
5. the global value, else an environment fallback, else the built-in default.

Several profiles apply to one user in assignment priority order (higher first);
at equal priority a user assignment comes before a group, and a group before a
domain. The first profile that holds a key decides it. A linked mailbox is a
separate subject: its own account's profiles apply while it is open.

The diff in the editor compares a profile with step 5 (what a user with no profile
gets); the preview (`GET /admin/features/effective?account=` or `?domain=`) runs
the whole chain and names the deciding scope and profile.

## What a profile controls

A feature is on when every boolean key it lists is `true`. Off means the route
answers `403 {"code":"feature_disabled","feature":<id>,"key":<key>}` before
anything runs (enforced in `feature_gate.rs` unless noted); the web app hides it.

| Category | Feature id | Kind | Keys | Refused at |
|---|---|---|---|---|
| Mail and compose | `compose.rich_text` | toggle | `cap.compose.rich_text` | JMAP `Email/set` create with an HTML body |
| | `compose.attachments` | toggle + limit | `cap.compose.attachments`, `limit.attachment_bytes` | `Email/set` create with attachment parts; `/jmap/upload` above the limit (413) |
| | `compose.share_links` | toggle | `cap.compose.share_links` | `/api/nextcloud/share-link` |
| | `compose.scheduled_send` | toggle | `cap.compose.scheduled_send` | `EmailSubmission/set` with `sendAt` (sending now stays possible) |
| | `compose.read_receipts` | toggle + settings | `cap.compose.read_receipts`, `receipts.dsnAllowed`, `mail.readReceipts` | `Email/set` create requesting a read receipt |
| | `compose.signatures` | toggle | `cap.signatures` | writes under `/api/account/signatures` |
| | `compose.identities` | toggle | `cap.identities` | writes under `/api/account/identities` |
| | `compose.sending` | limits | `limit.recipients_per_message`, `limit.send_per_hour`, `limit.send_per_day` | recipients per message on `Email/set`/`EmailSubmission/set`; the hourly and daily counts are not yet enforced |
| Reading and privacy | `privacy.remote_images` | toggle | `cap.remote_images` | `/api/image-proxy`, `POST /api/remote-images/grant` |
| | `privacy.direct_content` | toggle | `cap.remote_content.direct` | remote-content policy (`remote_policy.rs`) |
| | `privacy.trusted_links` | toggle + list | `cap.links.trusted`, `links.trustedDomains`, `links.trustedSubdomains` | `PUT /api/settings/links.*` |
| | `reading.zoom` | toggle | `cap.zoom` | `PUT /api/settings/reader.zoom` |
| Views and search | `views.focused` | toggle | `cap.mail.focused_inbox` | `PUT /api/settings/mail.focusedInbox` |
| | `views.events` | toggle | `cap.views.events` | `Email/query` on the Events view |
| | `views.attachments` | toggle | `cap.views.attachments` | web only (the messages stay reachable by search) |
| | `search.operators` | toggle | `cap.search.operators` | `Email/query` text with `from:`, `subject:`, `has:` ... |
| | `search.saved` | toggle | `cap.saved_searches` | writes under `/api/account/saved-searches` |
| Organise | `rules` | toggle | `cap.rules` | `MailRule/*`, `VacationResponse/*` (`feature_jmap.rs`) |
| | `rules.forward` | toggle | `cap.rules.forward` | `MailRule/set` with a forward action |
| | `sieve.raw` | toggle + limits | `cap.sieve.raw_edit`, `sieve.maxScripts`, `sieve.maxScriptBytes`, `sieve.allowedExtensions` | Sieve scope guard |
| | `forwarding.external` | toggle + limit | `cap.sieve.forward_external`, `sieve.maxRedirects` | Sieve |
| | `vacation` | toggle | `cap.vacation` | `VacationResponse/*` |
| | `labels`, `snooze`, `follow_up` | toggles | `cap.labels`, `cap.snooze`, `cap.follow_up` | `Email/set` adding a label, `snoozedUntil`, `followUpAt` (removing stays possible) |
| | `masked` | toggle | `cap.masked` | `/api/masked` |
| Calendar, contacts, tasks, notes, keys | `calendar`, `contacts`, `tasks`, `notes` | toggles | `cap.pim.*` | their JMAP families, Nextcloud PIM, `/api/calendar/*` |
| | `keys` | toggle | `cap.keys` | `CryptoKey/*`; the Keys module and compose sign/encrypt |
| Data | `import`, `export` | toggles + limits | `cap.import`, `import.maxMessages`, `import.maxMessageBytes`, `cap.export` | their routes |
| | `storage` | limit | `limit.storage_bytes` | stored only; the mail server's quota applies |
| Accounts and access | `accounts.linked` | toggle + limit | `accounts.linkingEnabled`, `limit.linked_mailboxes` | `POST /api/accounts/link` (409 at the limit) |
| | `access.protocols` | allowed values | `access.protocols` | `/jmap/session`, `/jmap/api`, `/api/v1` for an account of another kind |
| | `apps.native` | toggle | `cap.apps.native` | every signed-in request of a desktop or mobile app session |
| | `notifications.push` | toggle | `notifications.pushAllowed` | `/api/push/subscribe` |
| Integrations | `integrations.nextcloud`, `integrations.google`, `integrations.microsoft`, `plugins.ui`, `packages`, `ai_assistant`, `directory` | toggles (+ `limit.ai_calls_per_day`, `assist.profile`) | `cap.integration.*`, `cap.plugins`, `packages.allowed`, `cap.ai_assistant`, `cap.directory` | their routes |
| Developer access | `api_keys`, `mcp`, `oauth_apps` | toggles | `cap.api_keys`, `cap.mcp`, `cap.oauth_apps` | `POST /api/keys` and key use on `/api/v1`; `/mcp` with a key or token; OAuth consent |
| Personalisation | `appearance.custom` | toggle | `cap.appearance.custom` | `PUT /api/account/appearance` (theme, accent, custom and badge colours, density, font) |
| | `customisation.code` | toggle | `cap.appearance.custom_code` | `PUT /api/account/appearance/customization` |
| | `avatars`, `shortcuts`, `analytics` | toggles | `cap.avatars`, `cap.shortcuts`, `cap.analytics` | avatar changes on `PUT /api/accounts/layout`; `PUT /api/settings/shortcuts.*`; `POST /api/analytics/events` |

Every other setting a profile may hold (any non-global registry key) is in the
editor under **Other settings**. Not gateable by a profile, by design: signing in,
password, second factors and sessions (security policy has its own pages), and
reading and sending plain mail.

## Presets, import and export

`GET /admin/features` also returns `categories`, `profileKeys` (every key a profile
may hold, with its kind and bounds), `defaults` (step 5 above) and `presets`
(`full`, `standard`, `mail-only`, `minimal`). Presets are starting points in the
editor, not stored profiles. `GET /features/profiles/{id}/export` answers
`{"format":"mailwoman.featureProfile","version":1,"name":...,"values":...}`;
`POST /features/profiles/import` checks such a document exactly as a save does
and answers its normalised `name` and `values` without storing anything.
