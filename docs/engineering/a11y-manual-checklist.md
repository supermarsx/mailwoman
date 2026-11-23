# Accessibility — manual verification checklist (WCAG 2.2 AA)

Companion to the automated `axe-core` gate (`.github/workflows/a11y.yml` →
`apps/web/e2e/a11y.spec.ts`). axe catches ~30–40% of WCAG issues — the machine-checkable
ones (contrast, ARIA required-children/parent, names/roles, labels). **This checklist is
the other 60%**: the things only a human with a screen reader, a keyboard, and the OS
accessibility switches can confirm. Run it before a release that touches UI (SPEC §24 /
`docs/ROADMAP-1.0.md` L21).

Scope: the audited screens are login/consent, admin sign-in, the mailbox + **Ribbon**, the
**Settings** dialog + menus, the **Calendar month grid**, and the reader **Security panel**.

## How to run

- **Screen readers:** NVDA (Windows, free), VoiceOver (macOS, `Cmd+F5`), Orca (Linux). Test at
  least one per platform you ship. Turn the screen off / eyes closed for the SR passes.
- **Keyboard only:** unplug the mouse. `Tab` / `Shift+Tab`, arrows, `Home`/`End`, `Enter`/`Space`,
  `Esc`. Nothing may require a pointer.
- **OS switches:** reduced motion, high contrast / forced colors, and a 200% browser zoom.

Mark each item Pass / Fail / N-A. A Fail routes to the owning web area (e1 mail · e2 pim ·
e3 shell/settings · e4 crypto) — the same routing the axe gate uses.

---

## 1. Keyboard operability (WCAG 2.1.1, 2.1.2, 2.4.3, 2.4.7)

- [ ] Every interactive control is reachable by `Tab` in a sensible order; nothing is skipped.
- [ ] A **visible focus indicator** is present on every focused control (the `vars.a11y.focusRing`
      token) — never focus that lands with no visible ring.
- [ ] **No keyboard trap:** focus can always move on with `Tab`/`Shift+Tab` (except an intentional
      modal trap, which `Esc` must release).
- [ ] **Ribbon** (mailbox toolbar, `role="tablist"`): `Tab` reaches it as **one** stop; `←/→`
      (and `Home`/`End`) move between Home/View/Folder tabs with selection following focus.
- [ ] **Message list:** `↑/↓`, `Home`/`End` move a roving cursor; the virtual window scrolls the
      focused row into view; `Enter` opens it.
- [ ] **Calendar month grid:** arrows move one day (←/→) / one week (↑/↓), `Home`/`End` = week
      ends, `PageUp`/`PageDown` = ±month (`Shift` = ±year), `Enter`/`Space` opens the focused day.
      Focus stays continuous across month boundaries.
- [ ] **Dialogs** (Compose, Settings, Event editor, key Generate/Import, contacts import/merge):
      focus moves **into** the dialog on open, is **trapped** while open, `Esc` closes, and focus
      **returns to the trigger** on close.
- [ ] **Menus / view switcher / admin nav** (roving tabindex): one `Tab` stop, arrows traverse.

## 2. Screen-reader semantics (WCAG 1.3.1, 4.1.2, 4.1.3)

- [ ] Landmarks announce: `main`, the labelled `nav`s (Mailboxes / Apps), search, dialogs
      (`aria-modal` "dialog"), regions (`Message security details`).
- [ ] **Ribbon** reads as a tab list: "tab, selected, N of M"; the panel is associated
      (`aria-controls` / `aria-labelledby`).
- [ ] **Message list** rows announce position ("N of M") and unread/current state.
- [ ] **Calendar grid** reads as a grid; each day cell announces the **full date + event count**
      (e.g. "Tuesday, July 14, 2026, 2 events"); moving focus re-announces via the live region.
- [ ] **Security panel** verdict badges announce pass/fail **as text** ("DKIM passed",
      "Signature verified"), never colour-only; the glyph is decorative (`aria-hidden`).
- [ ] **Toasts / status** (send undo, saved, errors) announce via live regions without stealing
      focus; `role="alert"` for errors, `role="status"` for polite updates.
- [ ] **Untrusted text is bidi-isolated** (subjects, sender/display names, filenames): a crafted
      RTL-override subject must NOT reorder surrounding UI (the `exe.png`↔`gnp.exe` spoof).
- [ ] Every icon-only button has a meaningful accessible name (pin / tear-off / close / dismiss).

## 3. Reduced motion (WCAG 2.3.3)

- [ ] Enable the OS "reduce motion" setting. Reload. Transitions/animations collapse to instant
      (the `vars.a11y.motionDuration*` tokens + the global neutralizer) — no dialog slide, no
      toast fly-in, no spinner that relies on motion to convey state.

## 4. High contrast / forced colors (WCAG 1.4.3, 1.4.11, 1.4.1)

- [ ] Turn on Windows High Contrast / `forced-colors`. All text, borders, and focus rings remain
      visible; the focus ring **thickens** (`vars.a11y.focusRingWidth` → 3px).
- [ ] Status is never conveyed by colour alone (verdict badges keep their glyph + text; selection
      uses `aria-pressed`/`aria-selected`, not just a colour swap).
- [ ] Contrast ≥ 4.5:1 for normal text, ≥ 3:1 for large text — spot-check the muted/secondary
      text (list preview, dates, hints), which the axe `color-contrast` gate also enforces.

## 5. Target size & zoom (WCAG 2.5.8, 1.4.10, 1.4.4)

- [ ] Interactive targets are ≥ 24×24 CSS px (the `vars.a11y.touchTarget` token) — check the
      icon-only row actions, dialog ✕, sub-tab pin/tear-off/close, undo dismiss, calendar chips.
- [ ] At 200% browser zoom and 320px width, content **reflows** with no loss of function and **no
      horizontal page scroll** (inner regions may scroll; the page must not — see the RTL/pseudo
      smoke `i18n-rtl.spec.ts`).

## 6. Internationalization / RTL (SPEC §24)

- [ ] Under a right-to-left direction the whole UI **mirrors** (logical CSS properties); the
      **calendar grid** mirrors its day order and arrow-key direction.
- [ ] With a long-translation (pseudolocale) pass, no chrome clips or truncates; labels wrap
      rather than overflow.

---

### Sign-off

| Area (owner) | Keyboard | SR | Reduced motion | Forced colors | Target/zoom | RTL |
|---|---|---|---|---|---|---|
| Mailbox + Ribbon (e1) | | | | | | |
| Calendar grid (e2) | | | | | | |
| Shell / Settings / Admin (e3) | | | | | | |
| Security / crypto (e4) | | | | | | |
| Login / Consent (e3) | | | | | | |

Reviewer: __________________  Date: __________  Build/tag: __________
