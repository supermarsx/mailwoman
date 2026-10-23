// Service-worker registration: in a browser the worker is registered under the
// deploy prefix; in a Tauri shell none is, and one an earlier build registered is
// removed (t31-e7: a worker's fetches leave the WebView, so on Windows the shell's
// origin is answered by whatever listens on loopback port 80).

import { afterEach, describe, expect, it, vi } from 'vitest';
import { registerServiceWorker } from './register.ts';

interface G {
  __TAURI_INTERNALS__?: unknown;
}

function fakeServiceWorker(): { register: ReturnType<typeof vi.fn>; unregister: ReturnType<typeof vi.fn> } {
  const unregister = vi.fn(async () => true);
  const register = vi.fn(async () => ({}));
  Object.defineProperty(navigator, 'serviceWorker', {
    configurable: true,
    value: { register, getRegistrations: async () => [{ unregister }, { unregister }] },
  });
  return { register, unregister };
}

afterEach(() => {
  delete (globalThis as G).__TAURI_INTERNALS__;
  Reflect.deleteProperty(navigator, 'serviceWorker');
});

describe('registerServiceWorker', () => {
  it('registers the worker in a browser', async () => {
    const sw = fakeServiceWorker();
    await registerServiceWorker();
    expect(sw.register).toHaveBeenCalledWith('/sw.js', { type: 'classic', scope: '/' });
    expect(sw.unregister).not.toHaveBeenCalled();
  });

  it('registers nothing in a Tauri shell and removes earlier registrations', async () => {
    (globalThis as G).__TAURI_INTERNALS__ = {};
    const sw = fakeServiceWorker();
    await registerServiceWorker();
    expect(sw.register).not.toHaveBeenCalled();
    expect(sw.unregister).toHaveBeenCalledTimes(2);
  });
});
