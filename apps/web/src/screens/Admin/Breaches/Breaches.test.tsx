import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import Breaches, { type BreachApi } from './index.tsx';
import type { BreachDetail, BreachIncident, BreachList, BreachMeta, BreachSummary } from '../../../api/admin/breaches.ts';
import { renderInShell } from '../shell/testkit.tsx';

const nowSecs = Math.floor(Date.now() / 1000);

const summary = (over: Partial<BreachSummary> & { id: string }): BreachSummary => ({
  reference: 'BR-TEST0001', title: 'Mailbox exposed', domain: 'example.test', status: 'assessing', severity: 'risk', source: 'manual', type: 'mailbox-access',
  detectedAt: nowSecs - 3600, awareAt: nowSecs - 3600, deadlineAt: nowSecs + 71 * 3600, assignee: null, updatedAt: nowSecs, authorityNotifiedAt: null, affectedCount: 2, draft: false,
  clock: { phase: 'running', awareAt: nowSecs - 3600, deadlineAt: nowSecs + 71 * 3600, remainingSecs: 71 * 3600, elapsedSecs: 3600, notifiedAt: null }, ...over,
});
const list = (rows: BreachSummary[]): BreachList => ({
  incidents: rows, matched: rows.length,
  stats: { total: rows.length, open: rows.length, overdue: rows.filter((r) => r.clock.phase === 'overdue').length, dueSoon: 0, drafts: rows.filter((r) => r.draft).length, notifiedLate: 0, byStatus: { assessing: rows.length }, bySeverity: {}, created30d: [{ t: nowSecs, count: rows.length }] },
});
const meta: BreachMeta = {
  statuses: ['detected', 'assessing'], sources: ['manual'], severities: ['unassessed', 'none', 'risk', 'high'], types: ['other', 'mailbox-access'], phases: ['initial'],
  questions: [{ id: 'confidentiality', options: ['none', 'limited', 'broad'] }], variables: ['incident', 'dpo'], defaults: { subject: {}, body: {} }, enabled: true, noticeHours: 72,
};
const api = (rows: BreachSummary[]): BreachApi => ({ list: vi.fn(async () => list(rows)), meta: vi.fn(async () => meta), settings: vi.fn(async () => []) });

const incident = (over: Partial<BreachIncident> = {}): BreachIncident => ({
  ...summary({ id: 'b1' }), rev: 1, createdAt: nowSecs, noticeHours: 72, delayReasonsRequired: false, overdue: false,
  suggestion: { score: 0, level: 'unassessed', complete: false, answered: 0, total: 8 },
  playbook: [{ id: 'revoke-sessions', done: false, by: '', at: 0 }],
  body: {
    incidentType: 'mailbox-access', description: 'd', reporter: 'root', subjectCategories: [], dataCategories: [], approxSubjects: null, approxRecords: null, consequences: '', measuresTaken: '',
    measuresProposed: '', answers: {}, assessment: null, affected: [{ address: 'a@example.test', accountId: 'acct-1' }], affectedDomains: [], timeline: [], evidence: [], notNotifiableReason: '',
    closeReason: '', delayReasons: '', authorityNotices: [], subjectNotices: [], escalatedAt: null, auto: null,
  }, ...over,
});

const mocks = vi.hoisted(() => ({ detail: undefined as unknown as BreachDetail, lockdown: vi.fn(), move: vi.fn(), confirm: vi.fn(), preview: vi.fn() }));
vi.mock('../../../api/admin/breaches.ts', async (orig) => {
  const real = await orig<typeof import('../../../api/admin/breaches.ts')>();
  return {
    ...real,
    loadBreach: vi.fn(async () => mocks.detail),
    lockdown: mocks.lockdown, moveBreach: mocks.move, confirmBreach: mocks.confirm, previewSubjectNotice: mocks.preview,
  };
});

afterEach(() => { cleanup(); vi.clearAllMocks(); });

describe('Admin breach register', () => {
  it('lists incidents with their notice countdown', async () => {
    renderInShell(() => <Breaches api={api([summary({ id: 'b1' })])} />);
    const row = await screen.findByTestId('breach-row-b1');
    expect(within(row).getByTestId('breach-countdown')).toHaveAttribute('data-tone', 'ok');
    expect(screen.getByTestId('breach-tile-open')).toBeTruthy();
  });

  it('shows the overdue banner and an alert countdown past the deadline', async () => {
    const overdue = summary({ id: 'b2', deadlineAt: nowSecs - 3600, clock: { phase: 'overdue', awareAt: 0, deadlineAt: nowSecs - 3600, remainingSecs: -3600, elapsedSecs: 0, notifiedAt: null } });
    renderInShell(() => <Breaches api={api([overdue])} />);
    expect(await screen.findByTestId('breach-overdue-banner')).toBeTruthy();
    expect(within(screen.getByTestId('breach-row-b2')).getByTestId('breach-countdown')).toHaveAttribute('data-tone', 'overdue');
  });

  it('marks a draft and starts its clock from the detail view', async () => {
    const draft = summary({ id: 'b1', draft: true, awareAt: null, deadlineAt: null, clock: { phase: 'not-started', awareAt: null, deadlineAt: null, remainingSecs: null, elapsedSecs: null, notifiedAt: null } });
    mocks.detail = { incident: incident({ draft: true, awareAt: null, deadlineAt: null }), trail: [], deliveries: [] };
    mocks.confirm.mockResolvedValue(mocks.detail.incident);
    renderInShell(() => <Breaches api={api([draft])} />);
    fireEvent.click(await screen.findByTestId('breach-open-b1'));
    fireEvent.click(await screen.findByTestId('breach-confirm'));
    await waitFor(() => expect(mocks.confirm).toHaveBeenCalledWith('b1'));
  });

  it('requires a written reason before closing and sends it with the move', async () => {
    mocks.detail = { incident: incident(), trail: [], deliveries: [] };
    mocks.move.mockResolvedValue(mocks.detail.incident);
    renderInShell(() => <Breaches api={api([summary({ id: 'b1' })])} />);
    fireEvent.click(await screen.findByTestId('breach-open-b1'));
    const target = await screen.findByTestId('breach-target');
    fireEvent.change(target, { target: { value: 'closed' } });
    const button = screen.getByTestId('breach-move');
    expect(button).toBeDisabled();
    fireEvent.input(screen.getByTestId('breach-reason'), { target: { value: 'Authority was notified on the first day.' } });
    expect(button).not.toBeDisabled();
    fireEvent.click(button);
    await waitFor(() => expect(mocks.move).toHaveBeenCalledWith('b1', 'closed', 'Authority was notified on the first day.'));
  });

  it('asks for reasons for delay when the deadline has passed', async () => {
    mocks.detail = { incident: incident({ delayReasonsRequired: true, overdue: true }), trail: [], deliveries: [] };
    renderInShell(() => <Breaches api={api([summary({ id: 'b1' })])} />);
    fireEvent.click(await screen.findByTestId('breach-open-b1'));
    expect(await screen.findByTestId('breach-delay-required')).toBeTruthy();
    expect(screen.getByTestId('breach-delay-reasons')).toBeTruthy();
  });

  it('previews a lockdown before it can be applied', async () => {
    mocks.detail = { incident: incident(), trail: [], deliveries: [] };
    mocks.lockdown.mockResolvedValue({ dryRun: true, action: 'revoke-sessions', results: [{ address: 'a@example.test', status: 'would-apply', sessions: 2 }] });
    renderInShell(() => <Breaches api={api([summary({ id: 'b1' })])} />);
    fireEvent.click(await screen.findByTestId('breach-open-b1'));
    expect(screen.queryByTestId('breach-lock-apply')).toBeNull();
    fireEvent.click(await screen.findByTestId('breach-lock-dry-sessions'));
    expect(await screen.findByTestId('breach-lock-apply')).toBeTruthy();
    expect(mocks.lockdown).toHaveBeenCalledWith('b1', { action: 'revoke-sessions', dryRun: true });
    mocks.lockdown.mockResolvedValue({ dryRun: false, action: 'revoke-sessions', results: [] });
    await waitFor(() => expect(screen.getByTestId('breach-lock-apply')).not.toBeDisabled());
    fireEvent.click(screen.getByTestId('breach-lock-apply'));
    await waitFor(() => expect(mocks.lockdown).toHaveBeenLastCalledWith('b1', { action: 'revoke-sessions', dryRun: false, confirm: true }));
  });

  it('keeps send disabled until the audience has been previewed', async () => {
    mocks.detail = { incident: incident(), trail: [], deliveries: [] };
    mocks.preview.mockResolvedValue({ dryRun: true, channels: ['inbox'], recipients: [{ address: 'a@example.test', accountId: 'acct-1', locale: 'en', channels: ['inbox'], skipped: [] }], preview: { subject: 'Subject', body: 'Body', locale: 'en', unknownVariables: { subject: [], body: [] } } });
    renderInShell(() => <Breaches api={api([summary({ id: 'b1' })])} />);
    fireEvent.click(await screen.findByTestId('breach-open-b1'));
    expect(await screen.findByTestId('breach-send')).toBeDisabled();
    fireEvent.click(screen.getByTestId('breach-preview'));
    expect(await screen.findByTestId('breach-plan')).toHaveTextContent('Subject');
    await waitFor(() => expect(screen.getByTestId('breach-send')).not.toBeDisabled());
  });
});
