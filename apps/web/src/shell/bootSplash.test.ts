import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import {
  BOOT_GEOMETRY_KEY,
  BOOT_SESSION_KEY,
  bootSplashActive,
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
  it('stands before #root, so Solid never renders into or over it', () => {
    expect(html.indexOf('id="mw-boot"')).toBeLessThan(html.indexOf('id="root"'));
  });
});
