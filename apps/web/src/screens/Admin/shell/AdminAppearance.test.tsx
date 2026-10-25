import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { createRoot } from 'solid-js';
import { AdminAppearance } from './AdminAppearance.tsx';
import {
  ADMIN_APPEARANCE_KEY,
  WEBMAIL_APPEARANCE_KEY,
  createAdminAppearance,
  defaultAdminAppearance,
  parseAdminAppearance,
  resolveAdminTheme,
  type AdminAppearanceStore,
  type AdminAppearancePrefs,
} from './appearanceStore.ts';
import { ACCENT_IDS } from '../../../theme/accentMath.ts';
import { THEME_LIST } from '../../../theme/registry.ts';
import { defaultAppearance } from '../../../theme/appearance.ts';
import { expectNoAxeViolations, expectNoInlineStyle } from './uxTestkit.ts';

const root = (): HTMLElement => document.documentElement;

function memoryStore(initial: AdminAppearancePrefs | null = null): AdminAppearanceStore & { saved: (AdminAppearancePrefs | null)[] } {
  let value = initial;
  const saved: (AdminAppearancePrefs | null)[] = [];
  return {
    saved,
    load: () => value,
    save(p) {
      value = p;
      saved.push(p);
    },
  };
}

/** `matchMedia` reporting a dark OS, with a way to flip it. */
function stubColorScheme(dark: boolean): { flip(dark: boolean): void } {
  const listeners = new Set<(e: { matches: boolean }) => void>();
  let state = dark;
  vi.stubGlobal('matchMedia', (q: string) => ({
    get matches() {
      return q.includes('dark') ? state : false;
    },
    media: q,
    addEventListener: (_: string, fn: (e: { matches: boolean }) => void) => listeners.add(fn),
    removeEventListener: (_: string, fn: (e: { matches: boolean }) => void) => listeners.delete(fn),
  }));
  return {
    flip(next) {
      state = next;
      for (const fn of listeners) fn({ matches: next });
    },
  };
}

beforeEach(() => {
  localStorage.clear();
  for (const a of ['data-theme', 'data-accent', 'data-appearance', 'data-density']) root().removeAttribute(a);
});
afterEach(() => vi.unstubAllGlobals());

describe('appearance choice (pure)', () => {
  it('defaults to the webmail theme this browser chose, else Dark', () => {
    expect(defaultAdminAppearance(null, false)).toEqual({ theme: 'dark', accent: '', mode: 'dark' });
    expect(defaultAdminAppearance({ ...defaultAppearance(), theme: 'paper', chosen: false }, false)).toEqual({ theme: 'dark', accent: '', mode: 'dark' });
    expect(defaultAdminAppearance({ ...defaultAppearance(), theme: 'paper', accentId: 'teal', chosen: true }, false)).toEqual({
      theme: 'paper',
      accent: 'teal',
      mode: 'light',
    });
    expect(defaultAdminAppearance({ ...defaultAppearance(), mode: 'system', lightTheme: 'grove-light', darkTheme: 'grove-dark', chosen: true }, true)).toEqual({
      theme: 'grove-dark',
      accent: '',
      mode: 'system',
    });
  });

  it('the mode picks the light or dark variant of the chosen pack', () => {
    expect(resolveAdminTheme({ theme: 'grove-dark', accent: '', mode: 'light' }, true)).toBe('grove-light');
    expect(resolveAdminTheme({ theme: 'grove-dark', accent: '', mode: 'system' }, false)).toBe('grove-light');
    expect(resolveAdminTheme({ theme: 'grove-light', accent: '', mode: 'system' }, true)).toBe('grove-dark');
    expect(resolveAdminTheme({ theme: 'terminal', accent: '', mode: 'dark' }, false)).toBe('terminal');
  });

  it('rejects stored junk', () => {
    expect(parseAdminAppearance({ theme: 'nope' })).toBeNull();
    expect(parseAdminAppearance('x')).toBeNull();
    expect(parseAdminAppearance({ theme: 'paper', accent: 'mauve', mode: 'sideways' })).toEqual({ theme: 'paper', accent: '', mode: 'light' });
  });
});

describe('createAdminAppearance', () => {
  it('applies the default (webmail theme) to :root without storing anything', () => {
    stubColorScheme(false);
    localStorage.setItem(WEBMAIL_APPEARANCE_KEY, JSON.stringify({ ...defaultAppearance(), theme: 'ocean-dark', accentId: 'magenta', density: 'compact', chosen: true }));
    createRoot((dispose) => {
      const c = createAdminAppearance();
      expect(c.chosen()).toBe(false);
      expect(root()).toHaveAttribute('data-theme', 'ocean-dark');
      expect(root()).toHaveAttribute('data-accent', 'magenta');
      expect(root()).toHaveAttribute('data-appearance', 'dark');
      expect(root()).toHaveAttribute('data-density', 'compact');
      expect(localStorage.getItem(ADMIN_APPEARANCE_KEY)).toBeNull();
      dispose();
    });
  });

  it('a choice applies at once, is kept in mw.admin.appearance and leaves the webmail preference alone', () => {
    stubColorScheme(false);
    const webmail = JSON.stringify({ ...defaultAppearance(), theme: 'light', chosen: true });
    localStorage.setItem(WEBMAIL_APPEARANCE_KEY, webmail);
    createRoot((dispose) => {
      const c = createAdminAppearance();
      c.setTheme('terminal');
      expect(root()).toHaveAttribute('data-theme', 'terminal');
      expect(c.prefs().mode).toBe('dark');
      c.setAccent('blue');
      expect(root()).toHaveAttribute('data-accent', 'blue');
      expect(JSON.parse(localStorage.getItem(ADMIN_APPEARANCE_KEY)!)).toEqual({ theme: 'terminal', accent: 'blue', mode: 'dark' });
      expect(localStorage.getItem(WEBMAIL_APPEARANCE_KEY)).toBe(webmail);
      dispose();
    });
    createRoot((dispose) => {
      createAdminAppearance();
      expect(root()).toHaveAttribute('data-theme', 'terminal');
      dispose();
    });
  });

  it('"Match the system" follows the OS live, inside the chosen pack', () => {
    const os = stubColorScheme(false);
    createRoot((dispose) => {
      const c = createAdminAppearance({ store: memoryStore(), webmail: () => null });
      c.setTheme('plum-dark');
      c.setMode('system');
      expect(root()).toHaveAttribute('data-theme', 'plum-light');
      os.flip(true);
      expect(root()).toHaveAttribute('data-theme', 'plum-dark');
      expect(root()).toHaveAttribute('data-appearance', 'dark');
      c.setTheme('slate-light');
      expect(c.prefs().mode).toBe('system');
      expect(root()).toHaveAttribute('data-theme', 'slate-dark');
      dispose();
    });
  });

  it('reset forgets the choice and follows the webmail theme again', () => {
    stubColorScheme(false);
    const store = memoryStore({ theme: 'paper', accent: '', mode: 'light' });
    createRoot((dispose) => {
      const c = createAdminAppearance({ store, webmail: () => null });
      expect(root()).toHaveAttribute('data-theme', 'paper');
      c.reset();
      expect(store.saved.at(-1)).toBeNull();
      expect(c.chosen()).toBe(false);
      expect(root()).toHaveAttribute('data-theme', 'dark');
      dispose();
    });
  });
});

describe('AdminAppearance control', () => {
  function setup(store = memoryStore()) {
    stubColorScheme(false);
    let controller!: ReturnType<typeof createAdminAppearance>;
    const utils = render(() => {
      controller = createAdminAppearance({ store, webmail: () => null });
      return <AdminAppearance controller={controller} />;
    });
    return { ...utils, store, controller: () => controller };
  }

  it('a header button opens a modal dialog with mode, every theme and every accent', async () => {
    setup();
    const button = screen.getByRole('button', { name: 'Appearance' });
    expect(button).toHaveAttribute('aria-haspopup', 'dialog');
    fireEvent.click(button);
    const dlg = await screen.findByRole('dialog', { name: 'Console appearance' });
    expect(dlg).toHaveAttribute('aria-modal', 'true');
    const mode = within(dlg).getByRole('group', { name: 'Mode' });
    expect(within(mode).getAllByRole('radio').map((r) => (r as HTMLInputElement).value)).toEqual(['light', 'dark', 'system']);
    expect(within(mode).getByRole('radio', { name: 'Dark' })).toBeChecked();
    const themes = dlg.querySelectorAll('[data-theme-option]');
    expect([...themes].map((t) => t.getAttribute('data-theme-option')).sort()).toEqual(THEME_LIST.map((t) => t.id).sort());
    expect(themes).toHaveLength(18);
    const accents = dlg.querySelectorAll('[data-accent-option]');
    expect([...accents].map((a) => a.getAttribute('data-accent-option'))).toEqual([...ACCENT_IDS]);
    expect(within(dlg).getByRole('button', { name: 'Theme default' })).toHaveAttribute('aria-pressed', 'true');
  });

  it('picking a theme, an accent and a mode applies each at once and marks the pick with a check', async () => {
    const { store } = setup();
    fireEvent.click(screen.getByRole('button', { name: 'Appearance' }));
    const dlg = await screen.findByRole('dialog');
    const paper = dlg.querySelector<HTMLButtonElement>('[data-theme-option="paper"]')!;
    fireEvent.click(paper);
    expect(root()).toHaveAttribute('data-theme', 'paper');
    expect(paper).toHaveAttribute('aria-pressed', 'true');
    expect(paper.querySelector('svg[data-icon="check"]')).not.toBeNull();
    expect(within(dlg).getByRole('radio', { name: 'Light' })).toBeChecked();
    fireEvent.click(within(dlg).getByRole('button', { name: 'Magenta' }));
    expect(root()).toHaveAttribute('data-accent', 'magenta');
    fireEvent.click(within(dlg).getByRole('radio', { name: 'Dark' }));
    expect(root()).toHaveAttribute('data-theme', 'blueprint');
    expect(store.saved.at(-1)).toEqual({ theme: 'paper', accent: 'magenta', mode: 'dark' });
  });

  it('Escape and Close return focus to the button; Use the webmail theme forgets the choice', async () => {
    const { store } = setup(memoryStore({ theme: 'terminal', accent: '', mode: 'dark' }));
    const button = screen.getByRole('button', { name: 'Appearance' });
    button.focus();
    fireEvent.click(button);
    const dlg = await screen.findByRole('dialog');
    fireEvent.keyDown(dlg, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(document.activeElement).toBe(button);
    fireEvent.click(button);
    fireEvent.click(await screen.findByRole('button', { name: 'Use the webmail theme' }));
    expect(store.saved.at(-1)).toBeNull();
    await waitFor(() => expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Close' })));
    fireEvent.click(screen.getByRole('button', { name: 'Close' }));
    await waitFor(() => expect(document.activeElement).toBe(button));
  });

  it('has no axe violations and no inline style attribute (the previews are classes)', async () => {
    setup();
    fireEvent.click(screen.getByRole('button', { name: 'Appearance' }));
    const dlg = await screen.findByRole('dialog');
    await expectNoAxeViolations(dlg);
    expectNoInlineStyle(dlg);
  });
});
