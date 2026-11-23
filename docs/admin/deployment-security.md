---
screen: true
---
# Deployment checks

The **Deployment checks** tab of the **Security centre** (`#/security/deployment`)
lists how the server process that answers you is configured for transport security,
cookies, CSRF, sessions, second factors, sign-in limits, country policy, encryption at
rest and document previews. Each row is an observation of this process and of your
request; the tab changes nothing.

![The Deployment checks tab with the search field, the Status filter and the first rows of the table.](/screenshots/admin-access/deployment-security-checks.webp){.mw-shot}

Each row has a **Check**, a **Status** (**Present**, **Review needed** or
**Information**), the **Observed value**, and under **Configuration** links to the
settings that change it. **Find a check** and **Status** filter the table;
**Refresh assessment** reads it again. **Back to diagnostics** opens the
**Diagnostics** screen. The same count of rows that need review appears on the
**Security posture** tile of the Security centre **Overview**.

Opening the tab needs **diagnostics.read** granted for all domains. The answer is
not cached and contains no secrets, environment values or file paths.

## The checks

| Check | **Review needed** when | Change it with |
|---|---|---|
| **HTTP admission and request limits** | never (information) | `MW_HTTP_*` limits, at start; see [Configuration](./configuration.md) |
| **Inbound connection negotiation limits** | never (information) | listener limits, at start |
| **HTTPS request** | your request did not reach the server over HTTPS, directly or through a trusted proxy | TLS on the server or at a trusted reverse proxy ([Reverse proxy](/deploy/reverse-proxy)) |
| **Secure cookies** | session cookies would be sent without `Secure` | serving over HTTPS (an `https://` `MW_PUBLIC_URL`, or a trusted TLS proxy), or `MW_COOKIE_SECURE` |
| **Strict transport security** | never; shown as information when the server sends no HSTS header | HTTPS configuration |
| **Strict CSRF protection** | `MW_CSRF_STRICT` is off (the server's default) | `MW_CSRF_STRICT=true`; see [Hardening](/deploy/hardening) |
| **Administrator session limits** | never (information: the idle and absolute limits in force) | **Console idle timeout**, **Console session limit** |
| **Enabled administrators without a second factor** | at least one enabled administrator has neither an authenticator app nor a passkey | each administrator: [Administrator authenticator](./authenticator.md) |
| **Mailbox factor enrollment** | never (information: how many known accounts have a second factor) | [Two-factor & sign-in requirements](./security-center.md#two-factor-sign-in-requirements) |
| **Sign-in retry delay (seconds)** | the delay is 0 | **Sign-in retry cooldown** ([`signin.retryCooldownSecs`](/admin/settings#signin.retrycooldownsecs)) |
| **Temporary credential-failure locks** | never; always **Present** with the limits in force | [Temporary sign-in locks](./signin-locks.md) |
| **Country policy** | the sign-in country policy is on but no country database is loaded | [Sign-in country policy](./signin-geography.md) |
| **Encryption at rest** | encryption warnings are on and there are open findings | [Encryption](./encryption.md) |
| **Document preview isolation** | an isolated renderer is required but no worker is configured | document worker settings |

The session limits shown are the values the server applies after its own bounds,
not just what was saved (see [Sessions](./sessions.md#settings)).

## Review the checks that need attention

1. Open **Security centre** > **Deployment checks**.
2. Set **Status** to **Review needed**.
3. For each row, read the **Observed value** and use the link under
   **Configuration**, or the setting named in the table above.
4. Select **Refresh assessment** after the change. Settings that are read at start,
   such as `MW_CSRF_STRICT`, need a server restart first.

![The checks filtered to Review needed on a server reached over plain HTTP.](/screenshots/admin-access/deployment-security-review.webp){.mw-crop}

The example is a demo server reached over plain HTTP and started with
`MW_CSRF_STRICT=true`: **HTTPS request** and **Secure cookies** need review because
the request is not HTTPS, **Enabled administrators without a second factor** because
the only administrator has none, and **Encryption at rest** because the database is
not encrypted as a whole (only sealed columns). Without `MW_CSRF_STRICT=true`,
**Strict CSRF protection** would be listed too.

## What the checks do not test

The tab lists these limits itself under **What this assessment does not test**:

- Public TLS certificates, reverse-proxy configuration and firewall exposure are not
  tested.
- Upstream authentication and mailbox access are not tested.
- The configured document worker is not run; a configured worker does not show that
  its isolation works.
- A reported setting is evidence of configuration, not a guarantee of protection.

Each server process reports on itself. Other servers behind the same load balancer,
and the reverse proxy in front, can be configured differently; check them separately.

## Related

- [Security centre](./security-center.md)
- [Hardening](/deploy/hardening)
- [Reverse proxy](/deploy/reverse-proxy)
- [Sessions](./sessions.md)
