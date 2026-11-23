---
screen: true
---
# Inbound access

**Inbound access** limits which client IP addresses may use the admin console, and,
separately, the webmail and its APIs. Each of the two policies is off by default.
When a policy is on, a request from an address that is not on its list is refused
with `403`, before any sign-in or session check. Open it in the console under
**Security centre** > **Inbound access** (`#/security/inbound`).

![The Inbound access tab of the Security centre: Refresh saved policy and DNS status and the start of the Admin console and API card.](/screenshots/admin-access/inbound-allowlists-page.webp){.mw-shot}

Each surface has its own card. The admin card, with two rescue entries typed in but not saved:

![The Admin console and API card of Inbound access with two rescue entries typed in.](/screenshots/admin-access/inbound-allowlists-admin.webp){.mw-crop}

1. **Restrict this surface to the allowlist** turns the policy for this surface on.
2. **Rescue IPs and CIDRs**: literal addresses or networks, one per line.
3. **Allowed DNS hostnames**: names whose current addresses are also allowed.
{.mw-callouts}

The page has three cards: **Admin console and API**, **Webmail and account APIs**,
and **DNS resolution limits**, followed by **Current DNS grants** (the addresses
each hostname resolved to at the last refresh).

## Before you start

- You need **security.read** to view this page and **security.write** to change it,
  both granted for all domains. Saving also asks you to confirm it is you, unless you
  did in the last five minutes.
- Find the address the server sees for you: the **Trusted source IP** on
  **Security centre** > **Sign-in country policy** shows it. Behind a reverse proxy
  this is only your real address when the proxy is trusted (`MW_TRUSTED_PROXIES`,
  `MW_FORWARDED_MODE`; see [Reverse proxy](/deploy/reverse-proxy)). Otherwise every
  request appears to come from the proxy.

::: danger
The admin policy also covers the console's sign-in page and this editor. There is no
exemption for loopback addresses or for an administrator who is already signed in.
If you save a list that does not include your own address, you lose access to the
console at once, and getting it back needs a change on the server (see
[Recovering from a lockout](#recovering-from-a-lockout)).
:::

## Restrict a surface

1. Open **Security centre** > **Inbound access**.
2. In **Admin console and API** (or **Webmail and account APIs**), enter at least
   one address or network in **Rescue IPs and CIDRs**: one IPv4 or IPv6 address or
   CIDR per line, up to 128. Include an address you can always use for recovery.
3. Optionally add **Allowed DNS hostnames**, one lowercase ASCII name per line
   (`vpn.example.com`; international names in their `xn--` form). No URLs, ports,
   wildcards, IP addresses or trailing dots.
4. Select **Restrict this surface to the allowlist**.
5. Select **Review changes**. In **Apply inbound access policy?**, type `APPLY` and
   select **Apply confirmed policy**.

Nothing changes until the server accepts the save. If someone saved the policy while
you were editing, the save is refused; select **Reset draft to loaded policy** and
**Refresh saved policy and DNS status**, then edit again.

## What each policy covers

| Policy | Covers | Does not cover |
|---|---|---|
| **Admin console and API** | The console path and everything under it, including sign-in. | Anything else. |
| **Webmail and account APIs** | `/api`, `/jmap`, `/dav`, `/rc`, `/sso`, `/oidc`, `/oauth` and `/mcp` and everything under them: sign-in, mail, calendars and contacts over DAV, signed content links, single sign-on, API keys and WebSocket handshakes. | The web app's static files, `/healthz`, `/readyz`, `/metrics`, public key discovery (WKD) and client error reports. |

Neither policy covers SMTP, IMAP or other non-HTTP listeners, and neither replaces a
firewall. Open connections are not cut; each new request is checked, and a
WebSocket keeps the decision made at its handshake.

The policy does not change trusted proxies, API key scopes, request-protection
budgets or outbound access.

## Hostnames and DNS

A hostname allows only the addresses it currently resolves to (A and AAAA records),
never a whole network and never an address found by reverse lookup.

- Requests never wait for DNS. The server resolves the names in the background
  after a request or a status read, one refresh at a time.
- A name that has not resolved yet, has expired or failed allows nothing. An answer
  with more addresses than the limit allows nothing. After a failed lookup, the
  previous answer is kept only until its own expiry.
- An answer is kept for its DNS time-to-live, but never longer than the cache limit.
- With **Recursive DNS servers** empty, the operating system's resolvers are used.
  Up to four `IP:port` entries (`10.0.0.53:53`, `[fd00::53]:53`) replace them, with
  no fallback. The answers are only as trustworthy as those resolvers; Mailwoman does
  not validate DNSSEC.

| Field | Default | Range |
|---|---|---|
| **Lookup timeout (milliseconds)** | 2000 | 100 to 5000 |
| **Maximum cache lifetime (seconds)** | 300 | 30 to 3600 |
| **Maximum distinct hostnames** | 64 | 1 to 64 (across both policies) |
| **Maximum addresses per hostname** | 16 | 1 to 32 |
| **Concurrent lookups** | 4 | 1 to 4 |

**Current DNS grants** is a snapshot from the last refresh: for each name, whether it
grants access, the **Usable addresses**, **Validity at last refresh** and any **DNS
result code**. It is not a countdown and does not test connectivity.

## Several servers and failures

Every protected request reads the saved policy from the database (with a two-second
time limit), so a change applies on every server that shares the database without
waiting for DNS. If the policy cannot be read, or the stored policy is invalid,
requests to the covered paths are refused with `503`.

## Recovering from a lockout

Signing in does not get past the policy. To recover, change the stored policy on the
server: it is kept in the database under the key `security.inbound-allowlist.v1`.
Correct or remove that entry with the database tools for your deployment, then reload
the console from an allowed address.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Saving is refused with "Each enabled surface requires at least one rescue IP or CIDR." | A policy is on with an empty **Rescue IPs and CIDRs** list. | Add at least one address or network. |
| A hostname shows **No current grant** | It has not resolved yet, the lookup failed, or it returned more addresses than allowed. | Check the **DNS result code**, the resolvers and **Maximum addresses per hostname**. |
| Everyone is refused after turning the webmail policy on | The server sees the reverse proxy's address for every request. | Configure the proxy as trusted, or add its address only if every client really comes through it. |

## Related

- [IP ban list](./ip-bans.md): refuse individual addresses
- [Sign-in country policy](./signin-geography.md)
- [Security centre](./security-center.md)
- [Reverse proxy](/deploy/reverse-proxy)
