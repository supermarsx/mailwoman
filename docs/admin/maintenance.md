# Maintenance admission mode

Maintenance mode is disabled by default. It pauses admission of new mailbox requests without deleting sessions, removing credentials, cancelling accepted work, or changing upstream mail storage. Background synchronization and already accepted submissions are not a shutdown mechanism and may continue.

Administrators read and update `/admin/maintenance/mode` (under the configured administrator and application prefixes). Reading requires `maintenance.read`; updating requires unscoped `maintenance.write`, fresh central authentication for `maintenanceMode`, and the exact strong `If-Match` returned by the latest read. Missing revisions return 428; stale, weak, wildcard or multiple revisions return 412. The persisted configuration is:

```json
{"enabled":false,"message":"","retryAfterSecs":300}
```

The message is plain text, at most 1000 characters, with control characters refused. Clients must render it as text, never HTML. Retry time is 1–86400 seconds. Updates use an atomic compare-and-set across instances; transition audit records include enabled state and retry time, not the message contents.

When enabled, new mailbox API/login, JMAP, DAV, SSO, OIDC, OAuth and MCP requests receive HTTP 503, `Retry-After`, `Cache-Control: no-store`, and `{code:"maintenance",message,retryAfterSecs}`. Existing sessions become usable again after disabling mode, subject to their ordinary expiry/revocation policies. Administrator routes remain protected by their existing authentication and permissions and remain accessible for recovery. Static assets, health checks, public configuration, the sign-in challenge needed by administrator login, branding assets and explicit logout/revocation paths remain reachable. Unknown or corrupt stored maintenance configuration fails closed for governed user requests. The administrator GET then returns a conservative enabled fallback and top-level `configurationInvalid:true`, with an ETag derived from the invalid raw record. A fresh authorized PUT can repair that exact record; a concurrent change still returns412. The invalid bytes are never exposed. The response-only marker must not be included in PUT requests.

`/api/public-config` exposes only the safe `maintenance` configuration object for a pre-login notice. The setting is deployment-wide and stored as `maintenance.mode.v1`; it is not a per-user preference or a timer that automatically disables maintenance.
