// The settings model C client (worker f01): wire parsing, the shared map, and
// how refusals reach the UI (codes, never server English).

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import {activateAccountPreferences,suspendAccountPreferences} from './accountPreferenceLifecycle.ts';
import {
  clearSettings,
  deleteSetting,
  fetchSettings,
  loadSettings,
  parseEntry,
  putSetting,
  resetSetting,
  settings,
  settingsLoadStatus,
  SettingsApiError,
  writeSetting,
  type Fetcher,
} from './settings.ts';

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

const LOCKED = { value: 'dark', source: 'global', locked: true, lockedBy: 'global' };
const FREE = { value: 'cozy', source: 'default', locked: false };

describe('api/settings', () => {
  beforeEach(() => clearSettings());
  afterEach(() => suspendAccountPreferences());

  it('binds settings saves and resets to the verified storage authority', async () => {
    activateAccountPreferences('account','msa1_settings_authority_one');
    const fetcher=vi.fn<Fetcher>(async()=>json(200,FREE));
    await putSetting('density','compact',fetcher);
    await deleteSetting('density',fetcher);
    for(const [,init] of fetcher.mock.calls) expect(new Headers(init?.headers).get('x-mailwoman-storage-authority')).toBe('msa1_settings_authority_one');
  });

  it('refuses queued writes after same-ID authority replacement even before map cleanup', async () => {
    activateAccountPreferences('account','msa1_settings_authority_one');
    let finish!:(response:Response)=>void;
    const fetcher=vi.fn<Fetcher>(async (_url,init)=>init?.method
      ? new Promise<Response>(resolve=>{finish=resolve;}) : json(200,{settings:{}}));
    await loadSettings({fetcher});
    const first=writeSetting('density','compact');
    const queued=resetSetting('density');
    const rejected=expect(queued).rejects.toMatchObject({code:'account_changed'});
    activateAccountPreferences('account','msa1_settings_authority_two');
    finish(json(200,FREE));await first;await rejected;
    expect(fetcher).toHaveBeenCalledTimes(2);
    expect(settings()).toEqual({});
  });

  it('serializes rapid same-key edits and does not flash intermediate acknowledgements', async () => {
    const replies: Array<(response: Response) => void> = [];
    const methods: string[] = [];
    await loadSettings({fetcher: async (_url, init) => {
      if (!init?.method) return json(200, {settings: {density: FREE}});
      methods.push(init.method);
      return new Promise<Response>(resolve => replies.push(resolve));
    }});
    const first = writeSetting('density', 'compact');
    const second = resetSetting('density');
    expect(methods).toEqual(['PUT']);
    replies.shift()!(json(200, {...FREE, value:'compact'}));
    await first;
    await vi.waitFor(() => expect(methods).toEqual(['PUT', 'DELETE']));
    expect(settings().density?.value).toBe('cozy');
    replies.shift()!(json(200, {...FREE, value:'relaxed'}));
    await second;
    expect(settings().density?.value).toBe('relaxed');
  });

  it('does not replace a saved choice with an older background load', async () => {
    let finish!: (response: Response) => void;
    let loads = 0;
    await loadSettings({fetcher: async (_url, init) => {
      if (init?.method) return json(200, {...FREE, value:'compact'});
      if (++loads === 1) return json(200, {settings:{density:FREE}});
      return new Promise<Response>(resolve => {finish=resolve;});
    }});
    const refresh=loadSettings({refresh:true});
    expect(settingsLoadStatus()).toBe('ready');
    await writeSetting('density','compact');
    finish(json(200,{settings:{density:FREE}}));await refresh;
    expect(settings().density?.value).toBe('compact');
  });

  it('retains loaded values on a failed refresh but clears them on sign-out', async () => {
    await loadSettings({fetcher: async()=>json(200,{settings:{density:FREE}})});
    await loadSettings({refresh:true,fetcher:async()=>json(503,{})});
    expect(settings().density).toEqual(FREE);
    expect(settingsLoadStatus()).toBe('error');
    await loadSettings({refresh:true,fetcher:async()=>json(401,{})});
    expect(settings()).toEqual({});
  });

  it('never sends a queued edit after the account changes', async () => {
    let finish!: (response: Response)=>void;
    const fetcher=vi.fn<Fetcher>(async (_url,init)=>init?.method
      ? new Promise<Response>(resolve=>{finish=resolve;}) : json(200,{settings:{}}));
    await loadSettings({fetcher});
    const first=writeSetting('density','compact');
    const queued=writeSetting('density','relaxed');
    const rejected=expect(queued).rejects.toMatchObject({code:'account_changed'});
    clearSettings();finish(json(200,FREE));await first;await rejected;
    expect(fetcher).toHaveBeenCalledTimes(2);
    expect(settings()).toEqual({});
  });

  it('ignores an old account load after switching accounts', async () => {
    let resolve!: (response: Response) => void;
    const old = loadSettings({ fetcher: () => new Promise<Response>(r => { resolve = r; }) });
    clearSettings();
    await loadSettings({ fetcher: async () => json(200, { settings: { 'reader.colorMode': { value: 'original', source: 'user', locked: false } } }) });
    resolve(json(200, { settings: { 'reader.colorMode': { value: 'invert', source: 'user', locked: false } } }));
    await old;
    expect(settings()['reader.colorMode']?.value).toBe('original');
  });

  it('does not apply an old account write acknowledgement to the next account', async () => {
    let resolve!: (response: Response) => void;
    await loadSettings({ fetcher: async (_url, init) => init?.method === 'PUT' ? new Promise<Response>(r => { resolve = r; }) : json(200, { settings: {} }) });
    const old = writeSetting('reader.colorMode', 'invert');
    clearSettings();
    await loadSettings({ fetcher: async () => json(200, { settings: {} }) });
    resolve(json(200, { value: 'invert', source: 'user', locked: false }));
    await old;
    expect(settings()['reader.colorMode']).toBeUndefined();
  });

  it('parses entries and drops malformed ones', async () => {
    const fetcher = vi.fn<Fetcher>(async () =>
      json(200, {
        settings: {
          'appearance.theme': LOCKED,
          'appearance.density': FREE,
          broken: { value: 1, source: 'nowhere', locked: false },
          'not-an-object': 5,
        },
      }),
    );
    const map = await fetchSettings(fetcher);
    expect(fetcher).toHaveBeenCalledWith('/api/settings');
    expect(Object.keys(map).sort()).toEqual(['appearance.density', 'appearance.theme']);
    expect(map['appearance.theme']).toEqual(LOCKED);
    expect(parseEntry({ value: null, source: 'user', locked: false, userValue: 'x' })).toEqual({
      value: null,
      source: 'user',
      locked: false,
      userValue: 'x',
    });
    expect(parseEntry({ source: 'user', locked: false })).toBeNull();
  });

  it('PUT sends {value} to the encoded key and returns the new entry', async () => {
    const fetcher = vi.fn<Fetcher>(async () => json(200, { value: 'de', source: 'user', locked: false }));
    const entry = await putSetting('ui.language', 'de', fetcher);
    expect(entry.value).toBe('de');
    const [url, init] = fetcher.mock.calls[0]!;
    expect(url).toBe('/api/settings/ui.language');
    expect(init?.method).toBe('PUT');
    expect(JSON.parse(String(init?.body))).toEqual({ value: 'de' });
    const del = vi.fn<Fetcher>(async () => json(200, FREE));
    await deleteSetting('a/b', del);
    expect(del.mock.calls[0]![0]).toBe('/api/settings/a%2Fb');
  });

  it('a refusal carries the code and field, not the server text', async () => {
    const locked = vi.fn<Fetcher>(async () =>
      json(403, { code: 'setting_locked', error: 'appearance.theme is set by your administrator' }),
    );
    const err = await putSetting('appearance.theme', 'light', locked).catch((e: unknown) => e);
    expect(err).toBeInstanceOf(SettingsApiError);
    expect((err as SettingsApiError).code).toBe('setting_locked');
    expect((err as SettingsApiError).message).not.toContain('administrator');

    const invalid = vi.fn<Fetcher>(async () => json(422, { code: 'invalidValue', field: '/value' }));
    const e2 = (await putSetting('offline.budgetMb', 1e9, invalid).catch((e: unknown) => e)) as SettingsApiError;
    expect([e2.status, e2.code, e2.field]).toEqual([422, 'invalidValue', '/value']);

    const plain = vi.fn<Fetcher>(async () => new Response('server error', { status: 500 }));
    const e3 = (await fetchSettings(plain).catch((e: unknown) => e)) as SettingsApiError;
    expect(e3.code).toBe('http500');
  });

  it('loads the shared map once, and a 401 is the signed-out state', async () => {
    const fetcher = vi.fn<Fetcher>(async () => json(200, { settings: { 'appearance.theme': LOCKED } }));
    await Promise.all([loadSettings({ fetcher }), loadSettings(), loadSettings()]);
    expect(fetcher).toHaveBeenCalledTimes(1);
    expect(settings()['appearance.theme']?.locked).toBe(true);
    expect(settingsLoadStatus()).toBe('ready');

    clearSettings();
    await loadSettings({ fetcher: vi.fn<Fetcher>(async () => json(401, { error: 'unauthenticated' })) });
    expect(settingsLoadStatus()).toBe('signed-out');
    expect(settings()).toEqual({});
  });

  it('writes update the shared map; a locked refusal reloads it', async () => {
    let theme: unknown = FREE;
    const fetcher = vi.fn<Fetcher>(async (_url, init) => {
      if (init?.method === 'PUT') {
        return theme === LOCKED
          ? json(403, { code: 'setting_locked' })
          : json(200, { value: 'light', source: 'user', locked: false });
      }
      if (init?.method === 'DELETE') return json(200, FREE);
      return json(200, { settings: { 'appearance.theme': theme } });
    });
    await loadSettings({ fetcher });
    await writeSetting('appearance.theme', 'light');
    expect(settings()['appearance.theme']?.value).toBe('light');
    await resetSetting('appearance.theme');
    expect(settings()['appearance.theme']?.source).toBe('default');

    theme = LOCKED; // the administrator locks it meanwhile
    await expect(writeSetting('appearance.theme', 'light')).rejects.toBeInstanceOf(SettingsApiError);
    await vi.waitFor(() => expect(settings()['appearance.theme']?.locked).toBe(true));
  });
});
