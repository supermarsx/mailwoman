# External-audit preparation dossier

This directory is the **preparation dossier an external security auditor consumes**
before and during a funded engagement against Mailwoman. It is produced as part of the
1.0 hardening milestone (`docs/ROADMAP-1.0.md`, SPEC §25/§27).

> **The funded external audit itself is HUMAN-GATED.** This dossier only *prepares* the
> engagement. The actual audit **run** — a paid, independent third party exercising the
> crypto / MCP / plugin-sandbox / Assist surfaces — and the **resolution of its
> findings** are a hard, human-gated condition of the real 1.0 tag (SPEC §25 lines
> 1393–1394; ROADMAP-1.0.md line 16). No document here, and nothing in this milestone,
> discharges that condition. This milestone makes Mailwoman **1.0-ready**, not 1.0-tagged.

## Contents

- [`threat-model.md`](./threat-model.md) — the consolidated threat model across every
  security-sensitive surface: client-side crypto + zero-access key hierarchy, the MCP
  server, the WASM plugin sandbox, the Assist gateway, OAuth 2.1 / scoped API keys,
  at-rest encryption, and DLP. Per surface: assets, trust boundaries, adversary model,
  existing mitigations, and **residual risk**.
- [`surface-inventory.md`](./surface-inventory.md) — the security-surface inventory:
  network-reachable endpoints, trust-boundary crossings, every place untrusted input is
  parsed, the crypto primitives + their crates, the sandbox boundary, and the `deny.toml`
  bounded-ignore rationale.
- [`self-baseline.md`](./self-baseline.md) — how an auditor reproduces the baseline that
  already exists: the `zap-baseline` ZAP scan, `cargo deny check`, the JS license gate,
  and the live-E2E security assertions (Assist redaction, MCP send-gating, plugin
  capability denial, zero-access ciphertext-at-rest). What to run and where the current
  results live.

## Scope of an engagement (suggested)

An auditor should prioritise the four surfaces where a defect has the highest blast
radius and where Mailwoman makes the strongest claims:

1. **Client-side crypto & the zero-access key hierarchy** — the E2EE and at-rest promise.
2. **The WASM plugin sandbox** — arbitrary third-party code, host-mediated I/O.
3. **The MCP server & Assist gateway** — LLM-facing surfaces, prompt-injection blast
   radius, send-gating.
4. **The OAuth 2.1 AS + scoped API keys** — the authorization surface for all automation.

## Grounding

These documents summarise and cross-link the per-subsystem security docs already in
`docs/security/` — they do not restate them in full. The authoritative descriptions are:

- [`../crypto.md`](../crypto.md) · [`../zero-access.md`](../zero-access.md) ·
  [`../mcp.md`](../mcp.md) · [`../plugins.md`](../plugins.md) ·
  [`../../assist.md`](../../assist.md) · [`../api-keys-oauth.md`](../api-keys-oauth.md) ·
  [`../dlp.md`](../dlp.md) · [`../observability.md`](../observability.md) ·
  [`../password-change.md`](../password-change.md) · [`../max-security.md`](../max-security.md) ·
  [`../screen-capture.md`](../screen-capture.md) · [`../admin-panel.md`](../admin-panel.md)
- The supply-chain posture: [`../../../deny.toml`](../../../deny.toml).
- The coordinated-disclosure policy + honest scope boundaries: [`../../../SECURITY.md`](../../../SECURITY.md).
