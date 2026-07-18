import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { RethreadMaintenance } from './RethreadMaintenance.tsx';
import type { MaintenanceApi, RethreadSummary } from '../../api/maintenance.ts';
import type { AdminApi, UserSummary } from '../../state/slices/admin.ts';

// Only `listUsers` is exercised by the account picker; the rest are inert stubs.
function makeAdminApi(users: UserSummary[]): AdminApi {
  const api: Partial<AdminApi> = { listUsers: async () => users };
  return api as AdminApi;
}

function makeMaintenance(
  summary: RethreadSummary,
  opts: { reject?: boolean } = {},
): { api: MaintenanceApi; rethread: ReturnType<typeof vi.fn> } {
  const rethread = vi.fn(async (_accountId: string): Promise<RethreadSummary> => {
    if (opts.reject) throw new Error('boom');
    return summary;
  });
  return { api: { rethread }, rethread };
}

const USERS: UserSummary[] = [
  {
    accountId: 'acc-1',
    username: 'alice',
    domain: 'example.com',
    quota: null,
    flags: { zeroAccess: false, forcePasswordChange: false, remoteCacheWipe: false, disabled: false },
  },
];

const SUMMARY: RethreadSummary = { accounts: 1, messages: 42, threads: 12, reassigned: 7 };

describe('admin re-thread mailbox — confirm-gated JWZ backfill', () => {
  it('populates the account picker and disables the run button until an account is picked', async () => {
    const { api: maintenance, rethread } = makeMaintenance(SUMMARY);
    render(() => <RethreadMaintenance api={makeAdminApi(USERS)} maintenance={maintenance} />);

    await waitFor(() => expect(screen.getByRole('option', { name: 'alice@example.com' })).toBeInTheDocument());
    // no account chosen yet → run button disabled, no dialog, no POST
    expect(screen.getByTestId('admin-rethread-run')).toBeDisabled();
    expect(screen.queryByTestId('admin-rethread-dialog')).not.toBeInTheDocument();
    expect(rethread).not.toHaveBeenCalled();
  });

  it('opening the confirm dialog does NOT POST; only confirm fires the request', async () => {
    const { api: maintenance, rethread } = makeMaintenance(SUMMARY);
    render(() => <RethreadMaintenance api={makeAdminApi(USERS)} maintenance={maintenance} />);

    await waitFor(() => expect(screen.getByTestId('admin-rethread-account')).toBeInTheDocument());
    fireEvent.change(screen.getByTestId('admin-rethread-account'), { target: { value: 'acc-1' } });

    // pressing "Re-thread mailbox" opens the confirmation dialog — but must NOT POST
    fireEvent.click(screen.getByTestId('admin-rethread-run'));
    await waitFor(() => expect(screen.getByTestId('admin-rethread-dialog')).toBeInTheDocument());
    const dialog = screen.getByTestId('admin-rethread-dialog');
    expect(dialog).toHaveAttribute('role', 'dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    // the warning is announced (role="alert")
    expect(screen.getByText(/re-keys conversation grouping/i)).toHaveAttribute('role', 'alert');
    expect(rethread).not.toHaveBeenCalled();
  });

  it('cancelling the dialog closes it without POSTing', async () => {
    const { api: maintenance, rethread } = makeMaintenance(SUMMARY);
    render(() => <RethreadMaintenance api={makeAdminApi(USERS)} maintenance={maintenance} />);

    await waitFor(() => expect(screen.getByTestId('admin-rethread-account')).toBeInTheDocument());
    fireEvent.change(screen.getByTestId('admin-rethread-account'), { target: { value: 'acc-1' } });
    fireEvent.click(screen.getByTestId('admin-rethread-run'));
    await waitFor(() => expect(screen.getByTestId('admin-rethread-dialog')).toBeInTheDocument());

    fireEvent.click(screen.getByTestId('admin-rethread-cancel'));
    await waitFor(() => expect(screen.queryByTestId('admin-rethread-dialog')).not.toBeInTheDocument());
    expect(rethread).not.toHaveBeenCalled();
  });

  it('confirming POSTs the selected accountId and renders the returned summary', async () => {
    const { api: maintenance, rethread } = makeMaintenance(SUMMARY);
    render(() => <RethreadMaintenance api={makeAdminApi(USERS)} maintenance={maintenance} />);

    await waitFor(() => expect(screen.getByTestId('admin-rethread-account')).toBeInTheDocument());
    fireEvent.change(screen.getByTestId('admin-rethread-account'), { target: { value: 'acc-1' } });
    fireEvent.click(screen.getByTestId('admin-rethread-run'));
    await waitFor(() => expect(screen.getByTestId('admin-rethread-dialog')).toBeInTheDocument());
    fireEvent.click(screen.getByTestId('admin-rethread-confirm'));

    await waitFor(() => expect(rethread).toHaveBeenCalledWith('acc-1'));
    // the dialog closes and the summary (with the reassigned count) is shown
    await waitFor(() => expect(screen.getByTestId('admin-rethread-summary')).toBeInTheDocument());
    expect(screen.queryByTestId('admin-rethread-dialog')).not.toBeInTheDocument();
    const summary = screen.getByTestId('admin-rethread-summary').textContent ?? '';
    expect(summary).toContain('7'); // reassigned
    expect(summary).toContain('42'); // messages
  });

  it('shows an honest error state when the request fails', async () => {
    const { api: maintenance, rethread } = makeMaintenance(SUMMARY, { reject: true });
    render(() => <RethreadMaintenance api={makeAdminApi(USERS)} maintenance={maintenance} />);

    await waitFor(() => expect(screen.getByTestId('admin-rethread-account')).toBeInTheDocument());
    fireEvent.change(screen.getByTestId('admin-rethread-account'), { target: { value: 'acc-1' } });
    fireEvent.click(screen.getByTestId('admin-rethread-run'));
    await waitFor(() => expect(screen.getByTestId('admin-rethread-dialog')).toBeInTheDocument());
    fireEvent.click(screen.getByTestId('admin-rethread-confirm'));

    await waitFor(() => expect(rethread).toHaveBeenCalledWith('acc-1'));
    await waitFor(() => expect(screen.getByTestId('admin-rethread-error')).toBeInTheDocument());
    expect(screen.getByTestId('admin-rethread-error')).toHaveAttribute('role', 'alert');
    expect(screen.queryByTestId('admin-rethread-summary')).not.toBeInTheDocument();
  });
});
