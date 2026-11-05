import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { AdminDataRequests } from './index.tsx';
import * as api from '../../../api/admin/dataRequests.ts';
import type { AdminDataRequest, DataRequestList } from '../../../api/admin/dataRequests.ts';

vi.mock('../../../api/admin/dataRequests.ts', async importOriginal => ({
  ...await importOriginal<typeof import('../../../api/admin/dataRequests.ts')>(),
  listDataRequests: vi.fn(), loadDataRequest: vi.fn(), approveDataRequest: vi.fn(), rejectDataRequest: vi.fn(),
  extendDataRequest: vi.fn(), createDataRequestOnBehalf: vi.fn(), exportDataRequestLog: vi.fn(), runDataRequest: vi.fn(),
}));

const NOW = Math.floor(Date.now() / 1000);
const request = (over: Partial<AdminDataRequest> = {}): AdminDataRequest => ({
  id: 'r1', kind: 'erasure', status: 'blocked', createdAt: NOW - 86_400, updatedAt: NOW, dueAt: NOW + 5 * 86_400,
  details: '', requestedBy: 'self', needsConfirmation: false, canCancel: true, graceUntil: null, decisionReason: null,
  extensionDays: 0, extensionReason: null, export: null, downloadAvailable: false, downloads: 0, completion: null, dueState: 'dueSoon',
  address: 'alice@example.test', domain: 'example.test', assignee: null, verifiedAt: null, verifiedBy: null, approvedAt: NOW, approvedBy: 'root',
  originalDueAt: NOW + 5 * 86_400, answeredAt: null, error: null, legalHold: { active: true, source: 'policy', reason: 'listed in the legal hold setting' },
  requiresVerification: false, approvalMode: 'admin', ...over,
});
const list = (requests: AdminDataRequest[]): DataRequestList => ({
  requests, matched: requests.length, kinds: [],
  stats: { total: 4, open: 2, overdue: 1, dueSoon: 1, answered: 2, onTime: 1, avgAnswerHours: 12.5, byStatus: { blocked: 1, completed: 2 }, byKind: { erasure: 1 }, created30d: [{ t: NOW - 86_400, count: 3 }, { t: NOW, count: 1 }] },
});

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(api.listDataRequests).mockResolvedValue(list([request()]));
  vi.mocked(api.loadDataRequest).mockResolvedValue({ request: request(), trail: [{ id: 't1', ts: NOW, actor: 'system', kind: 'blocked-by-hold', detail: {} }] });
});

describe('Admin > Data requests', () => {
  it('shows the queue with a countdown, the stats and the legal hold on a blocked erasure', async () => {
    render(() => <AdminDataRequests />);
    const row = await screen.findByTestId('dr-row-r1');
    expect(row).toHaveTextContent('alice@example.test');
    expect(row).toHaveTextContent('5 day(s) left');
    expect(row).toHaveTextContent('Legal hold');
    expect(screen.getByTestId('dr-tile-open')).toHaveTextContent('2');
    expect(screen.getByTestId('dr-tile-overdue')).toHaveTextContent('1');
    expect(screen.getByTestId('dr-tile-ontime')).toHaveTextContent('50%');
    expect(screen.getByTestId('dr-chart-created')).toBeInTheDocument();
  });

  it('opens a request, shows why the erasure is gated, offers no approval while the hold applies, and lists the trail', async () => {
    render(() => <AdminDataRequests />);
    fireEvent.click(await screen.findByRole('button', { name: 'Open' }));
    const hold = await screen.findByTestId('dr-hold');
    expect(hold).toHaveTextContent('from the legal hold setting');
    expect(screen.getByText(/blocked by a legal hold/)).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Approve' })).toBeNull();
    expect(screen.getByTestId('dr-trail')).toHaveTextContent('Blocked by a legal hold');
  });

  it('needs a reason to reject or extend and sends it', async () => {
    vi.mocked(api.listDataRequests).mockResolvedValue(list([request({ status: 'awaiting_approval', legalHold: { active: false } })]));
    vi.mocked(api.loadDataRequest).mockResolvedValue({ request: request({ status: 'awaiting_approval', legalHold: { active: false } }), trail: [] });
    vi.mocked(api.rejectDataRequest).mockResolvedValue(request({ status: 'rejected', legalHold: { active: false } }));
    vi.mocked(api.extendDataRequest).mockResolvedValue(request({ legalHold: { active: false } }));
    render(() => <AdminDataRequests />);
    fireEvent.click(await screen.findByRole('button', { name: 'Open' }));
    const reject = await screen.findByRole('button', { name: 'Reject' });
    const extend = screen.getByRole('button', { name: 'Extend due date' });
    expect(reject).toBeDisabled();
    expect(extend).toBeDisabled();
    fireEvent.input(screen.getByTestId('dr-note'), { target: { value: 'complex request' } });
    fireEvent.click(extend);
    await waitFor(() => expect(api.extendDataRequest).toHaveBeenCalledWith('r1', 30, 'complex request'));
    await waitFor(() => expect(screen.getByTestId('dr-note')).toHaveValue(''));
    fireEvent.input(screen.getByTestId('dr-note'), { target: { value: 'not applicable' } });
    await waitFor(() => expect(screen.getByRole('button', { name: 'Reject' })).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: 'Reject' }));
    await waitFor(() => expect(api.rejectDataRequest).toHaveBeenCalledWith('r1', 'not applicable'));
  });

  it('applies filters and files a request on behalf of a person', async () => {
    vi.mocked(api.createDataRequestOnBehalf).mockResolvedValue(request({ status: 'awaiting_approval' }));
    render(() => <AdminDataRequests />);
    await screen.findByTestId('dr-row-r1');
    fireEvent.change(screen.getByLabelText('Due'), { target: { value: 'overdue' } });
    await waitFor(() => expect(api.listDataRequests).toHaveBeenLastCalledWith(expect.objectContaining({ due: 'overdue' })));
    fireEvent.input(screen.getAllByLabelText('Person')[0]!, { target: { value: 'bob@example.test' } });
    fireEvent.click(screen.getByRole('button', { name: 'File request' }));
    await waitFor(() => expect(api.createDataRequestOnBehalf).toHaveBeenCalledWith(expect.objectContaining({ address: 'bob@example.test', kind: 'access' })));
  });
});
