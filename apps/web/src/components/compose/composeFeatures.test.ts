import { beforeEach, expect, it, vi } from 'vitest';

const state = vi.hoisted(() => ({ map: {} as Record<string, { value: unknown; locked?: boolean }> }));
vi.mock('../../api/settings.ts', () => ({
  settings: () => state.map,
  loadSettings: vi.fn(async () => undefined),
}));

import { COMPOSE_FEATURES, COMPOSE_FEATURE_GROUPS, composeFeatureKey, composeFeatureOn, isComposeFeatureId, presetValue, presetValues, useComposeFeature } from './composeFeatures.ts';

beforeEach(() => { state.map = {}; });

it('lists every feature once with a valid registry id, a known group and a label', () => {
  const ids = COMPOSE_FEATURES.map((f) => f.id);
  expect(new Set(ids).size).toBe(ids.length);
  for (const f of COMPOSE_FEATURES) {
    expect(isComposeFeatureId(f.id)).toBe(true);
    expect(COMPOSE_FEATURE_GROUPS).toContain(f.group);
    expect(f.labelKey).toMatch(/^[a-z]/);
  }
  for (const id of ['layout.columns', 'layout.contentWidth', 'layout.pageColor', 'layout.themes', 'layout.defaultFont', 'layout.spacing', 'options.sentFolder', 'options.replyTo']) expect(ids).toContain(id);
});

it('rejects ids the registry family would refuse', () => {
  for (const bad of ['', 'Layout', '1a', 'a-b', 'a b', 'x'.repeat(60)]) expect(isComposeFeatureId(bad)).toBe(false);
});

it('falls back to defaultOn while the settings are unknown', () => {
  const on = useComposeFeature('layout.columns');
  expect(on()).toBe(true);
  expect(useComposeFeature('not.registered')()).toBe(true);
});

it('reads compose.feature.<id> from the settings map', () => {
  const on = useComposeFeature('layout.themes');
  state.map[composeFeatureKey('layout.themes')] = { value: false, locked: true };
  expect(on()).toBe(false);
  state.map[composeFeatureKey('layout.themes')] = { value: true };
  expect(on()).toBe(true);
  state.map[composeFeatureKey('layout.themes')] = { value: 'junk' };
  expect(composeFeatureOn('layout.themes', state.map)).toBe(true);
});

it('presets: everything turns all on, simple and locked-down switch advanced ones off', () => {
  const advanced = COMPOSE_FEATURES.find((f) => f.advanced)!;
  const basic = COMPOSE_FEATURES.find((f) => !f.advanced)!;
  expect(presetValues('everything', { admin: true }).every(([, v]) => v)).toBe(true);
  expect(presetValue('simple', advanced)).toBe(false);
  expect(presetValue('simple', basic)).toBe(true);
  expect(presetValue('lockedDown', advanced)).toBe(false);
  expect(presetValues('simple', { admin: false }).length).toBe(COMPOSE_FEATURES.filter((f) => !f.adminOnly).length);
});
