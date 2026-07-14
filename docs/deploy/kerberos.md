# EWS native Kerberos — BYO reverse-proxy (SPEC §6.5 R2)

> **Status:** scaffold (t10-e0). Filled by t10-e12 (docs + decision memo only — NO
> GSSAPI/C dependency is built).

## The constraint (honest)

Native Kerberos/SPNEGO (GSSAPI) needs a non-permissive, C-linked library (MIT
Kerberos / Heimdal / `libgssapi-sys`), which the project's **permissive-only,
no-openssl / no-`-sys`-C license floor forbids**. Mailwoman therefore does **not**
ship native Kerberos autonomously. Native support is **flagged as a human
license-floor-exception decision** — a feature-gated, off-by-default, non-permissive
dependency — not an autonomous build.

## Default path: BYO SPNEGO reverse-proxy

Deployments that need Kerberos SSO to on-prem Exchange put a **SPNEGO-terminating
reverse proxy** (IIS/ARR, Apache `mod_auth_gssapi`, or nginx with the SPNEGO module)
in front of EWS. The proxy negotiates Kerberos with the client/KDC and forwards an
authenticated request upstream; Mailwoman's EWS bridge talks to the proxy over the
already-shipped **NTLMv2 / Basic** path.

<!-- e12: fill the concrete IIS/ARR + Apache mod_auth_gssapi + nginx configs, the
keytab/SPN setup, a tested run against a fixture SPNEGO proxy, and the decision memo
flagging native Kerberos as a human license-floor-exception. -->
