import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { RethreadMaintenance, SearchIndexMaintenance } from './RethreadMaintenance.tsx';
import {
  MaintenanceApiError,
  type MaintenanceApi,
  type ReindexSummary,
  type RethreadSummary,
  type SearchIndexApi,
  type SearchIndexStatus,
} from '../../api/maintenance.ts';
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

const STATUS: SearchIndexStatus = {
  documents: 40,
  messages: 42,
  zeroAccessDocuments: 0,
  persistent: true,
  rebuilding: false,
  rebuildDone: 0,
  rebuildTotal: 0,
};

const REINDEXED: ReindexSummary = {
  accounts: 2,
  messages: 42,
  indexed: 41,
  removed: 3,
  zeroAccess: 1,
  failed: 0,
};

/** A search-index client for the re-thread tests, which do not look at that block. */
const IDLE_INDEX: SearchIndexApi = {
  status: async () => STATUS,
  reindexAll: async () => REINDEXED,
};

describe('admin re-thread mailbox — confirm-gated JWZ backfill', () => {
  it('populates the account picker and disables the run button until an account is picked', async () => {
    const { api: maintenance, rethread } = makeMaintenance(SUMMARY);
    render(() => <RethreadMaintenance api={makeAdminApi(USERS)} maintenance={maintenance} searchIndex={IDLE_INDEX} />);

    await waitFor(() => expect(screen.getByRole('option', { name: 'alice@example.com' })).toBeInTheDocument());
    // no account chosen yet → run button disabled, no dialog, no POST
    expect(screen.getByTestId('admin-rethread-run')).toBeDisabled();
    expect(screen.queryByTestId('admin-rethread-dialog')).not.toBeInTheDocument();
    expect(rethread).not.toHaveBeenCalled();
  });

  it('opening the confirm dialog does NOT POST; only confirm fires the request', async () => {
    const { api: maintenance, rethread } = makeMaintenance(SUMMARY);
    render(() => <RethreadMaintenance api={makeAdminApi(USERS)} maintenance={maintenance} searchIndex={IDLE_INDEX} />);

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
    render(() => <RethreadMaintenance api={makeAdminApi(USERS)} maintenance={maintenance} searchIndex={IDLE_INDEX} />);

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
    render(() => <RethreadMaintenance api={makeAdminApi(USERS)} maintenance={maintenance} searchIndex={IDLE_INDEX} />);

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
    render(() => <RethreadMaintenance api={makeAdminApi(USERS)} maintenance={maintenance} searchIndex={IDLE_INDEX} />);

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

describe('admin search index — status and rebuild', () => {
  it('shows how much is indexed and where, and does not rebuild until asked', async () => {
    const reindexAll = vi.fn(async () => REINDEXED);
    render(() => <SearchIndexMaintenance api={{ status: async () => STATUS, reindexAll }} />);

    await waitFor(() => expect(screen.getByTestId('admin-searchindex-count')).toBeInTheDocument());
    const count = screen.getByTestId('admin-searchindex-count').textContent ?? '';
    expect(count).toContain('40');
    expect(count).toContain('42');
    expect(screen.getByTestId('admin-searchindex-place')).toHaveTextContent(/on disk/i);
    expect(screen.queryByTestId('admin-searchindex-progress')).not.toBeInTheDocument();
    expect(reindexAll).not.toHaveBeenCalled();
  });

  it('says so when the index is kept in memory', async () => {
    const status = async (): Promise<SearchIndexStatus> => ({ ...STATUS, persistent: false });
    render(() => <SearchIndexMaintenance api={{ status, reindexAll: async () => REINDEXED }} />);

    await waitFor(() => expect(screen.getByTestId('admin-searchindex-place')).toBeInTheDocument());
    expect(screen.getByTestId('admin-searchindex-place')).toHaveTextContent(/in memory/i);
  });

  it('rebuilds on click, reports the result and reads the status again', async () => {
    const status = vi.fn(async () => STATUS);
    const reindexAll = vi.fn(async () => REINDEXED);
    render(() => <SearchIndexMaintenance api={{ status, reindexAll }} />);

    await waitFor(() => expect(screen.getByTestId('admin-searchindex-count')).toBeInTheDocument());
    expect(status).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByTestId('admin-searchindex-run'));

    await waitFor(() => expect(screen.getByTestId('admin-searchindex-summary')).toBeInTheDocument());
    expect(reindexAll).toHaveBeenCalledTimes(1);
    const summary = screen.getByTestId('admin-searchindex-summary').textContent ?? '';
    expect(summary).toContain('41'); // indexed
    expect(summary).toContain('3'); // removed
    await waitFor(() => expect(status).toHaveBeenCalledTimes(2));
    expect(screen.queryByTestId('admin-searchindex-error')).not.toBeInTheDocument();
  });

  it('shows progress and disables the button while the server is rebuilding', async () => {
    const status = async (): Promise<SearchIndexStatus> => ({
      ...STATUS,
      rebuilding: true,
      rebuildDone: 10,
      rebuildTotal: 42,
    });
    const reindexAll = vi.fn(async () => REINDEXED);
    render(() => <SearchIndexMaintenance api={{ status, reindexAll }} />);

    await waitFor(() => expect(screen.getByTestId('admin-searchindex-progress')).toBeInTheDocument());
    const progress = screen.getByTestId('admin-searchindex-progress').textContent ?? '';
    expect(progress).toContain('10');
    expect(progress).toContain('42');
    expect(screen.getByTestId('admin-searchindex-run')).toBeDisabled();
    fireEvent.click(screen.getByTestId('admin-searchindex-run'));
    expect(reindexAll).not.toHaveBeenCalled();
  });

  it('tells a refused concurrent rebuild apart from a failed one', async () => {
    const busy = vi.fn(async (): Promise<ReindexSummary> => {
      throw new MaintenanceApiError(409, 'busy');
    });
    const first = render(() => (
      <SearchIndexMaintenance api={{ status: async () => STATUS, reindexAll: busy }} />
    ));
    await waitFor(() => expect(screen.getByTestId('admin-searchindex-count')).toBeInTheDocument());
    fireEvent.click(screen.getByTestId('admin-searchindex-run'));
    await waitFor(() => expect(screen.getByTestId('admin-searchindex-error')).toBeInTheDocument());
    expect(screen.getByTestId('admin-searchindex-error')).toHaveTextContent(/already running/i);
    expect(screen.queryByTestId('admin-searchindex-summary')).not.toBeInTheDocument();
    first.unmount();

    const broken = vi.fn(async (): Promise<ReindexSummary> => {
      throw new MaintenanceApiError(500, 'boom');
    });
    render(() => <SearchIndexMaintenance api={{ status: async () => STATUS, reindexAll: broken }} />);
    await waitFor(() => expect(screen.getByTestId('admin-searchindex-count')).toBeInTheDocument());
    fireEvent.click(screen.getByTestId('admin-searchindex-run'));
    await waitFor(() => expect(screen.getByTestId('admin-searchindex-error')).toBeInTheDocument());
    expect(screen.getByTestId('admin-searchindex-error')).toHaveTextContent(/failed/i);
    expect(screen.getByTestId('admin-searchindex-error')).not.toHaveTextContent(/already running/i);
  });

  it('offers no rebuild in proxy mode, where the server has no index', async () => {
    const status = async (): Promise<SearchIndexStatus> => {
      throw new MaintenanceApiError(501, 'engine mode only');
    };
    render(() => <SearchIndexMaintenance api={{ status, reindexAll: async () => REINDEXED }} />);

    await waitFor(() =>
      expect(screen.getByTestId('admin-searchindex-unavailable')).toBeInTheDocument(),
    );
    expect(screen.queryByTestId('admin-searchindex-run')).not.toBeInTheDocument();
    expect(screen.queryByTestId('admin-searchindex-status-error')).not.toBeInTheDocument();
  });

  it('reports a status it could not read, and still offers the rebuild', async () => {
    const status = async (): Promise<SearchIndexStatus> => {
      throw new MaintenanceApiError(500, 'boom');
    };
    render(() => <SearchIndexMaintenance api={{ status, reindexAll: async () => REINDEXED }} />);

    await waitFor(() =>
      expect(screen.getByTestId('admin-searchindex-status-error')).toBeInTheDocument(),
    );
    expect(screen.queryByTestId('admin-searchindex-count')).not.toBeInTheDocument();
    expect(screen.getByTestId('admin-searchindex-run')).toBeEnabled();
  });
});
