// t79: a zoom step changes one custom property on the body frame's box; the
// frame element (and so the message document inside it) is never replaced.
import { afterEach, describe, expect, it, vi } from 'vitest';
import { createSignal } from 'solid-js';
import { render } from 'solid-js/web';
import type { SettingEntry, SettingsMap } from '../api/settings.ts';

const [map, setMap] = createSignal<SettingsMap>({});
vi.mock('../api/settings.ts', async (original) => {
  const actual = await original<typeof import('../api/settings.ts')>();
  return {
    ...actual,
    settings: () => map(),
    loadSettings: vi.fn(async () => undefined),
    writeSetting: vi.fn(async (key: string, value: unknown): Promise<SettingEntry> => {
      const entry: SettingEntry = { value, source: 'user', locked: false };
      setMap({ ...map(), [key]: entry });
      return entry;
    }),
  };
});

const { ReaderZoomBox } = await import('./ReaderZoomBox.tsx');
const zoom = await import('./readerZoom.ts');
const css = await import('./readerZoom.css.ts');

afterEach(() => { setMap({}); zoom.clearReaderZoom(); document.body.innerHTML = ''; });

describe('ReaderZoomBox', () => {
  it('scales through one custom property and keeps the same frame element', async () => {
    const host = document.createElement('div');
    document.body.append(host);
    const dispose = render(() => <ReaderZoomBox><iframe class="reader__frame" title="body" sandbox="" srcdoc="<p>Hi</p>" /></ReaderZoomBox>, host);
    const box = host.querySelector<HTMLElement>('[data-testid="reader-zoom-box"]')!;
    const frame = box.querySelector('iframe')!;
    const srcdoc = frame.getAttribute('srcdoc');
    expect(box.className).toBe(css.box);
    expect(box.style.getPropertyValue(css.zoomVar)).toBe('');
    for (const [step, scale] of [[zoom.zoomIn, '1.1'], [zoom.zoomIn, '1.25'], [zoom.zoomOut, '1.1']] as const) {
      step();
      expect(box.style.getPropertyValue(css.zoomVar)).toBe(scale);
      expect(box.querySelector('iframe')).toBe(frame);
    }
    zoom.setReaderZoom(200);
    expect(box.style.getPropertyValue(css.zoomVar)).toBe('2');
    expect(box.dataset['readerZoom']).toBe('200');
    zoom.resetZoom();
    expect(box.style.getPropertyValue(css.zoomVar)).toBe('');
    expect(box.querySelector('iframe')).toBe(frame);
    expect(frame.getAttribute('srcdoc')).toBe(srcdoc);
    for (let i = 0; i < 8; i++) await Promise.resolve();
    expect(box.querySelector('iframe')).toBe(frame);
    dispose();
  });
  it('follows a level stored by another window', () => {
    const host = document.createElement('div');
    document.body.append(host);
    const dispose = render(() => <ReaderZoomBox><span /></ReaderZoomBox>, host);
    setMap({ 'reader.zoom': { value: 150, source: 'user', locked: false } });
    expect(host.querySelector<HTMLElement>('[data-testid="reader-zoom-box"]')!.style.getPropertyValue(css.zoomVar)).toBe('1.5');
    dispose();
  });
});
