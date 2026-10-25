import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { expectNoAxeViolations } from '../shell/uxTestkit.ts';
import { stripIsolates } from '../../../i18n';
import { AllowlistPanel } from './Allowlist.tsx';
import {
  createPluginsSlice,
  EMPTY_ALLOWLIST,
  HIGH_POWER_CAPABILITIES,
  type AllowlistView,
} from '../../../state/slices/plugins.ts';
import { mockPluginsApi, type MockPluginsApi } from './testkit.ts';

const DIGEST_A = 'a'.repeat(64);
const DIGEST_B = 'b'.repeat(64);

const mockApi = (initial: AllowlistView): MockPluginsApi => mockPluginsApi([], initial);

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
  const PENDING: AllowlistView = { present: [{ pluginId: 'acme-filter', computedDigest: DIGEST_A, firstParty: false, approved: false }], pins: [] };
  const row = (): HTMLElement => document.querySelector(`[data-row-key="acme-filter:${DIGEST_A}"]`) as HTMLElement;

  it('shows the computed digest in full and the high-power + unsigned notes', async () => {
    render(() => <AllowlistPanel api={mockApi(PENDING)} />);
    await waitFor(() => expect(screen.getByTestId('allowlist-digest')).toHaveTextContent(DIGEST_A));
    expect(screen.getByTestId('allowlist-highpower-note')).toHaveTextContent(HIGH_POWER_CAPABILITIES[0]!);
    expect(screen.getByTestId('allowlist-unsigned-note')).toBeInTheDocument();
    expect(within(row()).getByTestId('allowlist-status')).toHaveTextContent('Not approved');
  });

  it('a first-party id offers no approve action and explains precedence', async () => {
    render(() => <AllowlistPanel api={mockApi({ present: [{ pluginId: 'spam-rspamd', computedDigest: DIGEST_B, firstParty: true, approved: false }], pins: [] })} />);
    await waitFor(() => expect(screen.getByTestId('allowlist-firstparty-note')).toBeInTheDocument());
    expect(screen.queryByRole('button', { name: /Approve digest/ })).toBeNull();
  });

  it('approving asks first: the dialog shows the digest, wants the id typed, and only confirm calls the API', async () => {
    const api = mockApi(PENDING);
    render(() => <AllowlistPanel api={api} />);
    await waitFor(() => expect(row()).not.toBeNull());
    fireEvent.click(within(row()).getByRole('button', { name: /Approve digest/ }));
    const dialog = await screen.findByRole('alertdialog');
    expect(within(dialog).getByTestId('allowlist-dialog-digest')).toHaveTextContent(DIGEST_A);
    const confirm = within(dialog).getByRole('button', { name: 'Approve digest' });
    expect(confirm).toBeDisabled();
    expect(api.calls).toEqual([]);
    fireEvent.input(within(dialog).getByRole('textbox'), { target: { value: 'acme-filter' } });
    fireEvent.click(confirm);
    await waitFor(() => expect(api.calls).toEqual([`approveDigest:acme-filter:${DIGEST_A}`]));
    await waitFor(() => expect(within(row()).getByTestId('allowlist-status')).toHaveTextContent('Approved'));
    await waitFor(() => expect(screen.getByTestId('allowlist-notice')).toBeInTheDocument());
  });

  it('cancelling does not call the API', async () => {
    const api = mockApi(PENDING);
    render(() => <AllowlistPanel api={api} />);
    await waitFor(() => expect(row()).not.toBeNull());
    fireEvent.click(within(row()).getByRole('button', { name: /Approve digest/ }));
    fireEvent.click(within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Cancel' }));
    await waitFor(() => expect(screen.queryByRole('alertdialog')).toBeNull());
    expect(api.calls).toEqual([]);
  });

  it('an approved component offers a confirm-gated revoke', async () => {
    const api = mockApi({ present: [{ ...PENDING.present[0]!, approved: true }], pins: [] });
    render(() => <AllowlistPanel api={api} />);
    await waitFor(() => expect(row()).not.toBeNull());
    fireEvent.click(within(row()).getByRole('button', { name: /^Revoke/ }));
    fireEvent.click(within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Revoke pin' }));
    await waitFor(() => expect(api.calls).toEqual([`revokeDigest:acme-filter:${DIGEST_A}`]));
  });

  it('renders stored pins for oversight, marking a revoked one', async () => {
    const api = mockApi({
      present: [],
      pins: [
        { pluginId: 'acme-filter', digestHex: DIGEST_A, name: 'Acme', version: '1.0', source: null, note: null, approvedBy: 'root', approvedAt: '2026-10-01T10:00:00Z', revoked: false },
        { pluginId: 'old-filter', digestHex: DIGEST_B, name: null, version: null, source: null, note: null, approvedBy: 'root', approvedAt: '2026-09-01T10:00:00Z', revoked: true },
      ],
    });
    const { container } = render(() => <AllowlistPanel api={api} />);
    await waitFor(() => expect(screen.getAllByTestId('allowlist-pin-state')).toHaveLength(2));
    expect(screen.getAllByTestId('allowlist-pin-state').map((e) => e.textContent)).toEqual(['Approved', 'Revoked']);
    expect(stripIsolates(container.textContent ?? '')).toMatch(/Approved by root on/);
    await expectNoAxeViolations(container);
  });
});
