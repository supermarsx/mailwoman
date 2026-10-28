import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { SettingEntry } from '../api/settings.ts';

const state: { status: string; map: Record<string, SettingEntry> } = { status: 'ready', map: {} };
const writeSetting = vi.fn<(key: string, value: unknown) => Promise<SettingEntry>>();

vi.mock('../api/settings.ts', () => ({
  settings: () => state.map,
  settingsLoadStatus: () => state.status,
  writeSetting: (key: string, value: unknown) => writeSetting(key, value),
}));

const {
  LEGACY_RIBBON_MARKER,
  legacyRibbonCollapsedPending,
  migrateLegacyRibbonCollapsed,
  noteLegacyRibbonCollapsed,
} = await import('./legacyRibbonCollapsed.ts');

const KEY = 'ui.bar.mailRibbon';
const entry = (over: Partial<SettingEntry> = {}): SettingEntry => ({ value: 'pinned', source: 'default', locked: false, ...over });

describe('legacy ribbonCollapsed -> ui.bar.mailRibbon', () => {
  beforeEach(() => {
    localStorage.clear();
    writeSetting.mockReset();
    state.status = 'ready';
    state.map = { [KEY]: entry() };
  });

  it('does nothing without the marker', async () => {
    await migrateLegacyRibbonCollapsed();
    expect(writeSetting).not.toHaveBeenCalled();
  });

  it('writes auto once when the user has no value, then drops the marker', async () => {
    noteLegacyRibbonCollapsed();
    writeSetting.mockResolvedValue(entry({ value: 'auto', source: 'user', userValue: 'auto' }));
    await migrateLegacyRibbonCollapsed();
    expect(writeSetting).toHaveBeenCalledWith(KEY, 'auto');
    expect(localStorage.getItem(LEGACY_RIBBON_MARKER)).toBeNull();
    await migrateLegacyRibbonCollapsed();
    expect(writeSetting).toHaveBeenCalledTimes(1);
  });

  it('keeps a value the user already stored', async () => {
    noteLegacyRibbonCollapsed();
    state.map = { [KEY]: entry({ source: 'user', userValue: 'pinned' }) };
    await migrateLegacyRibbonCollapsed();
    expect(writeSetting).not.toHaveBeenCalled();
    expect(legacyRibbonCollapsedPending()).toBe(false);
  });

  it('respects an administrator lock', async () => {
    noteLegacyRibbonCollapsed();
    state.map = { [KEY]: entry({ locked: true, source: 'domain' }) };
    await migrateLegacyRibbonCollapsed();
    expect(writeSetting).not.toHaveBeenCalled();
    expect(legacyRibbonCollapsedPending()).toBe(false);
  });

  it('waits for the settings map and for a server that knows the key', async () => {
    noteLegacyRibbonCollapsed();
    state.status = 'loading';
    await migrateLegacyRibbonCollapsed();
    state.status = 'ready';
    state.map = {};
    await migrateLegacyRibbonCollapsed();
    expect(writeSetting).not.toHaveBeenCalled();
    expect(legacyRibbonCollapsedPending()).toBe(true);
  });

  it('keeps the marker when the write fails', async () => {
    noteLegacyRibbonCollapsed();
    writeSetting.mockRejectedValue(new Error('offline'));
    await migrateLegacyRibbonCollapsed();
    expect(legacyRibbonCollapsedPending()).toBe(true);
  });
});
