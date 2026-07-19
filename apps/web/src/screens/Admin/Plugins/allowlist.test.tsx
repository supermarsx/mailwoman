import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { AllowlistPanel } from './Allowlist.tsx';
import {
  createPluginsSlice,
  EMPTY_ALLOWLIST,
  HIGH_POWER_CAPABILITIES,
  type AllowlistView,
  type PluginInfo,
  type PluginsApi,
} from '../../../state/slices/plugins.ts';

const DIGEST_A = 'a'.repeat(64);
const DIGEST_B = 'b'.repeat(64);

/** A PluginsApi over a mutable allowlist view; records every allowlist mutation. */
function mockApi(initial: AllowlistView): PluginsApi & { calls: string[] } {
  const calls: string[] = [];
  let view = initial;
  return {
    calls,
    async list(): Promise<PluginInfo[]> {
      return [];
    },
    async approve() {},
    async enable() {},
    async disable() {},
    async grant() {},
    async setAllowUnsigned() {},
    async listAllowlist() {
      return view;
    },
    async approveDigest(pluginId, digestHex) {
      calls.push(`approveDigest:${pluginId}:${digestHex}`);
      view = {
        present: view.present.map((p) =>
          p.pluginId === pluginId && p.computedDigest === digestHex ? { ...p, approved: true } : p,
        ),
        pins: view.pins,
      };
    },
    async revokeDigest(pluginId, digestHex) {
      calls.push(`revokeDigest:${pluginId}:${digestHex}`);
      view = {
        present: view.present.map((p) => (p.pluginId === pluginId ? { ...p, approved: false } : p)),
        pins: view.pins,
      };
    },
    async uninstall(id) {
      calls.push(`uninstall:${id}`);
    },
  };
}

describe('Admin → Plugins allowlist: slice methods', () => {
  it('approveDigest / revokeDigest / uninstall call the API and reload the allowlist', async () => {
    const api = mockApi({
      present: [{ pluginId: 'acme-filter', computedDigest: DIGEST_A, firstParty: false, approved: false }],
      pins: [],
    });
    const slice = createPluginsSlice(api);

    await slice.approveDigest('acme-filter', DIGEST_A);
    expect(api.calls).toContain(`approveDigest:acme-filter:${DIGEST_A}`);
    // reload ran: the present row now reads approved
    expect(slice.allowlist().present[0]?.approved).toBe(true);

    await slice.revokeDigest('acme-filter', DIGEST_A);
    expect(api.calls).toContain(`revokeDigest:acme-filter:${DIGEST_A}`);
    expect(slice.allowlist().present[0]?.approved).toBe(false);

    await slice.uninstall('acme-filter');
    expect(api.calls).toContain('uninstall:acme-filter');
  });

  it('starts from an empty allowlist view', () => {
    const api = mockApi(EMPTY_ALLOWLIST);
    const slice = createPluginsSlice(api);
    expect(slice.allowlist()).toEqual(EMPTY_ALLOWLIST);
  });
});

describe('Admin → Plugins allowlist: panel', () => {
  it('shows the computed digest and, for a third-party component, the high-power + unsigned notes', async () => {
    const api = mockApi({
      present: [{ pluginId: 'acme-filter', computedDigest: DIGEST_A, firstParty: false, approved: false }],
      pins: [],
    });
    render(() => <AllowlistPanel api={api} />);

    await waitFor(() => expect(screen.getByTestId('allowlist-present-card')).toBeInTheDocument());
    // the exact digest the admin is trusting is shown in full (not truncated)
    expect(screen.getByTestId('allowlist-digest')).toHaveTextContent(DIGEST_A);
    // high-power caps are surfaced as not-grantable-to-third-party (names the cap)
    expect(screen.getByTestId('allowlist-highpower-note')).toHaveTextContent(HIGH_POWER_CAPABILITIES[0]!);
    // unsigned-but-pinned is a neutral informational note, present for third-party
    expect(screen.getByTestId('allowlist-unsigned-note')).toBeInTheDocument();
  });

  it('a first-party id offers no approve action and explains precedence', async () => {
    const api = mockApi({
      present: [{ pluginId: 'nextcloud', computedDigest: DIGEST_A, firstParty: true, approved: false }],
      pins: [],
    });
    render(() => <AllowlistPanel api={api} />);

    await waitFor(() => expect(screen.getByTestId('allowlist-present-card')).toBeInTheDocument());
    expect(screen.getByTestId('allowlist-status')).toHaveTextContent('First-party');
    expect(screen.getByTestId('allowlist-firstparty-note')).toBeInTheDocument();
    // no approve button for a first-party id
    expect(screen.queryByRole('button', { name: /Approve digest for/ })).not.toBeInTheDocument();
  });

  it('approving is confirm-gated: the dialog shows the digest and only confirm calls the API', async () => {
    const api = mockApi({
      present: [{ pluginId: 'acme-filter', computedDigest: DIGEST_A, firstParty: false, approved: false }],
      pins: [],
    });
    render(() => <AllowlistPanel api={api} />);

    await waitFor(() => expect(screen.getByTestId('allowlist-present-card')).toBeInTheDocument());

    // pressing Approve opens the confirmation — but must NOT call the API yet
    fireEvent.click(screen.getByRole('button', { name: 'Approve digest for acme-filter' }));
    await waitFor(() => expect(screen.getByTestId('allowlist-dialog')).toBeInTheDocument());
    const dialog = screen.getByTestId('allowlist-dialog');
    expect(dialog).toHaveAttribute('role', 'dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    // the exact digest being trusted is shown in the dialog
    expect(screen.getByTestId('allowlist-dialog-digest')).toHaveTextContent(DIGEST_A);
    expect(api.calls).not.toContain(`approveDigest:acme-filter:${DIGEST_A}`);

    // confirm fires the request
    fireEvent.click(screen.getByTestId('allowlist-confirm'));
    await waitFor(() => expect(api.calls).toContain(`approveDigest:acme-filter:${DIGEST_A}`));
    // dialog closes on success
    await waitFor(() => expect(screen.queryByTestId('allowlist-dialog')).not.toBeInTheDocument());
  });

  it('cancelling the confirm dialog does not call the API', async () => {
    const api = mockApi({
      present: [{ pluginId: 'acme-filter', computedDigest: DIGEST_A, firstParty: false, approved: false }],
      pins: [],
    });
    render(() => <AllowlistPanel api={api} />);

    await waitFor(() => expect(screen.getByTestId('allowlist-present-card')).toBeInTheDocument());
    fireEvent.click(screen.getByRole('button', { name: 'Approve digest for acme-filter' }));
    await waitFor(() => expect(screen.getByTestId('allowlist-dialog')).toBeInTheDocument());

    fireEvent.click(screen.getByTestId('allowlist-cancel'));
    await waitFor(() => expect(screen.queryByTestId('allowlist-dialog')).not.toBeInTheDocument());
    expect(api.calls).toEqual([]);
  });

  it('an approved component offers a confirm-gated revoke', async () => {
    const api = mockApi({
      present: [{ pluginId: 'acme-filter', computedDigest: DIGEST_A, firstParty: false, approved: true }],
      pins: [
        {
          pluginId: 'acme-filter',
          digestHex: DIGEST_A,
          name: 'Acme filter',
          version: '1.2.0',
          source: null,
          note: null,
          approvedBy: 'admin',
          approvedAt: '2026-07-19T00:00:00Z',
          revoked: false,
        },
      ],
    });
    render(() => <AllowlistPanel api={api} />);

    await waitFor(() => expect(screen.getByTestId('allowlist-present-card')).toBeInTheDocument());
    fireEvent.click(screen.getByRole('button', { name: 'Revoke pin for acme-filter' }));
    await waitFor(() => expect(screen.getByTestId('allowlist-dialog')).toBeInTheDocument());
    fireEvent.click(screen.getByTestId('allowlist-confirm'));
    await waitFor(() => expect(api.calls).toContain(`revokeDigest:acme-filter:${DIGEST_A}`));
  });

  it('renders stored pins for oversight, marking a revoked one', async () => {
    const api = mockApi({
      present: [],
      pins: [
        {
          pluginId: 'old-plugin',
          digestHex: DIGEST_B,
          name: null,
          version: null,
          source: null,
          note: null,
          approvedBy: 'admin',
          approvedAt: '2026-01-01T00:00:00Z',
          revoked: true,
        },
      ],
    });
    render(() => <AllowlistPanel api={api} />);

    await waitFor(() => expect(screen.getByTestId('allowlist-pin')).toBeInTheDocument());
    expect(screen.getByTestId('allowlist-pin')).toHaveTextContent(DIGEST_B);
    expect(screen.getByTestId('allowlist-pin')).toHaveTextContent('Revoked');
  });
});
