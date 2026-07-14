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

- [ ] **UI-plugin (TypeScript sandboxed) tier** (§22.2) — document-only in V7; V7 ships
      the engine (WASM) plugin tier only.
- [ ] **Calendar / tasks / reactions / voting / recall plugin-seam export** (§6.5, §22)
      — the frozen WIT world exports the **account-backend (MAIL)** interface only. The
      bridges already **implement and fixture-test** these surfaces (advertised via
      `capabilities()`), but they are **not yet drivable through the plugin seam** — the
      WIT export for them is a post-1.0 extension. Until then the UI's existing
      standards/header-convention fallbacks handle them. (See
      `docs/RELEASE-NOTES-26.8.md` and `docs/bridges/`.)
- [ ] **Rspamd / SpamAssassin trainer plugins** (§10.8) — the LanguageTool plugin
      already proves the plugin runtime end-to-end, so these were off the V7 path.
- [ ] **Masked-email plugins** (§28.4).
- [ ] **OAuth dynamic client registration** (V6 follow-up c) — bridges use
      Mailwoman-as-OAuth-*client*; admin-seeded / BYO client IDs suffice. Add only if
      third-party self-registration is wanted.
- [ ] **EWS native Kerberos/NTLM-SSO via system GSSAPI** (§6.5, R2) — pure-Rust
      NTLMv2 + Basic ship in V7; Kerberos needs non-permissive platform libs. BYO
      reverse-proxy-auth is the documented interim path; native Kerberos is post-1.0.
- [ ] **MSG/OFT deep write fidelity** (embedded objects, custom named properties,
      §28.8) — V7 ships the body + attachments + headers floor; deep fidelity is
      best-effort.
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
