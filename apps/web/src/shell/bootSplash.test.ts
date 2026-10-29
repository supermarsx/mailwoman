import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import {
  BOOT_ADMIN_CARD_KEY,
  BOOT_ADMIN_SESSION_KEY,
  BOOT_GEOMETRY_KEY,
  BOOT_SESSION_KEY,
  bootSplashActive,
  rememberAdminBootSession,
  rememberAdminLoginCard,
  rememberBootGeometry,
  rememberBootSession,
  releaseBootSplash,
  resetBootSplashForTests,
} from './bootSplash.ts';

const root = document.documentElement;

function mountSplash(): HTMLElement {
  const el = document.createElement('div');
  el.id = 'mw-boot';
  document.body.append(el);
  root.setAttribute('data-mw-boot', 'mail');
  return el;
}

beforeEach(() => {
  resetBootSplashForTests();
  localStorage.clear();
});
afterEach(() => {
  vi.useRealTimers();
  document.getElementById('mw-boot')?.remove();
  root.removeAttribute('data-mw-boot');
  root.removeAttribute('data-mw-boot-leaving');
  root.removeAttribute('data-reduced-motion');
  root.removeAttribute('style');
  document.body.innerHTML = '';
});

describe('boot splash (t52)', () => {
  it('fades out over the mounted UI, then removes itself', () => {
    vi.useFakeTimers();
    const splash = mountSplash();
    expect(bootSplashActive()).toBe(true);
    releaseBootSplash();
    expect(bootSplashActive()).toBe(false);
    // Two frames first, so the UI underneath is painted before the fade starts.
    expect(root.hasAttribute('data-mw-boot-leaving')).toBe(false);
    vi.advanceTimersToNextFrame();
    vi.advanceTimersToNextFrame();
    expect(root.hasAttribute('data-mw-boot-leaving')).toBe(true);
    vi.advanceTimersByTime(200);
    expect(splash.isConnected).toBe(false);
    expect(root.hasAttribute('data-mw-boot')).toBe(false);
  });

  it('goes at once under reduced motion, and only once', () => {
    root.setAttribute('data-reduced-motion', '');
    const splash = mountSplash();
    releaseBootSplash();
    expect(splash.isConnected).toBe(false);
    const again = mountSplash();
    releaseBootSplash();
    expect(again.isConnected).toBe(true);
  });

  it('is a no-op when the boot script never ran', () => {
    expect(bootSplashActive()).toBe(false);
    expect(() => releaseBootSplash()).not.toThrow();
  });

  it('remembers the session, and forgets the shell geometry on sign-out', () => {
    rememberBootSession(true);
    localStorage.setItem(BOOT_GEOMETRY_KEY, '{}');
    expect(localStorage.getItem(BOOT_SESSION_KEY)).toBe('1');
    rememberBootSession(false);
    expect(localStorage.getItem(BOOT_SESSION_KEY)).toBeNull();
    expect(localStorage.getItem(BOOT_GEOMETRY_KEY)).toBeNull();
  });

  it('remembers the admin console session separately from the mail one (t72)', () => {
    rememberAdminBootSession(true);
    expect(localStorage.getItem(BOOT_ADMIN_SESSION_KEY)).toBe('1');
    expect(localStorage.getItem(BOOT_SESSION_KEY)).toBeNull();
    rememberAdminBootSession(false);
    expect(localStorage.getItem(BOOT_ADMIN_SESSION_KEY)).toBeNull();
  });

  it('records the sign-in card height to a tenth of a pixel, with or without a logo (t72)', () => {
    const card = document.createElement('form');
    document.body.append(card);
    card.getBoundingClientRect = () => ({ height: 357.54, width: 360 }) as DOMRect;
    rememberAdminLoginCard(card);
    expect(JSON.parse(localStorage.getItem(BOOT_ADMIN_CARD_KEY)!)).toEqual({ v: 1, wide: window.innerWidth >= 761, height: 357.5, logo: false });
    card.append(document.createElement('img'));
    rememberAdminLoginCard(card);
    expect(JSON.parse(localStorage.getItem(BOOT_ADMIN_CARD_KEY)!).logo).toBe(true);
  });

  it('does not record a card that is not laid out (t72)', () => {
    rememberAdminLoginCard(document.createElement('form'));
    rememberAdminLoginCard(null);
    expect(localStorage.getItem(BOOT_ADMIN_CARD_KEY)).toBeNull();
  });

  it('clears the admin placeholder attributes when it goes (t72)', () => {
    root.setAttribute('data-reduced-motion', '');
    const splash = mountSplash();
    root.setAttribute('data-mw-boot', 'admin');
    root.setAttribute('data-mw-boot-logo', '');
    root.setAttribute('data-mw-boot-nopasskey', '');
    root.style.setProperty('--mwb-admin-card', '300px');
    releaseBootSplash();
    expect(splash.isConnected).toBe(false);
    for (const a of ['data-mw-boot', 'data-mw-boot-logo', 'data-mw-boot-nopasskey']) expect(root.hasAttribute(a)).toBe(false);
    expect(root.style.getPropertyValue('--mwb-admin-card')).toBe('');
  });

  it('does not record geometry for a shell that is not laid out', () => {
    document.body.innerHTML = '<div class="shell"><div class="mail-workspace"><div class="workspace-body"><div class="mail-pane"></div></div></div></div>';
    rememberBootGeometry();
    expect(localStorage.getItem(BOOT_GEOMETRY_KEY)).toBeNull();
  });
});

describe('index.html boot splash', () => {
  const html = readFileSync(join(dirname(fileURLToPath(import.meta.url)), '..', '..', 'index.html'), 'utf8');
  it('is decorative and carries no inline style or script (style-src / script-src \'self\')', () => {
    const splash = html.slice(html.indexOf('<div id="mw-boot"'), html.indexOf('<div id="root">'));
    expect(splash).toContain('aria-hidden="true"');
    expect(html).not.toMatch(/\sstyle=/);
    expect(html).not.toMatch(/<script(?![^>]*\bsrc=)[^>]*>/);
    expect(html).not.toMatch(/<style\b/);
  });
  it('has the admin sign-in card and console frame, inert, with the controls out of the tab order (t72)', () => {
    const splash = html.slice(html.indexOf('<div id="mw-boot"'), html.indexOf('<div id="root">'));
    expect(splash).toMatch(/<div class="mwb-admin" inert>/);
    expect(splash).toMatch(/<div class="mwb-console" inert>/);
    const controls = splash.match(/<(input|button)\b[^>]*>/g) ?? [];
    expect(controls).toHaveLength(4);
    for (const control of controls) expect(control).toContain('tabindex="-1"');
    expect(splash).not.toMatch(/\s(name|for)="/);
  });
  it('stands before #root, so Solid never renders into or over it', () => {
    expect(html.indexOf('id="mw-boot"')).toBeLessThan(html.indexOf('id="root"'));
  });
});
