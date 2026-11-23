# Browser storage authority

Several things are kept per account: preferences and appearance in the browser and on the server, saved drafts, analytics consent, push subscriptions. An upstream account id alone cannot safely name such data, because two different mail servers (or a proxy and a native account) can hand out the same id. Mailwoman therefore gives each verified signed-in identity an opaque name, the **storage authority**, and keys private data by it.

## The value

After a successful sign-in, the sign-in response and `GET /api/me` include `storageAuthority`, a string of the form `msa1_<64 hex characters>`. `accountId` stays the upstream account id.

The server computes it only for a session that is bound to an active identity whose binding has not been retired. It is `HMAC-SHA256` over the identity's fields, under a key derived from the server key for this purpose only. The fields are length-prefixed and depend on how the user signed in:

| Sign-in | Fields |
|---|---|
| Native account (engine mode, including SSO into a native account) | `native`, the account id |
| SSO into a non-native account | `sso`, the identity binding, the account id |
| Proxy to an upstream JMAP server | `proxy`, the identity binding, the account id, the discovery URL and the API URL |

Consequences:

- A native account has the same storage authority whichever way it signs in.
- The value reveals no URL, external subject or address.
- Signing in again or rotating the session keeps the same value. A retired or unbound session gets none, and personal settings then answer `503` with "Sign in again before loading personal settings."
- The value depends on the server key. If the server key is replaced, every storage authority changes, and data keyed by the old values is no longer found.

## How the server uses it

The storage authority is not a credential. The server always derives it from the session cookie; the browser cannot choose it. The browser sends it back in the `X-Mailwoman-Storage-Authority` header as a check that it is still talking about the same identity:

- A write to personal data without the header is refused with `428` ("Reload these settings before saving.").
- A header that differs from the session's value is refused with `409` (`accountChanged`, "The signed-in account changed. Reload these settings.").

This stops a tab that was opened under one account from writing into another account's data after the user switched accounts in another tab.

## Rules for the browser

- Private records in browser storage are restored only when both the storage authority and the account id match.
- Without a storage authority, nothing private is persisted or restored.
- Records that carry only an account id are never assigned to the signed-in account automatically, even when the ids match.

## Related

- [Personal settings and verified identities](./personal-settings-authority.md)
- [Encryption at rest](./encryption-at-rest.md) (the server key)
