# Exchange Web Services (EWS) bridge (V7)

The EWS bridge (`plugins/bridge-ews`) connects Mailwoman to on-premises Exchange
Server (2013–2019 / Subscription Edition) via Exchange Web Services (SOAP). It is a
first-party **WASM plugin** (`wasm32-wasip2`) implementing the engine account-backend
seam; once loaded, an EWS account looks like any other backend to the engine.

The SOAP subset is parsed/emitted with `quick-xml`. As with all bridges, the guest
uses the host `http-fetch` import — it opens no sockets.

## What it delivers

- **Mail** — sync/send/fetch through the real jail, indistinguishable from IMAP.
- Implemented and fixture-tested: calendar + free/busy + rooms, GAL, Out-of-Office
  (OOF), message-recall, and voting. **See the scope boundary below.**

## Authentication

EWS on-premises authentication supported in V7:

- **Basic** authentication.
- **NTLMv2** — a **pure-Rust, hand-rolled** implementation (MD4/MD5/HMAC-MD5, verified
  against RFC / MS-NLMP known-answer test vectors). **Zero new dependencies**, no
  GSSAPI, no C.

### Kerberos is a documented gap (BYO reverse-proxy)

**Kerberos / NTLM single sign-on via the system GSSAPI is not shipped in V7.** A
pure-Rust Kerberos/GSSAPI stack is not viable within Mailwoman's permissive-only
license floor. If your deployment requires Kerberos SSO:

- Front EWS with a **reverse proxy that terminates Kerberos** and presents Basic or
  NTLM to Mailwoman (the documented interim path), or
- use Basic / NTLMv2 directly where your Exchange allows it.

Native Kerberos is tracked for post-1.0 (`docs/ROADMAP-1.0.md`).

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

- **Mail is delivered through the real plugin jail.** The WIT world currently exports
  the **account-backend (MAIL)** interface only.
- **Calendar / free-busy / rooms / GAL / OOF / recall / voting are implemented and
  fixture-tested but not yet drivable through the plugin seam** — a post-1.0
  WIT-export extension. The UI's existing standards fallbacks handle those meanwhile.
- **Kerberos SSO is a BYO-reverse-proxy gap** (Basic + NTLMv2 ship). See above.
- The crate uses `deny(unsafe_code)` rather than `forbid` because the generated
  wit-bindgen glue is unsafe-by-construction and localized; the bridge's own logic is
  unsafe-free, and `mw-plugin` remains the `forbid(unsafe_code)` trust boundary.
