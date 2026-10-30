// t75: Save on a setting's page must not take the page down. The console renders
// each section under one <Suspense> (screens/Admin/index.tsx). After a write the
// page reloads the settings list, the setting and its domain/per-user rows, and
// reading any of those resources while it reloads used to suspend that boundary:
// the whole Settings page was swapped for the page skeleton and back. The page
// must stay mounted (the same nodes), show the saved value, and say "Saving…"
// and then "Saved" in place.

import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { Suspense, type JSX } from 'solid-js';
import { AdminSettings } from './index.tsx';
import type { AdminSetting } from '../../../api/admin/settings.ts';
import type { AdminReadResult } from '../shell/adminApi.ts';
import { Dashboard } from '../Dashboard/Dashboard.tsx';
import { engineDashboard } from '../Dashboard/fixtures.ts';
import { renderInShell } from '../shell/testkit.tsx';
import type { DashboardResult } from '../../../api/admin/dashboard.ts';
import { t } from '../../../test/i18n.ts';
import { loadRequestHistory } from '../../../api/admin/requestHistory.ts';

vi.mock('../../../api/admin/requestHistory.ts', () => ({
  loadRequestHistory: vi.fn(),
  loadNotFoundPaths: vi.fn(() => Promise.reject(new Error('not in this test'))),
}));

const BUDGET: AdminSetting = {
  key: 'offline.budgetMb',
  family: false,
  kind: { type: 'int', min: 0, max: 10240 },
  default: 250,
  nullable: false,
  scopes: 'global+domain+user',
  userOverride: true,
  category: 'offline',
  mapsTo: '',
  env: null,
  dedicatedRoute: null,
  scopeId: 'account',
  hot: true,
  ceiling: false,
  userVisible: true,
  globalValue: null,
  allowUserOverride: true,
  globalLocked: false,
  updatedBy: null,
  updatedAt: null,
  effective: { value: 250, source: 'default' },
  envState: null,
  overrides: {},
  etag: '"e0"',
};
const SAVED: AdminSetting = { ...BUDGET, globalValue: 512, effective: { value: 512, source: 'global' }, etag: '"e1"' };

function json(status: number, body: unknown, etag?: string): Response {
  const headers: Record<string, string> = { 'content-type': 'application/json' };
  if (etag) headers['ETag'] = etag;
  return new Response(JSON.stringify(body), { status, headers });
}

/** A gate: every call after `hold()` waits for `release()`. */
function gate() {
  let held = false;
  const waiting: (() => void)[] = [];
  return {
    hold: () => void (held = true),
    release: () => {
      held = false;
      for (const r of waiting.splice(0)) r();
    },
    pass: async () => {
      if (held) await new Promise<void>((r) => waiting.push(r));
    },
  };
}

/** The console's page boundary, counting how often its fallback is drawn. */
function boundary(ui: () => JSX.Element) {
  let fallbacks = 0;
  const Fallback = () => {
    fallbacks++;
    return <p data-testid="page-skeleton">Loading…</p>;
  };
  return { ui: () => <Suspense fallback={<Fallback />}>{ui()}</Suspense>, fallbacks: () => fallbacks };
}

describe('Save on a setting page keeps the page (t75)', () => {
  beforeEach(() => localStorage.clear());
  afterEach(() => vi.unstubAllGlobals());

  it('the page stays mounted with the saved value while the list, the setting and the rows reload', async () => {
    const reloads = gate();
    let saved = false;
    let listCalls = 0;
    const list = async (): Promise<AdminReadResult<AdminSetting[]>> => {
      listCalls++;
      await reloads.pass();
      return { kind: 'ok', data: [saved ? SAVED : BUDGET], etag: null };
    };
    const loadOne = async () => {
      await reloads.pass();
      return { setting: saved ? SAVED : BUDGET, etag: saved ? '"e1"' : '"e0"' };
    };
    const puts: unknown[] = [];
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: string, init?: RequestInit) => {
        const url = String(input);
        if (init?.method === 'PUT' && url.includes('/domains/')) {
          await reloads.pass();
          const row = { scope: 'domain', scopeId: 'example.org', value: 64, locked: false, updatedBy: 'root', updatedAt: 1, etag: '"d1"' };
          return json(200, row, '"d1"');
        }
        if (init?.method === 'PUT') {
          puts.push(JSON.parse(String(init.body)));
          saved = true;
          return json(200, SAVED, '"e1"');
        }
        if (url.includes('scope=')) {
          await reloads.pass();
          return json(200, { values: [], nextCursor: null });
        }
        return json(404, {});
      }),
    );

    const b = boundary(() => (
      <AdminSettings route={{ section: 'settings', rest: [BUDGET.key] }} load={list} loadOne={loadOne} />
    ));
    renderInShell(b.ui);
    const page = await screen.findByTestId('admin-settings');
    const detail = await screen.findByTestId('setting-detail');
    const domains = (await screen.findByRole('heading', { name: t('admin-settings-domains') })).closest('section')!;
    await waitFor(() => expect(within(domains).queryByTestId('table-empty')).not.toBeNull());
    const before = b.fallbacks();
    const input = screen.getByRole('spinbutton', { name: /Offline cache size/ }) as HTMLInputElement;

    reloads.hold();
    fireEvent.input(input, { target: { value: '512' } });
    fireEvent.click(screen.getByRole('button', { name: t('admin-settings-save') }));
    await waitFor(() => expect(puts).toEqual([{ globalValue: 512 }]));
    await waitFor(() => expect(listCalls).toBe(2));

    // The list is reloading: the page, the form and the saved value are all there.
    expect(screen.getByTestId('admin-settings')).toBe(page);
    expect(screen.getByTestId('setting-detail')).toBe(detail);
    expect(screen.queryByTestId('page-skeleton')).toBeNull();
    expect(input.isConnected).toBe(true);
    expect(input.value).toBe('512');
    expect(await screen.findByText(t('admin-settings-saved'))).toBeInTheDocument();

    reloads.release();
    await waitFor(() => expect(screen.getByTestId('source-badge').getAttribute('data-source')).toBe('stored'));
    expect(screen.getByTestId('setting-detail')).toBe(detail);
    expect(b.fallbacks()).toBe(before);

    // A domain value written: its rows, the setting and the list reload; still the same page.
    reloads.hold();
    fireEvent.input(within(domains).getByRole('textbox', { name: t('admin-settings-domain') }), { target: { value: 'example.org' } });
    fireEvent.input(within(domains).getByRole('spinbutton', { name: t('admin-settings-domain-value') }), { target: { value: '64' } });
    fireEvent.click(within(domains).getByRole('button', { name: t('admin-settings-domain-save') }));
    // Progress is in the shared toast; the mounted form keeps its readable action.
    expect(within(domains).getByRole('button', {name:t('admin-settings-domain-save')})).toHaveAttribute('aria-busy','true');
    expect(await screen.findByText(t('admin-ux-form-saving'))).toBeInTheDocument();
    expect(within(domains).queryByText(t('admin-ux-form-saving'))).toBeNull();
    expect(screen.getByTestId('setting-detail')).toBe(detail);
    reloads.release();
    await waitFor(() => expect(within(domains).getByRole('button', { name: t('admin-settings-domain-save') })).toBeInTheDocument());
    await waitFor(() => expect(screen.getAllByText(t('admin-settings-saved')).length).toBeGreaterThan(0));
    expect(screen.getByTestId('admin-settings')).toBe(page);
    expect(screen.getByTestId('setting-detail')).toBe(detail);
    expect(b.fallbacks()).toBe(before);
  });
});

describe('Dashboard refresh keeps the dashboard (t75)', () => {
  beforeEach(() => {
    localStorage.clear();
    const series = engineDashboard().traffic!.series.map((p) => ({ ...p, coverage: 'complete' as const }));
    vi.mocked(loadRequestHistory).mockResolvedValue({
      from: 1790000000, to: 1790003600, bucketSeconds: 60, series,
      coverage: { observedSeconds: 3600, requestedSeconds: 3600, gaps: [] },
      policy: { defaultRangeSeconds: 3600, maxRangeSeconds: 2592000, retentionDays: 30 },
      scope: 'server', timing: 'response-ready',
    });
  });
  it('Refresh reloads the figures in place, without the page skeleton', async () => {
    const reloads = gate();
    const load = vi.fn(async (): Promise<DashboardResult> => {
      await reloads.pass();
      return { kind: 'ok', data: engineDashboard() };
    });
    const b = boundary(() => <Dashboard />);
    renderInShell(b.ui, { loadDashboard: load });
    const tile = await screen.findByTestId('tile-users');
    const before = b.fallbacks();
    reloads.hold();
    fireEvent.click(screen.getByRole('button', { name: t('admin-dash-refresh') }));
    await waitFor(() => expect(load).toHaveBeenCalledTimes(2));
    expect(screen.getByTestId('tile-users')).toBe(tile);
    expect(screen.queryByTestId('page-skeleton')).toBeNull();
    reloads.release();
    await waitFor(() => expect(screen.getByRole('button', { name: t('admin-dash-refresh') })).not.toBeDisabled());
    expect(screen.getByTestId('tile-users')).toBe(tile);
    expect(b.fallbacks()).toBe(before);
  });
});
