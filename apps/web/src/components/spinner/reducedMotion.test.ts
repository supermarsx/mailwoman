// Under reduced motion no busy indicator may freeze: each one rotates, pulses
// or fades. jsdom cannot evaluate keyframes, so this pins the sources: the
// converted files carry the calm fallback and the dead `busy` style is gone.
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const src = (p: string): string => readFileSync(`${process.cwd()}/src/${p}`, 'utf8');

describe('reduced-motion busy indicators', () => {
  it.each([
    ['components/asyncState.css.ts', 'calmAnimation'],
    ['screens/Admin/shell/stats.css.ts', 'calmAnimation'],
    ['components/breadcrumbs/breadcrumbs.css.ts', 'calmAnimation'],
    ['screens/Admin/Diagnostics/diagnostics.css.ts', 'calmAnimation'],
    ['components/compose/compose-chrome.css', 'compose-status-calm'],
    ['components/address-contact-card.css', 'animation-duration: 3s'],
    ['viewers/sandbox.ts', 'mw-reader-image-pulse 3s'],
  ])('%s keeps a visible fallback', (file, needle) => {
    expect(src(file)).toContain(needle);
  });

  it('skeleton pulses and spinners are not set to animation none', () => {
    const a = src('components/asyncState.css.ts');
    expect(a).not.toMatch(/skeletonPulse=style\(\{[^}]*STILL/);
    expect(a).not.toMatch(/folderSkeleton(Icon|Line) = style\(\{[^\n]*STILL/);
    expect(src('screens/Admin/Diagnostics/diagnostics.css.ts')).not.toMatch(/running"\] svg`, \{[^\n]*animation: 'none'/);
    expect(src('components/compose/compose-chrome.css')).not.toMatch(/--spin svg \{ animation:none/);
  });

  it('the global reduced-motion rule exempts spinners and the unused shell busy style is gone', () => {
    expect(src('theme/themes.css.ts')).toContain('[data-mw-spinner]');
    expect(src('screens/Admin/shell/shell.css.ts')).not.toContain('export const busy');
  });
});
