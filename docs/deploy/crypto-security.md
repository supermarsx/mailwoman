# Deploying V4 crypto & security

V4 adds OpenPGP/S/MIME end-to-end encryption, a message Security panel, an
outbound DLP pipeline, sender controls (WKD/ARF), the max-security opening mode,
and the honest screen-capture watermark. This page is the operator reference:
environment variables and HTTP endpoints. For the model and rationale see
[`../security/`](../security/README.md).

All of it is additive and **off by default** — an operator opts in per feature.
The crypto that matters (private-key operations) runs in the browser and needs no
server configuration; the server only serves public keys, relays reports, loads
DLP config, and toggles the watermark.

## Environment

| Env | Default | Meaning |
|-----|---------|---------|
| `MW_WKD_DIR` | *(unset → WKD off)* | Directory of public keys to publish over Web Key Directory. Two layouts are accepted (see below). |
| `MW_DLP_RULES` | *(unset → no rules)* | A path to a JSON `[DlpRule]` file **or** an inline JSON array. See [`../security/dlp.md`](../security/dlp.md). Read per evaluation. |
| `MW_ABUSE_ADDRESS` | *(unset → ARF off)* | Destination address for ARF abuse/feedback reports emitted by report-phishing / report-junk sender controls. Required for `POST /api/security/report`. |
| `MW_ABUSE_SPOOL` | *(unset)* | Directory to write generated ARF reports as `<uuid>.eml`. When set, reports are spooled (`relayed:true`); otherwise the report is generated and logged. |
| `MW_WATERMARK` | `false` | Enable the deterrent screen-capture watermark overlay. Read [`../security/screen-capture.md`](../security/screen-capture.md) first — it is not a security control. |
| `MW_WATERMARK_OPACITY` | `0.08` | Overlay tile opacity, clamped to 0.0–1.0. |

These are engine/server-mode features; the private-key crypto is entirely
client-side and has no server env.

## Endpoints

| Method + path | Auth | Purpose |
|---------------|------|---------|
| `GET /.well-known/openpgpkey/hu/{hash}` | public | WKD **direct** method — key by z-base-32 hash; domain from `Host`. |
| `GET /.well-known/openpgpkey/{domain}/hu/{hash}` | public | WKD **advanced** method — domain in path. |
| `GET /.well-known/openpgpkey[/{domain}]/policy` | public | WKD policy probe (empty 200 signals a WKD-enabled domain). |
| `POST /api/security/report` | cookie | Emit an ARF report for a message (`{emailId, kind:"phishing"\|"junk", note?}`). Engine mode only; needs `MW_ABUSE_ADDRESS`. |
| `GET /api/security/dlp/config` | cookie | Read the active DLP rules parsed from `MW_DLP_RULES` (same shape the engine enforces). |
| `GET /api/security/watermark` | cookie | Watermark config for the SPA — the flag, opacity, viewer identity, server time, and the mandatory honesty note. |

WKD responses are `application/octet-stream`, public (no cookie), and
path-traversal-guarded (`hash` = 32 z-base-32 chars; `domain` rejects `..` / `/`).

### `MW_WKD_DIR` layout

Either layout works:

- **Address-named files** — `alice@example.org` with an optional
  `.asc`/`.pgp`/`.gpg`/`.key`/`.pub` extension, binary or armored. Armored input is
  dearmored to binary on serve.
- **Standard gpg-wks tree** — `<domain>/hu/<hash>` where `<hash>` is the WKD
  z-base-32 of the SHA-1 of the lowercased local part (the canonical `gpg-wks`
  layout).

If you terminate TLS at a reverse proxy, forward `Host` verbatim so the WKD direct
method resolves the right domain (and note the `openpgpkey.` vhost prefix is
stripped).

## What CI proves (and what only CI can)

The V4 CI jobs (`.github/workflows/ci.yml`) gate:

- **`wasm-build`** — builds `mw-crypto` + `mw-sanitize` to `wasm32` via `wasm-pack`
  on **Windows *and* Linux** and runs a Node smoke that does a real
  generateKey → encrypt → decrypt round-trip against the freshly built bundle. The
  two-OS wasm toolchain is the one thing only CI exercises end-to-end; a local dev
  build covers a single OS.
- **`crypto-interop`** — decrypts/verifies recorded GnuPG- and openssl
  (Outlook-style)-generated fixtures (`fixtures/crypto/**`), with no live keyserver
  or DNS.
- **`mail-auth-verdicts`** — DKIM verdicts (pass *and* fail) against a seeded
  offline resolver, plus Received-chain / attachment-risk / anomaly detection.
- **`e2e-crypto`** — Playwright drives the crypto/security UIs against the real
  engine + real WASM worker (booting greenmail + `mailwoman-engine`).

The `deny` job keeps the license floor permissive-only (no GPL/LGPL/AGPL); the
`rsa` "Marvin Attack" advisory (RUSTSEC-2023-0071) is accepted with a bounded
threat model because S/MIME RSA decryption runs **client-side** on the user's own
device — see `deny.toml` for the recorded rationale.
