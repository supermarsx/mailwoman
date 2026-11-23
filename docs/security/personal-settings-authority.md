# Personal settings and verified identities

Personal settings are stored on the server under an owner that comes from the verified signed-in identity, not from the raw upstream account id. This page explains which owner each kind of data uses and what happens to data stored under older, id-only keys.

## Owners

| Data | Owner | Depends on the server key |
|---|---|---|
| Interface preferences, mailbox appearance, mailbox CSS and script documents | The [storage authority](./browser-storage-authority.md) (`msa1_…`), in keys of the form `personal-v2:<kind>:<owner>` | Yes |
| Signatures (`personal_signature`), saved searches (`personal_saved_search`), sending identities, focused senders, notification settings, push subscriptions, security alerts and saved drafts | A personal owner `po1_…`: SHA-256 over the same identity fields as the storage authority, with its own domain prefix | No |

Every write still has to carry the session's storage authority in `X-Mailwoman-Storage-Authority` (`428` without it, `409` if it differs). Existing compare-and-set checks and administrator locks on appearance settings still apply. Administrator appearance documents have their own ownership and are not affected.

For native accounts the personal owner is derived only from an actual native account. A proxy session cannot reach a native account's data by presenting a matching account id.

Analytics consent is kept per storage authority and starts unset for each one. Recording events and the queued export both check the session's verified authority and its saved consent, so another identity with the same raw account id cannot give consent on someone else's behalf.

Feature assignments and the settings that administrators set per user or group use their own subject model, described in [Settings](/admin/settings).

## Data stored under older keys

Rows saved under the older keys `ui-preferences:<account>`, `appearance:<account>`, `appearance-customization:mailbox:<account>`, and in the older `signatures` and `saved_searches` tables, stay in the database but are not loaded. A raw account id cannot show which identity wrote them, even when a native account has the same id today.

Users can set their preferences again in **Settings**. To recover the old values, an administrator can read the old rows from a database backup, establish the original owner from independent evidence, and have that user reapply the values through the normal settings pages. There is no automatic reassignment. Keep the old rows until ownership is established.

## Moving the database

The store migration tool copies the signature and saved-search tables including their HTML and plain-text formats. It copies sealed values without resealing them, so the target must use the same server key. Preferences keyed by the storage authority are only found again under the same server key; the personal owner of signatures and saved searches does not depend on it. Neither property means that changing the server key is supported.

## Related

- [Browser storage authority](./browser-storage-authority.md)
- [Personal settings (user guide)](/user/settings/)
