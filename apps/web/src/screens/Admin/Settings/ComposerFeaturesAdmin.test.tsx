import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@solidjs/testing-library';

const api = vi.hoisted(() => ({
  putGlobal: vi.fn(async () => ({})), deleteGlobal: vi.fn(async () => ({})),
  putDomainValue: vi.fn(async () => ({})), deleteDomainValue: vi.fn(async () => undefined),
  listScopedValues: vi.fn(async () => ({ values: [], nextCursor: null })),
}));
vi.mock('../../../api/admin/settings.ts', () => api);

import { ComposerFeaturesAdmin } from './ComposerFeaturesAdmin.tsx';
import { COMPOSE_FEATURES, composeFeatureKey } from '../../../components/compose/composeFeatures.ts';
import type { AdminSetting } from '../../../api/admin/settings.ts';

afterEach(() => { cleanup(); Object.values(api).forEach((m) => m.mockClear()); });

const row = (id: string, extra: Partial<AdminSetting>): AdminSetting => ({ key: composeFeatureKey(id), globalValue: false, globalLocked: false, effective: { value: false, source: 'global' }, overrides: {}, etag: 'e1', ...extra } as AdminSetting);

it('lists every COMPOSE_FEATURES entry once', () => {
  const { container } = render(() => <ComposerFeaturesAdmin rows={[]} />);
  for (const f of COMPOSE_FEATURES) expect(container.querySelectorAll(`[data-feature="${f.id}"]`).length).toBe(1);
});

it('writes a global value with the lock flag', async () => {
  render(() => <ComposerFeaturesAdmin rows={[row('layout.columns', { globalValue: true, effective: { value: true, source: 'global' } })]} />);
  const item = document.querySelector('[data-feature="layout.columns"]')!;
  const lock = item.querySelectorAll('input')[1]!;
  fireEvent.click(lock);
  await waitFor(() => expect(api.putGlobal).toHaveBeenCalled());
  expect(api.putGlobal).toHaveBeenCalledWith(composeFeatureKey('layout.columns'), { globalValue: true, locked: true, allowUserOverride: false }, 'e1');
});

it('domain scope needs a valid domain and writes a domain value', async () => {
  render(() => <ComposerFeaturesAdmin rows={[]} />);
  fireEvent.change(screen.getByLabelText('Applies to'), { target: { value: 'domain' } });
  const box = document.querySelector('[data-feature="layout.themes"] input') as HTMLInputElement;
  expect(box.disabled).toBe(true);
  fireEvent.input(screen.getByLabelText('Domain'), { target: { value: 'example.com' } });
  expect(box.disabled).toBe(false);
  fireEvent.click(box);
  await waitFor(() => expect(api.putDomainValue).toHaveBeenCalledWith(composeFeatureKey('layout.themes'), 'example.com', { value: false, locked: false }, null));
});

it('a profile writes every feature as the default', async () => {
  render(() => <ComposerFeaturesAdmin rows={[]} />);
  fireEvent.change(screen.getByLabelText('Profiles'), { target: { value: 'simple' } });
  fireEvent.click(screen.getByRole('button', { name: 'Apply profile' }));
  await waitFor(() => expect(api.putGlobal).toHaveBeenCalledTimes(COMPOSE_FEATURES.length));
});
