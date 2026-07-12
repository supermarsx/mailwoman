import { describe, it, expect, beforeEach, vi } from 'vitest';
import { createThemeSlice } from './theme.ts';
import type { SliceContext } from './context.ts';

// The slice ignores its context; a minimal stub keeps the factory happy.
const ctx = { client: {}, showToast: vi.fn() } as unknown as SliceContext;

function root(): HTMLElement {
  return document.documentElement;
}

describe('theme slice', () => {
  beforeEach(() => {
    localStorage.clear();
    root().removeAttribute('data-theme');
    root().removeAttribute('data-density');
    root().style.removeProperty('--mw-accent');
    root().style.removeProperty('--mw-ui-font');
  });

  it('applies defaults to :root on creation', () => {
    const s = createThemeSlice(ctx);
    // jsdom has no matchMedia → default theme is light, density cozy.
    expect(s.density()).toBe('cozy');
    expect(root().getAttribute('data-theme')).toBe(s.theme());
    expect(root().getAttribute('data-density')).toBe('cozy');
  });

  it('setTheme reflects onto data-theme and persists', () => {
    const s = createThemeSlice(ctx);
    s.setTheme('grove-dark');
    expect(s.theme()).toBe('grove-dark');
    expect(root().getAttribute('data-theme')).toBe('grove-dark');
    const stored = JSON.parse(localStorage.getItem('mw.theme.prefs') ?? '{}');
    expect(stored.theme).toBe('grove-dark');
  });

  it('setAccent sets an inline --mw-accent override and clears it when empty', () => {
    const s = createThemeSlice(ctx);
    s.setAccent('#6d8a4e');
    expect(root().style.getPropertyValue('--mw-accent')).toBe('#6d8a4e');
    s.setAccent('');
    expect(root().style.getPropertyValue('--mw-accent')).toBe('');
  });

  it('setUiFont sets/removes the --mw-ui-font override', () => {
    const s = createThemeSlice(ctx);
    s.setUiFont('mono');
    expect(root().style.getPropertyValue('--mw-ui-font')).toContain('JetBrains Mono');
    s.setUiFont('default');
    expect(root().style.getPropertyValue('--mw-ui-font')).toBe('');
  });

  it('setDensity, setLayout and ribbon collapse round-trip', () => {
    const s = createThemeSlice(ctx);
    s.setDensity('compact');
    s.setLayout('ribbon');
    s.setRibbonCollapsed(true);
    expect(root().getAttribute('data-density')).toBe('compact');
    expect(s.layout()).toBe('ribbon');
    expect(s.ribbonCollapsed()).toBe(true);
  });

  it('restores persisted prefs from localStorage', () => {
    createThemeSlice(ctx).setTheme('amoled');
    // A fresh slice reads the persisted value back.
    const s2 = createThemeSlice(ctx);
    expect(s2.theme()).toBe('amoled');
    expect(root().getAttribute('data-theme')).toBe('amoled');
  });

  it('ignores a corrupt localStorage payload and falls back to defaults', () => {
    localStorage.setItem('mw.theme.prefs', '{ not json');
    const s = createThemeSlice(ctx);
    expect(s.density()).toBe('cozy');
  });
});
