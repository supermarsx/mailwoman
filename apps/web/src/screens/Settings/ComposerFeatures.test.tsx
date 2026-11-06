import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@solidjs/testing-library';

type Entry = { value: unknown; source: string; locked: boolean };
const fx = vi.hoisted(() => ({
  set: null as unknown as (v: Record<string, unknown>) => void,
  write: vi.fn(async (_k: string, _v: unknown) => ({})),
  reset: vi.fn(async (_k: string) => ({})),
}));
vi.mock('../../api/settings.ts', async () => {
  const { createSignal } = await import('solid-js');
  const [map, setMap] = createSignal<Record<string, unknown>>({});
  fx.set = setMap;
  return {
    settings: map,
    settingsLoadStatus: () => 'ready',
    loadSettings: vi.fn(async () => undefined),
    writeSetting: fx.write,
    resetSetting: fx.reset,
    SettingsApiError: class extends Error {},
  };
});

import { ComposerFeatures, COMPOSER_FEATURES_CARD, rowId } from './ComposerFeatures.tsx';
import { COMPOSE_FEATURES, composeFeatureKey } from '../../components/compose/composeFeatures.ts';

const setMap = (m: Record<string, Entry>): void => fx.set(m);
beforeEach(() => { setMap({}); fx.write.mockClear(); fx.reset.mockClear(); });
afterEach(cleanup);

it('renders every user-editable COMPOSE_FEATURES entry exactly once, and matches the search index', () => {
  const { container } = render(() => <ComposerFeatures />);
  for (const f of COMPOSE_FEATURES.filter((x) => !x.adminOnly)) expect(container.querySelectorAll(`#${rowId(f.id)}`).length).toBe(1);
  expect(COMPOSER_FEATURES_CARD.rows.map((r) => r.id).sort()).toEqual(COMPOSE_FEATURES.filter((x) => !x.adminOnly).map((f) => rowId(f.id)).sort());
});

it('writes compose.feature.<id> when a switch is toggled', () => {
  render(() => <ComposerFeatures />);
  fireEvent.click(document.getElementById(rowId('layout.columns'))!);
  expect(fx.write).toHaveBeenCalledWith(composeFeatureKey('layout.columns'), false);
});

it('shows the administrator lock note and disables a locked switch', () => {
  setMap({ [composeFeatureKey('layout.themes')]: { value: false, source: 'domain', locked: true } });
  render(() => <ComposerFeatures />);
  const box = document.getElementById(rowId('layout.themes')) as HTMLInputElement;
  expect(box.disabled).toBe(true);
  expect(box.checked).toBe(false);
  expect(screen.getAllByTestId('setting-lock-note').length).toBe(1);
});

it('master toggle writes every unlocked feature in the group', async () => {
  setMap({ [composeFeatureKey('layout.themes')]: { value: true, source: 'domain', locked: true } });
  render(() => <ComposerFeatures />);
  fireEvent.click(document.getElementById('cf-group-layout')!);
  await waitFor(() => expect(fx.write).toHaveBeenCalled());
  const keys = fx.write.mock.calls.map((c) => c[0]);
  expect(keys).not.toContain(composeFeatureKey('layout.themes'));
  expect(keys).toContain(composeFeatureKey('layout.columns'));
  expect(fx.write.mock.calls.every((c) => c[1] === false)).toBe(true);
});

it('applies the Simple profile and resets only user values', async () => {
  setMap({ [composeFeatureKey('layout.defaultFont')]: { value: false, source: 'user', locked: false } });
  render(() => <ComposerFeatures />);
  fireEvent.change(document.getElementById('cf-preset')!, { target: { value: 'simple' } });
  fireEvent.click(screen.getByRole('button', { name: 'Apply profile' }));
  await waitFor(() => expect(fx.write).toHaveBeenCalled());
  expect(fx.write).toHaveBeenCalledWith(composeFeatureKey('layout.columns'), false);
  expect(fx.write).toHaveBeenCalledWith(composeFeatureKey('layout.defaultFont'), true);
  await screen.findByText(/Profile applied/);
  fireEvent.click(screen.getByRole('button', { name: 'Reset to defaults' }));
  await waitFor(() => expect(fx.reset).toHaveBeenCalledWith(composeFeatureKey('layout.defaultFont')));
  expect(fx.reset).toHaveBeenCalledTimes(1);
});
