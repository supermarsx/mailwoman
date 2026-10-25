// Field hits in the command palette (t32-e71a, UX10).
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { CommandPalette, type AdminSearch, type PaletteField } from './CommandPalette.tsx';

const FIELDS: PaletteField[] = [
  { id: 'smtp.port', label: 'Relay port', sectionLabel: 'Mail handling', path: '/admin#/mail?field=smtp.port', keywords: ['MW_SMTP_PORT'] },
  { id: 'signin.newUsers', label: 'New users', sectionLabel: 'Sign-in policy', path: '/admin#/signin?field=signin.newUsers' },
];

function open(search: AdminSearch = async () => [], onOpen = vi.fn()) {
  render(() => <CommandPalette open onClose={() => undefined} sections={[]} fields={FIELDS} search={search} onOpen={onOpen} />);
  return { input: screen.getByRole('combobox'), onOpen };
}

describe('CommandPalette field hits', () => {
  it('lists no fields until two characters are typed', () => {
    const { input } = open();
    fireEvent.input(input, { target: { value: 'r' } });
    expect(screen.queryAllByRole('option')).toHaveLength(0);
  });

  it('matches label, id and keywords, shows the section, and opens ?field=', () => {
    const { input, onOpen } = open();
    fireEvent.input(input, { target: { value: 'mw_smtp' } });
    const [hit] = screen.getAllByRole('option');
    expect(hit).toHaveTextContent('Relay port');
    expect(hit).toHaveTextContent('Mail handling');
    expect(hit).toHaveTextContent('Field');
    fireEvent.input(input, { target: { value: 'newusers' } });
    expect(screen.getAllByRole('option')[0]).toHaveTextContent('New users');
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onOpen).toHaveBeenCalledWith('/admin#/signin?field=signin.newUsers');
  });

  it('a server hit for the same field is shown once', async () => {
    const search = vi.fn<AdminSearch>(async () => [
      { kind: 'setting', label: 'Relay port', path: '/admin#/mail?field=smtp.port' },
      { kind: 'user', label: 'porter@example.org', path: '/admin#/users/porter@example.org' },
    ]);
    const { input } = open(search);
    fireEvent.input(input, { target: { value: 'port' } });
    await waitFor(() => expect(screen.getAllByRole('option')).toHaveLength(2));
    expect(screen.getAllByRole('option').map((o) => o.textContent)).toEqual([
      expect.stringContaining('Relay port'),
      expect.stringContaining('porter@example.org'),
    ]);
  });
});
