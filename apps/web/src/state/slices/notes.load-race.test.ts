// `loadNotes()` when two calls overlap (t31-e4, 26.20).
//
// `loadNotes()` writes `notes` after its awaits. Opening Notes starts two loads
// (the surface switch and the module mount each call it). If the earlier one
// answers after the later one, it must not put its older rows back (a note
// created in between would vanish until the next reload), and it must not end
// the loading state of the later one.
//
// The backend is an in-memory mock whose `Note/get` answers are computed when
// asked (so a held answer is the one from before any later write) and delivered
// only once the test releases them. Same shape as
// `modules/calendar/controller.load-race.test.ts` (t30-h1).

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

/** A stateful mock client; the `Note/get` calls numbered in `hold` (1-based)
 *  wait for their gate's release before answering. */
function harness(hold: number[]): { client: Client; gates: Map<number, Gate> } {
  let store: Note[] = [mkNote('n1')];
  let loads = 0;
  let created = 0;
  const gates = new Map(hold.map((n) => [n, gate()] as const));
  const jmap = async (body: JmapRequest): Promise<JmapResponse> => {
    const names = body.methodCalls.map((c) => c[0]);
    if (names.includes('Note/get')) {
      loads += 1;
      const list = [...store];
      const g = gates.get(loads);
      if (g !== undefined) {
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
    // Note/set — only creates are used here.
    const call = body.methodCalls.find((c) => c[0] === 'Note/set');
    const create = ((call?.[1] ?? {}) as { create?: Record<string, Partial<Note>> }).create ?? {};
    const out: Record<string, { id: string }> = {};
    for (const [key, v] of Object.entries(create)) {
      const id = `srv-${(created += 1)}`;
      out[key] = { id };
      store = [mkNote(id, v), ...store];
    }
    return {
      methodResponses: [['Note/set', { accountId: 'acct1', created: out, updated: {}, destroyed: [] }, 's'] as Row],
      sessionState: 's',
    };
  };
  const session = async (): Promise<JmapSession> =>
    ({ primaryAccounts: { [CAP_NOTES]: 'acct1' }, accounts: { acct1: {} } }) as unknown as JmapSession;
  const client = { jmap, session, onNetwork: () => () => undefined } as unknown as Client;
  return { client, gates };
}

function withSlice(client: Client, run: (s: NotesSlice) => Promise<void>): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    createRoot((dispose) => {
      const ctx: SliceContext = { client, showToast: () => undefined };
      run(createNotesSlice(ctx)).then(resolve, reject).finally(dispose);
    });
  });
}

describe('notes slice — overlapping loadNotes()', () => {
  it('keeps the notes of the load that started last when an earlier one answers after it', async () => {
    const { client, gates } = harness([1]);
    await withSlice(client, async (s) => {
      const first = s.loadNotes();
      await gates.get(1)!.reached;

      // A note written while the first load waits, then a load that runs to the end.
      const note = await s.createNote({ title: 'Added while loading' });
      await s.loadNotes();
      expect(s.notes().some((n) => n.id === note.id)).toBe(true);

      gates.get(1)!.release();
      await first;

      expect(s.notes().some((n) => n.id === note.id)).toBe(true);
      expect(s.notes().map((n) => n.id).sort()).toEqual(['n1', 'srv-1']);
      expect(s.notesLoading()).toBe(false);
    });
  });

  it('stays loading until the latest call answers, whichever answers first', async () => {
    const { client, gates } = harness([1, 2]);
    await withSlice(client, async (s) => {
      const first = s.loadNotes();
      await gates.get(1)!.reached;
      const second = s.loadNotes();
      await gates.get(2)!.reached;
      expect(s.notesLoading()).toBe(true);

      gates.get(1)!.release();
      await first;
      expect(s.notesLoading()).toBe(true);

      gates.get(2)!.release();
      await second;
      expect(s.notesLoading()).toBe(false);
      expect(s.notes().map((n) => n.id)).toEqual(['n1']);
    });
  });
});
