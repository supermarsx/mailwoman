import { describe, it, expect, vi } from 'vitest';
import { createRoot } from 'solid-js';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { AppContext } from '../../state/context.ts';
import type { AppState } from '../../state/store.ts';
import { createNotesSlice, type NotesSlice } from '../../state/slices/notes.ts';
import type { SliceContext } from '../../state/slices/context.ts';
import type { Client, Me } from '../../api/client.ts';
import type { JmapRequest, JmapResponse, JmapSession } from '../../api/jmap-types.ts';
import { CAP_NOTES, type Note } from '../../api/pim-types.ts';
import { NotesModule } from './index.tsx';

// ── mock Note/* JMAP client (self-contained: the shared appHarness does not
//    speak the PIM surface, and it is not this module's file to edit) ─────────

const SESSION: JmapSession = {
  capabilities: {},
  accounts: { acct1: { name: 'T', isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
  primaryAccounts: { [CAP_NOTES]: 'acct1' },
  username: 'me@example.org',
  apiUrl: '/a', downloadUrl: '/d', uploadUrl: '/u', eventSourceUrl: '/e', state: 's0',
};

function mkNote(id: string, over: Partial<Note> = {}): Note {
  return {
    id,
    notebookId: 'default',
    title: `Note ${id}`,
    tags: [],
    color: '#94a3b8',
    pinned: false,
    bodyHtml: '',
    bodyText: '',
    links: [],
    createdAt: '2026-07-01T00:00:00Z',
    updatedAt: '2026-07-01T00:00:00Z',
    ...over,
  };
}

type Row = JmapResponse['methodResponses'][number];

function makeClient(seed: Note[]): Client {
  let store = [...seed];
  let counter = 0;
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    const names = body.methodCalls.map((c) => c[0]);
    if (names.includes('Note/get')) {
      return {
        methodResponses: [
          ['Note/query', { accountId: 'acct1', ids: store.map((n) => n.id) }, 'q'] as Row,
          ['Note/get', { accountId: 'acct1', state: 's', list: store, notFound: [] }, 'g'] as Row,
        ],
        sessionState: 's',
      };
    }
    if (names.includes('Note/set')) {
      const call = body.methodCalls.find((c) => c[0] === 'Note/set');
      const args = (call?.[1] ?? {}) as {
        create?: Record<string, Record<string, unknown>>;
        update?: Record<string, Record<string, unknown>>;
        destroy?: string[];
      };
      const created: Record<string, { id: string }> = {};
      for (const [k, v] of Object.entries(args.create ?? {})) {
        const id = `srv-${++counter}`;
        created[k] = { id };
        store = [mkNote(id, v as Partial<Note>), ...store];
      }
      for (const [id, patch] of Object.entries(args.update ?? {})) {
        store = store.map((n) => (n.id === id ? { ...n, ...(patch as Partial<Note>) } : n));
      }
      if (args.destroy !== undefined) store = store.filter((n) => !args.destroy!.includes(n.id));
      return {
        methodResponses: [['Note/set', { accountId: 'acct1', created, updated: {}, destroyed: [] }, 's'] as Row],
        sessionState: 's',
      };
    }
    return { methodResponses: body.methodCalls.map((c) => [c[0], {}, c[2]] as Row), sessionState: 's' };
  });
  return {
    login: vi.fn(async (): Promise<Me> => ({ username: 'me@example.org', accountId: 'acct1' })),
    logout: vi.fn(async () => undefined),
    me: vi.fn(async (): Promise<Me> => ({ username: 'me@example.org', accountId: 'acct1' })),
    session: vi.fn(async () => SESSION),
    jmap,
    sanitize: vi.fn(async (h: string) => h),
    onNetwork: vi.fn(() => () => undefined),
  };
}

function setup(seed: Note[]): { slice: NotesSlice } {
  const ctx: SliceContext = { client: makeClient(seed), showToast: () => undefined };
  let slice!: NotesSlice;
  createRoot(() => {
    slice = createNotesSlice(ctx);
  });
  const app = slice as unknown as AppState;
  render(() => (
    <AppContext.Provider value={app}>
      <NotesModule />
    </AppContext.Provider>
  ));
  return { slice };
}

async function selectFirst(): Promise<void> {
  const items = await screen.findAllByRole('option');
  fireEvent.click(items[0]!);
  await screen.findByLabelText('Note editor');
}

describe('NotesModule', () => {
  it('loads notes and lists pinned first', async () => {
    setup([mkNote('a', { title: 'Alpha' }), mkNote('b', { title: 'Bravo', pinned: true })]);
    const options = await screen.findAllByRole('option');
    // Bravo is pinned, so it sorts ahead of Alpha.
    expect(options[0]).toHaveTextContent('Bravo');
    expect(options[1]).toHaveTextContent('Alpha');
  });

  it('creates a new note and opens it', async () => {
    const { slice } = setup([]);
    await waitFor(() => expect(slice.notesLoading()).toBe(false));
    fireEvent.click(screen.getByRole('button', { name: '+ New note' }));
    await screen.findByLabelText('Note editor');
    await waitFor(() => expect(slice.notes().some((n) => n.title === 'Untitled note')).toBe(true));
  });

  it('edits the title', async () => {
    const { slice } = setup([mkNote('a', { title: 'Alpha' })]);
    await selectFirst();
    const title = screen.getByLabelText('Note title') as HTMLInputElement;
    fireEvent.input(title, { target: { value: 'Renamed' } });
    await waitFor(() => expect(slice.notes().find((n) => n.id === 'a')?.title).toBe('Renamed'));
  });

  it('pins a note from the detail pane', async () => {
    const { slice } = setup([mkNote('a', { title: 'Alpha' })]);
    await selectFirst();
    fireEvent.click(screen.getByRole('button', { name: 'Pin note' }));
    await waitFor(() => expect(slice.notes().find((n) => n.id === 'a')?.pinned).toBe(true));
  });

  it('changes the note color', async () => {
    const { slice } = setup([mkNote('a')]);
    await selectFirst();
    fireEvent.click(screen.getByRole('button', { name: 'Color #ef4444' }));
    await waitFor(() => expect(slice.notes().find((n) => n.id === 'a')?.color).toBe('#ef4444'));
  });

  it('adds and removes a tag', async () => {
    const { slice } = setup([mkNote('a')]);
    await selectFirst();
    const tagInput = screen.getByLabelText('Add tag') as HTMLInputElement;
    fireEvent.input(tagInput, { target: { value: 'Work' } });
    fireEvent.submit(tagInput.closest('form')!);
    await waitFor(() => expect(slice.notes().find((n) => n.id === 'a')?.tags).toEqual(['work']));

    fireEvent.click(screen.getByRole('button', { name: 'Remove tag work' }));
    await waitFor(() => expect(slice.notes().find((n) => n.id === 'a')?.tags).toEqual([]));
  });

  it('searches notes by title', async () => {
    setup([mkNote('a', { title: 'Groceries' }), mkNote('b', { title: 'Meeting notes' })]);
    await screen.findAllByRole('option');
    fireEvent.input(screen.getByLabelText('Search notes'), { target: { value: 'groc' } });
    await waitFor(() => {
      const opts = screen.getAllByRole('option');
      expect(opts).toHaveLength(1);
      expect(opts[0]).toHaveTextContent('Groceries');
    });
  });

  it('filters by tag chip', async () => {
    setup([mkNote('a', { title: 'Tagged', tags: ['home'] }), mkNote('b', { title: 'Untagged' })]);
    await screen.findAllByRole('option');
    fireEvent.click(screen.getByRole('button', { name: '#home' }));
    await waitFor(() => {
      const opts = screen.getAllByRole('option');
      expect(opts).toHaveLength(1);
      expect(opts[0]).toHaveTextContent('Tagged');
    });
  });

  it('adds a cross-link via the picker', async () => {
    const { slice } = setup([mkNote('a')]);
    await selectFirst();
    fireEvent.input(screen.getByLabelText('Link target id'), { target: { value: 'msg-42' } });
    fireEvent.submit(screen.getByLabelText('Add cross-link'));
    await waitFor(() => {
      const links = slice.notes().find((n) => n.id === 'a')?.links ?? [];
      expect(links).toContainEqual({ type: 'email', id: 'msg-42' });
    });
  });

  it('sanitizes editor output so no script is stored', async () => {
    const { slice } = setup([mkNote('a')]);
    await selectFirst();
    const body = screen.getByTestId('note-body') as HTMLDivElement;
    body.innerHTML = '<p>safe</p><script>alert(1)</script>';
    fireEvent.input(body);
    await waitFor(() => {
      const stored = slice.notes().find((n) => n.id === 'a')?.bodyHtml ?? '';
      expect(stored).toContain('<p>safe</p>');
      expect(stored.toLowerCase()).not.toContain('<script');
    });
  });
});
