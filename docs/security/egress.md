# Outbound connections (egress)

The server makes HTTP requests of its own: remote images for the reader, calendar
subscriptions, account autoconfiguration, OpenPGP key lookups, webhooks, OAuth,
country-database downloads, AI Assist endpoints and more. In many of these a caller
influences the destination, so each is a server-side request forgery (SSRF) risk.
This page describes the address policy that applies to them, how an operator's proxy
fits in, and the limits of both.

Operator setup for proxy routes is in [Egress proxy](/deploy/egress-proxy).

## The address policy

Outbound HTTP requests go through the `mw-egress` crate, which applies one policy:

- **Schemes:** only `http` and `https`. URLs with a user name or password in them are
  refused.
- **Resolve once, connect to what was checked.** The server resolves the host name
  itself, picks the first address the policy allows, and pins the connection to that
  address. A name that answers differently on a second lookup (DNS rebinding) cannot
  move the connection after the check. An IP literal in the URL is checked directly.
- **Refused addresses:** loopback, private (RFC 1918), link-local (including the
  cloud metadata address `169.254.169.254`), carrier-grade NAT (`100.64.0.0/10`),
  multicast, broadcast, unspecified, documentation, benchmarking (`198.18.0.0/15`),
  reserved `240.0.0.0/4` and `192.0.0.0/24`; for IPv6, loopback, unspecified,
  multicast, unique-local `fc00::/7`, link-local `fe80::/10` and documentation
  `2001:db8::/32`.
- **Embedded IPv4 is decoded.** IPv4-mapped and IPv4-compatible IPv6 addresses, the
  NAT64 well-known prefix `64:ff9b::/96`, 6to4, Teredo (both embedded addresses) and
  ISATAP are unwrapped and the IPv4 rules applied, so `::ffff:127.0.0.1` is refused
  as loopback. NAT64 network-specific prefixes are site configuration and are not
  decoded; a deployment that uses one needs its own network ACL.
- **Redirects are followed by hand.** Each `Location` goes through the same checks
  from the start, so a permitted host cannot redirect into a refused address.
- **Caps:** 10 seconds per request by default, at most 4 redirects, and a response
  size limit set per use (8 MiB for images; bulk downloads such as the country
  database have their own ceiling).
- **Nothing from the reader is forwarded:** no cookies, `Referer`, `Authorization`
  or browser `User-Agent`; each use sends a fixed `User-Agent`.
- **No ambient proxy.** Every HTTP client the server builds calls `no_proxy()`, so an
  `HTTP_PROXY`, `HTTPS_PROXY` or `ALL_PROXY` variable in the server's environment is
  ignored. (A proxy reached that way would resolve the name itself and defeat the
  pin.) A test scans the source and fails for any HTTP client built without it.

The policy decides where a request may go. It does not decide how often: rate limits
belong to each caller (the image proxy, for example, has a per-account limit).

### Profiles that allow private addresses

Some uses have a legitimate reason to reach the local network. Each is an explicit
operator setting, and none of them opens the metadata address:

| Use | Setting | What it allows | Still refused |
|---|---|---|---|
| Account autoconfiguration (looks up the domain from the address a user types) | `MW_AUTOCONFIG_ALLOW_PRIVATE`, default off | Private ranges | Loopback, link-local (`169.254.0.0/16`, `fe80::/10`), the AWS IPv6 metadata address, and the same ranges inside any embedded IPv4 form |
| Mail-server connections to an operator-configured endpoint | the endpoint's allow-private option | Private ranges and same-host loopback | Metadata, link-local and reserved addresses |
| Webhooks and SIEM delivery | the internal destination list in the admin console | Listed internal addresses, networks (at least `/8` IPv4 or `/16` IPv6) or one host name, optionally with a port | Loopback, link-local, the metadata address and their NAT64/6to4/Teredo/ISATAP forms, refused when an entry is saved and again on each request |

OpenPGP key-server and WKD lookups always use the strict policy: they carry an
attacker-influenced key id and have no reason to reach a private address.

On a hosted multi-tenant deployment, leave `MW_AUTOCONFIG_ALLOW_PRIVATE` off: with it
on, anyone who can type an e-mail address into the sign-in or account form can make
the server request an address on its local network.

## JMAP-proxy mode upstreams

In JMAP-proxy mode the upstream server is chosen in two steps: the sign-in request
names the server, and that server's session document names the API, download and
upload URLs that the proxy relays requests to with the user's credentials. Every
upstream request goes through one client constructor that enforces:

- **Same origin.** A relayed URL must have the same origin as the session's server
  URL. This is checked at sign-in for all three URLs and again on every relayed
  request.
- **Address policy.** If neither `MW_JMAP_UPSTREAMS` nor `MW_UPSTREAM_DEFAULT` names
  the origin, the strict policy applies, so private, loopback and metadata addresses
  are refused. If `MW_JMAP_UPSTREAMS` is set, only the listed origins are accepted,
  at whatever address they resolve to. An origin named in either variable is trusted
  at any address: naming an exact origin is how an operator uses an internal JMAP
  server.
- **Pinned, no ambient proxy, limited redirects.** The session lookup follows up to 4
  redirects on the same origin, checking and pinning each hop. Relayed API, download
  and upload requests follow none. The request timeout is 300 seconds.
- **Downloads are not rendered.** A relayed download is served with
  `Content-Disposition: attachment` and `X-Content-Type-Options: nosniff`, and any
  HTML, XML, script or multipart type is replaced with `application/octet-stream`.

## Routing through an operator proxy

An administrator can route outbound requests through their own HTTP `CONNECT` or
SOCKS5 proxy. Each request has a traffic class, a constant in the code that names what
it is for:

`remote_content`, `ics_feeds`, `keyservers`, `webhooks`, `admin_detect`, `pki`,
`dns_auth`, `unsubscribe`, `siem`, `analytics`, `nextcloud`, `oauth`,
`update_check`, `sieve`, `geoip`, `system_mail`, `plugins`, `package_tracking`,
`push`.

A route lists the classes it carries; a class that no route lists goes direct. A route
is global or scoped to one mail domain, and a domain route replaces the global route
for the classes it lists. The class and the signed-in account's domain pick the route;
no request field, header or destination does. A server process that has not loaded
the egress configuration refuses every class request rather than sending it direct.

![The Outbound proxy routes tab with no routes configured and the Add a route button.](/screenshots/security/egress-routes.webp){.mw-crop}

How the address policy is kept through a proxy:

- **The proxy gets an IP address, not a name.** The server resolves the name and
  applies the address policy as for a direct request, then asks the proxy to connect
  to that address. The SOCKS5 encoder has no code path for a domain-name address type.
- **TLS runs end to end to the origin.** The tunnel carries TLS with the origin's name
  (SNI), verified against the origin's certificate. A proxy that presents its own
  certificate fails verification.
- **Plain `http` origins are refused through a route** unless that route allows
  plaintext.
- **No fallback.** If the tunnel cannot be set up, the request fails; it is not
  retried direct.

Route kinds:

| Kind | Behaviour |
|---|---|
| `http` | HTTP `CONNECT` to an IP address. |
| `http` with TLS to the proxy | The same inside a TLS connection to the proxy, verified for the proxy's host name. The `Proxy-Authorization` credential is not sent in the clear. |
| `socks5` | SOCKS5 `CONNECT` to an IP address. |
| `socks5h` | SOCKS5 with the name resolved by the proxy (see below). |

On a routed request the strict policy applies and the internal destination list does
not: internal destinations are reachable only on a class that goes direct.

### What a proxy can still do

- **It sees where requests go.** The origin host name is in the HTTP `CONNECT` line,
  and the TLS SNI is sent in clear text. What the tunnel protects for an `https`
  origin is the content of the request and response.
- **It can connect somewhere other than the address it was given.** For an `https`
  origin, certificate verification makes such a request fail rather than talk to the
  wrong host. For a plain `http` origin on a route that allows plaintext, nothing
  bounds this.

The address policy protects your network from destinations a caller can name. It
does not protect you from your own proxy.

### Remote DNS (`socks5h`)

A `socks5h` route sends the origin's name to the proxy, so the proxy decides which
address it connects to and the server cannot check that address. Such a route is
refused until an administrator with the egress capability explicitly accepts that
hosts reachable from the proxy's network can be fetched. Who accepted it and when is
stored with the route and recorded in the admin audit log; changing the route's kind,
host, port or remote-DNS setting clears the acceptance.

What still applies on a `socks5h` route:

- An IP literal in a URL is checked by the server and sent as an address; private,
  loopback and metadata literals are refused.
- A name must contain a dot and must not match the list in
  `crates/mw-egress/data/remote_dns_denylist.txt`: `localhost`, `*.localhost`,
  `*.local`, `*.localdomain`, `*.internal`, `*.intranet`, `*.lan`, `*.home`,
  `*.home.arpa`, `*.corp`, `*.private`, `*.arpa`, `*.svc`, `*.cluster.local`,
  `*.consul`, `metadata.google.internal`, `metadata.goog`, `instance-data`,
  `instance-data.ec2.internal`.
- Redirects are checked the same way; TLS to the origin, the plaintext rule and the
  size and time limits are unchanged.

What does not apply: a public name that resolves, at the proxy, to an address inside
the proxy's network can be fetched. The denylist covers names that point inward by
convention, not names that do so through DNS.

## Remote images in messages

The reader's remote-content mode is
[`privacy.remoteContent`](/admin/settings#privacy.remotecontent):

| Mode | What happens |
|---|---|
| `block` | Remote images never load. |
| `ask` (default) | Remote images are blocked until you choose **Show images** or one of the **Always show images from …** options. They then load through the server. |
| `proxy` | Remote images load through the server for every message, without asking. |
| `direct` | The browser loads images straight from the sender's servers. Available only when the administrator turns on [`privacy.allowDirect`](/admin/settings#privacy.allowdirect) and the account has [`cap.remote_content.direct`](/admin/settings#cap.remote_content.direct) (both default off); otherwise it behaves as `ask`. |

![The remote content notice above a newsletter, with Show images and More image options.](/screenshots/security/remote-images-banner.webp){.mw-crop}

**Through the server** means the reader's browser contacts only Mailwoman. The server
fetches the image under the address policy above, checks the response type against
its magic bytes, refuses SVG, re-encodes the image to PNG without metadata in a
WebAssembly sandbox, and caches the result. The sender's server sees the Mailwoman
server's address, not the reader's, but a fetch still happens when the message is
opened, so a tracking image can still tell the sender that, and roughly when, the
message was displayed. With [`privacy.trackers`](/admin/settings#privacy.trackers)
on (the default), known tracker URLs are removed and images that decode to 2×2 pixels
or smaller are replaced with a transparent pixel.

**Direct** mode exposes the reader's IP address, browser and open time to whoever
hosts the image.

### What a grant covers

In `ask` mode the server serves an image only if a grant covers the message: `single`
(this message), `per-sender` (the message's first From address), `per-domain` (that
address's domain) or `all`. The server re-checks the grant on every request, before
its cache, so revoking a grant stops loading at once.

Grants are per account and per message, not per URL, and an image's host has no
relation to the sender's domain. A session holding a grant for a message can
therefore have the server fetch any public URL under that message's id. The address
policy and the per-account rate limit
([`privacy.ratePerMin`](/admin/settings#privacy.ratepermin), default 1200, and
[`privacy.rateBurst`](/admin/settings#privacy.rateburst), default 1024) still apply.
The grant protects the reader from being tracked by a sender they have not opted
into; it is not a URL allowlist and does not restrict the account holder.

The message frame has an opaque origin and cannot send the session cookie, so the
reader asks the server for signed image URLs. Each is an HMAC-SHA256 token, under a
key derived from the server key, bound to the account, the session, the message, the
URL and an expiry (one hour for images, five minutes for message parts and linked
files). Signing out or revoking the session invalidates its tokens.

## Reporting

Security contact and disclosure policy:
[SECURITY.md](https://github.com/supermarsx/mailwoman/blob/master/SECURITY.md).

## Related

- [Egress proxy setup](/deploy/egress-proxy)
- [Plugin security](./plugins.md)
- [AI Assist: data flows](/assist)
