import { describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import Retention, { type RetentionApi } from './index.tsx';
import { bucketLabels, classWarnings } from './model.ts';
import type { ArchiveSnapshot, ClassesSnapshot, UserRulesOverview } from '../../../api/admin/retention.ts';
import { renderInShell } from '../shell/testkit.tsx';

const classes = (over: Partial<ClassesSnapshot> = {}): ClassesSnapshot => ({
  enforce: false, enforceKey: 'retention.class.enforce', edgesDays: [30, 90, 365, 1095], warnings: [],
  classes: [
    { id: 'short', key: 'retention.class.short.days', days: 365, effectiveDays: 365, forever: false, ageBuckets: [5, 0, 0, 0, 0], entries: [
      { id: 'sessions', key: 'retention.sessions.idleDays', ownDays: 90, days: 90, keptForever: false, exceedsClass: false, archivable: false, ageBuckets: [5, 0, 0, 0, 0] },
    ] },
    { id: 'medium', key: 'retention.class.medium.days', days: 1095, effectiveDays: 1095, forever: false, ageBuckets: [0, 0, 0, 0, 0], entries: [] },
    { id: 'long', key: 'retention.class.long.days', days: 2555, effectiveDays: 2555, forever: false, ageBuckets: [1, 2, 3, 0, 4], entries: [
      { id: 'dlpAudit', key: 'retention.dlpAudit.days', ownDays: 0, days: 0, keptForever: true, exceedsClass: true, archivable: true, ageBuckets: [1, 2, 3, 0, 4] },
    ] },
  ],
  ...over,
});

const archive = (over: Partial<ArchiveSnapshot> = {}): ArchiveSnapshot => ({
  enabled: true, directory: 'archive', directoryValid: true, keyGeneration: 2, legalHold: false, longDays: 2555, longForever: false,
  totals: { segments: 2, rows: 1500, bytes: 20480, oldestVerification: 1_800_000_000, oldestRow: 1_500_000_000, failing: 1 },
  stores: [{ id: 'dlpAudit', segments: 2, rows: 1500, bytes: 20480 }],
  segments: [
    { id: 'seg-1', storeId: 'dlpAudit', seq: 1, rowCount: 1000, firstTs: 1_500_000_000, lastTs: 1_600_000_000, bytes: 10240, keyGen: 1, state: 'sealed', createdAt: 1, verifiedAt: 1, verifyStatus: 'ok' },
    { id: 'seg-2', storeId: 'dlpAudit', seq: 2, rowCount: 500, firstTs: 1_600_000_001, lastTs: 1_700_000_000, bytes: 10240, keyGen: 2, state: 'sealed', createdAt: 1, verifiedAt: 1, verifyStatus: 'contentChanged' },
  ],
  status: { verifiedAt: 1_800_000_000, checked: 2, ok: 1, failed: [{ segment: 'seg-2', verdict: 'contentChanged' }], purged: 0 },
  tables: ['dlpAudit', 'assistAudit'],
  keys: ['retention.archive.enabled'],
  ...over,
});

const users = (): UserRulesOverview => ({
  enabled: true, usersWithRules: 14, rules: 40, enabledRules: 36, runs30d: 300, matched30d: 9000, messagesChanged30d: 8123,
  lastJob: { at: 1_800_000_000, accounts: 14, changed: 120, errors: 0, skipped: 3 }, keys: ['retention.user.enabled'],
});

const tiers = (over: Partial<NonNullable<RetentionApi['tiers']>> = {}): NonNullable<RetentionApi['tiers']> => ({
  classes: vi.fn(async () => classes()),
  archive: vi.fn(async () => archive()),
  userRules: vi.fn(async () => users()),
  verify: vi.fn(async () => ({ checked: 2, ok: 1, failed: [['seg-2', 'contentChanged']] as [string, string][], purged: 0 })),
  search: vi.fn(async () => ({ hits: [{ segmentId: 'seg-1', row: { id: 'row-9', action: 'block' } }] })),
  restore: vi.fn(async () => ({ restored: 3 })),
  settings: vi.fn(async () => []),
  ...over,
});

const api = (t = tiers()): RetentionApi => ({
  load: vi.fn(async () => ({
    sweeper: { enabled: true, nextRunAt: null, lastRunAt: null, lastTotal: null, vacuumed: null, running: false, keys: {} },
    stores: [], logs: { status: { fileEnabled: false, directory: 'logs', currentBytes: 0, rotatedFiles: 0, rotatedBytes: 0 }, keys: [] },
  })),
  preview: vi.fn(), run: vi.fn(), settings: vi.fn(async () => []), tiers: t,
});

describe('Admin retention tiers', () => {
  it('lists each class with its stores, ceiling and the keep-forever and exceeds-class notes', async () => {
    renderInShell(() => <Retention api={api()} />);
    const long = await screen.findByTestId('retention-class-long');
    expect(long).toHaveTextContent('2,555 days');
    expect(within(long).getByTestId('retention-class-entry-dlpAudit')).toHaveTextContent('Kept forever');
    expect(within(long).getByTestId('retention-class-entry-dlpAudit')).toHaveTextContent('above the class ceiling');
    expect(within(long).getByTestId('retention-class-entry-dlpAudit')).toHaveTextContent('can be archived');
    expect(screen.getByTestId('retention-class-enforce')).toHaveTextContent('class limit is off');
    expect(screen.getByTestId('retention-age-long')).toBeInTheDocument();
    cleanup();
  });

  it('warns when the class ceilings are out of order or the long class is forever', async () => {
    const t = tiers({ classes: vi.fn(async () => classes({ warnings: ['longBelowMedium', 'longForever'] })) });
    renderInShell(() => <Retention api={api(t)} />);
    const warnings = await screen.findByTestId('retention-class-warnings');
    expect(warnings).toHaveTextContent('long class is set below the medium class');
    expect(warnings).toHaveTextContent('kept forever');
    cleanup();
  });

  it('shows the archive size, the last check and the files that fail it', async () => {
    renderInShell(() => <Retention api={api()} />);
    expect(await screen.findByTestId('retention-archive-segments')).toHaveTextContent('2');
    expect(screen.getByTestId('retention-archive-size')).toHaveTextContent('20 KiB');
    expect(screen.getByTestId('retention-archive-failing')).toHaveTextContent('1');
    expect(screen.getByTestId('retention-segment-seg-2')).toHaveTextContent('contentChanged');
    expect(screen.getByTestId('retention-segment-seg-1')).toHaveTextContent('Passed');
    cleanup();
  });

  it('verifies on request and reports failures', async () => {
    const t = tiers();
    renderInShell(() => <Retention api={api(t)} />);
    fireEvent.click(await screen.findByTestId('retention-archive-verify'));
    expect(await screen.findByTestId('retention-archive-message')).toHaveTextContent('1 of 2 file(s) failed the check.');
    expect(t.verify).toHaveBeenCalledTimes(1);
    cleanup();
  });

  it('finds an archived row and restores its file', async () => {
    const t = tiers();
    renderInShell(() => <Retention api={api(t)} />);
    const find = await screen.findByTestId('retention-archive-find');
    expect(find).toBeDisabled();
    fireEvent.input(screen.getByTestId('retention-archive-key'), { target: { value: 'row-9' } });
    fireEvent.click(find);
    const hits = await screen.findByTestId('retention-archive-hits');
    expect(hits).toHaveTextContent('row-9');
    expect(t.search).toHaveBeenCalledWith('dlpAudit', { key: 'row-9', account: '' });
    fireEvent.click(within(hits).getByTestId('retention-archive-restore'));
    await waitFor(() => expect(t.restore).toHaveBeenCalledWith('seg-1'));
    expect(await screen.findByTestId('retention-archive-message')).toHaveTextContent('3 row(s) put back.');
    cleanup();
  });

  it('shows the user rules overview and a legal-hold notice', async () => {
    const t = tiers({ archive: vi.fn(async () => archive({ legalHold: true, longForever: true })) });
    renderInShell(() => <Retention api={api(t)} />);
    expect(await screen.findByTestId('retention-userrules-users')).toHaveTextContent('14');
    expect(screen.getByTestId('retention-userrules-changed')).toHaveTextContent('8,123');
    expect(await screen.findByText(/legal hold. No file is removed/)).toBeInTheDocument();
    cleanup();
  });

  it('leaves the screen as it was when the server offers no tiers', async () => {
    const plain = { ...api(), tiers: undefined };
    renderInShell(() => <Retention api={plain} />);
    await screen.findByTestId('retention-stores');
    expect(screen.queryByTestId('retention-class-long')).toBeNull();
    cleanup();
  });
});

describe('tiers model', () => {
  it('labels the age buckets from the edges', () => {
    const labels = bucketLabels([30, 90], (id, args) => `${id}:${JSON.stringify(args)}`);
    expect(labels).toEqual([
      'admin-retention-age-under:{"days":30}',
      'admin-retention-age-between:{"from":30,"to":90}',
      'admin-retention-age-over:{"days":90}',
    ]);
    expect(classWarnings({ warnings: ['longForever'] })).toEqual(['admin-retention-warn-longForever']);
  });
});
