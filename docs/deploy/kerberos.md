# EWS Kerberos SSO — BYO reverse-proxy (SPEC §6.5 R2)

> **Status:** BYO reverse-proxy is the shipped, supported path for Kerberos SSO to
> on-prem Exchange. Native GSSAPI Kerberos is **not built** — it is flagged as a
> human license-floor-exception decision (see the memo at the bottom of this file).
> Basic + pure-Rust NTLMv2 ship natively (see `docs/bridges/ews.md`).

## The constraint (honest)

Native Kerberos/SPNEGO (GSSAPI) needs a non-permissive, C-linked library (MIT
Kerberos / Heimdal / `libgssapi-sys`), which the project's **permissive-only,
no-openssl / no-`-sys`-C license floor forbids**. There is no production-grade
pure-Rust GSSAPI/Kerberos stack that fits the floor, so Mailwoman does **not** speak
Kerberos itself. Instead, a Kerberos-capable **reverse proxy you already run** (or
stand up) terminates Kerberos in front of EWS, and Mailwoman's EWS bridge consumes the
proxied endpoint over its already-shipped **Basic / NTLMv2** path.

This is not "Kerberos supported." It is **Kerberos terminated at a BYO proxy**; the
bridge never holds a Kerberos ticket. State it plainly to operators.

## Architecture: two authentication legs

```
  Mailwoman EWS bridge            Reverse proxy                 On-prem Exchange
  (host http-fetch,               (Kerberos-capable:            (EWS endpoint,
   net_allowlist = proxy)         IIS+ARR / Apache / nginx)      Kerberos-protected)
        │                               │                              │
        │  ── Basic or NTLMv2 ─────────▶│                              │
        │     over TLS  (front leg)     │  ── Kerberos / SPNEGO ──────▶│
        │                               │     (KCD / S4U, back leg)    │
        │  ◀───────── proxied EWS SOAP response ───────────────────────│
```

- **Front leg (bridge ↔ proxy):** Basic or NTLMv2 over TLS — the path Mailwoman
  already ships. Mailwoman never negotiates Kerberos.
- **Back leg (proxy ↔ EWS):** Kerberos, spoken **entirely by the proxy**. The proxy
  translates the front-leg credential (or a service identity) into a Kerberos ticket
  for the EWS SPN, typically via **Kerberos Constrained Delegation (KCD) / protocol
  transition (S4U2Self + S4U2Proxy)**.

Because the back leg is where Kerberos lives, the proxy must be able to *originate*
Kerberos to the upstream — that is a KCD/impersonation configuration, not merely
"accept Negotiate from a browser." IIS+ARR and Apache `mod_auth_gssapi` both support
this; nginx's SPNEGO module is oriented at the downstream-Negotiate case and is a
weaker fit for upstream KCD (see Recipe C).

---

## Recipe A — IIS + Application Request Routing (ARR) + KCD  *(recommended for AD-joined estates)*

The Microsoft-native pattern (the same pre-authentication publishing TMG / Web
Application Proxy used). IIS accepts Basic over TLS from Mailwoman and uses KCD to
obtain a Kerberos ticket to EWS on behalf of the supplied user.

1. **Install** the ARR and URL Rewrite modules on a domain-joined Windows Server, and
   run the ARR app pool under a dedicated service account (e.g. `EXAMPLE\proxysvc`).
2. **Front-leg auth** on the proxy site: enable **Basic** (over TLS only) so Mailwoman
   can authenticate with the account credentials.
3. **Server farm:** create an ARR server farm pointing at the Exchange server(s);
   enable SSL to the backend.
4. **KCD delegation:** allow `proxysvc` to delegate to the EWS SPN. Classic
   constrained delegation sets, on the `proxysvc` account,
   `msDS-AllowedToDelegateTo = HTTP/mail.example.com` (use *resource-based* KCD on the
   Exchange computer account if you prefer). "Use any authentication protocol"
   enables protocol transition (S4U2Self) so a Basic front leg becomes a Kerberos
   back leg.
5. **Forward the KCD identity:** in `applicationHost.config`, set
   `<serverRuntime authenticatedUserOverride="UseAuthenticatedUser" />` on the site so
   ARR forwards the impersonated Kerberos identity upstream.
6. **URL Rewrite** inbound rule routes `^EWS/(.*)` to the farm.

Endpoint Mailwoman points at: `https://ews-proxy.example.com/EWS/Exchange.asmx`.

---

## Recipe B — Apache httpd + `mod_auth_gssapi` + `mod_proxy`  *(Linux)*

`mod_auth_gssapi` accepts a Basic username/password from Mailwoman, performs the
Kerberos AS exchange against the KDC to obtain credentials, then uses S4U2Proxy to
delegate to the EWS SPN; `mod_proxy` forwards the request.

```apache
# /etc/apache2/sites-enabled/ews-proxy.conf  (sketch — adjust to your distro layout)
<VirtualHost *:443>
    ServerName ews-proxy.example.com
    SSLEngine on
    SSLCertificateFile      /etc/ssl/ews-proxy.crt
    SSLCertificateKeyFile   /etc/ssl/ews-proxy.key

    <Location "/EWS/">
        AuthType GSSAPI
        AuthName "EWS Kerberos"

        # Accept Basic from Mailwoman and turn it into a Kerberos credential.
        GssapiBasicAuth      On
        GssapiBasicAuthMech  krb5
        GssapiCredStore      keytab:/etc/apache2/http.keytab
        GssapiCredStore      client_keytab:/etc/apache2/http.keytab

        # Constrained delegation (S4U2Proxy) to the upstream EWS SPN.
        GssapiDelegCcache    /run/apache2/krbcache
        GssapiUseS4U2Proxy   On
        GssapiImpersonate    On

        Require valid-user

        SSLProxyEngine       on
        ProxyPass            https://mail.example.com/EWS/
        ProxyPassReverse     https://mail.example.com/EWS/
    </Location>
</VirtualHost>
```

The proxy host account must itself be trusted for constrained delegation to
`HTTP/mail.example.com` in AD (same `msDS-AllowedToDelegateTo` / resource-based KCD as
Recipe A). Exact upstream-credential wiring varies by `mod_auth_gssapi` version — treat
the block as a sketch and validate against a fixture proxy (below) before production.

---

## Recipe C — nginx + SPNEGO module  *(downstream-Negotiate only — note the limitation)*

`spnego-http-auth-nginx-module` lets nginx **accept** `Negotiate` from a downstream
client and validate it against a keytab:

```nginx
server {
    listen 443 ssl;
    server_name ews-proxy.example.com;
    ssl_certificate     /etc/nginx/ews-proxy.crt;
    ssl_certificate_key /etc/nginx/ews-proxy.key;

    location /EWS/ {
        auth_gss on;
        auth_gss_realm EXAMPLE.COM;
        auth_gss_keytab /etc/nginx/http.keytab;
        auth_gss_service_name HTTP/ews-proxy.example.com;

        proxy_pass https://mail.example.com/EWS/;
    }
}
```

**Limitation (honest):** stock nginx does **not** originate Kerberos to the *upstream*
— it authenticates the downstream leg but forwards to EWS as an ordinary reverse proxy.
Since Mailwoman does not speak `Negotiate`, this recipe only helps when the back leg to
EWS accepts a non-Kerberos identity (e.g. an internal Basic listener) or when you place
Kerberos-originating logic elsewhere. For the credential-translation (Basic → Kerberos)
case that most on-prem deployments need, prefer **Recipe A (IIS+ARR+KCD)** or
**Recipe B (Apache mod_auth_gssapi)**.

---

## SPN and keytab setup (shared by all recipes)

Register a service principal for the proxy host and grant it constrained delegation to
the EWS SPN.

```powershell
# On a domain controller / admin box (Windows):

# 1. SPN for the proxy's HTTP service, mapped to its service account.
setspn -S HTTP/ews-proxy.example.com EXAMPLE\proxysvc

# 2. Constrained delegation: proxysvc may delegate to the EWS SPN.
#    (GUI: proxysvc account → Delegation → "Use any authentication protocol"
#     → add HTTP/mail.example.com. Or set the attribute directly.)
Set-ADUser proxysvc -Add @{ 'msDS-AllowedToDelegateTo' = 'HTTP/mail.example.com' }

# 3. Export a keytab for a Linux proxy (Recipe B/C).
ktpass -princ HTTP/ews-proxy.example.com@EXAMPLE.COM `
       -mapuser EXAMPLE\proxysvc -pass * `
       -crypto AES256-SHA1 -ptype KRB5_NT_PRINCIPAL -out http.keytab
```

On the Linux proxy, verify the keytab and clock skew (Kerberos requires < 5 min skew):

```bash
klist -kte /etc/apache2/http.keytab      # confirm principal + enctypes
kinit -kt /etc/apache2/http.keytab HTTP/ews-proxy.example.com@EXAMPLE.COM
timedatectl                              # NTP-synced; skew < 5 min
```

Ensure the EWS server publishes an SPN Kerberos can target (commonly
`HTTP/mail.example.com`, registered on the Exchange computer or service account).

---

## Wiring Mailwoman's EWS bridge to the proxy

The bridge reaches the proxy through the host `http-fetch` import under its manifest
`net_allowlist` — it opens **no sockets** and can reach **only** the hosts in that
allowlist (see `docs/security/plugins.md`). To use a BYO Kerberos proxy:

1. **Point the account at the proxy**, not at Exchange directly:
   `https://ews-proxy.example.com/EWS/Exchange.asmx`.
2. **Allowlist the proxy host** in the EWS bridge's `net_allowlist` so `http-fetch` is
   permitted to reach it:

   ```toml
   # plugins/bridge-ews manifest (excerpt)
   capabilities = ["account-backend", "net"]
   net_allowlist = ["ews-proxy.example.com"]   # the proxy, NOT mail.example.com
   ```

   With Exchange no longer contacted directly, `mail.example.com` need not be in the
   allowlist — the proxy re-originates TLS to it on the back leg.
3. **Provide credentials the proxy's front leg accepts** — Basic or NTLMv2, exactly the
   account-add flow already documented in `docs/bridges/ews.md`. TLS terminates at the
   proxy; the proxy performs the Kerberos back leg on the account's behalf.

From the engine's perspective nothing changes: the EWS account still loads as a normal
account-backend and syncs mail through the real jail; the Kerberos hop is invisible to
Mailwoman and lives entirely in operator-owned infrastructure.

## Verifying against a fixture proxy

The BYO path is exercised in CI against a **fixture reverse proxy** that stands in for
the SPNEGO terminator: the bridge is configured with the proxy host in `net_allowlist`
and Basic/NTLMv2 front-leg credentials, the fixture proxy accepts the front leg and
replays recorded EWS SOAP responses on the back leg, and the recorded exchange is
driven through the real engine (the same harness as the `bridge-fixtures` job in
`docs/bridges/ews.md`). This proves the bridge → proxy wiring, the allowlist gate, and
the front-leg auth without requiring a live KDC in CI. A live KDC + real Exchange is
left to the secret-gated `live-interop` job where those secrets are present.

---

## Native Kerberos — license-floor decision (HUMAN-GATED)

**This section records a decision a maintainer/human must make; Mailwoman does not
make it autonomously.**

Native GSSAPI Kerberos (so the EWS bridge speaks `Negotiate`/SPNEGO directly, with no
BYO proxy) is **technically buildable but blocked by policy**, not by capability. It
requires accepting **one non-permissive, `-sys`-C optional dependency** — the system
GSSAPI stack: `libgssapi` backed by **MIT krb5** or **Heimdal** (or an equivalent
`*-sys` crate linking them). That directly violates the project's current
**MIT-only / no-openssl / no-`-sys`-C license and supply-chain floor**
(`docs/deploy/hardening.md`, `deny.toml`, plan §7).

If a maintainer chooses to accept that exception, the constraints on any such build are:

- It **must be feature-gated and off by default** (e.g. a `kerberos-gssapi` cargo
  feature), so default builds keep the permissive/no-C floor and `cargo deny` stays
  green for everyone who does not opt in.
- It introduces a **C link + a non-MIT (MIT-krb5 is MIT-ish; Heimdal is BSD-3;
  distro packaging varies) system dependency**, so the default release artifacts,
  reproducibility posture, and `cargo deny` allowlist would all need explicit updates.
- It is a **deliberate license-floor exception**, documented as such, that only a
  human/maintainer can authorize. Mailwoman's autonomous pipeline will **not** add a
  C/`-sys`/non-permissive dependency on its own.

**Default and current state: not built.** The supported answer for Kerberos SSO is the
**BYO reverse-proxy** path above. Native Kerberos remains a flagged, human-gated
exception in `docs/ROADMAP-1.0.md`, not a shipped feature. Advertising "Kerberos
supported" without this proxy or this exception would be false.
