# Release notes — 26.8 (V7)

V7 delivers the last features on the SPEC roadmap before the 1.0 hardening milestone:
the WASM engine-plugin runtime, the LDAP/GAL directory, password-change backends,
Assist (AI), the Graph/EWS/Gmail bridges, MSG/OFT/DOCX export, and Nextcloud.

**V7 completion is not 1.0.** 1.0 is a separate hardening / accessibility / i18n /
audit milestone — see [`ROADMAP-1.0.md`](./ROADMAP-1.0.md).

## What's new

- **WASM plugin runtime** (`mw-plugin`, wasmtime + WASI-p2) — capability-gated,
  resource-limited, Ed25519-signed plugin host. See
  [`security/plugins.md`](./security/plugins.md).
- **LDAP/GAL directory** (`mw-directory`, read-only) — GAL search, group expand,
  S/MIME cert and photo lookup, LDAP-bind login. See [`deploy/ldap.md`](./deploy/ldap.md).
- **Password-change backends** (`mw-passwd`) — local / LDAP RFC 3062 / Dovecot HTTP /
  poppassd / HMAC webhook, plus zero-access re-wrap and sealed-credential re-seal. See
  [`security/password-change.md`](./security/password-change.md).
- **Assist (AI)** (`mw-assist`) — BYO-endpoint (OpenAI-compatible / Anthropic /
  local-process), capability-scoped, content-free audit, send always human-gated. See
  [`assist.md`](./assist.md).
- **Bridges** (`plugins/bridge-{graph,ews,gmail}`) — Microsoft Graph, on-prem EWS, and
  Gmail as WASM plugins. See [`bridges/`](./bridges/).
- **MSG/OFT/DOCX export** (`mw-export`) — Outlook message/template + Word export. See
  [`export/msg-oft-docx.md`](./export/msg-oft-docx.md).
- **Nextcloud** — attach/save/share-link + auto-configured CalDAV/CardDAV/tasks. See
  [`integrations/nextcloud.md`](./integrations/nextcloud.md).
- **V6 follow-ups closed:** (a) proxy-mode headless scoped-key REST reads now resolve
  via `sessions_by_account`; (b) the MCP unattended-send countersign resolver is real
  (reads the admin flag on the key), no longer a stub.

## Scope boundaries — read these (no overclaim)

Three limits are stated plainly here and in the relevant docs:

1. **Bridges deliver MAIL through the real plugin jail; PIM/reactions are implemented
   but not yet seam-wired.** The frozen WIT world exports the **account-backend (MAIL)**
   interface only. Each bridge additionally **implements and fixture-tests**
   calendar/tasks/reactions/voting/recall/free-busy (advertised via `capabilities()`),
   but those are **not yet drivable through the plugin seam** — that WIT export is a
   post-1.0 extension. Until then, the UI's existing standards/header-convention
   fallbacks handle those surfaces. (`docs/bridges/*.md`,
   `docs/security/plugins.md`.)
2. **EWS Kerberos is a BYO-reverse-proxy gap.** On-prem EWS ships **Basic + pure-Rust
   NTLMv2** (hand-rolled, zero new deps). Kerberos/GSSAPI SSO is not shipped — a
   pure-Rust Kerberos stack is not viable within the permissive license floor. Front
   EWS with a Kerberos-terminating reverse proxy as the interim path; native Kerberos
   is post-1.0. (`docs/bridges/ews.md`.)
3. **quick-xml DoS bounded-ignore (docx-rs, write-only).** `docx-rs` transitively pins
   quick-xml 0.36.2 with two reader-side DoS advisories (RUSTSEC-2026-0194 / -0195).
   Mailwoman uses `docx-rs` for DOCX **writing only** and never parses untrusted
   `.docx`/XML through it, so the vulnerable reader path is unreachable; it is a
   client-side export, not a network-reachable parser. This is a bounded, documented
   `cargo deny` ignore; every other quick-xml consumer in the tree is on the fixed
   0.41. (`deny.toml`, `docs/export/msg-oft-docx.md`.)

### Also deferred past 1.0

- The declarative **TypeScript UI-plugin tier** (§22.2) — V7 ships the engine (WASM)
  plugin tier only.
- **Calendar/tasks/reactions plugin-seam export** (item 1 above).
- **OAuth dynamic client registration** — bridges use Mailwoman as an OAuth *client*;
  admin-seeded/BYO client IDs suffice.
- **MSG/OFT deep write fidelity** (embedded objects, custom named properties) —
  best-effort; body + attachments + headers is the committed floor.
- **OIDC/SAML SSO** — not implemented; V7's committed auth scope is password-change +
  LDAP-bind login. Tracked in `ROADMAP-1.0.md`.

## Supply chain

`cargo deny check` is green. The only V7 license addition is
`Apache-2.0 WITH LLVM-exception` (wasmtime / cranelift / wit-bindgen — all permissive,
pure-Rust, no OpenSSL, no C `-sys`). The only V7 advisory ignores are the two bounded
quick-xml write-only advisories above. No `openssl` anywhere; `ldap3` is forced to
`tls-rustls-ring`.
