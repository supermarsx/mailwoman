# Domains and upstream routing

Open **Domains** in the administrator console to configure where mailbox accounts
connect. Server mode is process-wide: engine mode uses IMAP or POP3 with SMTP;
proxy mode uses a JMAP session URL. Clients cannot choose upstream servers at login.

The editor supports a preview before saving, conflict detection through `If-Match`,
sign-in allow/block lists, enable/disable, aliases, and web-host mappings. A preferred
domain expands a bare login such as `alice` to `alice@example.org`. A configured web
host mapping takes precedence over the preferred domain. Forwarded host headers are
used only from trusted proxies. Aliases share routing and policy with the canonical
domain; the username sent upstream retains the domain the user typed.

Domain upstream edits require an unscoped `domains.upstream_write` grant. Sign-in
lists require `domains.signin_policy` for that domain. Lists support case-insensitive
`*` patterns, block rules take precedence, and an empty allowlist permits all names.
The server checks these lists after checking credentials and returns the ordinary
authentication failure when a list denies access.

For a configured mail endpoint, select implicit TLS, STARTTLS, or plaintext and
password authentication. Other authentication mechanisms are not accepted by this
editor. Private LAN and loopback endpoints require the explicit private-address
option. The actual dial checks every DNS answer and connects to checked addresses;
link-local and metadata destinations remain refused. Default environment-configured
upstreams retain their existing deployment policy. JMAP requests retain the existing
JMAP upstream allowlist and redirect policy.

Discovery runs only when an authorized administrator requests it. HTTP discovery
uses the `admin_detect` egress route, including redirects, and returns a proposal
without saving it. An OAuth-only discovery result needs review because the domain
editor currently accepts password authentication only.

## Changing or removing a domain

Disabling a domain rejects subsequent sign-ins and invalidates its mailbox sessions
on their next authenticated request. Changing an engine upstream disconnects its
old runtime; users sign in again against the new endpoint. A proxy session is
revalidated and repointed to its configured JMAP authority on the next request.
Changing SMTP configuration through the editor also invalidates the cached runtime.

Deletion is refused while a domain is preferred or has signed-in accounts, unless
an unscoped administrator explicitly forces deletion. A stale editor revision
cannot delete a concurrently changed row, including when force is requested.

`GET /admin/domains/status` reports blocked-account counts grouped by domain and
operator-configured JMAP origin choices. Counts are recalculated from current
routing; configuring a missing domain clears its routing warning without a restart.

## Command line

Use the same database and server key as the deployment. Set `MW_MODE` to its server
mode when editing upstream configuration. These offline operator commands audit
changes and use the same upstream schema as the HTTP editor:

```sh
mailwoman admin domains create example.org
mailwoman admin domains set-upstream example.org '{"imap":{"host":"imap.example.org","port":993,"tls":"implicit"},"smtp":{"host":"smtp.example.org","port":587,"tls":"starttls"}}'
mailwoman admin domains prefer example.org
mailwoman admin domains alias example.org example.net
mailwoman admin domains host example.org webmail.example.org
mailwoman admin domains show example.org
mailwoman admin domains disable example.org
mailwoman admin domains enable example.org
mailwoman admin domains alias example.org example.net --remove
mailwoman admin domains host example.org webmail.example.org --remove
mailwoman admin domains unprefer
mailwoman admin domains delete example.org
```

## Existing installations

Domain details expose separate concurrency tokens: the response `ETag` protects
configuration edits, `mappingEtag` protects aliases and login hosts, and
`preferenceEtag` protects the global preferred-domain selection. Send the matching
token in `If-Match` to reject stale changes. Mapping and preference writes support
`?dryRun=true`; PUT also accepts a `dryRun` body field. Legacy callers may omit
`If-Match`. A mapping already owned by another domain returns 409; an alias that
shadows a canonical domain, or a canonical name already used as an alias, returns 422.

On first startup after this change, installations with neither a configured domain
upstream nor a default import known authorities from stored engine accounts or JMAP
sessions. The most common authority wins for each domain; imported rows record
`importedAt` and `upstreamImportAmbiguous` in their settings for operator review.
Interrupted imports resume without overwriting already configured domain rows.
The `v6:upstream_import:done` setting prevents a later restart from reimporting a
domain that an administrator deliberately removed.

Sessions without a resolvable operator upstream are revoked. Stored engine accounts
without matching routing are not reconnected at startup. Their account records and
cached mail remain available for recovery after the operator configures routing.
`MW_UPSTREAM_DEFAULT` remains a supported deployment default; in proxy mode a single
usable `MW_JMAP_UPSTREAMS` entry remains the compatibility fallback. Several origins
need explicit domain mapping rather than a client-selected server.
Trusted-header sessions retain their explicit `MW_HEADER_AUTH_JMAP_URL` while
header authentication remains enabled. Ordinary password sessions cannot use that
override; domain restrictions continue to apply to both authentication paths.

The unauthenticated `/api/public-config` endpoint returns only branding, theme,
accent, login notice, and the suggested login domain, with `Cache-Control: no-store`.
It does not expose the domain inventory or upstream addresses.
