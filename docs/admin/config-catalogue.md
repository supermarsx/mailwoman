# Effective configuration catalogue

`GET <admin-path>/config/effective` returns an array of configuration entries and an
HTTP `ETag`. The response is private and must not be cached. It requires an
unscoped `config.read` grant. Security, integrations, mail, observability and
settings entries additionally require their area's unscoped read capability.
Domain-only grants do not authorize this global view.

Each entry includes a stable `id`, class, group, source, documentation link,
restart metadata, a value status, and a row ETag. Classes use lower camel case
(`ui`, `readOnly`, `refused`, `cliOnly`, `build`). `href` and `searchText` contain
navigation and static catalogue metadata, never configured values.

Value status distinguishes the evidence behind the display:

- `resolved`: read from running application state or a subsystem's effective
  policy. The database location is captured after redaction at startup.
- `resolvedPolicy`: computed by the existing settings resolver for global policy.
  This does not assert that every declared key has a runtime consumer.
- `build`: a declared build constant.
- `unavailable`: this process cannot report the effective value. `value` is null.

`configuredInput` is the separately redacted environment input. It must never be
substituted for an unavailable effective value. A secret is represented only as
`{"set":true}` or `{"set":false}`. URLs omit user information, query strings and
fragments, including URLs nested in lists. Ambiguous connection strings are
redacted rather than echoed. The server-key entry reports whether the key exists
and whether it was configured persistently; it never reports key material.

The catalogue endpoint is read-only. `readOnly` describes that endpoint, while
`class`, `configuredBy` and settings metadata identify declared configuration
locations; they do not assert that a planned editor or integration is implemented.
Deployment trust inputs remain deployment inputs. Changing an
environment variable requires the relevant process to restart; the catalogue
does not edit the environment or restart services.

Settings writes continue through the existing settings or dedicated feature
routes, preserving validation, scope restrictions, locks, environment pins,
`If-Match`, impact previews and audit. Configuration export/import, snapshots,
CLI parity and a domain-scoped catalogue are separate work and are not exposed
by this endpoint.
