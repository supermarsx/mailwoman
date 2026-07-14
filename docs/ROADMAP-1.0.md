# Roadmap to Mailwoman 1.0

**Status:** skeleton authored by t7-e0 (V7 scaffold); the 1.0-gate checklist and the
deferred-items list were finalized by t7-e15 at the 26.8 CI/docs pass. t7-e17 applies
the release-time touch at the 26.8 (V7) tag.

V7 (release **26.8**) delivers the **last features** on the SPEC roadmap (§27): the
WASM plugin runtime, LDAP/GAL directory, password-change backends, Assist (AI), the
Graph/EWS/Gmail bridges, MSG/OFT/DOCX export, and Nextcloud. **V7 completion is not
1.0.** 1.0 is a distinct **hardening / accessibility / i18n / audit** milestone — no
new features, everything below must be green.

This document tracks the remaining 1.0 gate. It is a living checklist; each item
links to its SPEC section and owner as work begins.

---

## 1.0 release gate (SPEC §27, §7)

- [ ] **WCAG 2.2 AA audit** as a release gate — full keyboard operability,
      screen-reader-tested flows (including the calendar grid patterns),
      reduced-motion, high-contrast, and touch-target sizing.
- [ ] **Translations** for en / de / fr / es / pt-BR / nl / it / pl / ru / uk / zh /
      ja via Weblate; **RTL first-class** (mirrored layouts including the calendar;
      bidi-isolation for mixed-direction subjects).
- [ ] **Performance gates** measured in CI (SPEC §23): initial JS < 250 KB,
      cold-to-inbox < 1.5 s, search p95 < 50 ms, binary < 45 MB / image < 30 MB, etc.
- [ ] **External security audit** — crypto + web app **+ the new MCP, plugin
      sandbox, and Assist surfaces** — funded, run, and **findings resolved** (the
      SPEC's hard 1.0 condition).
- [ ] **Packaging / store presence** finalized (winget / notarized-macOS / AppImage /
      deb / rpm / Flatpak / F-Droid / Play / App-Store per §16), auto-update
      signing/staging, hosting-panel recipes (§18.1) CI-smoke-tested.

## Deferred-to-1.0-or-post items (from the V7 OUT list)

**Update (release 26.10):** the deferred-spec tail below shipped — additively, over the
frozen V7 surfaces — in the rolling `26.10` release. See the `26.10` entry in
`VERSIONING.md` for the full changelog and the two non-blocking follow-ups.

- [x] **UI-plugin (TypeScript sandboxed) tier** (§22.2) — **shipped in 26.10**: approved
      plugins render in an opaque-origin `<iframe sandbox="allow-scripts">` (no
      `allow-same-origin`) behind a deny-by-default `postMessage` broker, with an Ed25519
      signed registry + admin approval and an unsigned-plugin banner. The sandbox-escape
      gate (all 12 vectors) found no hole in a real browser. (V7 shipped the engine WASM
      tier only.)
- [x] **Calendar / tasks / reactions / voting / recall plugin-seam export** (§6.5, §22)
      — **shipped in 26.10**: a second `mailwoman:plugin-pim` WIT world (`calendar` /
      `tasks` / `bridge-parity` interfaces) is bound by the host via per-interface export
      probing, so the Graph/EWS/Gmail bridges now drive these surfaces **through the plugin
      jail** with honest per-provider support (Graph all six, EWS calendar + tasks, Gmail
      none). `mw-engine` prefers the bridge when a capability is genuinely advertised and
      otherwise keeps the byte-unchanged standards fallback; `account-backend`-only
      components (LanguageTool, Nextcloud) load unchanged. (See `docs/bridges/`.)
- [x] **Rspamd / SpamAssassin trainer plugins** (§10.8) — **shipped in 26.10** as jailed
      `wasm32-wasip2` classifiers reaching their daemons only through host `http-fetch`
      under a net allowlist (no C linkage), feeding a fail-soft `SpamHook` in
      `Engine::ingest` (a classifier failure never drops mail).
- [x] **Masked-email plugins** (§28.4) — **shipped in 26.10**: store-layer alias service +
      lifecycle + `/api/masked/*` routes. **On-send From-rewrite closed in 26.11**: a
      server-side `MaskedSubmitter` decorator rewrites the envelope `MAIL FROM` to the
      canonical alias for an account-owned enabled alias and fails closed on
      cross-account/disabled/deleted aliases (inner submitter never called).
- [x] **OAuth dynamic client registration** (V6 follow-up c) — **shipped in 26.10**:
      RFC 7591 register + RFC 7592 read/update/delete in `mw-oauth`, **default-disabled and
      ops-gated** (policy row + redirect-host-suffix allowlist + registration-access-tokens,
      no scope escalation). **Admin-enable route closed in 26.11**: admin-session-gated
      `GET/PUT /admin/oauth-dcr` (parity with SSO/UI-plugin admin), DCR still default-disabled.
- [ ] **EWS native Kerberos/NTLM-SSO via system GSSAPI** (§6.5, R2) — **partially addressed
      in 26.10**: the BYO SPNEGO reverse-proxy path is documented + fixture-tested
      (`docs/deploy/kerberos.md` — IIS+ARR+KCD / Apache `mod_auth_gssapi` / nginx SPNEGO
      recipes) on top of the shipped Basic + pure-Rust NTLMv2. **Native GSSAPI stays
      unbuilt**: it needs a non-permissive `-sys`-C dep, so it is a **flagged human
      license-floor decision** (feature-gated, off by default) the autonomous pipeline will
      not take.
- [x] **MSG/OFT deep write fidelity** (embedded objects, custom named properties,
      §28.8) — **shipped in 26.10**: additive `__nameid` named-property map + embedded-OLE
      message writing in `mw-export`; a message with no named properties or embedded objects
      stays byte-identical to the 26.9 floor.
- [x] **Gmail bridge** — this was the §27 scope-cut ladder's first candidate (R6) but
      was **NOT cut**: it shipped fully in V7 (true label semantics + history-ID delta
      sync). No follow-up needed.

## Open gaps found during V7 scaffolding (e0)

- [ ] **OIDC / SAML SSO login is NOT built** (SPEC §18.3 / §1; assumption (g)).
      A repository search at e0 found **no** OIDC/SAML/SSO implementation in any
      crate or the web app — it is **not** in V7's committed scope (which is
      password-*change* + LDAP-bind login, §18.3). If enterprise SSO is a 1.0
      requirement, it is **unbuilt work** and must be scheduled here; otherwise
      record the explicit decision to defer it past 1.0.
