import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { TwoFactorPolicy } from './TwoFactorPolicy.tsx';
import type { TwofaPolicyApi, TwofaPolicyInput, TwofaPolicyRow } from './twofaPolicy.ts';
import type { AdminApi, Domain } from '../../state/slices/admin.ts';

// Only `listDomains` feeds the add-form datalist; the rest are inert stubs.
function makeAdminApi(domains: Domain[]): AdminApi {
  const api: Partial<AdminApi> = { listDomains: async () => domains };
  return api as AdminApi;
}

/** A stateful mock policy client: `set` upserts into the backing rows so a
 *  subsequent `list` (the panel's refetch) reflects the change. */
function makePolicy(
  initial: TwofaPolicyRow[],
  opts: { rejectSet?: boolean } = {},
): { api: TwofaPolicyApi; set: ReturnType<typeof vi.fn>; rows: TwofaPolicyRow[] } {
  const rows = [...initial];
  const set = vi.fn(async (input: TwofaPolicyInput): Promise<void> => {
    if (opts.rejectSet) throw new Error('boom');
    const i = rows.findIndex((r) => r.scopeKind === input.scopeKind && r.scopeValue === input.scopeValue);
    const row: TwofaPolicyRow = { ...input };
    if (i >= 0) rows[i] = row;
    else rows.push(row);
  });
  const api: TwofaPolicyApi = { list: async () => [...rows], set };
  return { api, set, rows };
}

const DOMAINS: Domain[] = [
  { name: 'example.com', upstreamJson: '{}', allowlist: [], blocklist: [] },
  { name: 'example.org', upstreamJson: '{}', allowlist: [], blocklist: [] },
];

describe('admin require-two-factor policy (DQ2)', () => {
  it('reflects the loaded global + per-domain rows', async () => {
    const { api } = makePolicy([
      { scopeKind: 'global', scopeValue: '', require2fa: true },
      { scopeKind: 'domain', scopeValue: 'example.com', require2fa: true },
    ]);
    render(() => <TwoFactorPolicy policy={api} api={makeAdminApi(DOMAINS)} />);

    await waitFor(() => expect(screen.getByTestId('admin-2fa-global')).toBeChecked());
    await waitFor(() => expect(screen.getAllByTestId('admin-2fa-domain-row')).toHaveLength(1));
    expect(screen.getByText('example.com')).toBeInTheDocument();
  });

  it('defaults the global toggle to off when no policy is set', async () => {
    const { api } = makePolicy([]);
    render(() => <TwoFactorPolicy policy={api} api={makeAdminApi([])} />);
    await waitFor(() => expect(screen.getByTestId('admin-2fa-global')).not.toBeChecked());
    // No domain rules yet → the empty note shows.
    expect(screen.getByText('No per-domain requirements set.')).toBeInTheDocument();
  });

  it('toggling the global requirement POSTs a global upsert', async () => {
    const { api, set } = makePolicy([]);
    render(() => <TwoFactorPolicy policy={api} api={makeAdminApi([])} />);
    await waitFor(() => expect(screen.getByTestId('admin-2fa-global')).toBeInTheDocument());

    fireEvent.change(screen.getByTestId('admin-2fa-global'), { target: { checked: true } });
    await waitFor(() =>
      expect(set).toHaveBeenCalledWith({ scopeKind: 'global', scopeValue: '', require2fa: true }),
    );
    // The refetch reflects the new state.
    await waitFor(() => expect(screen.getByTestId('admin-2fa-global')).toBeChecked());
  });

  it('adds a per-domain rule (lower-cased) and shows it after refetch', async () => {
    const { api, set } = makePolicy([]);
    render(() => <TwoFactorPolicy policy={api} api={makeAdminApi(DOMAINS)} />);
    await waitFor(() => expect(screen.getByTestId('admin-2fa-add-domain')).toBeInTheDocument());

    fireEvent.input(screen.getByTestId('admin-2fa-add-domain'), { target: { value: 'Example.ORG' } });
    fireEvent.click(screen.getByTestId('admin-2fa-add-submit'));

    await waitFor(() =>
      expect(set).toHaveBeenCalledWith({ scopeKind: 'domain', scopeValue: 'example.org', require2fa: true }),
    );
    await waitFor(() => expect(screen.getByText('example.org')).toBeInTheDocument());
  });

  it('toggling an existing domain row upserts that domain', async () => {
    const { api, set } = makePolicy([
      { scopeKind: 'domain', scopeValue: 'example.com', require2fa: true },
    ]);
    render(() => <TwoFactorPolicy policy={api} api={makeAdminApi(DOMAINS)} />);
    await waitFor(() => expect(screen.getByLabelText('Require two-factor for example.com')).toBeChecked());

    fireEvent.change(screen.getByLabelText('Require two-factor for example.com'), { target: { checked: false } });
    await waitFor(() =>
      expect(set).toHaveBeenCalledWith({ scopeKind: 'domain', scopeValue: 'example.com', require2fa: false }),
    );
  });

  it('surfaces an honest error when a save fails', async () => {
    const { api } = makePolicy([], { rejectSet: true });
    render(() => <TwoFactorPolicy policy={api} api={makeAdminApi([])} />);
    await waitFor(() => expect(screen.getByTestId('admin-2fa-global')).toBeInTheDocument());

    fireEvent.change(screen.getByTestId('admin-2fa-global'), { target: { checked: true } });
    await waitFor(() =>
      expect(screen.getByText('Could not save the two-factor policy.')).toHaveAttribute('role', 'alert'),
    );
  });
});
