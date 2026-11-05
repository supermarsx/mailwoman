import { describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import CookiesTab from './CookiesTab.tsx';
import { configFromRows, share, COOKIE_GROUPS, type DetectedRow, type InventoryView } from './model.ts';
import type { CookiesApi } from './api.ts';
import type { AdminSetting } from '../../../api/admin/settings.ts';
import type { PrivacyTabContext } from '../Privacy/model.ts';
import { renderInShell } from '../shell/testkit.tsx';
import { consentConfig, resetConsentForTests } from '../../../cookies/runtime.ts';

const row = (key: string, value: unknown, kind: AdminSetting['kind'] = { type: 'bool' }): AdminSetting => ({
  key, family: false, kind, default: value, nullable: false, scopes: 'global+domain', userOverride: false, category: 'privacy', mapsTo: '', env: null, dedicatedRoute: null,
  scopeId: 'account', hot: true, ceiling: false, userVisible: false, globalValue: null, allowUserOverride: false, globalLocked: false, updatedBy: null, updatedAt: null,
  effective: { value, source: 'default' }, envState: null, overrides: {}, etag: '"v1"',
});
const rows: AdminSetting[] = [
  row('cookies.enabled', true), row('cookies.mode', 'optIn', { type: 'enum', values: ['optIn', 'optOut', 'noticeOnly'] }),
  row('cookies.functional.enabled', true), row('cookies.functional.default', true), row('cookies.analytics.enabled', true), row('cookies.analytics.default', false),
  row('cookies.policyVersion', 4, { type: 'int', min: 1, max: 1000000 }), row('cookies.layout', 'modal', { type: 'enum', values: ['banner', 'card', 'modal'] }),
];
const ctx = (): PrivacyTabContext => ({
  rows: () => rows, settingsError: () => undefined, canReadSettings: () => true, stats: () => undefined, statsError: () => undefined, statsLoading: () => false,
  updatedAt: () => null, range: () => '1h', setRange: () => undefined, refresh: () => undefined, focusKey: () => undefined, onDirty: () => undefined, onSaved: () => undefined, reloadSettings: () => undefined,
});
const zero = { acceptAll: 0, rejectAll: 0, custom: 0, notice: 0 };
const api = (over: Partial<CookiesApi> = {}): CookiesApi => ({
  inventory: vi.fn(async (): Promise<InventoryView> => ({
    categories: ['necessary', 'functional', 'analytics', 'marketing'], kinds: ['cookie', 'localStorage', 'sessionStorage', 'indexedDB'],
    builtin: [{ id: 'session', kind: 'cookie', pattern: 'mw_session', category: 'necessary', purpose: 'Keeps you signed in.', duration: 'Session', party: 'first', provider: 'This server' }],
    rules: [{ id: 'r1', kind: 'cookie', pattern: '_ga*', category: 'analytics', note: 'Vendor', createdAt: 1 }],
  })),
  detected: vi.fn(async (): Promise<DetectedRow[]> => [{ id: 'd1', kind: 'localStorage', name: 'vendor.key', firstSeen: 1, lastSeen: 2, seenCount: 3, status: 'unclassified' }]),
  stats: vi.fn(async () => ({
    from: 0, to: 86400 * 2, total: 5, truncated: false, decisions: { acceptAll: 3, rejectAll: 1, custom: 1, notice: 0 },
    categories: { functional: { accepted: 4, total: 5 }, analytics: { accepted: 3, total: 5 }, marketing: { accepted: 0, total: 5 } },
    regions: [{ region: 'PT', count: 4 }], signals: { none: 4, gpc: 1, dnt: 0 }, versions: [{ version: 4, count: 5 }],
    series: [{ t: 0, decisions: { ...zero, acceptAll: 2 } }, { t: 86400, decisions: { ...zero, acceptAll: 1, rejectAll: 1, custom: 1 } }],
  })),
  saveRule: vi.fn(async () => undefined), deleteRule: vi.fn(async () => undefined), classify: vi.fn(async () => undefined), ignore: vi.fn(async () => undefined),
  remove: vi.fn(async () => undefined), scan: vi.fn(async () => 0), exportCsv: vi.fn(async () => 'time\r\n'),
  ...over,
});

describe('Admin cookies tab', () => {
  it('shows the inventory with built-in entries and the administrator rules', async () => {
    renderInShell(() => <CookiesTab ctx={ctx()} api={api()} />);
    const table = await screen.findByTestId('cookie-inventory');
    expect(within(table).getByText('mw_session')).toBeInTheDocument();
    expect(within(table).getByText('Keeps you signed in.')).toBeInTheDocument();
    expect(within(table).getAllByText('_ga*')[0]!.closest('tr')).toHaveTextContent('Vendor');
    expect(screen.getByText(/Set how the cookie consent control behaves/)).toBeInTheDocument();
  });

  it('classifies a reported name, which adds a rule', async () => {
    const a = api();
    renderInShell(() => <CookiesTab ctx={ctx()} api={a} />);
    const detected = await screen.findByTestId('cookie-detected');
    fireEvent.change(within(detected).getByLabelText('Category of vendor.key'), { target: { value: 'analytics' } });
    fireEvent.click(within(detected).getByRole('button', { name: 'Classify' }));
    await waitFor(() => expect(a.classify).toHaveBeenCalledWith('d1', 'analytics'));
  });

  it('ignores and removes reported names', async () => {
    const a = api();
    renderInShell(() => <CookiesTab ctx={ctx()} api={a} />);
    const detected = await screen.findByTestId('cookie-detected');
    fireEvent.click(within(detected).getByRole('button', { name: 'Ignore' }));
    await waitFor(() => expect(a.ignore).toHaveBeenCalledWith('d1'));
    fireEvent.click(within(detected).getByRole('button', { name: 'Remove' }));
    await waitFor(() => expect(a.remove).toHaveBeenCalledWith('d1'));
  });

  it('adds a rule from the form and removes one', async () => {
    const a = api();
    renderInShell(() => <CookiesTab ctx={ctx()} api={a} />);
    await screen.findByTestId('cookie-inventory');
    const add = screen.getByRole('button', { name: 'Add rule' });
    expect(add).toBeDisabled();
    fireEvent.input(screen.getByLabelText('Name'), { target: { value: 'pixel_*' } });
    fireEvent.change(screen.getByLabelText('Category'), { target: { value: 'marketing' } });
    fireEvent.click(add);
    await waitFor(() => expect(a.saveRule).toHaveBeenCalledWith({ kind: 'cookie', pattern: 'pixel_*', category: 'marketing', note: '' }));
    fireEvent.click(screen.getByRole('button', { name: /Remove\s+_ga\*/ }));
    await waitFor(() => expect(a.deleteRule).toHaveBeenCalledWith('r1'));
  });

  it('does not accept a pattern that matches everything', async () => {
    renderInShell(() => <CookiesTab ctx={ctx()} api={api()} />);
    await screen.findByTestId('cookie-inventory');
    fireEvent.input(screen.getByLabelText('Name'), { target: { value: '***' } });
    expect(screen.getByRole('button', { name: 'Add rule' })).toBeDisabled();
  });

  it('shows consent statistics as chart cards with tables, and exports the log as CSV', async () => {
    const a = api();
    const click = vi.fn();
    const create = vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(click);
    Object.assign(URL, { createObjectURL: vi.fn(() => 'blob:x'), revokeObjectURL: vi.fn() });
    renderInShell(() => <CookiesTab ctx={ctx()} api={a} />);
    expect(await screen.findByTestId('cookie-total')).toHaveTextContent('5 decisions');
    expect(screen.getByTestId('cookie-chart-decisions')).toBeInTheDocument();
    expect(screen.getByTestId('cookie-chart-categories')).toHaveTextContent('Analytics (60 %)');
    const daily = screen.getByTestId('cookie-chart-daily');
    expect(within(daily).getByRole('table', { hidden: true })).toHaveTextContent('Accept all');
    expect(screen.getByText(/1 Global Privacy Control, 0 Do Not Track/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Export log (CSV)' }));
    await waitFor(() => expect(a.exportCsv).toHaveBeenCalled());
    await waitFor(() => expect(click).toHaveBeenCalled());
    create.mockRestore();
  });

  it('previews the control from the saved settings without recording anything', async () => {
    resetConsentForTests();
    renderInShell(() => <CookiesTab ctx={ctx()} api={api()} />);
    await screen.findByTestId('cookie-inventory');
    fireEvent.click(screen.getByRole('button', { name: 'Preview the control' }));
    const config = consentConfig();
    expect(config).toMatchObject({ enabled: true, mode: 'optIn', version: 4, layout: 'modal' });
    expect(config?.categories?.find((c) => c.id === 'analytics')?.enabled).toBe(true);
    expect(config?.inventory?.length).toBe(1);
    cleanup();
    resetConsentForTests();
  });
});

describe('Cookies model', () => {
  it('lists every cookies.* registry key in exactly one group', () => {
    const all = COOKIE_GROUPS.flatMap((g) => g.keys);
    expect(new Set(all).size).toBe(all.length);
    expect(all.every((k) => k.startsWith('cookies.'))).toBe(true);
  });
  it('builds a preview configuration and computes shares', () => {
    expect(configFromRows([], undefined)).toMatchObject({ enabled: true, mode: 'optIn', layout: 'banner', version: 1 });
    expect(share(1, 3)).toBe(33.3);
    expect(share(1, 0)).toBe(0);
  });
});
