// Per-user appearance sync (t19 e13, SPEC §17.3).
//
// Driven through the REAL theme slice rather than a stand-in seam: the thing
// worth proving is that a server payload survives the slice's re-validation and
// lands on `:root`, which a fake `appearancePrefs()` would not exercise.
//
// The conflict rules in `api/prefs.ts` are the reason this file is long — each
// branch of the reconcile is a case where a user could lose a setting, so each
// one is pinned by name.

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import {
  AppearanceSyncError,
  clearSyncMark,
  createAppearancePrefsApi,
  createAppearanceSync,
  startAppearanceSync,
  stopAppearanceSync,
  type AppearancePrefsApi,
  type StoredAppearance,
} from './prefs.ts';
import { createThemeSlice, type ThemeSlice } from '../state/slices/theme.ts';
import type { SliceContext } from '../state/slices/context.ts';
import type { AppearancePrefs } from '../theme/appearance.ts';

const ctx = { client: {}, showToast: vi.fn() } as unknown as SliceContext;

function slice(): ThemeSlice {
  return createThemeSlice(ctx);
}

const DEPLOYMENT = { theme: 'grove-light', brandName: 'Mailwoman', accent: null };

/** A recording stand-in for the three HTTP calls. */
function fakeApi(initial: StoredAppearance | Error): {
  api: AppearancePrefsApi;
  saved: AppearancePrefs[];
  resets: number;
  failSave: (e: Error | null) => void;
} {
  const saved: AppearancePrefs[] = [];
  let saveError: Error | null = null;
  const state = { resets: 0 };
  const api: AppearancePrefsApi = {
    load: async () => {
      if (initial instanceof Error) throw initial;
      return initial;
    },
    save: async (prefs) => {
      if (saveError !== null) throw saveError;
      saved.push(prefs);
      return 5_000;
    },
    reset: async () => {
      state.resets += 1;
    },
  };
  return {
    api,
    saved,
    get resets() {
      return state.resets;
    },
    failSave: (e) => {
      saveError = e;
    },
  };
}

const stored = (appearance: unknown, updatedAt: number | null): StoredAppearance => ({
  appearance,
  updatedAt,
  deploymentDefault: DEPLOYMENT,
});

/** Seed the mark as if this device had already synced `prefs` at `updatedAt`. */
function seedMark(prefs: AppearancePrefs, updatedAt: number): void {
  localStorage.setItem('mw.theme.sync', JSON.stringify({ updatedAt, prefs: JSON.stringify(prefs) }));
}

describe('appearance prefs API', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('GETs the account endpoint and narrows the response', async () => {
    const fetcher = vi.fn(
      async () =>
        new Response(
          JSON.stringify({
            appearance: { mode: 'fixed', theme: 'ocean-dark' },
            updatedAt: 42,
            deploymentDefault: { theme: 'grove-light', brandName: 'Acme', accent: '#123456' },
          }),
          { status: 200, headers: { 'content-type': 'application/json' } },
        ),
    );
    const got = await createAppearancePrefsApi(fetcher).load();

    expect(fetcher).toHaveBeenCalledWith('/api/account/appearance');
    expect(got.appearance).toEqual({ mode: 'fixed', theme: 'ocean-dark' });
    expect(got.updatedAt).toBe(42);
    expect(got.deploymentDefault.brandName).toBe('Acme');
    expect(got.deploymentDefault.accent).toBe('#123456');
  });

  it('PUTs the whole preference object and DELETEs on reset', async () => {
    const fetcher = vi.fn(
      async () => new Response(JSON.stringify({ ok: true, updatedAt: 9 }), { status: 200 }),
    );
    const api = createAppearancePrefsApi(fetcher);
    const s = slice();

    expect(await api.save(s.appearancePrefs())).toBe(9);
    const [, init] = fetcher.mock.calls[0] as [string, RequestInit];
    expect(init.method).toBe('PUT');
    expect(JSON.parse(String(init.body))).toEqual({ appearance: s.appearancePrefs() });

    await api.reset();
    expect((fetcher.mock.calls[1]?.[1] as RequestInit).method).toBe('DELETE');
  });

  it('treats a non-object appearance as nothing stored', async () => {
    // A scalar or array could never round-trip into a preference set; reading it
    // as "absent" keeps a junk value from reaching the slice at all.
    for (const junk of ['nope', 7, [], null]) {
      const fetcher = async (): Promise<Response> =>
        new Response(JSON.stringify({ appearance: junk, deploymentDefault: DEPLOYMENT }), {
          status: 200,
        });
      expect((await createAppearancePrefsApi(fetcher).load()).appearance).toBeNull();
    }
  });

  it('raises AppearanceSyncError carrying the status', async () => {
    const fetcher = async (): Promise<Response> =>
      new Response(JSON.stringify({ error: 'nope' }), { status: 401 });
    await expect(createAppearancePrefsApi(fetcher).load()).rejects.toMatchObject({
      name: 'AppearanceSyncError',
      status: 401,
      message: 'nope',
    });
  });
});

describe('appearance sync — the reconcile at start', () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute('data-theme');
  });
  afterEach(() => {
    stopAppearanceSync();
    vi.useRealTimers();
  });

  it('seeds the server when the account has nothing stored', async () => {
    const s = slice();
    s.setTheme('plum-dark');
    const f = fakeApi(stored(null, null));

    const sync = createAppearanceSync(s, { api: f.api, debounceMs: 0 });
    await sync.start();

    expect(f.saved).toHaveLength(1);
    expect(f.saved[0]?.theme).toBe('plum-dark');
    expect(sync.status()).toBe('synced');
  });

  it('adopts the account value on a device that has never synced', async () => {
    const s = slice();
    expect(s.theme()).not.toBe('ocean-dark');
    const f = fakeApi(stored({ mode: 'fixed', theme: 'ocean-dark', density: 'compact' }, 100));

    await createAppearanceSync(s, { api: f.api, debounceMs: 0 }).start();

    expect(s.theme()).toBe('ocean-dark');
    expect(s.density()).toBe('compact');
    // …and it reaches the DOM, not just the signals.
    expect(document.documentElement.getAttribute('data-theme')).toBe('ocean-dark');
  });

  it('adopting does not echo straight back as a write', async () => {
    const s = slice();
    const f = fakeApi(stored({ mode: 'fixed', theme: 'slate-dark' }, 100));

    await createAppearanceSync(s, { api: f.api, debounceMs: 0 }).start();

    // The adopt goes through `setAppearancePrefs`, which fires `subscribePrefs`.
    // Without the mark comparison that would immediately PUT the value we were
    // just handed — a write amplification loop across every device.
    expect(f.saved).toHaveLength(0);
  });

  it('keeps an offline edit the server has not superseded', async () => {
    const s = slice();
    const synced = s.appearancePrefs();
    seedMark(synced, 100);
    // The user changed the theme while this device could not reach the server.
    s.setTheme('grove-dark');

    // The server still holds what it held at the mark.
    const f = fakeApi(stored({ ...synced }, 100));
    const sync = createAppearanceSync(s, { api: f.api, debounceMs: 0 });
    await sync.start();

    // The local edit is pushed, NOT overwritten by the stale server value.
    expect(s.theme()).toBe('grove-dark');
    expect(f.saved).toHaveLength(1);
    expect(f.saved[0]?.theme).toBe('grove-dark');
  });

  it('takes the other device value when the server moved after this mark', async () => {
    const s = slice();
    const synced = s.appearancePrefs();
    seedMark(synced, 100);
    s.setTheme('grove-dark');

    // Another device wrote AFTER our mark: last write wins, and it is not ours.
    const f = fakeApi(stored({ mode: 'fixed', theme: 'amoled' }, 250));
    await createAppearanceSync(s, { api: f.api, debounceMs: 0 }).start();

    expect(s.theme()).toBe('amoled');
    expect(f.saved).toHaveLength(0);
  });

  it('does not overwrite an edit made while the load was in flight', async () => {
    const s = slice();
    let release!: () => void;
    const gate = new Promise<void>((r) => {
      release = r;
    });
    const saved: AppearancePrefs[] = [];
    const api: AppearancePrefsApi = {
      load: async () => {
        await gate;
        return stored({ mode: 'fixed', theme: 'ocean-light' }, 100);
      },
      save: async (p) => {
        saved.push(p);
        return 1;
      },
      reset: async () => undefined,
    };

    const pending = createAppearanceSync(s, { api, debounceMs: 0 }).start();
    s.setTheme('plum-dark'); // the user picks a theme mid-request
    release();
    await pending;

    // The response lost the race it started, so the user's pick stands.
    expect(s.theme()).toBe('plum-dark');
    expect(saved.at(-1)?.theme).toBe('plum-dark');
  });
});

describe('appearance sync — pushing changes', () => {
  beforeEach(() => {
    localStorage.clear();
  });
  afterEach(() => {
    stopAppearanceSync();
    vi.useRealTimers();
  });

  it('coalesces a burst of gallery clicks into one write', async () => {
    vi.useFakeTimers();
    const s = slice();
    const f = fakeApi(stored(null, null));
    const sync = createAppearanceSync(s, { api: f.api, debounceMs: 600 });
    await sync.start();
    f.saved.length = 0;

    s.setTheme('slate-light');
    s.setTheme('ocean-light');
    s.setTheme('plum-light');
    expect(f.saved).toHaveLength(0);

    vi.advanceTimersByTime(600);
    await vi.runAllTimersAsync();

    expect(f.saved).toHaveLength(1);
    expect(f.saved[0]?.theme).toBe('plum-light');
  });

  it('flush() sends a pending change immediately', async () => {
    const s = slice();
    const f = fakeApi(stored(null, null));
    const sync = createAppearanceSync(s, { api: f.api, debounceMs: 10_000 });
    await sync.start();
    f.saved.length = 0;

    s.setDensity('relaxed');
    await sync.flush();

    expect(f.saved).toHaveLength(1);
    expect(f.saved[0]?.density).toBe('relaxed');
  });

  it('stop() does not drop a change the user already made', async () => {
    const s = slice();
    const f = fakeApi(stored(null, null));
    const sync = createAppearanceSync(s, { api: f.api, debounceMs: 10_000 });
    await sync.start();
    f.saved.length = 0;

    s.setAccent('#6d8a4e');
    sync.stop();
    await sync.flush();

    expect(f.saved.at(-1)?.accent).toBe('#6d8a4e');
  });

  it('a signed-out session is local-only, not an error, and changes nothing', async () => {
    const s = slice();
    s.setTheme('grove-dark');
    const f = fakeApi(new AppearanceSyncError(401, 'unauthorized'));

    const sync = createAppearanceSync(s, { api: f.api, debounceMs: 0 });
    await sync.start();

    expect(sync.status()).toBe('local-only');
    // The app stays fully usable on device-local preferences.
    expect(s.theme()).toBe('grove-dark');
    expect(document.documentElement.getAttribute('data-theme')).toBe('grove-dark');
  });

  it('a failed push leaves local prefs alone and retries on the next change', async () => {
    const s = slice();
    const f = fakeApi(stored(null, null));
    f.failSave(new Error('network down'));

    const sync = createAppearanceSync(s, { api: f.api, debounceMs: 0 });
    await sync.start();
    expect(sync.status()).toBe('error');
    expect(f.saved).toHaveLength(0);

    // The mark was NOT advanced past the failure, so the next change carries the
    // whole current set up rather than assuming the earlier one landed.
    f.failSave(null);
    s.setTheme('slate-dark');
    await sync.flush();

    expect(f.saved).toHaveLength(1);
    expect(f.saved[0]?.theme).toBe('slate-dark');
    expect(sync.status()).toBe('synced');
  });

  it('reset() clears the account value and the local mark, not the current look', async () => {
    const s = slice();
    s.setTheme('ocean-dark');
    const f = fakeApi(stored(null, null));
    const sync = createAppearanceSync(s, { api: f.api, debounceMs: 0 });
    await sync.start();

    await sync.reset();

    expect(f.resets).toBe(1);
    expect(localStorage.getItem('mw.theme.sync')).toBeNull();
    // "Forget it server-side" is not "change what I am looking at".
    expect(s.theme()).toBe('ocean-dark');
  });

  it('exposes the deployment default from the load', async () => {
    const s = slice();
    const sync = createAppearanceSync(s, { api: fakeApi(stored(null, null)).api, debounceMs: 0 });
    expect(sync.deploymentDefault()).toBeNull();
    await sync.start();
    expect(sync.deploymentDefault()?.theme).toBe('grove-light');
  });
});

describe('the app-wide singleton', () => {
  beforeEach(() => {
    localStorage.clear();
    stopAppearanceSync();
  });
  afterEach(() => stopAppearanceSync());

  it('starts once however many callers ask for it', async () => {
    const s = slice();
    let loads = 0;
    const api: AppearancePrefsApi = {
      load: async () => {
        loads += 1;
        return stored(null, null);
      },
      save: async () => 1,
      reset: async () => undefined,
    };

    const a = startAppearanceSync(s, { api, debounceMs: 0 });
    const b = startAppearanceSync(s, { api, debounceMs: 0 });
    await a.flush();

    // Settings mounting must not restart a sync the app already booted.
    expect(b).toBe(a);
    expect(loads).toBe(1);
  });

  it('clearSyncMark makes the next start reconcile as a fresh device', async () => {
    const s = slice();
    seedMark(s.appearancePrefs(), 100);
    clearSyncMark();
    expect(localStorage.getItem('mw.theme.sync')).toBeNull();

    const f = fakeApi(stored({ mode: 'fixed', theme: 'hc-dark' }, 50));
    await createAppearanceSync(s, { api: f.api, debounceMs: 0 }).start();

    // No mark ⇒ adopt, even though the server value predates the old mark.
    expect(s.theme()).toBe('hc-dark');
  });
});
