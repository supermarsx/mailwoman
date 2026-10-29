// useBarMode (t38 §2.1): `pinned` until the settings map knows the key, the
// stored value after, an optimistic change that the server's refusal reverts
// with a toast, and no request at all for a locked key.

import { beforeEach, describe, expect, it } from 'vitest';
import { render, waitFor } from '@solidjs/testing-library';
import { clearSettings, loadSettings, type Fetcher } from '../../api/settings.ts';
import { toasts } from '../toast/toast.ts';
import { barSettingKey, isBarMode, useBarMode, type BarModeHandle } from './useBarMode.ts';

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

function capture(): { handle: () => BarModeHandle } {
  let handle: BarModeHandle | undefined;
  render(() => {
    handle = useBarMode('composerRibbon');
    return <span />;
  });
  return { handle: () => handle! };
}

beforeEach(() => clearSettings());

describe('useBarMode', () => {
  it('maps a bar to its registry key and accepts only the two modes', () => {
    expect(barSettingKey('mailTopBar')).toBe('ui.bar.mailTopBar');
    expect(barSettingKey('composerRibbon')).toBe('ui.bar.composerRibbon');
    expect(isBarMode('auto')).toBe(true);
    expect(isBarMode('hidden')).toBe(false);
  });

  it('is pinned before the settings load and against a server that does not know the key', async () => {
    const { handle } = capture();
    expect(handle().mode()).toBe('pinned');
    await loadSettings({ fetcher: async () => json(200, { settings: {} }) });
    expect(handle().mode()).toBe('pinned');
    expect(handle().locked()).toBe(false);
  });

  it('shows a change at once and keeps it when the server stores it', async () => {
    const puts: unknown[] = [];
    await loadSettings({
      fetcher: (async (_url, init) => {
        if (init?.method === 'PUT') {
          puts.push(JSON.parse(String(init.body)));
          return json(200, { value: 'auto', source: 'user', locked: false });
        }
        return json(200, { settings: { 'ui.bar.composerRibbon': { value: 'pinned', source: 'default', locked: false } } });
      }) as Fetcher,
    });
    const { handle } = capture();
    const done = handle().setMode('auto');
    expect(handle().mode()).toBe('auto');
    expect(await done).toBe(true);
    expect(handle().mode()).toBe('auto');
    expect(puts).toEqual([{ value: 'auto' }]);
  });

  it('reverts a refused change and says so in a toast', async () => {
    await loadSettings({
      fetcher: (async (_url, init) =>
        init?.method === 'PUT'
          ? json(503, {})
          : json(200, { settings: { 'ui.bar.composerRibbon': { value: 'auto', source: 'user', locked: false } } })) as Fetcher,
    });
    const { handle } = capture();
    expect(handle().mode()).toBe('auto');
    const done = handle().setMode('pinned');
    expect(handle().mode()).toBe('pinned');
    expect(await done).toBe(false);
    expect(handle().mode()).toBe('auto');
    await waitFor(() => expect(toasts().some((t) => t.tone === 'error')).toBe(true));
  });

  it('does not write a locked key', async () => {
    let writes = 0;
    await loadSettings({
      fetcher: (async (_url, init) => {
        if (init?.method) writes++;
        return json(200, { settings: { 'ui.bar.composerRibbon': { value: 'auto', source: 'domain', locked: true } } });
      }) as Fetcher,
    });
    const { handle } = capture();
    expect(handle().locked()).toBe(true);
    expect(await handle().setMode('pinned')).toBe(false);
    expect(handle().mode()).toBe('auto');
    expect(writes).toBe(0);
  });
});

// t39 D4: below 600 px the composer ribbon starts in auto-hide, but only while
// nobody has chosen; a stored or administered value always wins.
describe('useBarMode narrow-screen default (t39 D4)', () => {
  function viewport(width: number): () => void {
    const before = window.matchMedia;
    const listeners = new Set<() => void>();
    const list = (query: string): MediaQueryList => {
      const max = /max-width:\s*([\d.]+)px/.exec(query);
      return {
        get matches() { return max ? width <= Number(max[1]) : false; },
        media: query,
        onchange: null,
        addEventListener: (_: string, fn: () => void) => listeners.add(fn),
        removeEventListener: (_: string, fn: () => void) => listeners.delete(fn),
        addListener: () => undefined,
        removeListener: () => undefined,
        dispatchEvent: () => true,
      } as unknown as MediaQueryList;
    };
    Object.defineProperty(window, 'matchMedia', { configurable: true, value: list });
    (viewport as unknown as { resize: (w: number) => void }).resize = (w) => { width = w; for (const fn of listeners) fn(); };
    return () => Object.defineProperty(window, 'matchMedia', { configurable: true, value: before });
  }
  const resize = (w: number): void => (viewport as unknown as { resize: (w: number) => void }).resize(w);
  const settingsWith = (entry: unknown) => loadSettings({ fetcher: async () => json(200, { settings: entry ? { 'ui.bar.composerRibbon': entry } : {} }) });

  it('is auto at 390 px while the value is the registry default, and pinned again at 1280 px', async () => {
    const restore = viewport(390);
    try {
      const { handle } = capture();
      expect(handle().mode()).toBe('auto');
      await settingsWith({ value: 'pinned', source: 'default', locked: false });
      expect(handle().mode()).toBe('auto');
      resize(1280);
      expect(handle().mode()).toBe('pinned');
      resize(599);
      expect(handle().mode()).toBe('auto');
      resize(600);
      expect(handle().mode()).toBe('pinned');
    } finally {
      restore();
    }
  });

  it('keeps an explicit pinned, from the user or an administrator, at 390 px', async () => {
    const restore = viewport(390);
    try {
      for (const source of ['user', 'global', 'domain']) {
        clearSettings();
        await settingsWith({ value: 'pinned', source, locked: false });
        const { handle } = capture();
        expect(handle().mode(), source).toBe('pinned');
      }
      clearSettings();
      await settingsWith({ value: 'pinned', source: 'global', locked: true });
      expect(capture().handle().mode()).toBe('pinned');
    } finally {
      restore();
    }
  });

  it('applies to the composer ribbon only', async () => {
    const restore = viewport(390);
    try {
      let mail: BarModeHandle | undefined;
      render(() => {
        mail = useBarMode('mailRibbon');
        return <span />;
      });
      expect(mail!.mode()).toBe('pinned');
    } finally {
      restore();
    }
  });
});
