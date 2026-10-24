// The open note across a pushed reload (t32-h11, 26.20).
//
// When another device or session changes a note, the server pushes a change and
// the Notes module reloads (`refetchPim` -> `loadNotes()`). The open editor
// keeps what the user typed; if the open note changed elsewhere a notice says
// so, and if it was deleted elsewhere the editor closes with a notice.

import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { createRoot } from 'solid-js';
import { NotesModule } from './index.tsx';
import { AppContext } from '../../state/context.ts';
import type { AppState } from '../../state/store.ts';
import { createNotesSlice, type NotesSlice } from '../../state/slices/notes.ts';
import type { SliceContext } from '../../state/slices/context.ts';
import type { Client } from '../../api/client.ts';
import type { JmapRequest, JmapResponse, JmapSession } from '../../api/jmap-types.ts';
import { CAP_NOTES, type Note } from '../../api/pim-types.ts';

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

interface Server {
  rows: Note[];
}

function mockClient(srv: Server): Client {
  const jmap = async (body: JmapRequest): Promise<JmapResponse> => {
    const names = body.methodCalls.map((c) => c[0]);
    if (names.includes('Note/get')) {
      const list = srv.rows.map((n) => structuredClone(n));
      return {
        methodResponses: [
          ['Note/query', { accountId: 'acct1', ids: list.map((n) => n.id) }, 'q'] as Row,
          ['Note/get', { accountId: 'acct1', state: 's', list, notFound: [] }, 'g'] as Row,
        ],
        sessionState: 's',
      };
    }
    const args = (body.methodCalls[0]?.[1] ?? {}) as { update?: Record<string, Partial<Note>>; destroy?: string[] };
    for (const [id, patch] of Object.entries(args.update ?? {})) {
      srv.rows = srv.rows.map((n) => (n.id === id ? { ...n, ...patch } : n));
    }
    srv.rows = srv.rows.filter((n) => !(args.destroy ?? []).includes(n.id));
    return {
      methodResponses: [['Note/set', { accountId: 'acct1', created: {}, updated: {}, destroyed: [] }, 's'] as Row],
      sessionState: 's',
    };
  };
  const session = async (): Promise<JmapSession> =>
    ({ primaryAccounts: { [CAP_NOTES]: 'acct1' }, accounts: { acct1: {} } }) as unknown as JmapSession;
  return { jmap, session, onNetwork: () => () => undefined } as unknown as Client;
}

function setup(seed: Note[]): { slice: NotesSlice; srv: Server } {
  const srv: Server = { rows: seed };
  const ctx: SliceContext = { client: mockClient(srv), showToast: () => undefined };
  let slice!: NotesSlice;
  createRoot(() => {
    slice = createNotesSlice(ctx);
  });
  render(() => (
    <AppContext.Provider value={slice as unknown as AppState}>
      <NotesModule />
    </AppContext.Provider>
  ));
  return { slice, srv };
}

async function open(title: string): Promise<void> {
  fireEvent.click(await screen.findByRole('option', { name: new RegExp(title) }));
  await screen.findByLabelText('Note editor');
}

const titleInput = (): HTMLInputElement => screen.getByLabelText('Note title') as HTMLInputElement;
const body = (): HTMLDivElement => screen.getByTestId('note-body') as HTMLDivElement;

function typeBody(html: string): void {
  body().innerHTML = html;
  fireEvent.input(body());
}

describe('NotesModule — the open note across a reload', () => {
  it('keeps the editor and the typed text when the reload brings equal rows', async () => {
    const { slice } = setup([mkNote('a', { title: 'Alpha' }), mkNote('b', { title: 'Bravo' })]);
    await open('Alpha');
    fireEvent.input(titleInput(), { target: { value: 'Alpha draft' } });
    typeBody('<p>typed text</p>');

    await slice.loadNotes();
    expect(screen.getByLabelText('Note editor')).toBeInTheDocument();
    expect(titleInput().value).toBe('Alpha draft');
    expect(body().innerHTML).toBe('<p>typed text</p>');
    expect(screen.queryByText(/changed on another device/)).toBeNull();
  });

  it('shows no notice when only another note changed', async () => {
    const { slice, srv } = setup([mkNote('a', { title: 'Alpha' }), mkNote('b', { title: 'Bravo' })]);
    await open('Alpha');
    typeBody('<p>typed text</p>');

    srv.rows = srv.rows.map((n) => (n.id === 'b' ? { ...n, title: 'Bravo two' } : n));
    await slice.loadNotes();
    expect(await screen.findByRole('option', { name: /Bravo two/ })).toBeInTheDocument();
    expect(body().innerHTML).toBe('<p>typed text</p>');
    expect(screen.queryByText(/changed on another device/)).toBeNull();
  });

  it('keeps the typed text and says so when the open note changed elsewhere', async () => {
    const { slice, srv } = setup([mkNote('a', { title: 'Alpha' })]);
    await open('Alpha');
    fireEvent.input(titleInput(), { target: { value: 'Mine' } });
    typeBody('<p>my text</p>');
    // The typed text has been saved; the change elsewhere comes after it.
    await waitFor(() => expect(srv.rows[0]).toMatchObject({ title: 'Mine', bodyHtml: '<p>my text</p>' }));

    srv.rows = srv.rows.map((n) => (n.id === 'a' ? { ...n, title: 'Theirs', bodyHtml: '<p>their text</p>' } : n));
    await slice.loadNotes();
    expect(await screen.findByText(/changed on another device/)).toBeInTheDocument();
    expect(titleInput().value).toBe('Mine');
    expect(body().innerHTML).toBe('<p>my text</p>');
  });

  it('shows a change made elsewhere in fields the user has not typed in', async () => {
    const { slice, srv } = setup([mkNote('a', { title: 'Alpha', bodyHtml: '<p>old</p>' })]);
    await open('Alpha');

    srv.rows = srv.rows.map((n) => (n.id === 'a' ? { ...n, title: 'Theirs', bodyHtml: '<p>their text</p>' } : n));
    await slice.loadNotes();
    expect(await screen.findByText(/changed on another device/)).toBeInTheDocument();
    expect(titleInput().value).toBe('Theirs');
    expect(body().innerHTML).toBe('<p>their text</p>');
  });

  it('closes the editor with a notice when the open note was deleted elsewhere', async () => {
    const { slice, srv } = setup([mkNote('a', { title: 'Alpha' }), mkNote('b', { title: 'Bravo' })]);
    await open('Alpha');

    srv.rows = srv.rows.filter((n) => n.id !== 'a');
    await slice.loadNotes();
    await waitFor(() => expect(screen.queryByLabelText('Note editor')).toBeNull());
    expect(screen.getByText(/was deleted on another device/)).toBeInTheDocument();

    // Opening another note clears the notice.
    await open('Bravo');
    expect(screen.queryByText(/was deleted on another device/)).toBeNull();
  });
});
