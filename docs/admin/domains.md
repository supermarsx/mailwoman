# Domains and upstream routing

Open **Domains** in the administrator console to configure where mailbox accounts
connect. Server mode is process-wide: engine mode uses IMAP or POP3 with SMTP;
proxy mode uses a JMAP session URL. Clients cannot choose upstream servers at login.

The editor supports a preview before saving, conflict detection through `If-Match`,
sign-in allow/block lists, enable/disable, aliases, and web-host mappings. A preferred
domain expands a bare login such as `alice` to `alice@example.org`. A configured web
host mapping takes precedence over the preferred domain. Forwarded host headers are
used only from trusted proxies. Aliases share routing and policy with the canonical
domain. The application retains the address the user typed as its account identity;
an explicit backend username format can change only the name used to authenticate
to that backend.

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

## Backend selection and authentication names

Version 2 configurations contain up to 32 named backends, grouped by service:
mail (IMAP, POP3, or JMAP), submission (SMTP), and rules (ManageSieve). Each
backend has an enabled flag, a numeric priority, and an optional preferred flag.
Selection uses the enabled preferred backend first, then the lowest priority
number, then the backend ID. Only one preferred backend is allowed per service.
Selection does not retry a password against another backend after authentication
fails. A configured domain with no enabled mail backend refuses login.

### Mail backend failover

Failover is off by default. Turn it on with **Fail over to the next mail
backend when the selected one cannot be reached** on the domain's Connections
page, or `"failover": true` in a version 2 configuration. It applies to IMAP and
POP3 mail backends only; submission (SMTP), rules (ManageSieve), JMAP and EWS
are never failed over.

- The chain is the selected mail backend followed by the other enabled mail
  backends of the same protocol, by priority then ID. One attempt checks at
  most three backends.
- Every backend in the chain must use the same connection security, the same
  password-authentication policy and the same username format as the selected
  one; only the host, port and private-network permission may differ. Saving a
  chain that breaks this is refused.
- Before credentials are sent, each candidate gets the same credential-free
  check the diagnostics use: DNS, TCP connect, implicit TLS or STARTTLS, and
  the greeting. Only a DNS, connect, timeout, TLS or certificate failure moves
  to the next candidate. An address-policy refusal stops the attempt.
- The first candidate that passes is the only backend that receives the
  password. If it then rejects the password, or fails for any other reason, the
  sign-in or reconnect fails; the password is never tried on another backend.
- A backend that failed its check is moved behind the healthy ones for 30
  seconds, doubling with each consecutive failure up to 15 minutes. When that
  time ends it is checked first again, and one success clears its record, so
  new connections return to it. This state is per process and starts empty
  after a restart. Connections already open on a fallback stay there until they
  reconnect.
- The account keeps the selected backend as its identity (host and port), so
  failing over does not create a second account or change its ID.
- Each event is written at WARN level with the component
  `mailwoman::backend_failover` and appears under **Operations → Logs →
  Application logs**: `event` is `check-failed`, `failover`,
  `selected-backend-resting` or `returned-to-selected`, with the domain, the
  backend IDs and the failure reason. Host names and credentials are not
  logged.
- The chain is part of the domain's authentication binding. Turning failover
  on while a domain has more than one eligible backend, or changing the chain,
  asks existing accounts of that domain to sign in again.

The process-wide engine/proxy mode still applies; a backend list does not enable
mixed JMAP proxy and IMAP/POP3 engine operation within one running server.

Socket backends can authenticate with the full email address, local username,
`DOMAIN\user`, `user@realm`, or a custom template. Templates accept `{user}`,
`{domain}`, `{email}`, and `{realm}` and must include `{user}` or `{email}`.
Each service can use its own format. Formatting never changes account IDs,
session identity, or policy identity. JMAP currently requires the full-email
format. Changing the selected endpoint or authentication format requires a fresh
sign-in rather than silently reusing an existing authenticated connection.

## Connection security

Fixed security uses one selected method. Auto security tries the administrator's
ordered list before sending credentials. It advances only when the server does
not support the attempted method; certificate, hostname, network, and timeout
failures stop selection. Plaintext must be explicitly permitted before it can
appear in that list. Allowing plaintext permits an unencrypted connection when
the preceding methods are unavailable.

An optional TLS server name supports connecting to an IP address while verifying
the certificate against a DNS name. A public PEM CA bundle adds private trust
anchors to the standard roots. Certificate verification remains enabled, and the
process-wide minimum TLS version still applies. These settings are used by IMAP,
POP3, SMTP, and ManageSieve, including STARTTLS upgrades. Existing sessions pin the
selected security method on reconnection; changing configuration requires login
again. SSH tunneling is not provided by these endpoint settings.

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
