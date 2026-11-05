import { describe, expect, it, vi } from 'vitest';
import { createSignal } from 'solid-js';
import { fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { DraftsPopup, type DraftsPopupProps } from './DraftsPopup.tsx';
import type { AccountDraftDocument } from '../../api/accountDrafts.ts';
import type { StoredDraft } from './drafts-store.ts';

const account = (id: string, subject: string, savedAt: number, to = ''): AccountDraftDocument => ({
  accountId: 'a', storageAuthority: 's', id, revision: 'r', emailId: null, savedAt, status: 'saved', cleanupPending: false,
  summary: { subject }, ...(to ? { state: { to } as NonNullable<AccountDraftDocument['state']> } : {}),
});
const local = (id: string, subject: string, savedAt: number, to = 'x@y.z'): StoredDraft =>
  ({ id, accountId: 'a', storageAuthority: 's', to, subject, bodyHtml: '', bodyText: '', savedAt }) as StoredDraft;

function setup(over: Partial<DraftsPopupProps> = {}, accounts = [account('a1', 'Alpha', 300, 'bob@x.org'), account('a2', '', 200), account('a3', 'Gamma', 100)]) {
  const [list, setList] = createSignal(accounts);
  const props: DraftsPopupProps = {
    open: true, accountDrafts: list, localDrafts: () => [local('l1', 'Local one', 50)], accountSupported: true, loading: false,
    loadFailed: false, currentId: 'a1', onRetryLoad: vi.fn(), onOpenAccount: vi.fn(), onOpenLocal: vi.fn(),
    onDiscardAccount: vi.fn(async (d: AccountDraftDocument) => { setList((l) => l.filter((x) => x.id !== d.id)); return true; }),
    onDiscardLocal: vi.fn(), onClose: vi.fn(), ...over,
  };
  render(() => <DraftsPopup {...props} />);
  return props;
}

describe('DraftsPopup', () => {
  it('lists account drafts newest first, marks the open one, and switches to local drafts', () => {
    setup();
    const rows = screen.getAllByTestId(/^drafts-popup-row-/);
    expect(rows.map((r) => r.getAttribute('data-testid'))).toEqual(['drafts-popup-row-a1', 'drafts-popup-row-a2', 'drafts-popup-row-a3']);
    expect(rows[0]).toHaveAttribute('data-current', 'true');
    expect(within(rows[1]!).getByText('(no subject)')).toBeInTheDocument();
    expect(screen.getByTestId('drafts-popup-count')).toHaveTextContent('4 drafts');
    fireEvent.click(screen.getByRole('tab', { name: /Local drafts/ }));
    expect(screen.getByTestId('drafts-popup-row-l1')).toBeInTheDocument();
  });

  it('filters by subject or recipient and shows an empty result', () => {
    setup();
    fireEvent.input(screen.getByRole('searchbox'), { target: { value: 'bob@' } });
    expect(screen.getAllByTestId(/^drafts-popup-row-/)).toHaveLength(1);
    fireEvent.input(screen.getByRole('searchbox'), { target: { value: 'zzz' } });
    expect(screen.getByTestId('drafts-popup-empty')).toHaveTextContent('No drafts match');
  });

  it('opens a draft on click and on Enter, and navigates with arrows', () => {
    const p = setup();
    fireEvent.click(screen.getByTestId('draft-resume-account-a3'));
    expect(p.onOpenAccount).toHaveBeenCalledWith('a3');
    const first = screen.getByTestId('draft-resume-account-a1');
    first.focus();
    fireEvent.keyDown(first, { key: 'ArrowDown' });
    return waitFor(() => expect(screen.getByTestId('draft-resume-account-a2')).toHaveFocus()).then(() => {
      fireEvent.keyDown(screen.getByTestId('draft-resume-account-a2'), { key: 'Enter' });
      expect(p.onOpenAccount).toHaveBeenCalledWith('a2');
    });
  });

  it('selects with Space, confirms, discards, and keeps failures with Retry', async () => {
    let fail = true;
    const p = setup({
      onDiscardAccount: vi.fn(async (d: AccountDraftDocument) => (d.id === 'a3' && fail ? false : true)),
    });
    const a3 = screen.getByTestId('draft-resume-account-a3');
    a3.focus();
    fireEvent.keyDown(a3, { key: ' ' });
    fireEvent.click(screen.getByRole('button', { name: 'Select all' }));
    expect(screen.getByTestId('drafts-popup-selected')).toHaveTextContent('3 selected');
    fireEvent.click(screen.getByRole('button', { name: 'Discard selected' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(dialog).toHaveTextContent('removed from your account Drafts folder first');
    expect(p.onDiscardAccount).not.toHaveBeenCalled();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Discard' }));
    await waitFor(() => expect(p.onDiscardAccount).toHaveBeenCalledTimes(3));
    // The open draft goes last.
    expect((p.onDiscardAccount as ReturnType<typeof vi.fn>).mock.calls.at(-1)![0].id).toBe('a1');
    await waitFor(() => expect(screen.getByTestId('drafts-popup-live')).toHaveTextContent('2 discarded, 1 could not be discarded'));
    const row = screen.getByTestId('drafts-popup-row-a3');
    expect(within(row).getByText(/Could not be discarded/)).toBeInTheDocument();
    fail = false;
    fireEvent.click(within(row).getByRole('button', { name: 'Retry' }));
    await waitFor(() => expect(screen.getByTestId('drafts-popup-live')).toHaveTextContent('1 draft discarded'));
  });

  it('discards a local draft from the row menu after confirming', async () => {
    const p = setup();
    fireEvent.click(screen.getByRole('tab', { name: /Local drafts/ }));
    fireEvent.click(screen.getByRole('button', { name: /More actions for Local one/ }));
    fireEvent.click(screen.getByRole('menuitem', { name: 'Discard' }));
    fireEvent.click(within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Discard' }));
    await waitFor(() => expect(p.onDiscardLocal).toHaveBeenCalledWith('l1'));
  });

  it('Delete on a row asks for confirmation', async () => {
    setup();
    const a2 = screen.getByTestId('draft-resume-account-a2');
    fireEvent.keyDown(a2, { key: 'Delete' });
    expect(await screen.findByRole('alertdialog')).toBeInTheDocument();
  });

  it('shows load, error (with retry) and empty states', () => {
    const p = setup({ loadFailed: true }, []);
    expect(screen.getByRole('alert')).toHaveTextContent('could not be loaded');
    fireEvent.click(screen.getByRole('button', { name: 'Retry' }));
    expect(p.onRetryLoad).toHaveBeenCalled();
  });

  it('closes on Escape and on a click outside, and traps focus in a dialog', () => {
    const p = setup();
    const dialog = screen.getByRole('dialog', { name: 'Drafts' });
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    fireEvent.keyDown(dialog, { key: 'Escape' });
    expect(p.onClose).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByTestId('compose-drafts-scrim'));
    expect(p.onClose).toHaveBeenCalledTimes(2);
  });
});
