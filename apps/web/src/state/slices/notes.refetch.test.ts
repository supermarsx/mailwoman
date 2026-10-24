// `loadNotes()` answering while this client is writing (t32-h11, 26.20).
//
// A change pushed from another device or session makes the open Notes module
// reload. That reload can be answered from data read before a write this
// client has just made reached the server; applied as-is it would drop a note
// just created or put back the title just typed. A reload also must not hand
// out new objects for notes whose content did not change. The latest-wins
// guarantee of `notes.load-race.test.ts` (t31-e4) still holds.

import { describe, it, expect } from 'vitest';
import { createRoot } from 'solid-js';
import type { Client } from '../../api/client.ts';
import type { JmapRequest, JmapResponse, JmapSession } from '../../api/jmap-types.ts';
import { CAP_NOTES, type Note } from '../../api/pim-types.ts';
import type { SliceContext } from './context.ts';
import { createNotesSlice, type NotesSlice } from './notes.ts';

interface Gate {
  reached: Promise<void>;
  reach(): void;
  release(): void;
  released: Promise<void>;
}

function gate(): Gate {
  let reach: () => void = () => undefined;
  let release: () => void = () => undefined;
  const reached = new Promise<void>((r) => {
    reach = r;
  });
  const released = new Promise<void>((r) => {
    release = r;
  });
  return { reached, reach, release, released };
}

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
  client: Client;
  /** The server's rows; tests change them to stand for another device. */
  rows: Note[];
  /** Hold the next `Note/get` (computed now, answered on release). */
  holdNextLoad(): Gate;
  /** Hold the next `Note/set` until release (the write is applied on release). */
  holdNextSet(): Gate;
  /** Refuse the next `Note/set` create. */
  refuseNextCreate(): void;
}

function server(seed: Note[]): Server {
  const s: Server = {
    rows: seed.map((n) => ({ ...n })),
    client: undefined as unknown as Client,
    holdNextLoad: () => (loadGate = gate()),
    holdNextSet: () => (setGate = gate()),
    refuseNextCreate: () => {
      refuse = true;
    },
  };
  let loadGate: Gate | null = null;
  let setGate: Gate | null = null;
  let refuse = false;
  let created = 0;
  const jmap = async (body: JmapRequest): Promise<JmapResponse> => {
    const names = body.methodCalls.map((c) => c[0]);
    if (names.includes('Note/get')) {
      // Fresh objects every time, as a real answer would be.
      const list = s.rows.map((n) => structuredClone(n));
      const g = loadGate;
      loadGate = null;
      if (g !== null) {
        g.reach();
        await g.released;
      }
      return {
        methodResponses: [
          ['Note/query', { accountId: 'acct1', ids: list.map((n) => n.id) }, 'q'] as Row,
          ['Note/get', { accountId: 'acct1', state: 's', list, notFound: [] }, 'g'] as Row,
        ],
        sessionState: 's',
      };
    }
    const g = setGate;
    setGate = null;
    if (g !== null) {
      g.reach();
      await g.released;
    }
    const args = (body.methodCalls[0]?.[1] ?? {}) as {
      create?: Record<string, Partial<Note>>;
      update?: Record<string, Partial<Note>>;
      destroy?: string[];
    };
    const out: Record<string, { id: string }> = {};
    const notCreated: Record<string, { type: string }> = {};
    for (const [key, v] of Object.entries(args.create ?? {})) {
      if (refuse) {
        refuse = false;
        notCreated[key] = { type: 'forbidden' };
        continue;
      }
      const id = `srv-${(created += 1)}`;
      out[key] = { id };
      s.rows = [mkNote(id, v), ...s.rows];
    }
    for (const [id, patch] of Object.entries(args.update ?? {})) {
      s.rows = s.rows.map((n) => (n.id === id ? { ...n, ...patch } : n));
    }
    for (const id of args.destroy ?? []) s.rows = s.rows.filter((n) => n.id !== id);
    return {
      methodResponses: [
        ['Note/set', { accountId: 'acct1', created: out, notCreated, updated: {}, destroyed: [] }, 's'] as Row,
      ],
      sessionState: 's',
    };
  };
  const session = async (): Promise<JmapSession> =>
    ({ primaryAccounts: { [CAP_NOTES]: 'acct1' }, accounts: { acct1: {} } }) as unknown as JmapSession;
  s.client = { jmap, session, onNetwork: () => () => undefined } as unknown as Client;
  return s;
}

function withSlice(
  client: Client,
  run: (s: NotesSlice, toasts: string[]) => Promise<void>,
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    createRoot((dispose) => {
      const toasts: string[] = [];
      const ctx: SliceContext = { client, showToast: (kind) => toasts.push(kind) };
      run(createNotesSlice(ctx), toasts).then(resolve, reject).finally(dispose);
    });
  });
}

describe('notes slice — a reload answering while this client writes', () => {
  it('keeps a note created while a reload was out, when the reload answers without it', async () => {
    const srv = server([mkNote('n1')]);
    await withSlice(srv.client, async (s) => {
      await s.loadNotes();
      const held = srv.holdNextLoad();
      const reload = s.loadNotes();
      await held.reached;

      const note = await s.createNote({ title: 'Saved during the reload' });
      expect(s.notes().some((n) => n.id === note.id)).toBe(true);

      held.release();
      await reload;
      expect(s.notes().map((n) => n.id)).toContain(note.id);

      // The next reload lists it from the server, and it stays.
      await s.loadNotes();
      expect(s.notes().map((n) => n.id).sort()).toEqual(['n1', note.id].sort());
    });
  });

  it('keeps the title typed while a reload was out, when the reload answers the old title', async () => {
    const srv = server([mkNote('n1', { title: 'Old' })]);
    await withSlice(srv.client, async (s) => {
      await s.loadNotes();
      const held = srv.holdNextLoad();
      const reload = s.loadNotes();
      await held.reached;

      await s.updateNote('n1', { title: 'Typed' });
      held.release();
      await reload;
      expect(s.notes().find((n) => n.id === 'n1')?.title).toBe('Typed');
    });
  });

  it('keeps the typed title while its write is still out, when a later reload answers first', async () => {
    const srv = server([mkNote('n1', { title: 'Old' })]);
    await withSlice(srv.client, async (s) => {
      await s.loadNotes();
      const write = srv.holdNextSet();
      const saving = s.updateNote('n1', { title: 'Typed' });
      await write.reached;

      await s.loadNotes();
      expect(s.notes().find((n) => n.id === 'n1')?.title).toBe('Typed');

      write.release();
      await saving;
      await s.loadNotes();
      expect(s.notes().find((n) => n.id === 'n1')?.title).toBe('Typed');
    });
  });

  it('does not put back a note deleted while a reload was out', async () => {
    const srv = server([mkNote('n1'), mkNote('n2')]);
    await withSlice(srv.client, async (s) => {
      await s.loadNotes();
      const held = srv.holdNextLoad();
      const reload = s.loadNotes();
      await held.reached;

      await s.deleteNote('n2');
      held.release();
      await reload;
      expect(s.notes().map((n) => n.id)).toEqual(['n1']);
      expect(s.noteDeletedElsewhere('n2')).toBe(false);
    });
  });

  it('takes a change made elsewhere once no write of this client is newer', async () => {
    const srv = server([mkNote('n1', { title: 'Old' })]);
    await withSlice(srv.client, async (s) => {
      await s.loadNotes();
      await s.updateNote('n1', { title: 'Mine' });
      const before = s.noteChangedElsewhere('n1');

      srv.rows = srv.rows.map((n) => (n.id === 'n1' ? { ...n, title: 'Theirs' } : n));
      await s.loadNotes();
      expect(s.notes().find((n) => n.id === 'n1')?.title).toBe('Theirs');
      expect(s.noteChangedElsewhere('n1')).toBeGreaterThan(before);
    });
  });

  it('marks a note removed elsewhere, and only that one', async () => {
    const srv = server([mkNote('n1'), mkNote('n2')]);
    await withSlice(srv.client, async (s) => {
      await s.loadNotes();
      srv.rows = srv.rows.filter((n) => n.id !== 'n2');
      await s.loadNotes();
      expect(s.notes().map((n) => n.id)).toEqual(['n1']);
      expect(s.noteDeletedElsewhere('n2')).toBe(true);
      expect(s.noteDeletedElsewhere('n1')).toBe(false);
    });
  });

  it('keeps the same objects for notes a reload did not change', async () => {
    const srv = server([mkNote('n1'), mkNote('n2')]);
    await withSlice(srv.client, async (s) => {
      await s.loadNotes();
      const [a, b] = [s.notes().find((n) => n.id === 'n1'), s.notes().find((n) => n.id === 'n2')];

      srv.rows = srv.rows.map((n) => (n.id === 'n2' ? { ...n, title: 'Changed' } : n));
      await s.loadNotes();
      expect(s.notes().find((n) => n.id === 'n1')).toBe(a);
      expect(s.notes().find((n) => n.id === 'n2')).not.toBe(b);
      expect(s.noteChangedElsewhere('n1')).toBe(0);
    });
  });

  it('shows an error and adds no note when the server refuses a create', async () => {
    const srv = server([mkNote('n1')]);
    await withSlice(srv.client, async (s, toasts) => {
      await s.loadNotes();
      srv.refuseNextCreate();
      await expect(s.createNote({ title: 'Refused' })).rejects.toThrow();
      expect(toasts).toContain('error');
      expect(s.notes().map((n) => n.id)).toEqual(['n1']);
    });
  });
});
