// t99: each account's last settings map is shown at once when that account is
// opened again in this page, then refreshed in place; never across accounts,
// never across a sign-in.
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { activateAccountPreferences, suspendAccountPreferences } from './accountPreferenceLifecycle.ts';
import { clearSettings, clearSettingsSnapshots, loadSettings, settings, settingsLoadStatus, writeSetting, type Fetcher } from './settings.ts';
import { setSwrCachesEnabled } from './swrCache.ts';
import { installLinkedMailbox } from './linkedMailboxSession.ts';

const json = (body: unknown) => new Response(JSON.stringify(body), { status: 200, headers: { 'content-type': 'application/json' } });
const entry = (value: unknown) => ({ value, source: 'user', locked: false });
const A = ['account-a', 'msa1_authority_aaaaaaaaaa'] as const;
const B = ['account-b', 'msa1_authority_bbbbbbbbbb'] as const;

function gate() {
  let open!: () => void;
  const opened = new Promise<void>((r) => { open = r; });
  return { opened, open };
}

describe('settings snapshots (t99)', () => {
  beforeEach(() => { setSwrCachesEnabled(true); clearSettings(); clearSettingsSnapshots(); });
  afterEach(() => { suspendAccountPreferences(); clearSettings(); clearSettingsSnapshots(); setSwrCachesEnabled(false); });

  it('shows the account\'s last map at once on return, then replaces it with the server answer', async () => {
    activateAccountPreferences(...A);
    await loadSettings({ fetcher: async () => json({ settings: { density: entry('compact') } }) });
    // Switch to B.
    clearSettings();
    activateAccountPreferences(...B);
    await loadSettings({ fetcher: async () => json({ settings: { density: entry('cozy') } }) });
    expect(settings()['density']?.value).toBe('cozy');
    // Back to A: A's map is there before the request answers.
    clearSettings();
    activateAccountPreferences(...A);
    const slow = gate();
    const done = loadSettings({ fetcher: async () => { await slow.opened; return json({ settings: { density: entry('spacious') } }); } });
    expect(settingsLoadStatus()).toBe('ready');
    expect(settings()['density']?.value).toBe('compact');
    slow.open();
    await done;
    expect(settings()['density']?.value).toBe('spacious');
  });

  it('never shows one account\'s map for another', async () => {
    activateAccountPreferences(...A);
    await loadSettings({ fetcher: async () => json({ settings: { density: entry('compact') } }) });
    clearSettings();
    activateAccountPreferences(...B);
    const slow = gate();
    const done = loadSettings({ fetcher: async () => { await slow.opened; return json({ settings: {} }); } });
    expect(settingsLoadStatus()).toBe('loading');
    expect(settings()).toEqual({});
    slow.open();
    await done;
  });

  it('a write updates the kept map', async () => {
    activateAccountPreferences(...A);
    const fetcher = vi.fn<Fetcher>(async (_url, init) => init?.method === 'PUT' ? json(entry('compact')) : json({ settings: { density: entry('cozy') } }));
    await loadSettings({ fetcher });
    await writeSetting('density', 'compact');
    clearSettings();
    activateAccountPreferences(...B);
    clearSettings();
    activateAccountPreferences(...A);
    const slow = gate();
    const done = loadSettings({ fetcher: async () => { await slow.opened; return json({ settings: { density: entry('compact') } }); } });
    expect(settings()['density']?.value).toBe('compact');
    slow.open();
    await done;
  });

  it('a sign-in or sign-out drops every kept map', async () => {
    activateAccountPreferences(...A);
    await loadSettings({ fetcher: async () => json({ settings: { density: entry('compact') } }) });
    installLinkedMailbox(null); // what sign-in and sign-out do
    clearSettings();
    activateAccountPreferences(...A);
    const slow = gate();
    const done = loadSettings({ fetcher: async () => { await slow.opened; return json({ settings: {} }); } });
    expect(settingsLoadStatus()).toBe('loading');
    slow.open();
    await done;
  });
});
