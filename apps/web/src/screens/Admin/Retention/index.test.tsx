import { describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import Retention, { type RetentionApi } from './index.tsx';
import { keepsForever, limitKeys, limitMessage, topBy, formatBytes } from './model.ts';
import type { DryRun, RetentionSnapshot, RetentionStore } from '../../../api/admin/retention.ts';
import type { AdminSetting } from '../../../api/admin/settings.ts';
import { renderInShell } from '../shell/testkit.tsx';

const store = (over: Partial<RetentionStore> & { id: string }): RetentionStore => ({
  owner: 'sweeper', unit: 'age', key: `retention.${over.id}.days`, days: 90, defaultDays: 90, fixed: false,
  rows: 10, bytes: 2048, lastSweepAt: 1_800_000_000, lastRows: 2, purged: 5, error: null, ...over,
});
const snapshot = (): RetentionSnapshot => ({
  sweeper: {
    enabled: true, nextRunAt: 1_800_086_400, lastRunAt: 1_800_000_000, lastTotal: 7, vacuumed: false, running: false,
    keys: { enabled: 'retention.sweeper.enabled' },
  },
  stores: [
    store({ id: 'alertEvents' }),
    store({ id: 'outbox', days: 0, rows: 3, bytes: null, purged: 0 }),
    store({ id: 'rememberedDevices', unit: 'grace', key: 'retention.expiredCredentials.graceDays', days: 1, defaultDays: 1 }),
    store({ id: 'authHistory', owner: 'existing', key: 'logs.auth.retentionDays', days: 7, lastSweepAt: null, purged: null }),
    store({ id: 'protectionEvents', owner: 'existing', key: null, days: 30, fixed: true, lastSweepAt: null, purged: null }),
  ],
  logs: {
    status: { fileEnabled: true, directory: 'logs', currentBytes: 2048, rotatedFiles: 3, rotatedBytes: 1_048_576 },
    keys: ['logging.level'],
  },
});
const setting = (key: string, value: unknown, kind: AdminSetting['kind']): AdminSetting => ({
  key, family: false, kind, default: value, nullable: false, scopes: 'global', userOverride: false, category: 'privacy', mapsTo: '', env: null, dedicatedRoute: null,
  scopeId: 'account', hot: true, ceiling: false, userVisible: false, globalValue: null, allowUserOverride: false, globalLocked: false, updatedBy: null, updatedAt: null,
  effective: { value, source: 'default' }, envState: null, overrides: {}, etag: '"v1"',
});
const dry: DryRun = {
  dryRun: true, total: 12, heldAccounts: 1,
  stores: [
    { id: 'alertEvents', rows: 9, more: false, skipped: null, error: null },
    { id: 'outbox', rows: 0, more: false, skipped: 'keep', error: null },
    { id: 'rememberedDevices', rows: 3, more: true, skipped: null, error: null },
  ],
};
const api = (over: Partial<RetentionApi> = {}): RetentionApi => ({
  load: vi.fn(async () => snapshot()),
  preview: vi.fn(async () => dry),
  run: vi.fn(async () => ({ runId: 'r1' })),
  settings: vi.fn(async () => [
    setting('retention.alertEvents.days', 90, { type: 'int', min: 0, max: 3650 }),
    setting('logging.level', 'info', { type: 'enum', values: ['error', 'warn', 'info', 'debug', 'trace'] }),
  ]),
  ...over,
});

describe('Admin retention', () => {
  it('lists every store with its limit, size, last sweep and rows removed', async () => {
    renderInShell(() => <Retention api={api()} />);
    const table = await screen.findByTestId('retention-stores');
    expect(within(table).getByTestId('retention-row-alertEvents')).toHaveTextContent('90 days');
    expect(within(table).getByTestId('retention-row-alertEvents')).toHaveTextContent('2.0 KiB');
    expect(within(table).getByTestId('retention-row-rememberedDevices')).toHaveTextContent('1 days after expiry');
    expect(within(table).getByTestId('retention-row-authHistory')).toHaveTextContent('Its own job');
    expect(within(table).getByTestId('retention-row-protectionEvents')).toHaveTextContent('30 days (fixed)');
    cleanup();
  });

  it('warns when a limit of 0 keeps rows forever, and only for sweeper stores', async () => {
    renderInShell(() => <Retention api={api()} />);
    expect(await screen.findByTestId('retention-warning-outbox')).toHaveTextContent('until it is deleted by hand');
    expect(screen.queryByTestId('retention-warning-alertEvents')).toBeNull();
    cleanup();
  });

  it('shows the dry-run counts first and runs only after confirmation', async () => {
    const a = api();
    renderInShell(() => <Retention api={a} />);
    fireEvent.click(await screen.findByTestId('retention-preview'));
    const result = await screen.findByTestId('retention-dry-result');
    expect(result).toHaveTextContent('12');
    expect(result).toHaveTextContent('1 account(s) are under a legal hold');
    expect(result).toHaveTextContent('3+');
    expect(a.run).not.toHaveBeenCalled();
    fireEvent.click(within(result).getByTestId('retention-run'));
    await waitFor(() => expect(a.run).toHaveBeenCalledTimes(1));
    expect(await screen.findByText('The sweep started. Counts update when it finishes.')).toBeInTheDocument();
    cleanup();
  });

  it('edits limits and log settings in place', async () => {
    renderInShell(() => <Retention api={api()} />);
    await screen.findByTestId('retention-stores');
    await waitFor(() => expect(screen.getAllByText(/Alert|alert|Fired alert records/).length).toBeGreaterThan(0));
    expect(screen.getByTestId('retention-log-status')).toHaveTextContent('3 rotated file(s)');
    cleanup();
  });

  it('reports a failed load', async () => {
    renderInShell(() => <Retention api={api({ load: vi.fn(async () => { throw new Error('nope'); }) })} />);
    expect(await screen.findByRole('alert')).toBeInTheDocument();
    cleanup();
  });
});

describe('retention model', () => {
  const s = snapshot().stores;
  it('describes limits and finds the keep-forever warnings', () => {
    expect(limitMessage(s[0]!)).toEqual({ id: 'admin-retention-limit-days', args: { days: 90 } });
    expect(limitMessage(s[1]!).id).toBe('admin-retention-limit-forever');
    expect(limitMessage(s[2]!).id).toBe('admin-retention-limit-grace');
    expect(limitMessage(s[4]!).id).toBe('admin-retention-limit-fixed');
    expect(s.filter(keepsForever).map((x) => x.id)).toEqual(['outbox']);
  });
  it('lists each editable key once and ranks charts', () => {
    expect(limitKeys(s)).toEqual(['retention.alertEvents.days', 'retention.outbox.days', 'retention.expiredCredentials.graceDays']);
    expect(topBy(s, (x) => x.rows).map((x) => x.id)).toEqual(['alertEvents', 'rememberedDevices', 'authHistory', 'protectionEvents', 'outbox']);
    expect(formatBytes(512)).toBe('512 B');
    expect(formatBytes(1_048_576)).toBe('1.0 MiB');
  });
});
