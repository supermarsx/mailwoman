// The pre-paint boot script (t52). It is plain JavaScript that Vite does not
// bundle, so these tests run the file's own text, the way the browser does.
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { defaultAppearance, resolveThemeName, type AppearancePrefs } from '../theme/appearance.ts';
import { THEME_REGISTRY } from '../theme/registry.ts';
import type { ThemeName } from '../theme/contract.css.ts';

const SOURCE = readFileSync(join(dirname(fileURLToPath(import.meta.url)), 'boot.js'), 'utf8');
const run = (): void => { new Function(SOURCE)(); };
const root = document.documentElement;
const BOOT_ATTRS = ['data-theme', 'data-appearance', 'data-density', 'data-accent', 'data-badge', 'data-mw-boot', 'data-reduced-motion'];

function snapshot(prefs: AppearancePrefs, theme: ThemeName, extra: Record<string, unknown> = {}): void {
  localStorage.setItem('mw.boot.theme', JSON.stringify({
    v: 1, theme, appearance: THEME_REGISTRY[theme].appearance, mode: prefs.mode, light: prefs.lightTheme, dark: prefs.darkTheme,
    schedule: prefs.schedule, density: prefs.density, accent: prefs.accentId, badge: prefs.badgeColor, font: '', props: {}, ...extra,
  }));
}

function mockDark(dark: boolean): void {
  vi.stubGlobal('matchMedia', (q: string) => ({ matches: q.includes('dark') ? dark : false, media: q }));
}

beforeEach(() => {
  localStorage.clear();
  for (const a of BOOT_ATTRS) root.removeAttribute(a);
  root.removeAttribute('style');
  document.head.querySelectorAll('meta[name="mw-console"]').forEach((m) => m.remove());
  mockDark(false);
});
afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
  localStorage.clear();
  for (const a of BOOT_ATTRS) root.removeAttribute(a);
  root.removeAttribute('style');
});

describe('boot.js', () => {
  it('leaves the CSS defaults alone with nothing stored, and draws no mailbox', () => {
    run();
    expect(root.hasAttribute('data-theme')).toBe(false);
    expect(root.getAttribute('data-mw-boot')).toBe('plain');
  });

  it('replays the last applied fixed theme, density, accent, badge and font', () => {
    const prefs: AppearancePrefs = { ...defaultAppearance(), theme: 'light', density: 'compact', accentId: 'teal' as AppearancePrefs['accentId'], badgeColor: 'accent', chosen: true };
    snapshot(prefs, 'light', { font: 'system-ui, sans-serif', props: { '--x-accent': '#123456', color: 'red' } });
    run();
    expect(root.getAttribute('data-theme')).toBe('light');
    expect(root.getAttribute('data-appearance')).toBe('light');
    expect(root.style.getPropertyValue('color-scheme')).toBe('light');
    expect(root.getAttribute('data-density')).toBe('compact');
    expect(root.getAttribute('data-accent')).toBe('teal');
    expect(root.getAttribute('data-badge')).toBe('accent');
    expect(root.style.getPropertyValue('--mw-ui-font')).toBe('system-ui, sans-serif');
    expect(root.style.getPropertyValue('--x-accent')).toBe('#123456');
    // Only custom properties are replayed from storage.
    expect(root.style.getPropertyValue('color')).toBe('');
  });

  it.each([true, false])('resolves "follow system" the way the theme layer does (OS dark: %s)', (dark) => {
    mockDark(dark);
    const prefs: AppearancePrefs = { ...defaultAppearance(), mode: 'system', lightTheme: 'light', darkTheme: 'dark', chosen: true };
    // Stored while the OS said the opposite: the custom accent was for that pack.
    snapshot(prefs, dark ? 'light' : 'dark', { props: { '--x-accent': '#123456' } });
    run();
    const expected = resolveThemeName(prefs, { systemDark: dark, now: new Date() });
    expect(root.getAttribute('data-theme')).toBe(expected);
    expect(root.getAttribute('data-appearance')).toBe(dark ? 'dark' : 'light');
    expect(root.style.getPropertyValue('--x-accent')).toBe('');
  });

  it.each(['03:00', '12:00', '21:30'])('resolves the schedule the way the theme layer does at %s', (time) => {
    vi.useFakeTimers({ toFake: ['Date'] });
    vi.setSystemTime(new Date(`2026-10-29T${time}:00`));
    const prefs: AppearancePrefs = { ...defaultAppearance(), mode: 'schedule', chosen: true };
    snapshot(prefs, 'dark');
    run();
    expect(root.getAttribute('data-theme')).toBe(resolveThemeName(prefs, { systemDark: false, now: new Date() }));
  });

  it('draws the mailbox only when a session was open, and not on consent pages', () => {
    localStorage.setItem('mw.boot.session', '1');
    run();
    expect(root.getAttribute('data-mw-boot')).toBe('mail');
    history.replaceState(null, '', '/oauth/authorize');
    run();
    expect(root.getAttribute('data-mw-boot')).toBe('plain');
    history.replaceState(null, '', '/');
  });

  it('does nothing on the admin console page', () => {
    snapshot({ ...defaultAppearance(), theme: 'light', chosen: true }, 'light');
    localStorage.setItem('mw.boot.session', '1');
    const meta = document.createElement('meta');
    meta.name = 'mw-console';
    meta.content = 'admin';
    document.head.append(meta);
    run();
    expect(root.hasAttribute('data-theme')).toBe(false);
    expect(root.hasAttribute('data-mw-boot')).toBe(false);
  });

  it('applies stored geometry only for the same layout class, and clamps it', () => {
    const wide = window.innerWidth >= 768;
    localStorage.setItem('mw.boot.geometry', JSON.stringify({ v: 1, wide, side: 300, tabs: 43, top: 70, list: -1, lead: 99999, row: 64 }));
    run();
    expect(root.style.getPropertyValue('--mwb-side')).toBe('300px');
    expect(root.style.getPropertyValue('--mwb-row')).toBe('64px');
    expect(root.style.getPropertyValue('--mwb-list')).toBe('');
    expect(root.style.getPropertyValue('--mwb-lead')).toBe('');
    root.removeAttribute('style');
    localStorage.setItem('mw.boot.geometry', JSON.stringify({ v: 1, wide: !wide, side: 300 }));
    run();
    expect(root.style.getPropertyValue('--mwb-side')).toBe('');
  });

  it('survives corrupt storage', () => {
    localStorage.setItem('mw.boot.theme', '{not json');
    localStorage.setItem('mw.boot.geometry', '[]');
    expect(run).not.toThrow();
    expect(root.hasAttribute('data-theme')).toBe(false);
  });
});
