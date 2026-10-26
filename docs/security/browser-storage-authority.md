# Browser storage authority

Authenticated login and `/api/me` may return `storageAuthority`, an opaque
`msa1_` value. It identifies the namespace for private browser persistence;
`accountId` remains the upstream JMAP account identifier.

The server derives this value only from an active verified session-principal
binding. Native sessions backed by a stored native account share that account's
namespace across authentication methods. Proxy sessions bind the verified
identity, account ID, and canonical discovery and API URLs. Non-native SSO
sessions bind the verified issuer identity and account ID. The value is a
purpose-separated HMAC; it does not disclose URLs, external subjects, or email
addresses. Session rotation preserves the namespace, while retired or unbound
sessions cannot obtain it.

Clients must require both the authority and account ID when restoring private
records. Missing authority disables persistence and automatic restore. Legacy
records containing only a raw account ID have no reliable realm provenance and
must not be automatically assigned to the currently signed-in account, even
when that account is native. This contract does not itself change the separate
server-side settings and feature-profile subject keys.
