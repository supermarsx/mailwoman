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
