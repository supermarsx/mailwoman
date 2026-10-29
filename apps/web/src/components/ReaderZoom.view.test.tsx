import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createSignal } from 'solid-js';
import { render } from 'solid-js/web';
import type { SettingEntry, SettingsMap } from '../api/settings.ts';

const [map, setMap] = createSignal<SettingsMap>({});
const writes: Array<[string, unknown]> = [];
vi.mock('../api/settings.ts', async (original) => ({
  ...(await original<typeof import('../api/settings.ts')>()),
  settings: () => map(),
  loadSettings: vi.fn(async () => undefined),
  writeSetting: vi.fn(async (key: string, value: unknown): Promise<SettingEntry> => {
    writes.push([key, value]);
    const entry: SettingEntry = { value, source: 'user', locked: false };
    setMap({ ...map(), [key]: entry });
    return entry;
  }),
}));

const { ReaderZoom, zoomMenuActions, zoomRowAction } = await import('./ReaderZoom.tsx');
const { clearReaderZoom } = await import('./readerZoom.ts');

function mount(): { host: HTMLElement; dispose: () => void } {
  const host = document.createElement('div');
  document.body.append(host);
  const dispose = render(() => <ReaderZoom />, host);
  return { host, dispose: () => { dispose(); host.remove(); } };
}
const byId = (host: HTMLElement, id: string): HTMLButtonElement => host.querySelector<HTMLButtonElement>(`[data-testid="${id}"]`)!;

beforeEach(() => { setMap({}); writes.length = 0; clearReaderZoom(); });

describe('ReaderZoom', () => {
  it('is one labelled group of three named buttons with tooltips', () => {
    const { host, dispose } = mount();
    const group = host.querySelector('[role="group"]')!;
    expect(group.getAttribute('aria-label')).toBeTruthy();
    for (const id of ['reader-zoom-out', 'reader-zoom-level', 'reader-zoom-in']) {
      const button = byId(host, id);
      expect(button.tagName).toBe('BUTTON');
      expect(button.getAttribute('aria-label')).toBeTruthy();
      expect(button.getAttribute('title')).toBeTruthy();
      expect(button.hasAttribute('style')).toBe(false);
    }
    dispose();
  });
  it('zooms out, in and resets with the buttons, showing the level', () => {
    const { host, dispose } = mount();
    expect(byId(host, 'reader-zoom-level').textContent).toContain('100');
    byId(host, 'reader-zoom-in').click();
    byId(host, 'reader-zoom-in').click();
    expect(byId(host, 'reader-zoom-level').textContent).toContain('125');
    byId(host, 'reader-zoom-level').click();
    expect(byId(host, 'reader-zoom-level').textContent).toContain('100');
    byId(host, 'reader-zoom-out').click();
    expect(byId(host, 'reader-zoom-level').textContent).toContain('90');
    expect(writes.map(([, value]) => value)).toEqual([110, 125, 100, 90]);
    dispose();
  });
  it('disables the button at either end of the ladder and everything when locked', () => {
    setMap({ 'reader.zoom': { value: 200, source: 'user', locked: false } });
    const { host, dispose } = mount();
    expect(byId(host, 'reader-zoom-in').disabled).toBe(true);
    expect(byId(host, 'reader-zoom-out').disabled).toBe(false);
    setMap({ 'reader.zoom': { value: 50, source: 'global', locked: true } });
    expect(byId(host, 'reader-zoom-out').disabled).toBe(true);
    expect(byId(host, 'reader-zoom-level').disabled).toBe(true);
    dispose();
  });
  it('folds into More as zoom out, zoom in and reset zoom', () => {
    setMap({ 'reader.zoom': { value: 150, source: 'user', locked: false } });
    const items = zoomMenuActions();
    expect(items.map(item => item.id)).toEqual(['zoom:out', 'zoom:in', 'zoom:reset']);
    expect(items[0]!.separatorBefore).toBe(true);
    expect(items.every(item => item.disabled === false)).toBe(true);
    void items[2]!.run();
    expect(writes.at(-1)).toEqual(['reader.zoom', 100]);
    expect(zoomMenuActions()[2]!.disabled).toBe(true);
    const row = zoomRowAction();
    expect(row.id).toBe('zoom');
    expect(row.menu().length).toBe(3);
  });
});
