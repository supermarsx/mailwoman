# Self-run security baseline (external-audit prep)

This is the **starting baseline an external auditor reproduces on day one** using tooling
that already exists in the repository — no new CI or code is introduced by this dossier. It
tells an auditor exactly what to run, where the gate lives, and what "green" already means,
so the paid engagement starts from a known-good floor instead of rediscovering it.

> **HUMAN-GATED AUDIT.** Reproducing this baseline is **not** the audit. The funded
> external audit **run** (an independent third party actively testing crypto / MCP /
> plugin-sandbox / Assist) and the **resolution of its findings** are a hard, human-gated
> condition of the actual 1.0 tag (SPEC §25 lines 1393–1394; §27; ROADMAP-1.0.md line 16).
> This baseline is the *input* to that engagement, not a substitute for it.

All jobs referenced below live in the single workflow `.github/workflows/ci.yml` (baseline
release 26.8.0). Commands assume a checkout at the repo root.

## 1. Supply chain — licenses, advisories, bans, sources {#supply-chain}

**Gate:** the `deny` CI job → `cargo deny check licenses advisories bans sources` against
[`deny.toml`](../../../deny.toml). This is the **authoritative** supply-chain gate.

```sh
cargo install cargo-deny        # or use EmbarkStudios/cargo-deny-action@v2 as CI does
cargo deny check licenses advisories bans sources
```

**Current posture (26.8):** GREEN. Permissive-license floor (GPL/LGPL/AGPL denied by
omission); `openssl` + `sequoia-openpgp` banned; `yanked = "deny"`; unknown
registry/git denied. The only advisory ignores are the four **bounded, documented**
ignores (RSA-Marvin, quick-xml write-only DoS, Tauri unmaintained) — see
[`surface-inventory.md`](./surface-inventory.md#6-supply-chain-posture--the-denytoml-bounded-ignores)
for each boundary. `cargo deny check advisories` reports **zero `vulnerability`-class**
entries; the ignores are `unmaintained`/reader-DoS only.

**Auditor action:** confirm each bounded-ignore boundary empirically (no server-side RSA
decrypt; no untrusted DOCX/XML through `docx-rs`; Tauri advisories are Linux-desktop/build
only).

## 2. JS runtime licenses

**Gate:** the `js-licenses` CI job (currently `continue-on-error` — cargo-deny is the
authoritative floor; this tightens once verified).

```sh
pnpm -C apps/web dlx license-checker-rseidelsohn --production --summary \
  --onlyAllow "MIT;Apache-2.0;BSD-2-Clause;BSD-3-Clause;ISC;0BSD;Zlib;MPL-2.0;CC0-1.0;Unlicense;Python-2.0"
```

Runtime web deps only. The 1.0 milestone adds `axe-core` (**MPL-2.0**, dev/test only, never
shipped) to the allowlist — flagged for explicit vetting in the 1.0 plan.

## 3. Dynamic scan — OWASP ZAP baseline {#zap}

**Gate:** the `zap-baseline` CI job (SPEC §27 exit gate). It boots the mock stack and runs
the ZAP baseline spider. It is `continue-on-error` (the scan is slow/flaky) and uploads an
HTML report artifact (`zap-baseline-report.html`).

```sh
docker compose -f docker-compose.dev.yml up -d --build mock mailwoman
./scripts/wait-for-health.sh http://localhost:8080/healthz 240
docker run --network host -v "$PWD:/zap/wrk:rw" ghcr.io/zaproxy/zaproxy:stable \
  zap-baseline.py -t http://localhost:8080 -I -m 5 -r zap-baseline-report.html
docker compose -f docker-compose.dev.yml down -v
```

`-I` = warnings do not fail (only FAIL-level rules); `-m 5` caps the spider. **Auditor
action:** treat this baseline scan as a floor, then run an authenticated/active ZAP scan
(out of scope for the CI baseline) against the live surfaces in
[`surface-inventory.md`](./surface-inventory.md#1-network-reachable-endpoints-mw-server).

## 4. Live-E2E security assertions (already enforced)

These CI jobs assert the security-critical behaviors described in the threat model against
**real** services (or deterministic mocks), not unit stubs. An auditor should re-run them
and then attempt to falsify each assertion.

### Zero-access ciphertext-at-rest (`e2e-v6`)
The Rust live harness (`cargo test -p mw-server --test v6_e2e -- --test-threads=1`) spawns
an in-process server + JMAP mock against **real Postgres + Valkey** and drives the DoD
headline proof: a **direct `psql` query confirms rows are ciphertext at rest**, plus
SQLite⇄Postgres backend parity. Boots via
`docker compose -f docker-compose.ci.yml up -d --wait postgres valkey`.

**Auditor action:** independently query the DB and confirm no plaintext body/subject/index
is recoverable for a zero-access account; confirm caching is disabled for those accounts.

### MCP send-gating (`e2e-v6` / `e2e-v7` + `mw-mcp` tests)
The safety tests assert the transmit path is **never reached** without `send` scope, that a
`send`-without-`unattended_send` call **lands in the Outbox** (not transmitted), and that
`unattended_send` without an admin countersignature returns **403**.

**Auditor action:** attempt to reach `mail.send` transmission with every scope combination;
confirm the Outbox is the only outcome absent a countersigned key.

### Plugin capability + resource-limit denial (`jail`) {#plugin-jail}
`cargo test -p languagetool --test jail_load` loads the **real LanguageTool WASM
component** and asserts: it loads only when its capability is granted; `http-fetch` outside
`net_allowlist` is denied; the DLP hook is denied without its capability; and the
memory/deadline ceilings trip cleanly (typed error, no host crash).

```sh
rustup target add wasm32-wasip2
sh plugins/languagetool/build.sh
cargo test -p languagetool --test jail_load
```

**Auditor action:** author a hostile fixture plugin and attempt sandbox escape, off-
allowlist network, secret exfiltration (`oauth-token`), and resource exhaustion.

### Assist redaction / E2EE-never-forwarded / content-free audit (`assist-mock`)
`cargo test -p mw-assist -- --include-ignored` against a deterministic, offline
OpenAI-compatible (+ Anthropic) mock (`scripts/mock-assist`, `docker-compose.ci.yml` service
`mock-assist`) asserts capability grant/deny, redaction, **E2EE decrypted content is never
forwarded** (default exclusion), the **content-free audit** (rows contain no mail content),
and **send-always-gated**.

**Auditor action:** verify the redactor against realistic PII/secret corpora; confirm no
content path bypasses the data-class ceiling; confirm the audit truly stores no content.

### Directory (LDAP/GAL) live conformance (`directory-vs-openldap`)
`cargo test -p mw-directory -- --include-ignored` against **real OpenLDAP** (seeded LDIF),
including LDAPS/StartTLS — the tests self-SKIP with a clear log if `MW_TEST_LDAP_URL` is
unset (never a silent pass). Confirms `ldap3` uses **rustls, not native-tls/openssl**.

### Crypto E2E (`e2e-crypto`)
Playwright `--project=crypto` drives the V4 crypto/security flows against the engine + the
**WASM crypto worker** (greenmail + engine mode + a DLP rule), exercising the client-side
private-key model end-to-end.

### Supporting jobs
`wasm-plugin-build` (asserts every first-party plugin is a real WASM component preamble),
`bridge-fixtures` (recorded Graph/EWS/Gmail replay through the real engine),
`crypto-interop`, `mail-auth-verdicts`, `imap`/`managesieve`/`caldav-carddav` conformance.
The nightly `live-interop` job (secret-gated, non-blocking, never on PRs) exercises real
M365/Workspace/Ollama/OpenAI/Anthropic endpoints when secrets are present.

## 5. Baseline summary for the auditor

| Baseline | How | Current state | Auditor next step |
|---|---|---|---|
| Supply chain | `cargo deny check …` | GREEN; 4 bounded ignores | confirm each boundary |
| JS licenses | `license-checker-rseidelsohn` | permissive-only (non-blocking) | confirm no copyleft ships |
| Dynamic scan | `zap-baseline.py` | baseline floor (continue-on-error) | authenticated + active scan |
| Zero-access at rest | `mw-server --test v6_e2e` | ciphertext-at-rest proven | independent DB inspection |
| MCP send-gating | `mw-mcp` + `e2e-v6/v7` | Outbox-only default proven | falsify transmit path |
| Plugin sandbox | `jail` job | capability + limits proven | hostile-plugin escape attempt |
| Assist governance | `assist-mock` | redaction + content-free audit proven | redactor corpus + bypass attempt |
| Directory TLS | `directory-vs-openldap` | rustls, live OpenLDAP | MITM/downgrade attempt |

Start here, then exercise the four crown-jewel surfaces (crypto worker, plugin host, MCP,
Assist) adversarially. **The funded external audit run + findings resolution remain
human-gated and are required before the actual 1.0 tag.**
