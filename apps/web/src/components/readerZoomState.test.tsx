import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createSignal } from 'solid-js';
import { render } from 'solid-js/web';
import type { SettingEntry, SettingsMap } from '../api/settings.ts';
import { activateAccountPreferences, suspendAccountPreferences } from '../api/accountPreferenceLifecycle.ts';

const state = vi.hoisted(() => ({
  handlers: new Map<string, (e: KeyboardEvent) => boolean | void>(),
  writes: [] as Array<[string, unknown]>,
  fail: null as Error | null,
}));
const [map, setMap] = createSignal<SettingsMap>({});

vi.mock('../api/settings.ts', async (original) => {
  const actual = await original<typeof import('../api/settings.ts')>();
  return {
    ...actual,
    settings: () => map(),
    settingsLoadStatus: vi.fn(() => 'ready'),
    loadSettings: vi.fn(async () => undefined),
    writeSetting: vi.fn(async (key: string, value: unknown): Promise<SettingEntry> => {
      state.writes.push([key, value]);
      if (state.fail) throw state.fail;
      const entry: SettingEntry = { value, source: 'user', locked: false };
      setMap({ ...map(), [key]: entry });
      return entry;
    }),
  };
});
vi.mock('../keymap/index.ts', () => ({
  useActions: (handlers: Record<string, (e: KeyboardEvent) => boolean | void>) => {
    for (const [id, handler] of Object.entries(handlers)) state.handlers.set(id, handler);
  },
}));

const zoom = await import('./readerZoom.ts');
const { SettingsApiError } = await import('../api/settings.ts');
const css = await import('./readerZoom.css.ts');

const flush = async (): Promise<void> => { for (let i = 0; i < 4; i++) await Promise.resolve(); };

beforeEach(() => {
  suspendAccountPreferences();
  localStorage.clear();
  setMap({});
  state.writes = [];
  state.fail = null;
  state.handlers.clear();
  zoom.clearReaderZoom();
});

describe('durable zoom intent', () => {
  const authority = 'zoom_test_authority_alice';
  const slot = `mw.reader-zoom:v1:${authority}:pending`;
  const entry = (value: number, locked = false): SettingEntry => ({ value, source: 'user', locked });
  const read = () => JSON.parse(localStorage.getItem(slot) ?? 'null') as {token:string;next:number;base:number;inflight:number|null}|null;
  async function owner(): Promise<void> { activateAccountPreferences('alice', authority); await flush(); }
  it('retains the latest rapid click across pagehide while an older write is in flight, then replays after reload', async () => {
    const { writeSetting } = await import('../api/settings.ts');
    await owner();
    let finish!: (value: SettingEntry) => void;
    vi.mocked(writeSetting).mockImplementationOnce(() => new Promise(resolve => { finish = resolve; }));
    zoom.zoomIn(); zoom.zoomIn(); zoom.zoomIn();
    window.dispatchEvent(new Event('pagehide'));
    expect(read()).toMatchObject({ next: 150, base: 100, inflight: 110 });
    suspendAccountPreferences(); // Old page is gone; old transport must not touch new state.
    setMap({ 'reader.zoom': entry(110) });
    await owner(); await flush();
    expect(state.writes.at(-1)).toEqual(['reader.zoom', 150]);
    expect(zoom.readerZoom()).toBe(150);
    finish(entry(110)); await flush();
    expect(zoom.readerZoom()).toBe(150);
    expect(read()).toBeNull();
  });
  it('does not replay against a newer server choice or an administrator lock', async () => {
    for (const locked of [false, true]) {
      suspendAccountPreferences();
      localStorage.setItem(slot, JSON.stringify({token:'pending',next:150,base:100,inflight:110}));
      setMap({ 'reader.zoom': entry(locked ? 100 : 175, locked) });
      await owner();
      expect(state.writes).toHaveLength(0);
      expect(read()).toBeNull();
    }
  });
  it('leaves pending intent intact when the settings refresh failed', async () => {
    const { settingsLoadStatus } = await import('../api/settings.ts');
    vi.mocked(settingsLoadStatus).mockReturnValueOnce('error');
    localStorage.setItem(slot, JSON.stringify({token:'pending',next:150,base:100,inflight:110}));
    await owner();
    expect(state.writes).toHaveLength(0);
    expect(read()?.next).toBe(150);
  });
  it('an older acknowledgement never deletes another window\'s pending token', async () => {
    const { writeSetting } = await import('../api/settings.ts');
    await owner();
    let finish!: (value: SettingEntry) => void;
    vi.mocked(writeSetting).mockImplementationOnce(() => new Promise(resolve => { finish = resolve; }));
    zoom.zoomIn(); zoom.zoomIn();
    localStorage.setItem(slot, JSON.stringify({token:'other-window',next:175,base:100,inflight:110}));
    finish(entry(110)); await flush();
    window.dispatchEvent(new Event('pagehide')); await flush();
    expect(read()?.token).toBe('other-window');
    expect(state.writes).toHaveLength(0);
  });
  it('account transition discards a queued level and ignores an old completion while the new account writes', async () => {
    const { writeSetting } = await import('../api/settings.ts');
    await owner();
    let finishOld!: (value: SettingEntry) => void;
    let finishNew!: (value: SettingEntry) => void;
    vi.mocked(writeSetting).mockImplementationOnce(() => new Promise(resolve => { finishOld = resolve; }));
    zoom.zoomIn(); zoom.zoomIn();
    activateAccountPreferences('bob', 'zoom_test_authority_bob'); await flush();
    expect(zoom.readerZoom()).toBe(100);
    vi.mocked(writeSetting).mockImplementationOnce(() => new Promise(resolve => { finishNew = resolve; }));
    zoom.zoomOut(); zoom.zoomOut();
    finishOld(entry(110)); await flush();
    expect(state.writes).toHaveLength(0);
    expect(zoom.readerZoom()).toBe(75);
    finishNew(entry(90)); await flush();
    window.dispatchEvent(new Event('pagehide')); await flush();
    expect(state.writes).toEqual([['reader.zoom',75]]);
  });
  it('does not send an older queued level after another window acknowledged and cleared the journal', async () => {
    const { writeSetting } = await import('../api/settings.ts');
    await owner();
    let finish!: (value: SettingEntry) => void;
    vi.mocked(writeSetting).mockImplementationOnce(() => new Promise(resolve => { finish = resolve; }));
    zoom.zoomIn(); zoom.zoomIn();
    expect(read()?.next).toBe(125);
    localStorage.removeItem(slot);
    finish(entry(110)); await flush();
    window.dispatchEvent(new Event('pagehide')); await flush();
    expect(state.writes).toHaveLength(0);
    expect(read()).toBeNull();
  });
  it('does not refresh the shared settings map without a pending intent', async () => {
    const { loadSettings } = await import('../api/settings.ts');
    vi.mocked(loadSettings).mockClear();
    await owner();
    expect(loadSettings).not.toHaveBeenCalled();
  });
  it('rejects malformed pending levels instead of replaying unvalidated storage', async () => {
    localStorage.setItem(slot, JSON.stringify({token:'pending',next:151,base:100,inflight:null}));
    await owner();
    expect(state.writes).toHaveLength(0);
  });
});

describe('zoom steps', () => {
  it('uses the fixed ladder from 50% to 200%', () => {
    expect(zoom.ZOOM_STEPS).toEqual([50, 67, 75, 90, 100, 110, 125, 150, 175, 200]);
  });
  it('snaps any value to the nearest step and falls back to 100%', () => {
    expect(zoom.snapZoom(113)).toBe(110);
    expect(zoom.snapZoom(120)).toBe(125);
    expect(zoom.snapZoom('150')).toBe(150);
    expect(zoom.snapZoom(1000)).toBe(200);
    expect(zoom.snapZoom(0)).toBe(50);
    for (const bad of [undefined, null, 'x', '', Number.NaN, {}]) expect(zoom.snapZoom(bad)).toBe(100);
  });
  it('steps one rung at a time and clamps at both ends', () => {
    expect(zoom.stepZoom(100, 1)).toBe(110);
    expect(zoom.stepZoom(100, -1)).toBe(90);
    expect(zoom.stepZoom(67, -1)).toBe(50);
    expect(zoom.stepZoom(50, -1)).toBe(50);
    expect(zoom.stepZoom(200, 1)).toBe(200);
  });
});

describe('zoom state and persistence', () => {
  it('reads the server preference and defaults to 100%', () => {
    expect(zoom.readerZoom()).toBe(100);
    setMap({ 'reader.zoom': { value: 150, source: 'domain', locked: false } });
    expect(zoom.readerZoom()).toBe(150);
  });
  it('applies a change at once and stores it as the user preference', async () => {
    zoom.zoomIn();
    expect(zoom.readerZoom()).toBe(110);
    expect(state.writes).toEqual([['reader.zoom', 110]]);
    await flush();
    expect(map()['reader.zoom']?.value).toBe(110);
    zoom.zoomIn(); zoom.zoomIn();
    expect(zoom.readerZoom()).toBe(150);
    zoom.resetZoom();
    expect(zoom.readerZoom()).toBe(100);
    await new Promise(resolve => setTimeout(resolve, 460)); await flush();
    expect(state.writes.at(-1)).toEqual(['reader.zoom', 100]);
  });
  it('collapses steps taken while a write is in flight into one write of the last level (t79)', async () => {
    for (let i = 0; i < 3; i++) zoom.zoomIn();
    expect(zoom.readerZoom()).toBe(150);
    expect(state.writes).toEqual([['reader.zoom', 110]]);
    await flush();
    // The next write waits out the spacing after the first one started.
    expect(state.writes).toEqual([['reader.zoom', 110]]);
    await new Promise(resolve => setTimeout(resolve, 460)); await flush();
    expect(state.writes).toEqual([['reader.zoom', 110], ['reader.zoom', 150]]);
    expect(zoom.readerZoom()).toBe(150);
    expect(map()['reader.zoom']?.value).toBe(150);
  });
  it('writes a step on its own at once, outside a burst (t79)', async () => {
    zoom.zoomIn();
    await flush();
    await new Promise(resolve => setTimeout(resolve, 200));
    zoom.zoomIn();
    await flush();
    expect(state.writes).toEqual([['reader.zoom', 110], ['reader.zoom', 125]]);
  });
  it('sends a step still waiting when the page goes away (t79)', async () => {
    zoom.zoomIn(); zoom.zoomIn();
    await flush();
    expect(state.writes).toEqual([['reader.zoom', 110]]);
    window.dispatchEvent(new Event('pagehide'));
    await flush();
    expect(state.writes).toEqual([['reader.zoom', 110], ['reader.zoom', 125]]);
  });
  it('writes nothing more when the steps come back to the stored level (t79)', async () => {
    zoom.zoomIn(); zoom.zoomIn(); zoom.zoomOut();
    expect(zoom.readerZoom()).toBe(110);
    await new Promise(resolve => setTimeout(resolve, 460)); await flush();
    expect(state.writes).toEqual([['reader.zoom', 110]]);
    expect(zoom.readerZoom()).toBe(110);
  });
  it('writes nothing when the level would not change', () => {
    setMap({ 'reader.zoom': { value: 200, source: 'user', locked: false } });
    zoom.zoomIn();
    zoom.setReaderZoom(200);
    expect(state.writes).toEqual([]);
  });
  it('does not change a level the administrator locked', () => {
    setMap({ 'reader.zoom': { value: 125, source: 'global', locked: true } });
    zoom.zoomOut();
    expect(zoom.readerZoom()).toBe(125);
    expect(state.writes).toEqual([]);
  });
  it('keeps the chosen level for the session when it cannot be stored', async () => {
    state.fail = new SettingsApiError(503, 'http503', null);
    zoom.zoomOut();
    await flush();
    expect(zoom.readerZoom()).toBe(90);
  });
  it('drops the chosen level when a lock arrived since the map loaded', async () => {
    setMap({ 'reader.zoom': { value: 100, source: 'default', locked: false } });
    state.fail = new SettingsApiError(403, 'setting_locked', null);
    zoom.zoomIn();
    await flush();
    expect(zoom.readerZoom()).toBe(100);
  });
});

describe('zoom shortcuts and wheel are scoped to the reader', () => {
  function mount(active = () => true): { root: HTMLElement; outside: HTMLButtonElement; dispose: () => void } {
    const host = document.createElement('div');
    const outside = document.createElement('button');
    document.body.append(host, outside);
    let root!: HTMLElement;
    const dispose = render(() => {
      const el = <section class="reader"><button type="button" class="inside">Reply</button><iframe class="reader__frame" title="Message" /></section> as HTMLElement;
      root = el;
      zoom.useReaderZoom({ root: () => root, active });
      return el;
    }, host);
    const frame = root.querySelector('iframe')!;
    frame.getBoundingClientRect = () => ({ left: 0, top: 100, right: 500, bottom: 600, width: 500, height: 500, x: 0, y: 100, toJSON: () => ({}) });
    return { root, outside, dispose: () => { dispose(); host.remove(); outside.remove(); } };
  }
  const press = (target: Element, key: string): KeyboardEvent => {
    const e = new KeyboardEvent('keydown', { key, ctrlKey: true, bubbles: true, cancelable: true });
    target.dispatchEvent(e);
    return e;
  };
  let view: ReturnType<typeof mount> | undefined;
  afterEach(() => { view?.dispose(); view = undefined; });

  it('registers zoomIn, zoomOut and zoomReset in the shortcut dispatcher', () => {
    view = mount();
    expect([...state.handlers.keys()].sort()).toEqual(['zoomIn', 'zoomOut', 'zoomReset']);
  });
  it('handles the keys while focus is in the reader', () => {
    view = mount();
    const inside = view.root.querySelector<HTMLButtonElement>('.inside')!;
    inside.focus();
    expect(state.handlers.get('zoomIn')!(press(inside, '='))).toBe(true);
    expect(zoom.readerZoom()).toBe(110);
    expect(state.handlers.get('zoomOut')!(press(inside, '-'))).toBe(true);
    expect(state.handlers.get('zoomOut')!(press(inside, '-'))).toBe(true);
    expect(zoom.readerZoom()).toBe(90);
    expect(state.handlers.get('zoomReset')!(press(inside, '0'))).toBe(true);
    expect(zoom.readerZoom()).toBe(100);
  });
  it('declines outside the reader, so the browser keeps its page zoom', () => {
    view = mount();
    view.outside.focus();
    expect(state.handlers.get('zoomIn')!(press(view.outside, '='))).toBe(false);
    expect(state.handlers.get('zoomReset')!(press(view.outside, '0'))).toBe(false);
    expect(zoom.readerZoom()).toBe(100);
    expect(state.writes).toEqual([]);
  });
  it('declines in a background reader tab', () => {
    view = mount(() => false);
    const inside = view.root.querySelector<HTMLButtonElement>('.inside')!;
    inside.focus();
    expect(state.handlers.get('zoomIn')!(press(inside, '='))).toBe(false);
  });
  it('lets the wheel reach the reader over the body only while Ctrl/Cmd is held', () => {
    view = mount();
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Control', ctrlKey: true }));
    expect(view.root.classList.contains(css.gesture)).toBe(true);
    document.dispatchEvent(new KeyboardEvent('keyup', { key: 'Control' }));
    expect(view.root.classList.contains(css.gesture)).toBe(false);
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Meta', metaKey: true }));
    window.dispatchEvent(new Event('blur'));
    expect(view.root.classList.contains(css.gesture)).toBe(false);
  });
  it('zooms with Ctrl + wheel over the body and leaves other wheels alone', () => {
    view = mount();
    const over = (deltaY: number, ctrlKey = true, clientY = 300): WheelEvent => {
      const e = new WheelEvent('wheel', { deltaY, ctrlKey, clientX: 50, clientY, bubbles: true, cancelable: true });
      view!.root.dispatchEvent(e);
      return e;
    };
    expect(over(-50).defaultPrevented).toBe(true);
    expect(zoom.readerZoom()).toBe(110);
    over(50); over(50);
    expect(zoom.readerZoom()).toBe(90);
    // A small pinch step accumulates before it zooms.
    over(-20); expect(zoom.readerZoom()).toBe(90);
    over(-40); expect(zoom.readerZoom()).toBe(100);
    // Plain scrolling, and Ctrl + wheel over the header, are not ours.
    expect(over(-100, false).defaultPrevented).toBe(false);
    expect(over(-100, true, 50).defaultPrevented).toBe(false);
    expect(zoom.readerZoom()).toBe(100);
  });
  it('consumes coalesced wheel distance, retains partial notches and clamps huge events', () => {
    view = mount();
    const wheel = (deltaY:number):void => { view!.root.dispatchEvent(new WheelEvent('wheel',{deltaY,ctrlKey:true,clientX:50,clientY:300,bubbles:true,cancelable:true})); };
    wheel(-175); expect(zoom.readerZoom()).toBe(150); // Three full notches, -25 remaining.
    wheel(-25); expect(zoom.readerZoom()).toBe(175);
    wheel(-720); expect(zoom.readerZoom()).toBe(200);
    wheel(720); expect(zoom.readerZoom()).toBe(50);
    wheel(-1e12); expect(zoom.readerZoom()).toBe(200);
    wheel(1e12); expect(zoom.readerZoom()).toBe(50);
  });
});
