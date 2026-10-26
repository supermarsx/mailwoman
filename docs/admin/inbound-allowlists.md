# Inbound IP and hostname allowlists

The inbound policy is disabled by default. It controls the source IP of HTTP
requests to the admin console/API and, separately, the webmail service APIs.
It does not add trusted proxies, change API-key scopes, exempt request budgets,
or grant outbound network access.

`GET /admin/security/inbound-allowlist` requires unscoped `security.read` and
returns `{config,status}` with an ETag. `PUT` requires unscoped `security.write`,
fresh `banManage` proof, and the current `If-Match`. The configured admin path
replaces `/admin` when customized. Missing revisions return 428; concurrent
changes return 412. Changes are audited without recording resolved addresses.

```json
{
  "admin": {"enabled": false, "cidrs": [], "hostnames": []},
  "webmail": {"enabled": false, "cidrs": [], "hostnames": []},
  "dns": {
    "servers": [],
    "timeoutMs": 2000,
    "cacheSeconds": 300,
    "maxHosts": 64,
    "maxAddresses": 16,
    "concurrency": 4
  }
}
```

Each enabled surface needs at least one literal rescue IP/CIDR. Check that the
operator's actual trusted source is covered before enabling the admin policy.
There is no automatic loopback or signed-in administrator bypass. Up to 128
literal entries are accepted per surface. Hostnames must be lowercase ASCII DNS
names (use IDNA ASCII for international names), without URLs, ports, wildcards,
trailing dots, or IP literals. DNS names grant only their currently resolved
addresses, not whole subnets or addresses obtained through reverse DNS.

## DNS limits and trust

An empty `servers` list uses the operating system's configured recursive DNS
servers. An explicit list contains at most four IP:port sockets, for example
`["10.0.0.53:53", "[fd00::53]:53"]`; it replaces the OS servers, with no public
resolver fallback. These are dedicated DNS exchanges over UDP/TCP, not HTTP
proxy requests. Resolver configuration is a privileged network-trust decision.
Answers are only as trustworthy as those configured recursive resolvers; this
feature does not claim independent DNSSEC authentication.

The bounds are 100–5000 ms per lookup, a 30–3600 second cache cap, 1–64 distinct
hostnames across the two surfaces, 1–32 addresses per hostname and 1–4 concurrent
lookups. Both A and AAAA are queried using absolute names. Search suffixes and
the hosts file do not extend the policy. Actual DNS TTL is respected even when
shorter than the configured cap; the cap never extends an answer's lifetime.

Requests do not wait for DNS. A single bounded background refresh runs per
server instance, triggered by traffic or a status read. Newly configured names
grant nothing until successfully resolved. Failed lookups retain earlier
answers only until their original expiry. An oversized answer grants nothing;
it is not truncated into a permissive partial answer. Expired, unresolved and
failed names grant no access. Refresh status includes usable addresses, remaining
validity, bounded error codes, and whether a refresh is in progress.

Every protected request reads the durable configuration with a two-second
deadline. This makes changes visible across instances without waiting for DNS
expiry. A revision change cancels the local refresh and discards old DNS grants;
completion from a superseded configuration cannot restore them. Configuration
read failure or invalid stored configuration returns 503; a source mismatch or
unknown source IP returns 403. Both responses are generic and `no-store`.

## Covered surfaces and limits

The admin policy covers the configured admin path and all its child routes,
including login and allowlist management. The webmail policy covers `/api`,
`/jmap`, `/dav`, `/rc`, `/sso`, `/oidc`, `/oauth` and `/mcp` and their children,
including signed-content URLs, API keys and websocket handshakes. Existing
connections are not forcibly terminated; new requests are checked. HTTP upgrades
retain the admission decision from their handshake.

Shared static assets, health/readiness, public WKD discovery, `/metrics`, and
error-report ingestion are outside these two route groups and retain their own
controls. The policy does not secure SMTP/IMAP listener ports or firewall rules.
The legacy `ip_allow` table and its domain/account scope columns are not used by
this global policy. Domain/account-specific hostname policies are not supported.

Forwarded headers are considered only through the existing trusted-proxy
configuration. Supplying `X-Forwarded-For` without a trusted transport peer never
substitutes for the actual client IP. If all rescue sources are removed by an
operator, recovery requires a controlled deployment-side restoration of the
`security.inbound-allowlist.v1` setting; ordinary authentication cannot bypass
the network policy.

## Admin editor

Admin → Inbound access presents separate admin and webmail rules, bounded DNS
controls, and the current per-host resolution status. Status refresh preserves
unsaved edits; if the saved revision changes, reset and refresh before editing
again. The validity column is a snapshot, not a live countdown. Every save
requires an explicit typed confirmation, the current ETag and the server
BanManage fresh-proof flow. No configuration is saved optimistically. Rescue
addresses must be checked against the operator's real trusted source before
enabling a policy. The UI does not test or guarantee that recovery access works.
