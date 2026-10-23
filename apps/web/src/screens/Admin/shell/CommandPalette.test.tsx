import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { CommandPalette, createAdminSearch, type AdminSearch, type PaletteSection } from './CommandPalette.tsx';

afterEach(() => vi.unstubAllGlobals());

const sections: PaletteSection[] = [
  { id: 'users', label: 'Users', icon: 'users', path: '/admin#/users' },
  { id: 'domains', label: 'Domains', icon: 'domains', path: '/admin#/domains' },
];

function Harness(props: { search: AdminSearch; onOpen?: (p: string) => void }) {
  const [open, setOpen] = createSignal(true);
  return (
    <>
      <button type="button" onClick={() => setOpen(true)}>
        Open
      </button>
      <CommandPalette open={open()} onClose={() => setOpen(false)} sections={sections} search={props.search} onOpen={props.onOpen ?? (() => undefined)} />
    </>
  );
}

describe('CommandPalette', () => {
  it('a combobox over a listbox of sections; arrows move the active option', async () => {
    render(() => <Harness search={async () => []} />);
    const input = await screen.findByRole('combobox', { name: 'Section, user, domain or setting' });
    await waitFor(() => expect(document.activeElement).toBe(input));
    const options = screen.getAllByRole('option');
    expect(options.map((o) => o.textContent)).toEqual(['UsersSection', 'DomainsSection']);
    expect(input).toHaveAttribute('aria-activedescendant', options[0]!.id);
    fireEvent.keyDown(input, { key: 'ArrowDown' });
    expect(input).toHaveAttribute('aria-activedescendant', options[1]!.id);
    expect(options[1]).toHaveAttribute('aria-selected', 'true');
    fireEvent.keyDown(input, { key: 'ArrowDown' });
    expect(input).toHaveAttribute('aria-activedescendant', options[0]!.id);
  });

  it('server hits for users appear after the sections; Enter opens the chosen path', async () => {
    const search = vi.fn<AdminSearch>(async () => [{ kind: 'user', label: 'alice@example.org', path: '/admin#/users/alice@example.org' }]);
    const opened = vi.fn();
    render(() => <Harness search={search} onOpen={opened} />);
    const input = await screen.findByRole('combobox');
    fireEvent.input(input, { target: { value: 'ali' } });
    const hit = await screen.findByRole('option', { name: /alice@example.org/ });
    expect(within(hit).getByText('User')).toBeInTheDocument();
    expect(search).toHaveBeenCalledWith('ali', expect.any(AbortSignal));
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(opened).toHaveBeenCalledWith('/admin#/users/alice@example.org');
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('without a server search route it says only sections are searched, and shows no invented hits', async () => {
    render(() => <Harness search={async () => 'unavailable'} />);
    const input = await screen.findByRole('combobox');
    fireEvent.input(input, { target: { value: 'use' } });
    expect(await screen.findByText(/only sections are listed/)).toBeInTheDocument();
    expect(screen.getAllByRole('option').map((o) => o.textContent)).toEqual(['UsersSection']);
  });

  it('Escape closes and focus returns to where it was', async () => {
    const opener = (): HTMLElement => screen.getByRole('button', { name: 'Open' });
    const { unmount } = render(() => <Harness search={async () => []} />);
    const dlg = await screen.findByRole('dialog', { name: 'Search the admin console' });
    fireEvent.keyDown(dlg, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    opener().focus();
    fireEvent.click(opener());
    await screen.findByRole('dialog');
    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' });
    await waitFor(() => expect(document.activeElement).toBe(opener()));
    unmount();
  });
});

describe('createAdminSearch', () => {
  it('404 means the server has no search route', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('', { status: 404 })));
    expect(await createAdminSearch('')('ali', new AbortController().signal)).toBe('unavailable');
  });

  it('keeps only hits that point inside the admin console', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () =>
        new Response(
          JSON.stringify({
            hits: [
              { kind: 'user', label: 'alice', path: '/admin#/users/alice' },
              { kind: 'user', label: 'evil', path: 'https://evil.example/' },
            ],
          }),
          { status: 200 },
        ),
      ),
    );
    const hits = await createAdminSearch('')('a', new AbortController().signal);
    expect(hits).toEqual([{ kind: 'user', label: 'alice', path: '/admin#/users/alice' }]);
  });
});
