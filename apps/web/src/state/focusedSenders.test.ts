import {afterEach, beforeEach, describe, expect, it, vi} from 'vitest';
import {createRoot} from 'solid-js';
import {activateAccountPreferences, suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {createFocusedSenders, LEGACY_FOCUS_KEY, readLegacyOverrides, type FocusedSendersApi} from './focusedSenders.ts';
import {parseSenders, type FocusedSender} from '../api/focusedSenders.ts';

const AUTH = 'authority-0123456789abcdef';
function fakeApi(initial: FocusedSender[] = []): FocusedSendersApi & {rows: FocusedSender[]; imports: Array<Record<string, string>>} {
  const api = {
    rows: [...initial],
    imports: [] as Array<Record<string, string>>,
    load: vi.fn(async () => [...api.rows]),
    save: vi.fn(async (email: string, tab: 'focused' | 'other' | null) => {
      api.rows = api.rows.filter(row => row.email !== email);
      if (tab) api.rows.push({email, tab, updatedAt: 1});
    }),
    import: vi.fn(async (senders: Record<string, 'focused' | 'other'>) => {
      api.imports.push(senders);
      for (const [email, tab] of Object.entries(senders)) if (!api.rows.some(row => row.email === email)) api.rows.push({email, tab, updatedAt: 2});
      return [...api.rows].sort((a, b) => a.email.localeCompare(b.email));
    }),
  };
  return api;
}
const flush = () => new Promise(resolve => setTimeout(resolve, 0));

beforeEach(() => {localStorage.clear(); suspendAccountPreferences();});
afterEach(() => {suspendAccountPreferences(); localStorage.clear();});

describe('focused senders (t94)', () => {
  it('loads the server rows for the signed-in mailbox and clears them on a switch', async () => {
    const api = fakeApi([{email: 'news@shop.example', tab: 'other', updatedAt: 5}]);
    activateAccountPreferences('acct1', AUTH);
    await createRoot(async dispose => {
      const store = createFocusedSenders(api);
      await flush();
      expect(store.overrides()).toEqual({'news@shop.example': 'other'});
      api.rows = [];
      activateAccountPreferences('acct2', AUTH);
      expect(store.overrides()).toEqual({});
      await flush();
      expect(api.load).toHaveBeenCalledTimes(2);
      dispose();
    });
  });

  it('moves the old localStorage entries to the server once and removes them', async () => {
    localStorage.setItem(LEGACY_FOCUS_KEY, JSON.stringify({'Old@Legacy.example': 'focused', 'news@shop.example': 'focused', bogus: 'other', 'x@y.example': 'spam'}));
    const api = fakeApi([{email: 'news@shop.example', tab: 'other', updatedAt: 5}]);
    activateAccountPreferences('acct1', AUTH);
    await createRoot(async dispose => {
      const store = createFocusedSenders(api);
      await flush();
      expect(api.imports).toEqual([{'old@legacy.example': 'focused', 'news@shop.example': 'focused'}]);
      // The server's own row wins.
      expect(store.overrides()).toEqual({'news@shop.example': 'other', 'old@legacy.example': 'focused'});
      expect(localStorage.getItem(LEGACY_FOCUS_KEY)).toBeNull();
      await store.reload();
      expect(api.import).toHaveBeenCalledTimes(1);
      dispose();
    });
  });

  it('keeps the local copy when the import fails', async () => {
    localStorage.setItem(LEGACY_FOCUS_KEY, JSON.stringify({'a@x.example': 'other'}));
    const api = fakeApi();
    api.import = vi.fn(async () => {throw new Error('503');});
    activateAccountPreferences('acct1', AUTH);
    await createRoot(async dispose => {
      createFocusedSenders(api);
      await flush();
      expect(localStorage.getItem(LEGACY_FOCUS_KEY)).not.toBeNull();
      dispose();
    });
  });

  it('applies a change at once and reverts it when the server refuses', async () => {
    const api = fakeApi();
    activateAccountPreferences('acct1', AUTH);
    await createRoot(async dispose => {
      const store = createFocusedSenders(api);
      await flush();
      const pending = store.set('A@X.example', 'other');
      expect(store.overrides()).toEqual({'a@x.example': 'other'});
      await pending;
      expect(api.save).toHaveBeenCalledWith('a@x.example', 'other');
      api.save = vi.fn(async () => {throw new Error('409');});
      await expect(store.set('a@x.example', 'focused')).rejects.toThrow('409');
      expect(store.overrides()).toEqual({'a@x.example': 'other'});
      await expect(store.set('b@x.example', 'focused')).rejects.toThrow();
      expect(store.overrides()).toEqual({'a@x.example': 'other'});
      dispose();
    });
  });

  it('reads only valid legacy entries and rejects a malformed server list', () => {
    localStorage.setItem(LEGACY_FOCUS_KEY, '{not json');
    expect(readLegacyOverrides()).toEqual({});
    localStorage.setItem(LEGACY_FOCUS_KEY, JSON.stringify(['a@x.example']));
    expect(readLegacyOverrides()).toEqual({});
    expect(() => parseSenders([{email: 'a@x.example', tab: 'spam', updatedAt: 1}])).toThrow();
    expect(() => parseSenders({})).toThrow();
    expect(parseSenders([{email: 'a@x.example', tab: 'other', updatedAt: 1}])).toEqual([{email: 'a@x.example', tab: 'other', updatedAt: 1}]);
  });
});
