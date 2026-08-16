# Routing outbound fetches through your own proxy

Mailwoman makes server-side outbound fetches (remote images, webcal, autoconfig,
key lookups, webhooks). 26.20 adds the machinery for an operator to require those
to go through an HTTP `CONNECT` or SOCKS5 proxy they control — see the status
note below for which parts of it are wired.

The threat model and the guarantees — including the one this design cannot make —
are in [`../security/egress.md`](../security/egress.md). **Read that before
deciding to enable this**; the summary is that Mailwoman still enforces its own
address policy and never asks the proxy to resolve a name, but it cannot prove
the proxy dialled the address it was given.

> ## ⚠️ STATUS — verify before publishing this page
>
> **This page was written before the feature was fully wired, and describes the
> intended complete behaviour.** At the time of writing, at commit `439ebb6`:
>
> * The transport (HTTP `CONNECT`, SOCKS5, IP-literal only), the config store
>   (migration `0026`, sealed password), the admin API (list / save / delete) and
>   the admin UI **have landed**.
> * **No fetch path consumes a configured route yet.** `list_egress_proxies` and
>   `get_egress_proxy` have no callers outside the store and the admin API, so
>   saving a route currently routes nothing. Wiring it into the hardened fetches
>   is a separate, later task.
> * **The route-test endpoint is not implemented.** The admin UI's Test button is
>   built against an agreed contract; the server half does not exist yet, so the
>   button cannot succeed.
>
> **Whoever tags the release must re-check both and either delete this banner or
> cut the corresponding sections.** Shipping this page as-is would describe a
> capability an operator cannot actually obtain — which is the exact class of
> claim this project has had to correct before.

---

## Configuring a route

Admin console → **Egress**. A route is:

| field | notes |
|---|---|
| **Id** | Your name for the route. It is the primary key: editing it would create a second route rather than rename this one, so it is fixed once saved. |
| **Scheme** | `http` (CONNECT) or `socks5`. |
| **Host**, **Port** | The proxy itself. |
| **Username** | Optional. Required if you set a password. |
| **Password** | Optional. **Write-only** — see below. |
| **Allow plaintext** | Off by default. Permits `http` origins through this route. |

### The password is write-only, and what that means when you edit

The server **never returns the password**, not even masked — the API carries only
a `hasCredentials` flag. So the admin form cannot show it to you, and cannot send
it back.

**Leave the password field blank to keep the stored one.** Editing a route's host
or port with the field blank preserves the credential; type a new value only when
you want to replace it. (Sending an explicit empty value clears it.)

This is worth stating because the alternative behaviour is a real hazard and was
briefly present during development: if an omitted password were written as empty,
every edit to an unrelated field would silently destroy the route's
authentication, and you would not find out until the next fetch failed.

---

## Testing a route

Each route has a **Test** button that performs a live attempt and reports what
happened. It reports an **outcome** and the **stage** it reached:

| outcome | meaning |
|---|---|
| `connected` | Reached the origin through the route. |
| `authRejected` | The proxy refused the credentials. |
| `refusedByPolicy` | Mailwoman's own address policy refused the target. |
| `dnsFailed` | The name did not resolve. |
| `unreachable` | The proxy could not be reached. |
| `originTlsFailed` | The tunnel opened, but the origin's TLS did not verify. |
| `routeInvalid` | The route's own configuration is not usable. |

The **stage** (`dns`, `connect`, `tunnel`, `origin`) is where it stopped, which
is usually what tells you which thing to fix: the same refusal at `tunnel` and at
`origin` are different faults. The result also states whether the proxy was
**actually traversed** — taken from the transport's own progress, not from the
fact that a route is configured.

A test that could not be *run* (the route no longer exists, you are not
authenticated, the probe itself failed) is reported separately from a verdict. It
means nothing was learned about the route, which is different from learning the
route is broken.

---

## Fail-closed, and what that looks like when it goes wrong

**If the proxy is unavailable, the fetch fails. It does not fall back to a direct
connection.** This is deliberate: a fallback would mean your routing requirement
silently stops applying exactly when the proxy is down.

The operational consequence is that **a briefly unavailable proxy looks like a
bug** — remote images stop loading, webcal refreshes fail. If you are diagnosing
that, the route's Test button is the fastest way to distinguish "the proxy is
down" from "Mailwoman refused the target", and the audit row records which
actually happened rather than what was configured.

---

## What is audited

Every route change — save and delete — writes an `audit_log` row. The row records
`scheme://host:port`, the only form of a route that may be logged, and never the
credentials. (`audit_log` is append-only by design: there is no update or delete
method, so a secret written into it could never be redacted afterwards. That is
why the rule is absolute rather than best-effort.)

Once fetches are routed, the audit row for a fetch is specified to record whether
the proxy was **actually** traversed — taken from the transport's own progress,
not from the configuration. Setting that field from the intended configuration
rather than the observed connection is exactly the bug that makes an audit log
worthless, so it is worth knowing it was designed against.

---

## Not supported (deliberately)

* **Per-origin routing rules.** One default route per deployment. Host-pattern
  routing is separately threat-modelled work, not an extension of this.
* **Per-account proxies.** An SSRF primitive with extra steps.
* **A user-selectable proxy.** Admin-only.
* **HTTP/2 over the tunnel, and connection pooling through a route.** No measured
  need; pooling in particular interacts with the DNS pin and would need its own
  "the pin still holds on hop N" proof.
