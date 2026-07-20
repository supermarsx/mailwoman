# Versioning

Mailwoman uses a **rolling release** scheme in **`YY.N`** format.

- **`YY`** — two-digit calendar year of the release (e.g. `26` for 2026).
- **`N`** — the release number within that year, starting at **1** and
  **resetting to 1 each new calendar year**.

Examples, in order: `26.1`, `26.2`, `26.3`, … then `27.1`, `27.2`, …

There is no separate major/minor/patch. Each tagged release is a self-contained
rolling snapshot; the sequence is strictly increasing within a year, and the
year boundary resets `N`.

## Git tags

Releases are tagged **bare** (no `v` prefix): `26.1`, `26.2`, …
Tags are annotated and signed where possible.

## Package manifests (semver-shaped ecosystems)

Cargo and npm require semver (`X.Y.Z`). We map `YY.N` → **`YY.N.0`**:

- `Cargo.toml` `[workspace.package] version` → `26.1.0`
- `apps/web/package.json` `version` → `26.1.0`

The third component (`.0`) is reserved for the rare out-of-band hotfix to an
already-tagged release (`26.1.1`); normal forward progress increments `N`
(`26.2`), not the patch field.

## Release checklist

1. Bump `version` in `Cargo.toml` and `apps/web/package.json` to `YY.N.0`.
2. Update this file's example if the year rolled over.
3. Commit `chore(release): YY.N`.
4. Tag: `git tag -a YY.N -m "Mailwoman YY.N"` then `git push origin YY.N`.

## History

- **`26.17`** — a polish + defense-in-depth tag that closes the four feature carryovers and the six
  LOW security-hardening notes deferred from 26.16, with **net-zero new third-party crates** (every
  item lands in existing files on `std`/already-vendored deps). Three additive migrations
  (**`0019`**/**`0020`**/**`0021`**), both dialects in lockstep, BIGINT-as-bool preserved. **Note
  metadata sealed at rest** (§7 PIM): a note's `title`, `tags`, `color`, and `pinned` flag were
  plaintext columns; migration **`0019`** adds sealed BLOB columns under `ServerKey`, `upsert_note`
  seals the four values and blanks the legacy plaintext columns to neutral defaults, and
  `note_from_row` unseals (falling back to the plaintext column only for a not-yet-backfilled row).
  The one load-bearing SQL that referenced these — `list_notes`'s `ORDER BY pinned DESC` — is
  re-homed: the query now orders by `updated_at DESC, id` (both untouched) and a **Rust stable sort
  by `pinned`** reproduces the exact prior order after decrypt, so **no plaintext sort key remains at
  rest** (Option A — a separate deterministic sort-key column was rejected because it would leak the
  ordering relationship we are sealing). A one-shot **idempotent store-open backfill** seals+blanks
  any pre-upgrade row; the redundant `idx_notes_pinned` index is dropped. Note filtering was already
  Rust-side, so the engine query path is unchanged. **`Identity.signatureName` persistence** (§7.2):
  the display name for a signing identity was accepted by the prefs route then dropped; migration
  **`0020`** adds the `identities.signature_name` column and it now round-trips through `v2.rs` and
  the prefs route (the web client already carried it). **`ar` app-wide locale negotiation**: Arabic
  joins the negotiated `LOCALES` set (12 → 13) with RTL layout via the existing direction resolver;
  the shipped `ar` catalog is a stub, so absent keys fall back to `en` through the normal fallback
  chain (a complete `ar` UI remains a follow-up). **Trusted Types now enforced** (§7.5): the web
  shell registers a `default` Trusted-Types policy in `main.tsx` (guarded on `window.trustedTypes`)
  before boot, and the server CSP re-enables `require-trusted-types-for 'script'` — the shell CSP
  and the tightened image-proxy CSP are now equal, and the drift-guard test asserts that equality.
  **Six 26.16 LOW hardening notes closed**: **L3** — the SSRF denylist now decodes NAT64
  (`64:ff9b::/96`) and 6to4 (`2002::/16`) embedded IPv4 and re-checks it, so a smuggled private
  target is refused; **L5** — ManageSieve egress resolves the user-supplied host and blocks
  cloud-metadata / loopback / link-local **while keeping RFC1918 reachable** (syncing to your own
  internal Sieve server is a legitimate case, so a full deny-by-default gate would over-block); **L1**
  — TOTP login is no longer replayable within a code's time window: `totp_verify` returns the matched
  step counter and a compare-and-swap `last_step` advance (migration **`0021`**) rejects a re-used or
  regressed counter; **L6** — MCP RFC 8707 audience enforcement is **default-on**, deriving the
  canonical resource from the configured public origin when `MW_MCP_RESOURCE` is unset (the env var
  still overrides; with no public origin configured, enforcement stays off) — a wrong-audience token
  is now rejected by default, and API-key auth stays exempt; **L2** — an opt-in `MW_RENDER_JAIL=strict`
  makes a failed Landlock setup **fatal** under a required jail, while the default stays best-effort so
  rendering still works on kernels < 5.13 (the hostile parse already runs in the syscall-less wasm
  guest). **L4** is a deliberate posture decision, not a code change: the image-proxy fetch stays
  session-authed + SSRF-filtered + re-encoded in the jail and is **not** grant-gated by default (full
  grant-gating would thread message-id/sender through the proxy URL and risk breaking already-granted
  loads); a per-account rate-limit is tracked as a follow-up. **`cargo deny` clean**; no
  openssl/`-sys`/C; license floor holds. **Adversarial security review: GO (0 critical / 0 high / 0
  medium)** — all six 26.16 LOW notes verified closed; 6 new LOW notes open a 26.18 hardening backlog,
  the prioritized one being that the note-metadata backfill blanks plaintext in place, so pre-upgrade
  notes can leave plaintext residue in SQLite free-pages/WAL and Postgres dead tuples until a VACUUM
  (new notes are unaffected). **Live-E2E: 8 legs green vs real infrastructure, 0 wiring bugs** —
  note-metadata sealed at rest (raw-store scan) with pinned-first order preserved and the backfill
  idempotent (verified on both SQLite and live Postgres), `signatureName` round-trip, `ar` negotiation
  winning, the shipped bundle served under enforced Trusted Types, a TOTP code rejected on replay
  within its window, a NAT64/6to4-smuggled private target refused, MCP default-on audience enforcement
  with API-key exemption, and the narrowed Sieve egress refusing metadata/loopback while allowing
  RFC1918. Two legs loud-skip on this host and are covered by CI: the browser-boot Trusted-Types
  pixel check (Playwright) and, when Docker is unavailable, the live-Postgres note-seal leg.
- **`26.16`** — the largest milestone since V7: the 7 material spec gaps an independent audit found
  still open at 26.15 (plus ~34 minor ones), closed across 13 parallel executor lanes, adversarially
  reviewed and live-verified. Net-new third-party crates for the whole milestone: **`seccompiler` +
  `landlock`** (both pure-Rust, MIT/Apache, no `-sys`/C, Linux-gated) — 2FA added **zero** (its
  primitives were already vendored). **Login two-factor auth** (§7.4): a hand-rolled, license-floor-clean
  relying-party stack in the new `mw-mfa` crate — `webauthn-rs` is banned (it pulls `openssl`, a hard
  `deny.toml` `[bans]` entry), so WebAuthn attestation-`none` assertion verification is implemented on the
  already-vendored `p256` (ES256) + `ed25519-dalek` (EdDSA) + `sha2` + a hand-written definite-length CBOR
  reader (no `ciborium` in-tree), alongside RFC 6238 TOTP and argon2-hashed recovery codes. Secrets sealed
  under `ServerKey` in migration **`0015`** (`cose_public_key` stored unsealed — it is public). The login
  gate runs after credential validation and **before any `create_session` in all three branches** (proxy,
  engine, header-auth) — an enrolled factor is **required with no password-only downgrade**; challenges and
  recovery codes are single-use; sign-counter regression is rejected. Admins can require 2FA globally or
  **per-domain** (`twofa_policy`, BIGINT-as-bool). **Kernel sandbox jail** (§7.5): the new `mw-sandbox`
  crate applies Linux seccomp-BPF (default-kill allowlist; no socket/execve/ptrace) + Landlock (deny-all
  FS) + PID/net namespaces + rlimits to the render child, **fail-closed** (a jail-expected-but-absent path
  refuses — `503` — rather than parsing in-process); non-Linux is a documented degraded mode, reported by a
  new `mailwoman doctor`. **WASM 2nd-layer media jail**: hostile CFB/MSG/OFT parsing + image re-encode now
  run inside a zero-host-import `wasm32` core module (`mw-media-wasm`) in a wasmtime Pulley interpreter
  (no JIT W^X page → survives the seccomp jail + systemd MDWE); the native `from_oft` in-process parse is
  **removed entirely** from the server runtime. **Anonymizing image proxy** (§7.2): `GET /api/image-proxy`
  fetches attacker-controlled email images under a deny-by-default SSRF policy — http/https only, DNS
  resolved once then the fetch **pinned to that IP** (anti-rebinding), every redirect hop re-validated,
  loopback/link-local/private/ULA/CGNAT/multicast + cloud-metadata (`169.254.169.254`) refused (IPv4-mapped
  v6 unwrapped and re-checked), size/timeout/concurrency caps, no `Cookie`/`Referer`/`Auth` forwarded,
  bytes re-encoded in the media jail, session-gated (never an open relay), content-hash cached — plus a
  4-grant model (single / all / per-sender / per-domain) over migration **`0016`**, tracker classification
  + count, and a tightened CSP (`style-src 'self'`, no `'unsafe-inline'`). **Rich-text compose**: the plain
  `<textarea>` is replaced by a lazy-loaded ProseMirror editor (all-MIT, self-hosted, 75 KB-gzip chunk)
  with a plain-text/format-flowed toggle, feeding the existing send path unchanged. **Conversation
  threading UI**: the flat message list groups on the already-plumbed JWZ `threadId`. **Bridge OAuth token
  acquisition**: device-code/auth-code/refresh flows against Microsoft/Google over the host rustls client,
  sealed token cache in migration **`0018`**, replacing `DeniedOAuthProvider`. **Prefs backends + Settings
  UI**: 2FA enrolment/sessions/signatures/notification-rules/keyboard-presets/offline-policy/RTL web
  screens over new prefs HTTP routes (migration **`0017`**; saved-searches reuse the frozen `0003` table).
  Plus **JMAP completeness** (`Thread/get`+`changes`, `SearchSnippet/get`, `VacationResponse/get|set`,
  `Quota/get`, `Email/copy|import|parse`), **MCP RFC 8707 audience enforcement** + real PIM tool backends +
  inbound-webhook action sink + OTLP span export + a hand-rolled Sentry/GlitchTip relay (off by default, no
  `sentry` crate/native-tls, no mail content), **crypto tail** (S/MIME AES-GCM AuthEnvelopedData, VKS key
  lookup, full Autocrypt Setup Message, PQC store-key wrap invoked at boot), **PIM** (calendar sharing/ACL,
  NL quick-add, categories, event attachments, webcal subscribe via the SSRF-hardened fetcher, VJOURNAL
  export), **mbox/EML/Maildir import**, **MSG/DOCX export**, **attachment-content search**, and a
  **container + supply-chain** pass (musl-static image, hardened compose/Helm, cosign/SBOM/scan workflow,
  fuzz targets). A **SPEC-honesty pass** corrected over-claims to match what actually ships (the PQ-hybrid
  TLS `X25519MLKEM768` "on by default" claim removed — only the banned `aws-lc-rs` C dep provides it).
  **`cargo deny` clean**; no openssl/`-sys`/C. **Adversarial security review: GO (0 critical / 0 high /
  0 medium)** — 6 LOW hardening notes deferred to 26.17. **Live-E2E: 22/22 green vs real Postgres, 0 wiring
  bugs** — the SSRF block (private/metadata/loopback all refused live), the 2FA no-downgrade gate (enrolled
  user gets no session cookie password-only; a virtual WebAuthn authenticator asserted and a tampered
  signature was refused), sealed-secrets-at-rest, and the tightened-CSP shell all proven against real
  infrastructure. The release gate itself caught a real **date-dependent latent bug** the narrower e2e run
  missed — `pim/quick-add`'s bare-time branch read the wall clock instead of the injected reference date, so
  it went red the moment the system clock rolled past the test's hardcoded day; fixed pre-tag by threading
  the reference date into the helper. **Deferred to 26.17 — all since shipped in 26.17**: sealing Note
  title/tags/color/pinned (a new migration adds sealed columns and the pinned-first sort moved into Rust,
  so no plaintext sort key remains at rest — no sortable-index leak), `Identity.signatureName` persistence,
  `ar` app-wide locale negotiation, a web default Trusted-Types policy (re-enabling
  `require-trusted-types-for` in the CSP), and the 6 LOW security-hardening notes. **Floor/platform-blocked**
  (still not buildable under the license floor or pending platform work, unchanged from prior tags): A8
  semantic search re-rank (needs an ML embedding model) — the embedding capability ships, the index re-rank
  does not; the iOS/native shell; native GSSAPI Kerberos; and a first-party S3 blob backend.
- **`26.15`** — three previously-stubbed-or-pinned-shut seams lit up, all net-zero new dependencies.
  **New-file blob upload**: `POST /jmap/upload/{accountId}` is now a real handler (was a 501 stub) — it
  authenticates, reads the body under the advertised `maxSizeUpload` (50 MB → `413` over limit), seals the
  bytes under `ServerKey` and writes them through a pluggable `UploadBackend` (filesystem default, rooted at
  `MW_UPLOAD_DIR`, sealed at rest, path-traversal-safe server-minted hex keys, per-account on-disk isolation),
  records metadata + `storage_key` in migration **`0012`** (no bytes in the DB), and returns a `blobId` on the
  reserved **`U`** prefix (`U`+hex — collision-free against the pure-64-hex stableIds, routed in `fetch_blob`
  before the `get_message` path). That `blobId` becomes a real attachment on an outgoing `Email/set` create
  through the existing `compose_from_spec`/`fetch_blob` seam. A **symmetric `proxy_upload`** mirrors
  `proxy_download` for proxy mode. Retention is TTL (24h for unreferenced uploads) via an explicit one-shot
  `mailwoman maintenance gc-uploads [--older-than <dur>]` CLI (never automatic). Web: a file picker in Compose
  uploads to `session.uploadUrl` and feeds the returned `blobId` into the existing attachment/send plumbing.
  S3 stays trait-boundary-and-config-surface only (no impl, no dep — no MIT/pure-Rust/`-sys`-free S3 client was
  adoptable without blowing the license floor). **Persistent plugin byte-storage**: the `HostKv` hard stub
  (`get`→`None`, `put`→no-op) is replaced by a sealed, quota-bounded, store-backed KV over migration **`0013`**
  — values sealed at rest, namespace **(plugin_id, account_id)** derived host-side from the bound `HostState`
  (never guest args; a deployment-wide plugin uses `account_id=''`), per-value 64 KiB / per-namespace 5 MiB /
  1000-key quota enforced at `put` (over-quota fails visibly), whole-namespace purge on uninstall, no TTL. The
  WIT `host` interface gains `kv-delete`/`kv-list` **additively**, keeping the package `@0.1.0` (t12
  `basic-credentials` precedent — committed `.wasm` fixtures keep linking). The deny-by-default
  `store:kv-scoped` admin gate is unchanged — 26.15 only makes the grant persistent. **Third-party component
  loading** (security-core): `resolve_component` is widened from first-party-pinned-only to
  first-party-pinned-**OR**-admin-pinned-digest. An admin reviews a specific component's SHA-256 and approves
  that exact 64-hex value into the new admin-managed **`0014`** `plugin_allowlist` (BIGINT-as-bool `revoked`,
  never native Postgres BOOLEAN). The compiled-in `FIRST_PARTY_DIGESTS` table is checked **FIRST and
  terminally** — a first-party id never consults the allowlist, even on a first-party miss/tamper (returns
  `None`, no fall-through), so a colliding allowlist row can never override or spoof a first-party identity;
  approve-time additionally rejects any allowlist entry whose id collides with a first-party id. On every load
  the SHA-256 is recomputed over a single in-memory buffer and the same bytes are handed to `PluginHost::load`
  (no re-read → no TOCTOU); mismatch, a revoked row, or an absent row is a **hard refuse (`None`) + an audit
  entry**. Third-party bytes load from a separate `MW_THIRDPARTY_PLUGIN_DIR`, run in the identical wasm sandbox
  with the identical deny-by-default capability model (nothing auto-granted — allowlisting authorizes the bytes
  to run, not any capability). A maintained `HIGH_POWER` capability set (account-backend / send-as-user class)
  is refused to any non-first-party plugin at **grant time** — provenance-gated, not overridable by admin
  action. As defense-in-depth, an Ed25519 signed-registry (`TrustRoot`/`signature::decide`, `ed25519-dalek`
  already vendored) is **also** verified when a signature is present; the digest pin alone remains sufficient to
  load (with an unsigned-allowed banner + audit). Admin surface: approve/revoke/uninstall routes on
  `/admin/plugins/*` (revoke sets `revoked=1` **and** disables the plugin — effective next load; hot-unloading a
  running instance stays out of scope) plus a web allowlist panel that surfaces each present component's
  computed digest for review. **Net zero new dependency-graph nodes** (`async-trait`/`sha2`/`ed25519-dalek` all
  already vendored; the seal reuses `ServerKey`/XChaCha20, the digest reuses the existing `sha2` pin); no
  openssl/`-sys`/C; **`cargo deny` clean** (1105 packages, byte-identical set vs 26.14). Verified: **1238 Rust**
  tests (0 failed, 11 ignored, desktop/mobile-excluded) + **759 web**; the live-E2E wave came up **8/8 green vs
  real Postgres + Dovecot + a real filesystem backend with 0 wiring bugs** — the third-party negatives that
  would be a CVE if they passed (no-approval / revoked / one-tampered-byte / first-party-id collision) were each
  **refused live**, and a HIGH_POWER cap was refused to a third-party plugin even when an admin attempted the
  grant. **E8 adversarial security review of the loosened boundary: GO (0 critical / 0 high)** — one LOW
  (revoke handler not lowercasing the URL digest) was fixed pre-tag; three INFO deferred. **Test gate note**:
  the Rust gate runs `cargo test --lib --tests` — a pre-existing, transient rustdoc ICE on
  `crates/mw-store/src/stores_v6.rs` (rustc 1.95.0, "could not resolve trait item being implemented",
  untouched by 26.15: `git diff b3f8020..HEAD` on that file is empty) was reported during a doctest phase;
  at release the full workspace doctest phase re-ran clean, so the milestone is unaffected either way and the
  `--lib --tests` target is the authoritative unit+integration gate. **Still deferred**: iOS shell (needs
  macOS + Xcode + a paid Apple account); an S3 `UploadBackend` impl (trait boundary shipped, impl held back by
  the license floor); native GSSAPI Kerberos (license-floor C dep). Rolling `YY.N` retained (this is 26.15, not
  a "1.0" tag).
- **`26.14`** — follow-ups closing the residuals 26.13 left open. **`tls-exporter` (RFC 9266) channel
  binding** across `mw-imap`/`mw-smtp`/`mw-pop3`: on TLS 1.3 the SCRAM-`PLUS` client now computes the
  RFC 9266 exporter binding (`export_keying_material`, label `"EXPORTER-Channel-Binding"`, empty
  context, 32 bytes) and sets the gs2 cb-name to `tls-exporter`; on TLS 1.2 it keeps
  `tls-server-end-point` (RFC 5929). This is the piece that lets **SCRAM-SHA-256-PLUS login actually
  COMPLETE** — proven live for **IMAP and POP3 against real Dovecot 2.4.4 over TLS 1.3** (the exact
  acceptance 26.13 could not reach, since Dovecot implements `tls-exporter`, not
  `tls-server-end-point`). SMTP `-PLUS` stays unit/mock-proven (no channel-binding-capable submission
  server in-env — the same honest disposition as 26.13; identical per-crate design). No config knob.
  **Server-metadata admin editor**: the write-capable `MetadataView` is now mounted under `/admin`
  (admin selects a provisioned account), reaching the account's backend through an admin-gated
  `/jmap/api` passthrough for `ServerMetadata/get|set` + `MailboxRights/get|set` (normal JMAP auth
  unweakened, fail-closed); the `mw_admin_session` cookie Path was broadened `/admin`→`/`
  (HttpOnly + SameSite=Strict + Secure unchanged, so the CSRF/XSS posture is identical) so the browser
  sends it to `/jmap/api`. **JWZ historical backfill** (admin opt-in): an idempotent one-shot re-thread
  of existing mail via the shipped full JWZ set algorithm, exposed as a `mailwoman maintenance rethread
  <account>` CLI subcommand AND an admin-panel button behind an explicit confirmation (it re-keys thread
  grouping — never automatic), over `POST /admin/maintenance/rethread`; new-ingest-only stays the
  default, no migration (reuses the `messages`/`threads` tables). Idempotency is machine-checked
  (`reassigned==0` on re-run); live-proven on SQLite and **Postgres**. **Blob-attachment honoring** on
  `Email/set` create — attachments whose `blobId` resolves to an existing stored message/part
  (forward / attach-from-mail) now ride into the built + sent message via `mail-builder`; an unresolved
  blobId is a clean `notCreated`, never a panic. New-file upload stays de-scoped (the `jmap_upload`
  stub / upload blob-store is a separate seam). **Net zero new dependency-graph nodes** (`tls-exporter`
  uses the already-vendored rustls `export_keying_material`); no openssl/`-sys`/C; `cargo deny` clean.
  Verified: **1196 Rust** tests (0 failed, 11 ignored, desktop/mobile-excluded) + **745 web**; the
  live-E2E wave (7 tests vs real Dovecot 2.4.4 / TLS 1.3 / Postgres 16) came up clean — **no
  feature-code bug this milestone** (the discipline still ran in full). **Still deferred**: iOS shell
  (needs macOS + Xcode + a paid Apple account); new-file blob upload; third-party (non-bundled) plugin
  byte-storage (needs a trust model + a bytes persistence seam); a live SMTP `-PLUS` proof (needs a
  channel-binding-capable submission server). Rolling `YY.N` retained (this is 26.14, not a "1.0" tag).
- **`26.13`** — buildable residuals & deferrals left after 26.12, closed additively. **SCRAM-`PLUS`
  channel binding completed** across `mw-imap`/`mw-smtp`/`mw-pop3`: `tls-server-end-point` (RFC 5929)
  now hashes the TLS leaf with the cert's own signature digest (SHA-256/384/512, floor SHA-256 — the
  prior SHA-256-only assumption is gone), plumbed through each protocol's TLS upgrade (SMTP/POP3
  previously dropped the leaf cert before auth). The client is correct and its binding computation is
  live-proven byte-exact against real certs; **note**: Dovecot 2.4.x advertises `SCRAM-SHA-256-PLUS`
  but implements only `tls-unique`/`tls-exporter`, **not** `tls-server-end-point`, so full `-PLUS`
  login-acceptance can't be proven against it (a server interop gap, not a client defect) — the login
  leg is unit/mock-proven. `tls-exporter` (RFC 9266, TLS 1.3-native) is a noted future interop
  enhancement, not shipped here. **IMAP ACL (RFC 4314) + METADATA (RFC 5464)**: full protocol commands
  in `mw-imap` (GETACL/SETACL/DELETEACL/LISTRIGHTS/MYRIGHTS + GET/SETMETADATA, sent to the upstream
  server which stays the enforcement point), an engine read-through JMAP seam (`MailboxRights/get|set`,
  `ServerMetadata/get|set` — no persistence, no migration, no new frozen type), and a web mailbox ACL
  editor (the 11 RFC 4314 rights bits as labeled checkboxes, write affordances gated on the caller's
  `a` right) + server-metadata view. Live-verified vs real Dovecot (ACL+METADATA plugins): grant →
  GETACL shows the identifier → revoke issues a real **DELETEACL** (identifier gone, not zero-rights).
  Metadata `mbox`-none = server scope, `Some` = mailbox; `value` NIL = remove. **JWZ threading**: the
  27-LOC References-head heuristic is replaced by the canonical JWZ algorithm (containers, id_table,
  reference linking, root set, subject-gather, empty-container prune); applied **new-ingest-only** (no
  historical re-key), keyed off the existing `messages.message_id` column so **no migration** —
  incremental ingest gathers the reply-chain member set, runs JWZ, repairs truncated `References`.
  Live-proven on SQLite and **live Postgres** (reply-before-original convergence, sibling repair).
  **GeoIP/ASN**: a pure-Rust BYO `.mmdb` reader (`maxminddb`, ISC, no C) resolves country + ASN from an
  admin-supplied MaxMind DB via `MW_GEOIP_DB`, cached per path; **no DB is bundled** (admin-supplied
  only). Live-proven against MaxMind's Apache-2.0 test DBs. **Net zero new dependency-graph nodes**
  (`x509-cert`/`maxminddb` already resolved in-tree); no openssl/`-sys`/C; `cargo deny` clean. Verified:
  **1170 Rust** tests (0 failed, 11 ignored, desktop/mobile-excluded baseline) + **737 web**; the
  live-E2E wave (`13 passed`, vs real Dovecot + live Postgres + GeoIP fixtures) again earned its keep —
  it caught a real **METADATA GET literal-parsing** bug (Dovecot returns values as synchronizing
  literals `{n}`; the parser returned the length marker) that every unit test passed, fixed + re-verified
  live before this tag. **Deferred**: iOS shell (still needs macOS + Xcode + a paid Apple account — not
  buildable here); IMAP ACL/METADATA *editing UI* is shipped, ACL `SETACL` write is exposed in the web
  editor (server-metadata editing stays admin-gated); `tls-exporter` channel binding. Rolling `YY.N`
  scheme retained (this is 26.13, not a "1.0" tag).
- **`26.12`** — spec-conformance closure: real SPEC-feature gaps the 2026-07-16 audit found
  in otherwise-complete code, closed additively over the frozen V0–V7 surfaces. **HTML
  sanitizer CSS-rewrite**: `mw-sanitize` no longer wholesale-strips CSS — it parses both inline
  `style=` and `<style>` blocks (via `cssparser`, MPL-2.0, already in-tree through ammonia),
  keeps an allowlist of ~130 visual properties, namespaces every selector under
  `.mw-email-body`, drops `position:fixed/sticky`, `@import`, external `url()` (only internal
  `cid:` survives) and every non-`@media`/`@supports` at-rule, clamps `z-index` to 1000, and
  drops `expression()`/`javascript:` values; the public `sanitize_email_html` signature and the
  wasm cdylib are unchanged. **Sieve source parser + web rules UI**: a hand-rolled
  recursive-descent `mw-sieve::parse` (zero new dep) is the round-trip inverse of the existing
  codegen; a new `apps/web` rules module ships a condition/action builder, a raw-Sieve editor
  with lint surface, a where-it-runs indicator, and a dry-run preview over the existing MailRule
  JMAP/ManageSieve path. **EWS real auth**: the bridge's NTLM-only placeholder-constant +
  hardcoded-endpoint auth is replaced with a Basic path (empty NT domain) alongside NTLMv2, keyed
  by per-account, host-held, sealed credentials — an additive `0011 ews_account_cred` table (both
  dialects, INTEGER/BIGINT 0/1 booleans, secret sealed with XChaCha20-Poly1305, `0001`–`0010`
  untouched), reached through a new additive `basic-credentials(account)` import on the frozen
  `host` WIT interface (backward-compatible; pre-t12 guests don't import it). The empty-account
  handle the guest passes ("one instance backs one account") is now resolved host-side to the
  plugin instance's bound account, fixing EWS auth end-to-end (and a latent same-shape gap in the
  OAuth bridges). **Compose sign-on-send**: the `sign` toggle now folds into encrypt
  (`signWithKeyRef` unwrapped at the worker boundary) for a signed-AND-encrypted `PGP MESSAGE`; a
  clear-signed sign-only branch emits a real RFC 9580 `PGP SIGNED MESSAGE` with the body inline
  (previously the body was discarded); and the reader now verifies the embedded signature on
  decrypt (`signerPublicKey` threaded additively through `DecryptRequest` → `Reader`, resolving
  the sender key from the keyring), so encrypt+sign mail reads back as "Signature verified".
  Encrypt-on-send and plain sends are byte-unchanged. **SASL + IMAP extensions**: SCRAM-SHA-256
  (and -PLUS) + OAUTHBEARER across `mw-imap`/`mw-pop3`/`mw-smtp` (PBKDF2 derived from in-tree
  `hmac`+`sha2`; no new dep), plus IMAP SORT + THREAD (RFC 5256) advertised through `BackendCaps`.
  **SMTP extensions**: DSN (`RET`/`ENVID`/`NOTIFY`/`ORCPT`), REQUIRETLS (fails closed when
  unadvertised), SMTPUTF8, and CHUNKING/BDAT. **Engine security**: DLP now evaluates the
  previously-unread `dictionaries` + `classification` conditions and adds a `notify`/`notify-admin`
  action; SPF is evaluated (origin IP from the top Received hop via `mail-auth`); the S/MIME
  recipient-cert lookup is wired to the GAL/LDAP `gal_lookup_cert` seam; identities are pulled
  from the server (source `"server"`) beyond the single seeded identity. **Search** gains fuzzy
  (`~`) and prefix/wildcard (`*`) queries within the existing p95 budget. **Autoconfig** adds a
  `.well-known/jmap` rung and a live SRV resolver (`hickory-resolver`, MIT/Apache, already in-tree
  via mail-auth). **Calendar** adds a side-by-side conflict resolver (consuming the previously
  unused `queryFreeBusy` free/busy grid), a distinct schedule view (no longer aliasing agenda),
  attendee `ROLE`/`CUTYPE` parse/emit + pickers, `RDATE` and `RECURRENCE-ID` overrides on
  expansion, and `.hol` export. **Packaging + CI**: the workspace version is now the single source
  of truth — `scripts/stamp-version.sh` stamps winget/flatpak/fdroid/both `tauri.conf.json`/both
  shell `package.json`, and `packaging.yml` parses it and compares every manifest (the three
  hardcoded `26.8.0` literals are gone); desktop/mobile unit tests + clippy run in a dedicated CI
  job and `desktop-e2e` is activated (honestly `continue-on-error`-gated for the hosted-runner
  WebView2↔msedgedriver pin); false 501/stub/"until eN"/"NOT mounted" doc comments were scrubbed.
  **Net zero new dependency graph nodes** (`cssparser`/`hickory-resolver` already resolved in-tree;
  SCRAM reuses `hmac`/`sha2`); no openssl/`-sys`/C; `cargo deny` clean (MPL-2.0 `cssparser` note
  recorded, permitted). Verified: **1101 Rust** tests (0 failed, 11 ignored) across the workspace
  with the `mailwoman-desktop`/`mailwoman-mobile` crates excluded (they need a generated
  `bundle-hash.json` fresh-checkout artifact and run in their own dedicated CI job, where desktop's
  11 unit tests pass) + **714 web** tests; live-E2E green — **17 backend** live tests
  (IMAP/POP3 SCRAM + SORT/THREAD vs a SCRAM-only Dovecot; SMTP DSN/SMTPUTF8/BDAT; S/MIME GAL cert
  vs real OpenLDAP; autoconfig `.well-known/jmap`+SRV; EWS Basic + per-account sealed creds through
  the jail on live Postgres) and the browser compose wire-assertion gate (a sent message is
  byte-verified genuinely encrypted, and — signed — reads back "Signature verified"), plus sieve
  round-trip, calendar resolver, and a real sanitizer CSS render. The EWS auth bug and both compose
  signing holes (encrypt+sign fold, clear-signed sign-only) plus the decrypt-side verify gap were
  each found by that live gate — "unit-green ≠ wired" — and fixed + re-verified before this tag.
  **Honest deferrals**: iOS shell (needs macOS + Xcode + a paid Apple account and a macOS runner —
  unbuildable on this toolchain); GeoIP/ASN enrichment (a BYO-database admin hook only — no
  permissively-redistributable DB is bundled; SPF itself shipped); full JWZ threading (the
  References/In-Reply-To heuristic stays — a ~250–400 LOC rewrite with incremental-ingest
  blast-radius); and IMAP ACL (4314) / METADATA (5464) editing UI (detection/read only — the
  editing surface is deferred). **Minor residual**: SCRAM channel-binding — the non-`PLUS`
  mechanisms are complete across all three protocols, and IMAP `-PLUS` assumes SHA-256 certificate
  leaves. **Artifact note**: the `mw-crypto` browser crypto-worker (`apps/web/src/wasm/mw-crypto/*`)
  is **git-tracked** (shipped committed, not built at package time); the committed bytes were
  functionally verified (native unit + Node wasm-runtime smoke tests + the live browser gate) but
  were hand-assembled on Windows due to a local toolchain gap (the vendored `wasm-opt` was invoked
  without the bulk-memory feature flags and this box's `rustc` omitted the `target_features`
  section) — the canonical artifact is regenerated by CI (e9) on Linux via the stock
  `build-wasm.sh` toolchain. Rolling `YY.N` scheme retained (this is 26.12, not a "1.0" tag).
- **`26.11`** — closes the two non-blocking follow-ups documented in `26.10`, both
  server-side and additive over the frozen surfaces. **Masked-email on-send From-rewrite**:
  a server-side `MaskedSubmitter` decorator wraps the standards-account submitter at the
  single construction seam (`engine_mode.rs::register()`). When a submitted message's
  envelope `From` is one of the sending account's own masked aliases and that alias is
  enabled, the envelope `MAIL FROM` is rewritten to the canonical stored alias (keeping the
  real address out of the Return-Path) and `lastUsedAt` is bumped. An alias owned by another
  account, a disabled alias, a deleted (tombstoned) alias, or a store error all fail
  **closed** — the inner submitter is never called, so the message is never sent. An ordinary
  non-alias `From` is forwarded byte-unchanged. It rides an additive
  `get_masked_email_by_addr` store lookup (no schema/migration edit), and bridge/plugin
  accounts are intentionally not wrapped (a provider rejects a foreign `From`; masked aliases
  are a standards-account feature). **OAuth DCR admin-enable route**: admin-session-gated
  `GET/PUT /admin/oauth-dcr` (parity with the SSO and UI-plugin admin routes), fail-closed on
  a disabled panel or missing/unknown session. Dynamic Client Registration **stays
  default-disabled** — enabling it is now an explicit admin action through the panel rather
  than config/CLI only; the default-off posture is unchanged. **Net zero new
  Rust/npm dependencies**; no openssl; no schema/migration edit; no mw-engine feature-code
  change. Verified: **1047 Rust** tests (144 suites) + the web suite; `cargo deny` clean with
  no new advisory ignore and no openssl; combined verify + live-E2E green — the masked
  send-path proven across a 5-scenario matrix (owned+enabled rewrite, cross-account /
  disabled / deleted fail-closed with the inner submitter never reached, non-alias
  byte-unchanged) driving the real engine JMAP submission path, and the DCR admin toggle
  proven end-to-end (unauth 401 → admin login → enable flips `/oauth/register` 403→201 →
  disable returns it to 403) on **SQLite and live Postgres**. Rolling `YY.N` scheme retained
  (this is 26.11, not a "1.0" tag).
- **`26.10`** — the deferred-spec tail: bridge PIM through the plugin seam, spam
  classifiers, masked email, OAuth dynamic client registration, a sandboxed TypeScript
  UI-plugin tier, and MSG/OFT deep write fidelity — all additive over the frozen V7
  surfaces, with a comprehensive live-E2E pass. **Bridge personal-information management
  is now drivable through the WASM plugin jail.** The plugin ABI gains a second
  `mailwoman:plugin-pim` world (`calendar` / `tasks` / `bridge-parity` interfaces) that
  the host binds via **per-interface export probing** — a component that exports only the
  frozen `account-backend` interface (LanguageTool, Nextcloud) loads byte-unchanged and
  advertises no PIM caps. The Graph/EWS/Gmail bridges wire their existing calendar / tasks /
  reactions / voting / recall / focused-sync implementations to the new exports with
  **honest per-provider support**: Graph advertises all six, EWS binds calendar + tasks
  only (its legacy coarse caps overclaim parity; the per-interface `supports-*` funcs are
  false), and Gmail advertises none — so `mw-engine` routes PIM to the bridge when a
  capability is genuinely advertised and otherwise keeps the **byte-unchanged standards
  fallback** (a plain IMAP/DAV account is unaffected). Two first-party **spam classifiers**
  ship as jailed `wasm32-wasip2` components (`spam-rspamd` talking to a real rspamd scan
  worker, `spam-spamassassin` via a SPAMC→HTTP relay) reaching their daemons only through
  the host `http-fetch` egress under a net allowlist (no C linkage). They feed a
  **fail-soft `SpamHook`** in `Engine::ingest` that runs on genuinely-new INBOX arrivals
  only: a `Spam` verdict tags `$Junk` and moves to Junk, while any classifier failure,
  denied host, non-INBOX message, or `Ham`/`Unknown` verdict delivers the message
  byte-unchanged (a classifier can never drop mail). A **masked-email** alias service
  (store repo + `/api/masked/*` routes) generates, enables/disables, and deletes
  per-account aliases. **OAuth 2.0 Dynamic Client Registration** (RFC 7591 register +
  RFC 7592 read/update/delete) is additive to `mw-oauth`, **default-disabled and
  ops-gated**: enabled only via an `oauth_dcr` policy row, with a redirect-host-suffix
  allowlist, optional initial-access-token, per-client registration-access-tokens, and no
  scope escalation. A **sandboxed TypeScript UI-plugin tier** renders approved plugins
  inside an **opaque-origin `<iframe sandbox="allow-scripts">`** (no `allow-same-origin`,
  host CSP `connect-src 'none'`) behind a **deny-by-default `postMessage` broker** —
  ungranted capabilities and off-allowlist methods are rejected before any host call — with
  an **Ed25519 signed registry**, admin approval, and an unsigned-plugin banner the guest
  cannot reach. **MSG/OFT deep write fidelity** adds a `__nameid` named-property map
  (MS-OXMSG) and embedded-OLE message writing to `mw-export`, additively: a message with no
  custom named properties or embedded objects stays byte-identical to the 26.9 floor.
  **EWS Kerberos** ships as a documented **BYO SPNEGO reverse-proxy** path (IIS+ARR+KCD /
  Apache mod_auth_gssapi / nginx SPNEGO recipes) on top of the shipped Basic + pure-Rust
  NTLMv2 — native GSSAPI stays a **flagged human license-floor decision** (it needs a
  non-permissive `-sys`-C dep, so the autonomous pipeline will not add it). **Net zero new
  Rust/npm dependencies**; no openssl / no `-sys` C; `0010` migration added both dialects,
  `0001`–`0009` untouched; the SQLite-default + browser-cookie paths are unchanged.
  Verified: **1023 Rust** tests (138 suites) + **671 web** tests; `cargo deny` clean with
  no new advisory ignore and no openssl anywhere; a comprehensive live-E2E gate green —
  **13 backend live-E2E** tests (bridge PIM through the real jail + engine matching the
  honest support matrix, standards fallback proven byte-unchanged, spam fail-soft vs the
  real components plus a gated real-daemon leg, DCR vs the real AuthServer on SQLite and
  Postgres, MSG/OFT deep round-trip), and **12 browser live-E2E** passing with 1 honest
  skip — the UI-plugin **sandbox-escape gate found no hole** (all 12 escape vectors —
  parent cookies/DOM/location, session token, storage, off-allowlist network — blocked by
  the browser and the broker). Rolling `YY.N` scheme retained (this is 26.10, not a "1.0"
  tag). Non-blocking 26.10 follow-ups (documented, not release-gating) — **both now CLOSED in
  `26.11`**: (a) **masked-email on-send From-rewrite** — the store-layer alias service +
  lifecycle + routes shipped here; automatic envelope rewrite on send needed a per-send
  alias→target seam, which `26.11` implemented server-side (a `MaskedSubmitter` decorator at
  the submission construction seam rather than through the jail, so the `masked-email`
  `message-out` component stays an identity passthrough); and (b) an optional
  **`PUT /admin/oauth-dcr` admin toggle** — DCR shipped here config/CLI-enabled; `26.11`
  added the admin-session-gated `GET/PUT /admin/oauth-dcr` route (DCR stays default-disabled).
- **`26.9`** — enterprise SSO + the accessibility/i18n/perf/packaging hardening pass.
  **Full OIDC and SAML 2.0 single sign-on** as login backends (new `mw-sso` crate),
  configured per-deployment/domain via the admin panel + a `0009` `sso_config` table
  and surfaced as "Sign in with <IdP>" on the login screen: OIDC over the
  `openidconnect` crate (discovery, auth-code + **PKCE**, JWKS ID-token validation,
  userinfo, RP-logout — RustCrypto/rustls, **no openssl**), and a **hand-rolled
  pure-Rust SAML SP** (SP metadata, AuthnRequest, HTTP-POST ACS, exclusive-C14N +
  XML-DSig RSA/ECDSA-SHA256 validation, audience/replay defenses — no `samael`,
  no openssl/libxml) with a content-free login audit and first-login defaulting to
  allowlist/deny. **Both flows are proven end-to-end live against a real Keycloak
  26.0** (headless + real-browser → authenticated inbox). This milestone also folds
  in the 1.0-readiness hardening: a **WCAG 2.2 AA** audit + fixes across every web
  screen (calendar ARIA grid, ribbon tablist, dialog focus, non-color verdict
  badges) gated by axe in CI; **Fluent i18n** with an `en` baseline, a 12-locale
  structure + Weblate config + RTL/bidi plumbing (human translation pending);
  **§23 performance budgets** measured-and-gated in CI (cold-load, render, bundle,
  binary/image); and **packaging recipes** (Flatpak/F-Droid/winget/deb/rpm/AppImage/
  macOS-notarize). Structural size work: the five first-party plugin `.wasm`
  components are **externalized** from the server binary to a plugins dir, each
  **SHA-256 digest-pinned** (fail-closed integrity), and the §23 binary/image budgets
  are revised to measured-realistic values (binary <91MB, image <205MB = measured
  ×1.15, documented) since the full V7 feature set (wasmtime JIT + all protocols +
  crypto) is inherently larger than the original core-build targets. Security posture
  is best-effort self-hardening + a published external-audit-prep dossier (no funded
  audit — open-source). Verified: 934 Rust + 633 web tests; cargo-deny clean with no
  new advisory ignore and **no openssl anywhere**; live SSO E2E green vs real
  Keycloak. Rolling `YY.N` scheme retained (this is 26.9, not a "1.0" tag).
  Remaining ops follow-ups (not release-gating): store/signing account provisioning +
  submissions, and human translation review via Weblate.
- **`26.8`** — V7: extensibility, directory, AI, and Exchange/Gmail bridges (the
  last feature milestone before 1.0). A **WASM engine-plugin runtime** (`mw-plugin`
  over wasmtime + the WASI-p2 component model): capability-deny-by-default, per-
  plugin resource limits (epoch-deadline + memory ceiling + optional fuel → a
  clean `LimitExceeded`, never a host panic), an Ed25519 signed registry, and a
  host-mediated ABI (no ambient network/fs — outbound HTTP and OAuth tokens are
  host-held) — the jail is the security boundary, proven live with a real loaded
  component (out-of-allowlist host denied, resource trip observed). An **LDAP/GAL
  directory** (`mw-directory`, ldap3 over rustls — no openssl): GAL search in
  recipient fields, distribution-group expand-before-send, S/MIME cert + photo
  lookup, multi-directory priority, StartTLS/LDAPS, read-only. **Password-change
  backends** (`mw-passwd`): local/LDAP-3062/Dovecot/poppassd/HMAC-webhook, with
  client-side zero-access key-hierarchy re-wrap and coordinated credential re-seal.
  An **Assist (AI) subsystem** (`mw-assist`): a BYO-endpoint gateway (OpenAI-
  compatible/Anthropic/local-process, hand-rolled over rustls — no LLM SDK) with
  per-capability scoping, data-class ceilings, **E2EE content never forwarded by
  default**, content-free audit, a "what left the device" disclosure, and — by
  construction — no capability that sends/accepts/deletes (send stays human-gated;
  the assistant reuses the MCP tool surface). **Graph, EWS, and Gmail bridges** as
  first-party `wasm32-wasip2` plugins implementing the frozen `AccountBackend`
  trait — indistinguishable from IMAP to the engine, quirks isolated to the bridge,
  OAuth tokens never in the guest, EWS using **hand-rolled pure-Rust NTLMv2** (zero
  new deps); they boot-load from the registry and are full **read + send** accounts.
  Plus **MSG/OFT/DOCX export** (`mw-export` via cfb/docx-rs), a **Nextcloud** attach/
  share-link plugin, GAL/Assist/Nextcloud wired into the mailbox compose+read UX,
  and both V6 follow-ups closed (proxy-mode headless scoped-key REST reads; the real
  MCP unattended-send countersign resolver). New crates: mw-plugin, mw-directory,
  mw-passwd, mw-assist; new `plugins/` (bridge-graph/ews/gmail, languagetool,
  nextcloud). Verified: 846 Rust + 579 web tests; cargo-deny clean; a live E2E gate
  (12/12) against **real OpenLDAP + a real jailed plugin + a mock Assist endpoint**
  — plugin-backed account serves JMAP identically to IMAP via the boot path, bridge
  send routes to the provider exactly once, Assist redaction proven — which caught
  three real deployment gaps (bridge mail-sync cursor, LDAP-3062 result-code
  handling, and boot-time plugin loading) that were fixed before release.
  **Honest scope boundaries** (not overclaimed): bridges deliver **mail** through
  the jail — bridge calendar/tasks/reactions are implemented and fixture-tested but
  reachable only through a **post-1.0 WIT-export extension**; EWS **Kerberos** is a
  documented BYO-reverse-proxy gap (Basic + NTLMv2 ship); third-party (non-bundled)
  plugin byte-storage is post-1.0; and a bounded `quick-xml`-reader-DoS advisory
  ignore is scoped to write-only DOCX export. **V7 completion is not 1.0** — the
  distinct 1.0 hardening gate (WCAG 2.2 AA, translations, perf budgets, and a funded
  external audit incl. the MCP/plugin/Assist surfaces) is enumerated in
  `docs/ROADMAP-1.0.md`.
- **`26.7`** — V6: server depth — zero-access storage, admin, API/OAuth, MCP,
  Postgres, cache. An **optional zero-access (zero-knowledge) storage mode**:
  the client-side key hierarchy (Argon2id/WebAuthn-PRF → root key → KEK →
  per-account data keys) is built on the existing V4 `mw-crypto` WASM, rows are
  sealed with XChaCha20-Poly1305 (AAD = table‖row‖schema-version), and a
  device-pairing QR+SAS flow transfers the root key device-to-device with the
  server relaying only ciphertext. Its scope is stated honestly: the server at
  rest sees ciphertext, opaque IDs, sizes, and timestamps, and because it still
  proxies live IMAP/SMTP a malicious *active* server is a stronger adversary
  that this mode does **not** defend against — it protects data at rest, and
  search stays a client-built encrypted index. A **pluggable PostgreSQL
  backend** now sits behind `mw-store` alongside SQLite (backend chosen by DSN;
  `mailwoman migrate-store` copies SQLite→Postgres), a **layered cache**
  (`mw-cache`: moka→Valkey/Redis→store) with a per-class scope matrix that
  structurally excludes zero-access plaintext from Redis/memory, a **full admin
  panel** (domains/users/quotas/policy/integrations/observability + an
  append-only audit log, mirrored to a `mailwoman admin` CLI), **scoped API keys
  + an OAuth 2.1 AS** (mandatory PKCE + RFC 8707 resource indicators; keys
  Argon2id-hashed, shown once, with per-key scope/expiry/IP-allowlist/rate-limit
  enforced on `/api/v1`), an **MCP server** (`/mcp` + `mailwoman mcp-stdio`; ten
  scoped tools, mail content carrying untrusted-provenance labels, and send
  disabled by default — routed to the Outbox unless an admin-countersigned
  `unattended-send` key is used), plus HMAC-signed webhooks, a REST convenience
  layer, and OTLP/Prometheus observability (rustls throughout — no openssl). New
  crates: mw-cache, mw-admin, mw-oauth, mw-mcp (Postgres lands inside mw-store).
  SQLite single-user and the browser cookie path are unchanged. Verified: 624
  Rust + 529 web tests; cargo-deny clean with zero new advisory ignores; and a
  live E2E gate driving the real stack (`postgres:16` + `valkey:8` + a spawned
  server) 7/7 green — admin provisioning+audit, OAuth consent→scoped-key→REST
  enforcement matrix, MCP gated-send→Outbox, backend parity (SQLite==Postgres),
  and zero-access ciphertext-at-rest proven by a direct Postgres query. One
  Postgres-only backend bug (i64 bound into a BOOLEAN column) was caught by that
  live gate and fixed before release.
- **`26.6`** — V5: thin native shells. Tauri v2 desktop (Windows/macOS/Linux)
  and mobile (Android/iOS) shells that reuse the **same SPA bundle** as the web
  app behind a feature-detected `Platform` capability layer (`isTauri()` →
  native path, browser path unchanged). Native auth via bearer token (keychain-
  backed: DPAPI on Windows, Keychain on macOS, Keystore on Android); a
  self-contained mode that spawns the bundled mw-server on loopback; bundle-
  integrity gate on launch; native screen-capture protection
  (`WDA_EXCLUDEFROMCAPTURE` / `FLAG_SECURE`). Background delivery: a server
  WebPush/VAPID relay over **`web-push-native`** (pure-Rust RFC 8188/http-ece,
  no openssl C), UnifiedPush on Android, and a Service-Worker `mw-push-wake`
  consumer that resyncs a backgrounded tab. Verified: 496 Rust + 475 web tests;
  cargo-deny clean (Tauri tree vetted — permissive-only, unmaintained-only
  advisory ignores documented); desktop shell launched live on Windows
  (integrity gate, keychain, self-contained spawn, capture protection); Android
  CI-gated; iOS/APNs documented. Live-E2E gaps caught + fixed: CSP
  `wasm-unsafe-eval` for the crypto worker, `CryptoKey.id` serde default,
  calendar list/instances shape parity, `web-push`→`web-push-native` openssl
  swap, mobile command registration, and the dead `mw-push-wake` consumer.
- **`26.5`** — V4: crypto & security depth. OpenPGP + S/MIME end-to-end
  encryption with **private-key operations in a client-side WASM build** of
  mw-crypto (keys never reach the server unencrypted); decrypted mail is
  sanitized in-worker (mw-sanitize wasm) before the sandboxed iframe. A
  Security panel with DKIM/SPF/DMARC/ARC verdicts, Received-chain, signature
  and attachment-risk analysis, and sender controls that emit **real Sieve
  rules**. Engine-side DLP on the outbound path (PAN/IBAN/national-id
  detectors → warn/block, redacted audit). The three-position max-security
  opening switch. Hybrid X25519+ML-KEM-768 store-key wrapping. Server: WKD
  publishing, ARF abuse reports, an honest watermark overlay. New crate:
  mw-crypto (native + wasm). Verified: 430 Rust + 432 web tests; wasm build on
  Windows + Linux; PGP/S-MIME interop against recorded GnuPG/Thunderbird/
  Outlook fixtures; 8 live Playwright specs (browser-generated key →
  encrypt → send → decrypt → in-worker sanitize; DKIM pass/fail; DLP block;
  max-security). Two "unit-green but CSP/JMAP-dead" gaps caught + fixed at the
  live-E2E gate.
- **`26.4`** — V3: personal-information management. Calendar (all views —
  day/3-day/work-week/week/month/tri-month/schedule/agenda/year — recurrence,
  reminders, attendees, iTIP invites, free/busy, conflict detection),
  tasks (VTODO + My Day + subtasks), encrypted-at-rest notes (rich text,
  tags/colors/pins, cross-links), and contacts (address books, groups, merge,
  vCard/CSV import/export, Compose autocomplete) — synced over CalDAV/CardDAV,
  serialized as iCalendar/vCard, behind a Mailwoman-native PIM surface reusing
  the JMAP envelope. New crates: mw-ics, mw-dav, mw-carddav. Server adds
  calendar/addressbook sharing + a holiday feed. Verified: 367 Rust + 312 web
  tests; Radicale CalDAV/CardDAV conformance (engine<->real-CalDAV round-trip);
  live Playwright E2E across all four modules through the real UI. Four
  end-to-end contract gaps caught + fixed at the E2E gate before release.
- **`26.3`** — V2: modern mail layer + theming. Engine-side Tantivy search
  (operators + saved searches), offline (Service Worker + encrypted OPFS +
  replay queue), WebSocket/SSE realtime push, multi-window (BroadcastChannel),
  the modern mail UX (tags/pins/snooze/sweep/undo-send/outbox/send-later/
  follow-up/focused+unified inbox/virtualized list), Sieve rules, identities,
  EML/mbox/TXT/Markdown export, the vanilla-extract design-token theming system
  (light/dark/HC/AMOLED + Grove woody themes) with self-hosted font puller and
  an optional ribbon preset, and sandboxed embedded attachment viewers
  (image/PDF/video) + a global Attachments module. Server gains a rustls-acme
  TLS listener, per-message CSP + CSRF/session hardening, and a blob-download
  route. New crates: mw-search, mw-sieve, mw-export. Verified: 283 Rust + 214
  web tests; live-stack Playwright E2E across all V2 features (offline, push,
  multi-window, viewers, search operators, theming, export). Six real
  end-to-end gaps caught and fixed at the E2E gate before release.
- **`26.2`** — V1: real mail backends. IMAP4rev2 + POP3 + SMTP submission +
  MIME parse/build behind a frozen `AccountBackend` seam, driven by
  `mw-engine` which presents the same JMAP surface the web UI already speaks
  (engine mode vs V0 proxy mode, config-switched). Sync ladder
  (QRESYNC/CONDSTORE/UID-window + POP3 UIDL), engine-side JWZ threading,
  autoconfig ladder, encrypted message cache. New crates: mw-imap, mw-pop3,
  mw-smtp, mw-mime, mw-engine, mw-autoconfig. Greenmail/Dovecot CI
  conformance + a Playwright E2E driving a real IMAP account through the
  unmodified web UI.
- **`26.1`** — first rolling release. V0 walking skeleton (SPEC §27): wired
  webmail path (SolidJS client → mw-server JMAP proxy + sanitize worker →
  JMAP upstream), Docker/CI, E2E. Supersedes the pre-adoption `v0.0.0`
  placeholder tag, which was removed.
