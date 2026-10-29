// t52: a reload must not paint the Dark defaults while the session resolves.
// `bindAppearanceToAccount` used to reset to the defaults on EVERY account
// observation, including the first `null` (session not resolved yet) and the
// signed-in account it then resolved to, so a Light user saw Dark until the
// server's copy arrived.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { bindAppearanceToAccount, stopAppearanceSync } from './prefs.ts';
import { activateAccountPreferences, suspendAccountPreferences } from './accountPreferenceLifecycle.ts';
import { defaultAppearance, mergeAppearancePrefs, type AppearancePrefs } from '../theme/appearance.ts';

const AUTHORITY = 'msa1_0123456789abcdef0123';

function seam(start: AppearancePrefs) {
  let current = start;
  return {
    appearancePrefs: () => current,
    setAppearancePrefs: vi.fn((next: Partial<AppearancePrefs>) => { current = mergeAppearancePrefs(current, next); }),
    subscribePrefs: () => () => undefined,
    current: () => current,
  };
}

const light: AppearancePrefs = { ...defaultAppearance(), theme: 'light', chosen: true };

describe('appearance account binding at boot (t52)', () => {
  let dispose: (() => void) | undefined;
  beforeEach(() => {
    localStorage.clear();
    suspendAccountPreferences();
    // The sync's first GET never answers: these tests are about what is shown before it does.
    vi.stubGlobal('fetch', vi.fn(() => new Promise(() => undefined)));
  });
  afterEach(() => {
    dispose?.();
    stopAppearanceSync();
    vi.unstubAllGlobals();
    localStorage.clear();
  });

  it('keeps this device\'s look while the session is still unresolved', () => {
    const prefs = seam(light);
    dispose = bindAppearanceToAccount(prefs);
    expect(prefs.setAppearancePrefs).not.toHaveBeenCalled();
    expect(prefs.current().theme).toBe('light');
  });

  it('starts a known account from what it last agreed with the server', () => {
    localStorage.setItem(`mw.theme.sync:v2:${encodeURIComponent(AUTHORITY)}`, JSON.stringify({ updatedAt: 1, prefs: JSON.stringify(light) }));
    const prefs = seam(defaultAppearance());
    dispose = bindAppearanceToAccount(prefs);
    activateAccountPreferences('user-1', AUTHORITY);
    expect(prefs.current().theme).toBe('light');
  });

  it('still gives an account this device never synced the defaults, not the previous look', () => {
    const prefs = seam(light);
    dispose = bindAppearanceToAccount(prefs);
    activateAccountPreferences('user-2', AUTHORITY);
    expect(prefs.current().theme).toBe('dark');
  });

  it('resets on sign-out once an account was seen', () => {
    localStorage.setItem(`mw.theme.sync:v2:${encodeURIComponent(AUTHORITY)}`, JSON.stringify({ updatedAt: 1, prefs: JSON.stringify(light) }));
    const prefs = seam(defaultAppearance());
    dispose = bindAppearanceToAccount(prefs);
    activateAccountPreferences('user-1', AUTHORITY);
    suspendAccountPreferences();
    expect(prefs.current().theme).toBe('dark');
  });
});
