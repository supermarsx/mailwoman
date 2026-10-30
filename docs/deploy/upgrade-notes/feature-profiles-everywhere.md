# Upgrade note: feature profiles cover every gateable feature (26.20, t98)

Applies to every deployment that uses feature profiles, and to API clients that
call features an administrator can now turn off.

## What changes

- **About 30 new profile switches** (`cap.*`, all default `true`), one allowed-values
  list (`access.protocols`, default `["imap","pop3","ews"]`) and one limit
  (`limit.linked_mailboxes`, default `0` = only the server maximum). The full list
  with what each one refuses is in [settings.md](../../admin/settings.md#feature-and-integration-permissions)
  and [feature-governance.md](../../admin/feature-governance.md#what-a-profile-controls).
- **The server refuses a feature that is off**, with
  `403 {"code":"feature_disabled","feature":"<id>","key":"<key>"}` (a limit:
  `409`/`413 {"code":"limit_exceeded","limit":"<key>","max":<n>}`), before any part
  of the request runs and before anything reaches the mail server. The web app
  hides what is off; hiding is never the only control except for the Attachments
  view, whose messages stay reachable through search.
- **The profile editor** groups features by category, has search, allow all /
  disallow all / clear per group, a diff against the server default, presets
  (Full, Standard, Mail only, Minimal / kiosk), clone, JSON import and export
  (`mailwoman.featureProfile` version 1), and an effective-value preview for a user
  or a domain.

## Nothing changes on upgrade

- No database migration. Profiles, assignments, groups and stored values are read
  as before; a profile only holds the keys it was saved with.
- Every new switch defaults to allowed and every new limit to "no extra limit", so
  a user keeps every feature they had.
- One existing key is now enforced where it was not before: a profile or setting
  that already stored **`cap.api_keys = false`**, **`cap.oauth_apps = false`**,
  **`cap.remote_images = false`** or **`cap.rules.forward = false`** now takes effect
  (key creation and key use, OAuth consent, the image proxy and image grants,
  forwarding rule actions). These values were stored by an administrator but had
  no effect; check them before upgrading:

  ```sh
  sqlite3 mailwoman.db "SELECT scope_kind, scope_id, key, value FROM setting_value WHERE key IN ('cap.api_keys','cap.oauth_apps','cap.remote_images','cap.rules.forward') AND value='false'; SELECT key, global_value FROM setting_policy WHERE key IN ('cap.api_keys','cap.oauth_apps','cap.remote_images','cap.rules.forward');"
  ```

- `cap.user_css` and `cap.user_background` (never enforced) are no longer listed
  as catalogue features; profiles that hold them keep the values, shown under
  "Other settings". Custom CSS and scripts are now `cap.appearance.custom_code`,
  below the existing deployment switches.

## API clients

A client that uses a feature a profile turns off gets the `403` above instead of
the operation. `GET /api/features` lists every feature with `enabled`, its limits
and allowed values, and their provenance; check it before offering a feature.
