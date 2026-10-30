import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { entryName, installBundleRecovery, showReloadNotice, isChunkLoadError, recoverFromChunkError, RELOAD_WINDOW_MS } from './bundleRecovery.ts';
import { registerNavigationGuard } from '../shell/navigationGuard.ts';

const chunkError = new TypeError('Failed to fetch dynamically imported module: https://mail.example/assets/Dashboard-CdQvJhSt.js');

function deps(over: Partial<Parameters<typeof recoverFromChunkError>[1]> = {}) {
  let clock = 1_000_000;
  const reload = vi.fn();
  const notify = vi.fn();
  return {
    reload,
    notify,
    tick: (ms: number) => { clock += ms; },
    value: {
      running: () => 'index-OLD11111.js',
      server: async () => 'index-NEW22222.js',
      reload,
      notify,
      now: () => clock,
      wait: async () => {},
      ...over,
    },
  };
}

beforeEach(() => { sessionStorage.clear(); });
afterEach(() => { sessionStorage.clear(); });

describe('chunk-load failure recovery (t102)', () => {
  it('recognises the browsers\' module download failures and nothing else', () => {
    expect(isChunkLoadError(chunkError)).toBe(true);
    expect(isChunkLoadError(new TypeError('error loading dynamically imported module: /assets/x.js'))).toBe(true);
    expect(isChunkLoadError(new TypeError('Importing a module script failed.'))).toBe(true);
    expect(isChunkLoadError(new Error('Unable to preload CSS for /assets/a.css'))).toBe(true);
    expect(isChunkLoadError(new Error('boom'))).toBe(false);
    expect(isChunkLoadError(null)).toBe(false);
  });

  it('reads the entry script from the shell (root and console forms)', () => {
    expect(entryName('<script type="module" crossorigin src="./assets/index-AbC12345.js" integrity="x"></script>')).toBe('index-AbC12345.js');
    expect(entryName('<script type="module" src="/mail/assets/index-Zz9.js"></script>')).toBe('index-Zz9.js');
    expect(entryName('<script src="./assets/boot-1.js"></script>')).toBeNull();
  });

  it('shows the notice and reloads once when the server has a newer build', async () => {
    const d = deps();
    await expect(recoverFromChunkError(chunkError, d.value)).resolves.toBe(true);
    expect(d.notify).toHaveBeenCalledOnce();
    expect(d.reload).toHaveBeenCalledOnce();
  });

  it('loop guard: no second reload into the same build, nor within the window', async () => {
    const d = deps();
    await recoverFromChunkError(chunkError, d.value);
    // After the reload the page still fails (the new build's chunk is missing too).
    await expect(recoverFromChunkError(chunkError, d.value)).resolves.toBe(false);
    d.tick(RELOAD_WINDOW_MS + 1);
    await expect(recoverFromChunkError(chunkError, d.value)).resolves.toBe(false);
    // A newer build again, but inside the window of the last reload: still no.
    d.tick(-RELOAD_WINDOW_MS);
    await expect(recoverFromChunkError(chunkError, { ...d.value, server: async () => 'index-THIRD333.js' })).resolves.toBe(false);
    // Past the window, a different build: allowed once more.
    d.tick(RELOAD_WINDOW_MS + 1);
    await expect(recoverFromChunkError(chunkError, { ...d.value, server: async () => 'index-THIRD333.js' })).resolves.toBe(true);
    expect(d.reload).toHaveBeenCalledTimes(2);
  });

  it('does not reload for the same build, an unknown server build, or another error', async () => {
    const same = deps({ server: async () => 'index-OLD11111.js' });
    await expect(recoverFromChunkError(chunkError, same.value)).resolves.toBe(false);
    const offline = deps({ server: async () => null });
    await expect(recoverFromChunkError(chunkError, offline.value)).resolves.toBe(false);
    const other = deps();
    await expect(recoverFromChunkError(new Error('render failed'), other.value)).resolves.toBe(false);
    expect(same.reload).not.toHaveBeenCalled();
    expect(offline.reload).not.toHaveBeenCalled();
    expect(other.reload).not.toHaveBeenCalled();
  });

  it('never reloads over unsaved work', async () => {
    const remove = registerNavigationGuard(() => false, undefined);
    try {
      const d = deps();
      await expect(recoverFromChunkError(chunkError, d.value)).resolves.toBe(false);
      expect(d.reload).not.toHaveBeenCalled();
    } finally { remove(); }
  });

  it('does not reload when the loop guard cannot be stored', async () => {
    const spy = vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new Error('blocked'); });
    try {
      const d = deps();
      await expect(recoverFromChunkError(chunkError, d.value)).resolves.toBe(false);
      expect(d.reload).not.toHaveBeenCalled();
    } finally { spy.mockRestore(); }
  });

  it('shows the notice on its own when no toast region is mounted', () => {
    showReloadNotice('A new version is available, reloading…');
    const el = document.querySelector('[data-testid="bundle-reload-notice"]');
    expect(el?.getAttribute('role')).toBe('status');
    expect(el?.textContent).toBe('A new version is available, reloading…');
    el?.remove();
  });

  it('listens for Vite preload errors and unhandled import rejections', () => {
    const target = new EventTarget() as unknown as Window;
    const add = vi.spyOn(target, 'addEventListener');
    const off = installBundleRecovery(target);
    expect(add.mock.calls.map((c) => c[0])).toEqual(['vite:preloadError', 'unhandledrejection']);
    off();
  });
});
