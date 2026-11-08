import { fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { beforeEach, expect, it, vi } from 'vitest';
import { AccountActivity, ACTIVITY_ACTIONS } from './AccountActivity.tsx';
import { apiFetch } from '../../api/http.ts';
import { loadCatalog, t } from '../../i18n';
vi.mock('../../api/http.ts', () => ({ apiFetch: vi.fn() }));
vi.mock('../../components/TimeHelpers.tsx', () => ({ messageTimeZone: () => 'America/New_York' }));
beforeEach(async () => { vi.mocked(apiFetch).mockReset(); await loadCatalog('security'); await loadCatalog('account-activity'); });
const entry = { id: 'audit-1', timestamp: '2026-11-08T22:37:48.689600982+00:00', action: 'linked-credential-renewed', actor: 'system', target: 'account:owner', detail: { link: 'first-link' } };
it('renders readable timezone-aware events in supplied order without merging repeated renewals and retains exact details', async () => {
  vi.mocked(apiFetch).mockResolvedValue(new Response(JSON.stringify({ events: [entry, { ...entry, id: 'audit-2', detail: { link: 'different-link' } }, { ...entry, id: 'audit-3', action: 'future-action' }] })));
  render(() => <AccountActivity />);
  const list = await screen.findByRole('list', { name: 'Account activity' });
  const rows = within(list).getAllByRole('listitem'); expect(rows).toHaveLength(3);
  expect(within(rows[0]!).getByText('Linked mailbox sign-in renewed')).toBeVisible();
  expect(rows[0]!.querySelector('time')).toHaveTextContent('5:37:48 PM');
  expect(rows[0]!.querySelector('time')).toHaveAttribute('datetime', entry.timestamp);
  expect(within(rows[2]!).getByText('Account event')).toBeVisible();
  const details = rows[1]!.querySelector('details')!; expect(details).not.toHaveAttribute('open');
  fireEvent.click(within(rows[1]!).getByText('Event details'));
  expect(within(rows[1]!).getByText('different-link', { exact: false })).toBeVisible();
  expect(within(rows[1]!).getByText('audit-2')).toBeVisible();
  expect(within(rows[1]!).getByText('Automatic system activity')).toBeVisible();
  expect(within(rows[1]!).getByText(entry.timestamp)).toBeVisible();
  expect(apiFetch).toHaveBeenCalledWith(expect.stringContaining('/api/account/security/activity'), { credentials: 'same-origin' });
});
it('gives every known event a human label', () => { for (const action of ACTIVITY_ACTIONS) expect(t(`account-activity-${action}`)).not.toBe(`account-activity-${action}`); });
it('keeps malformed timestamps readable and preserves the original value in details', async () => {
  vi.mocked(apiFetch).mockResolvedValue(new Response(JSON.stringify({ events: [{ ...entry, timestamp: 'invalid-timestamp' }] })));
  render(() => <AccountActivity />); expect(await screen.findByText('Time unavailable')).toBeVisible();
  expect(screen.getByText('invalid-timestamp')).not.toBeVisible();
  fireEvent.click(screen.getByText('Event details')); expect(screen.getByText('invalid-timestamp')).toBeVisible();
});
it('shows a local loading state and a quiet empty state', async () => {
  let resolve!: (response: Response) => void; vi.mocked(apiFetch).mockReturnValue(new Promise(r => { resolve = r; }));
  render(() => <AccountActivity />); expect(screen.getByRole('status')).toHaveTextContent('Loading account activity');
  resolve(new Response(JSON.stringify({ events: [] })));
  expect(await screen.findByText('No account activity yet.')).toBeVisible(); expect(screen.queryByRole('status')).not.toBeInTheDocument();
});
it('keeps request failure local and retries successfully', async () => {
  vi.mocked(apiFetch).mockResolvedValueOnce(new Response('', { status: 503 })).mockResolvedValueOnce(new Response(JSON.stringify({ events: [entry] })));
  render(() => <AccountActivity />); expect(await screen.findByRole('alert')).toHaveTextContent('Account activity could not be loaded.');
  fireEvent.click(screen.getByRole('button', { name: 'Retry' }));
  await waitFor(() => expect(screen.getByRole('list')).toBeVisible()); expect(apiFetch).toHaveBeenCalledTimes(2);
});
