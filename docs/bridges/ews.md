# Exchange Web Services (EWS) bridge (V7)

The EWS bridge (`plugins/bridge-ews`) connects Mailwoman to on-premises Exchange
Server (2013–2019 / Subscription Edition) via Exchange Web Services (SOAP). It is a
first-party **WASM plugin** (`wasm32-wasip2`) implementing the engine account-backend
seam; once loaded, an EWS account looks like any other backend to the engine.

The SOAP subset is parsed/emitted with `quick-xml`. As with all bridges, the guest
uses the host `http-fetch` import — it opens no sockets.

## What it delivers

- **Mail** — sync/send/fetch through the real jail, indistinguishable from IMAP.
- **Calendar + free/busy + rooms, GAL, Out-of-Office (OOF), message-recall, and
  voting** — implemented, mapped, and fixture-tested. The bridge targets the
  `plugin-pim` WIT world, so these PIM/parity surfaces are exported across the plugin
  seam (not mail-only) and round-trip through the real jail in the recorded-fixture
  suite. **See the scope boundary below** for what is seam-proven vs.
  user-surface-complete.

## Authentication

EWS on-premises authentication supported in V7:

- **Basic** authentication.
- **NTLMv2** — a **pure-Rust, hand-rolled** implementation (MD4/MD5/HMAC-MD5, verified
  against RFC / MS-NLMP known-answer test vectors). **Zero new dependencies**, no
  GSSAPI, no C.

### Kerberos SSO is via a BYO reverse-proxy (not native)

**Kerberos / SPNEGO single sign-on via the system GSSAPI is not spoken by the bridge.**
A production pure-Rust Kerberos/GSSAPI stack is not viable within Mailwoman's
permissive-only, no-openssl / no-`-sys`-C license floor, so the bridge does not hold a
Kerberos ticket. This is **not** "Kerberos supported"; it is one of:

- **BYO reverse-proxy (the supported path).** Front EWS with a Kerberos-capable reverse
  proxy (IIS+ARR+KCD, Apache `mod_auth_gssapi`, or nginx) that terminates Kerberos on
  the back leg and accepts the bridge's already-shipped **Basic / NTLMv2** on the front
  leg. Concrete config sketches, SPN/keytab setup, and the `net_allowlist` wiring are in
  **`docs/deploy/kerberos.md`**.
- **Basic / NTLMv2 directly** where your Exchange accepts them without Kerberos.

**Native GSSAPI Kerberos is a human-gated license-floor exception**, not an autonomous
build — it would require accepting a non-permissive `-sys`-C GSSAPI dependency,
feature-gated and off by default. See the decision memo at the bottom of
`docs/deploy/kerberos.md`. It is tracked as such in `docs/ROADMAP-1.0.md`.

### App registration / endpoint

On-prem EWS is typically reached at `https://mail.example.com/EWS/Exchange.asmx`.
Provide the EWS URL and the account credentials (Basic or NTLMv2) when adding the
account. There is no cloud app registration for on-prem EWS; for Exchange Online use
the **Graph bridge** instead.

## CI

The `bridge-fixtures` job replays recorded SOAP request/response pairs (including an
NTLM handshake) through the real engine. The nightly, secret-gated `live-interop` job
hits a real EWS endpoint only when secrets are present.

## Scope boundary (honest)

- **Mail is delivered through the real plugin jail** and is drivable end-to-end today.
- **Calendar / free-busy / rooms / GAL / OOF / recall / voting are implemented and
  exported through the `plugin-pim` WIT seam** (the `calendar`, `tasks`, and
  `bridge-parity` interfaces) and round-trip through the real jail in the fixture
  suite. What is not yet fully proven is the **end-to-end engine→CalDAV/JMAP
  user-facing surface** and **live-tenant interop** (the nightly, secret-gated
  `live-interop` job). Treat these surfaces as seam-proven and fixture-proven, not yet
  user-surface-complete against a live endpoint; where a surface is not yet driven
  through the engine end-to-end, the UI's existing standards fallbacks handle it.
- **Kerberos SSO is via a BYO reverse-proxy** (Basic + NTLMv2 ship natively; native
  GSSAPI is a human-gated license-floor exception). See above and
  `docs/deploy/kerberos.md`.
- The crate uses `deny(unsafe_code)` rather than `forbid` because the generated
  wit-bindgen glue is unsafe-by-construction and localized; the bridge's own logic is
  unsafe-free, and `mw-plugin` remains the `forbid(unsafe_code)` trust boundary.
