import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { DirectorySearch } from './DirectorySearch.tsx';
import { GroupExpand } from './GroupExpand.tsx';
import { ContactSecurity } from './ContactSecurity.tsx';
import type { GalEntry } from './index.ts';

function okJson(body: unknown): Response {
  return new Response(JSON.stringify(body), { status: 200, headers: { 'content-type': 'application/json' } });
}

const alice: GalEntry = { dn: 'cn=alice,ou=people', displayName: 'Alice Ng', mail: 'alice@corp', isGroup: false };
const allStaff: GalEntry = { dn: 'cn=all-staff,ou=groups', displayName: 'All Staff', mail: 'all@corp', isGroup: true };

describe('GAL search in recipient fields', () => {
  it('renders directory matches and marks distribution groups', async () => {
    const fetcher = vi.fn(async () => okJson({ entries: [alice, allStaff], page: 0, hasMore: false }));
    const picked: GalEntry[] = [];
    render(() => (
      <DirectorySearch query="a" debounceMs={0} fetcher={fetcher} onPick={(e) => picked.push(e)} />
    ));

    await waitFor(() => expect(screen.getByRole('listbox', { name: 'Directory matches' })).toBeInTheDocument());
    expect(screen.getByText('Alice Ng')).toBeInTheDocument();
    expect(screen.getByText('All Staff')).toBeInTheDocument();
    // Only the group carries the Group badge.
    expect(screen.getAllByTestId('group-badge')).toHaveLength(1);

    fireEvent.click(screen.getByText('Alice Ng'));
    expect(picked).toHaveLength(1);
    expect(picked[0]?.mail).toBe('alice@corp');
  });

  it('does not query for an empty field', async () => {
    const fetcher = vi.fn(async () => okJson({ entries: [], page: 0, hasMore: false }));
    render(() => <DirectorySearch query="   " debounceMs={0} fetcher={fetcher} onPick={() => {}} />);
    await Promise.resolve();
    expect(fetcher).not.toHaveBeenCalled();
  });
});

describe('distribution-group expand-before-send ("who is actually in this?")', () => {
  it('expands the group to its concrete members and offers to replace it', async () => {
    const members: GalEntry[] = [
      { dn: 'cn=alice,ou=people', displayName: 'Alice Ng', mail: 'alice@corp', isGroup: false },
      { dn: 'cn=bob,ou=people', displayName: 'Bob Roy', mail: 'bob@corp', isGroup: false },
    ];
    const fetcher = vi.fn(async () => okJson({ members }));
    let replaced: GalEntry[] | null = null;
    render(() => <GroupExpand group={allStaff} fetcher={fetcher} onExpand={(m) => (replaced = m)} />);

    // Renders the prompt, not the members, until asked.
    expect(screen.getByTestId('group-expand')).toBeInTheDocument();
    expect(screen.queryByTestId('member-count')).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Who is actually in this?' }));
    await waitFor(() => expect(screen.getByTestId('member-count')).toHaveTextContent('2 recipients'));
    expect(screen.getByText('Alice Ng')).toBeInTheDocument();
    expect(screen.getByText('Bob Roy')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: /Replace group with 2 recipients/ }));
    expect(replaced).not.toBeNull();
    expect(replaced!).toHaveLength(2);
  });
});

describe('per-contact security tab (cert / photo rows)', () => {
  it('shows published S/MIME certs from the directory', async () => {
    const fetcher = vi.fn(async (input: string) => {
      if (input.startsWith('/api/directory/cert')) {
        return okJson({ certs: [{ derB64: 'AAA=', fingerprint: 'AB:CD', notAfter: '2999-01-01' }] });
      }
      return okJson({ photoB64: null });
    });
    render(() => <ContactSecurity email="alice@corp" fetcher={fetcher} />);

    await waitFor(() => expect(screen.getByTestId('cert-row')).toBeInTheDocument());
    expect(screen.getByText('AB:CD')).toBeInTheDocument();
    expect(screen.getByTestId('cert-status')).toHaveTextContent('Current');
  });
});
