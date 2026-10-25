import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { SavedSearches } from './SavedSearches.tsx';
import type { SettingsService } from './service.ts';
import type { SavedSearch } from './types.ts';

function fixture(initial: SavedSearch[] = []) {
  let rows = initial;
  const upsertSavedSearch = vi.fn(async (search: SavedSearch) => { rows = [{ ...search, id: search.id || 'created' }]; });
  const service = { listSavedSearches: vi.fn(async () => rows), upsertSavedSearch, deleteSavedSearch: vi.fn(async () => { rows = []; }) } as unknown as SettingsService;
  const changed = vi.fn();
  render(() => <SavedSearches service={service} onChanged={changed} />);
  return { upsertSavedSearch, changed, service };
}

describe('Search folder editors', () => {
  it('creates a combined filter on its own page and refreshes folders after save', async () => {
    const { upsertSavedSearch, changed } = fixture();
    fireEvent.click(await screen.findByRole('button', { name: 'New search folder' }));
    expect(screen.queryByTestId('saved-search-list')).not.toBeInTheDocument();
    fireEvent.input(screen.getByLabelText('Folder name'), { target: { value: 'Finance unread' } });
    fireEvent.input(screen.getByLabelText('Sender contains'), { target: { value: 'finance@example.test' } });
    fireEvent.click(screen.getByLabelText('Unread messages only'));
    fireEvent.change(screen.getByLabelText('Attachments'), { target: { value: 'true' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(changed).toHaveBeenCalledOnce());
    expect(upsertSavedSearch).toHaveBeenCalledWith({ id: '', name: 'Finance unread', asFolder: true, queryJson: JSON.stringify({ from: 'finance@example.test', notKeyword: '$seen', hasAttachment: true }) });
    expect(await screen.findByTestId('saved-search-list')).toHaveTextContent('Finance unread');
  });

  it('preserves existing filter fields and refuses to discard edits silently', async () => {
    const { upsertSavedSearch } = fixture([{ id: 's1', name: 'Old', queryJson: '{"minSize":100,"subject":"invoice"}', asFolder: true }]);
    fireEvent.click(await screen.findByRole('button', { name: 'Edit' }));
    fireEvent.input(screen.getByLabelText('Folder name'), { target: { value: 'Invoices' } });
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(await screen.findByRole('alertdialog')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /Keep editing/i }));
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(upsertSavedSearch).toHaveBeenCalledOnce());
    expect(JSON.parse(upsertSavedSearch.mock.calls[0]![0].queryJson)).toEqual({ minSize: 100, subject: 'invoice' });
  });

  it('does not replace a malformed stored query with an unrestricted search', async () => {
    const { upsertSavedSearch } = fixture([{ id: 's1', name: 'Broken', queryJson: 'invalid', asFolder: true }]);
    fireEvent.click(await screen.findByRole('button', { name: 'Edit' }));
    expect(screen.getByRole('alert')).toHaveTextContent('invalid filter');
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
    expect(upsertSavedSearch).not.toHaveBeenCalled();
  });
});
