// A capped-height box must own a scrollport (t24-e13).
//
// The Compose dialog carried `max-height: 90vh` on a `display:flex` column with no
// `overflow`. That does not clamp anything useful: flex children default to
// `min-height:auto`, so they refuse to shrink and simply overflow the capped box —
// and `.compose__backdrop` is `position:fixed` with `place-items:center`, which
// provides no scrollport either, so the overflow was unreachable rather than merely
// off-screen.
//
// Measured in Chromium at Playwright's 1280x720 viewport, on the dialog's DEFAULT
// state: the box clamped to 648px while its content was 805px, leaving the footer's
// Send button at y=798..835 — entirely below the fold. Playwright reports that as
// "element is visible, enabled and stable / element is outside of the viewport",
// which is exactly what t24-e12 recorded against e2e, e2e-engine and e2e-crypto.
// Confirmed as a real click failure and not just geometry: with `overflow-y:visible`
// forced back on, `locator.click()` times out on the same message; with
// `overflow-y:auto` it succeeds.
//
// This is a source-level guard rather than a layout test because it needs no browser
// and no built bundle, and the property it pins is the one that was actually missing.
// The behavioural coverage is the Send click itself, which several e2e specs already
// perform (imap-engine, modern-ux, offline, theming, happy-path).

import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

/** `selector { ...decls... }` pairs, good enough for this flat stylesheet. */
function rules(css: string): { selector: string; body: string }[] {
  const out: { selector: string; body: string }[] = [];
  for (const m of css.matchAll(/([^{}]+)\{([^}]*)\}/g)) {
    const raw = (m[1] ?? '').trim().split('\n').pop() ?? '';
    out.push({ selector: raw.trim(), body: m[2] ?? '' });
  }
  return out;
}

describe('app.css — a capped-height box owns a scrollport', () => {
  const css = readFileSync(resolve(process.cwd(), 'src/styles/app.css'), 'utf8');

  it('every rule with max-height also declares overflow', () => {
    const offenders = rules(css)
      .filter((r) => /(^|[\s;])max-height\s*:/.test(r.body))
      .filter((r) => !/overflow(-x|-y)?\s*:/.test(r.body))
      .map((r) => r.selector);
    expect(
      offenders,
      'a max-height with no overflow does not clamp — children overflow it and become unreachable',
    ).toEqual([]);
  });

  it('.compose specifically keeps its scrollport (the regression that was fixed)', () => {
    const compose = rules(css).find((r) => r.selector === '.compose');
    expect(compose, '.compose rule must exist').toBeDefined();
    expect(compose!.body).toMatch(/max-height\s*:\s*90vh/);
    expect(compose!.body, 'without this the Send button lands below the fold').toMatch(
      /overflow-y\s*:\s*auto/,
    );
  });
});
