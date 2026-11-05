import { describe, expect, it } from 'vitest';
import { THEME_LIST } from './registry.ts';
import { ACCENT_IDS, accentContextOf } from './accentMath.ts';
import { contrastRatio } from './contrast.ts';
import { applyDerivedLink, derivedLinkFor, DERIVED_LINK_PROPERTY, linkColor } from './accentLink.ts';

describe('accent-derived link colour', () => {
  it('is null without an accent, so the theme link stays', () => {
    expect(derivedLinkFor('light', '', '')).toBeNull();
    const root = document.createElement('div');
    root.style.setProperty(DERIVED_LINK_PROPERTY, '#123456');
    applyDerivedLink(root, 'light', '', '');
    expect(root.style.getPropertyValue(DERIVED_LINK_PROPERTY)).toBe('');
  });

  it('reads 4.5:1 on every surface for every theme x named accent and a pale custom accent', () => {
    for (const entry of THEME_LIST) {
      const surfaces = accentContextOf(entry.palette).surfaces;
      for (const id of [...ACCENT_IDS, 'custom'] as const) {
        const link = id === 'custom' ? derivedLinkFor(entry.id, '', '#ffee99') : derivedLinkFor(entry.id, id, '');
        expect(link, `${entry.id}/${id}`).toMatch(/^#[0-9a-f]{6}$/);
        for (const s of surfaces) expect(contrastRatio(link!, s), `${entry.id}/${id} on ${s}`).toBeGreaterThanOrEqual(4.5);
      }
    }
  });

  it('falls back to the theme link token', () => {
    expect(linkColor).toContain(DERIVED_LINK_PROPERTY);
  });
});
