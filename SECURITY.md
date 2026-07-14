# Security policy

Mailwoman is a security-sensitive mail client: it handles end-to-end-encrypted mail,
private keys, an at-rest zero-access store, a WASM plugin sandbox, an MCP server, and an
AI Assist gateway. We take reports against any of these seriously and coordinate
disclosure with reporters.

This policy covers coordinated disclosure, supported versions, and the **honest scope
boundaries** of Mailwoman's security properties. The detailed threat model, surface
inventory, and self-run baseline for auditors live in
[`docs/security/audit-prep/`](docs/security/audit-prep/).

## Reporting a vulnerability

**Please report privately — do not open a public issue for a suspected vulnerability.**

- Preferred: **GitHub private vulnerability reporting** on
  <https://github.com/supermarsx/mailwoman> → *Security* → *Report a vulnerability*. This
  opens a private advisory thread with the maintainers.
- If you cannot use GitHub's private reporting, open a minimal public issue asking a
  maintainer for a private channel — **without** any vulnerability detail — and we will
  arrange one.

Please include: affected version/commit, the surface (crypto worker, plugin sandbox, MCP,
Assist, OAuth/API keys, zero-access, DLP, rendering, packaging), reproduction steps, and
impact. A proof-of-concept helps but is not required.

### Coordinated disclosure window

- We aim to **acknowledge** a report within a few days.
- We follow a **90-day coordinated-disclosure window** (SPEC §3): we work to ship a fix and
  publish an advisory within 90 days of a validated report, and we will credit reporters who
  wish to be credited.
- If a fix needs longer, we will say so and agree a revised timeline with the reporter. We
  ask reporters not to publicly disclose before the coordinated date.
- Fixes ship as a normal release; security-relevant releases carry a published advisory.

### Safe harbour

Good-faith security research that respects this policy, avoids privacy violations and
service disruption, and does not access or modify data beyond what is necessary to
demonstrate a finding, is welcome. Test only against your own instance or accounts you
control.

## Supported versions

Mailwoman uses a **rolling `YY.N`** version scheme (`VERSIONING.md`): `YY` is the year and
`N` increments per release, resetting each year. There is no long-term-support branch — the
project is developed in the open and moves forward on the rolling line.

| Version line | Supported |
|---|---|
| Latest rolling release (currently the `26.x` line, baseline `26.8.0`) | Yes — security fixes land here |
| Any older release | No — upgrade to the latest release |

Security fixes are applied to the current rolling line; users on older releases should
upgrade. (Note: SPEC §27 refers to a "1.0" maturity milestone; per `VERSIONING.md` and the
1.0 hardening plan, "1.0" is a maturity label on the rolling line — the actual release tag
remains rolling `YY.N` unless a documented scheme change is adopted.)

## Release integrity

Per SPEC §3, release artifacts are intended to be **signed (Sigstore/cosign) with SLSA
provenance**. Provisioning the signing accounts and store credentials is **human-gated** and
tracked as part of the 1.0 packaging work; verify signatures where published.

## Honest scope boundaries

Mailwoman does not overclaim. These boundaries are stated plainly so reporters and users
understand what is and is not protected. They are documented in full in
[`docs/security/audit-prep/surface-inventory.md`](docs/security/audit-prep/surface-inventory.md#7-honest-scope-boundaries-do-not-audit-as-shipped).

- **Zero-access protects data at rest, not a malicious active server.** It defends stored
  mail/PIM against a curious operator or a breached/stolen store. It does **not** defend
  against a fully malicious server that actively proxies your live IMAP/SMTP traffic. See
  [`docs/security/zero-access.md`](docs/security/zero-access.md).
- **Prompt injection is bounded, not solved.** The MCP server and Assist reduce the blast
  radius with provenance labels and least-authority scopes, but a client that ignores
  provenance or an over-granted key can still be steered by hostile mail. See
  [`docs/security/mcp.md`](docs/security/mcp.md).
- **PQC is groundwork.** The shipped post-quantum work is a hybrid X25519 + ML-KEM-768
  key-wrap of the at-rest store key; **`ml-kem` is unaudited** and this is not a user-facing
  E2EE claim. TLS hybrid is not enabled. See [`docs/security/crypto.md`](docs/security/crypto.md).
- **The plugin sandbox covers the engine (WASM) tier.** The TypeScript UI-plugin tier is
  not implemented; the plugin WIT exports the mail account-backend (calendar/tasks/reactions
  are fixture-tested but not yet seam-wired).
- **MCP unattended send** is unreachable without an admin-countersigned key; by default
  agent-initiated sends land in the Outbox for human confirmation.
- **DLP is advisory/best-effort**, not a confidentiality control; a determined insider can
  evade content detectors.
- **The screen-capture watermark** is a deterrent with stated limits, not a DRM control. See
  [`docs/security/screen-capture.md`](docs/security/screen-capture.md).
- **EWS Kerberos** is a documented bring-your-own gap (reverse-proxy auth); **OIDC/SAML SSO**
  is not built.

## External security audit (human-gated)

A **funded, independent external security audit** of the crypto, MCP, plugin-sandbox, and
Assist surfaces — **and the resolution of its findings** — is a hard, human-gated condition
of the actual 1.0 release tag (SPEC §25 / §27). The 1.0 hardening milestone **prepares** that
engagement: the threat model, surface inventory, and reproducible self-run baseline an
auditor consumes are in [`docs/security/audit-prep/`](docs/security/audit-prep/). Preparation
is complete; **the audit run and findings resolution have not yet occurred and cannot be
self-completed.** Until they do, treat the self-assessed properties above as pending
independent review.
