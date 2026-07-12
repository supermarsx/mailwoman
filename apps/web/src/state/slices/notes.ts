// Notes store slice (plan §2.5, §3 e6 — Web Notes module). Owns the `Note/*`
// surface for the web client: load, create, edit, pin, color, tag, delete, plus
// the title/tag + body-substring search and the tag filter. Disjoint file — no
// `store.ts` collision with the other PIM slices (same discipline as V2).
//
// Bodies transit the envelope in the clear (same-origin cookie-authed channel);
// they are sealed only at rest server-side (plan §1.6) — so the CLIENT holds
// plaintext HTML and is responsible for allowlist-sanitizing the editor output
// before it is sent (see `modules/notes/sanitize.ts`). Mock-backed until e10.

import { createSignal, createMemo, type Accessor } from 'solid-js';
import { CAP_CORE, type Id, type Invocation, type JmapRequest } from '../../api/jmap-types.ts';
import { responseFor } from '../../api/jmap.ts';
import { CAP_NOTES, type Note, type NoteLink } from '../../api/pim-types.ts';
import type { SliceContext } from './context.ts';

/** The default notebook every account has until multi-notebook lands (e8). */
export const DEFAULT_NOTEBOOK_ID = 'default';

/** Themed note colors (token-agnostic swatches; the chip renders the raw value). */
export const NOTE_COLORS: readonly string[] = [
  '#facc15', // amber
  '#f97316', // orange
  '#ef4444', // red
  '#ec4899', // pink
  '#a855f7', // violet
  '#3b82f6', // blue
  '#14b8a6', // teal
  '#22c55e', // green
  '#94a3b8', // slate (default / "no color")
];

/** The neutral color a new note starts with. */
export const DEFAULT_NOTE_COLOR = '#94a3b8';

// ── JMAP builders (Note/* over the existing envelope) ───────────────────────

const NOTES_USING = [CAP_CORE, CAP_NOTES];

/** The Note props the list needs (bodies included — client holds plaintext). */
const NOTE_PROPERTIES = [
  'id',
  'notebookId',
  'title',
  'tags',
  'color',
  'pinned',
  'bodyHtml',
  'bodyText',
  'links',
  'createdAt',
  'updatedAt',
] as const;

/** Query the account's notes newest-first, then hydrate in one round-trip. */
export function noteQuery(accountId: Id, limit = 500): JmapRequest {
  return {
    using: NOTES_USING,
    methodCalls: [
      [
        'Note/query',
        {
          accountId,
          sort: [
            { property: 'pinned', isAscending: false },
            { property: 'updatedAt', isAscending: false },
          ],
          limit,
        },
        'q',
      ],
      [
        'Note/get',
        { accountId, '#ids': { resultOf: 'q', name: 'Note/query', path: '/ids' }, properties: [...NOTE_PROPERTIES] },
        'g',
      ],
    ],
  };
}

/** A single `Note/set` (create / update / destroy) over the notes capability. */
export function noteSet(accountId: Id, ops: Record<string, unknown>): JmapRequest {
  const call: Invocation = ['Note/set', { accountId, ...ops }, 's'];
  return { using: NOTES_USING, methodCalls: [call] };
}

interface NoteGetResponse {
  accountId: Id;
  state: string;
  list: Note[];
  notFound: Id[];
}

interface NoteSetResponse {
  accountId: Id;
  created?: Record<string, Partial<Note>> | null;
  updated?: Record<string, Partial<Note> | null> | null;
  destroyed?: Id[] | null;
  notCreated?: Record<string, unknown> | null;
}

/** The mutable subset a create/update carries (server assigns id + timestamps). */
type NotePatch = Partial<Omit<Note, 'id' | 'createdAt'>>;

// ── Helpers (pure, exported for tests) ───────────────────────────────────────

/** Best-effort unique id for optimistic inserts before the server answers. */
function localId(): string {
  const c = globalThis.crypto;
  if (c !== undefined && typeof c.randomUUID === 'function') return c.randomUUID();
  return `note-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

/** Strip tags to a plain-text projection for the body substring search. */
export function noteBodyText(note: Note): string {
  if (note.bodyText.length > 0) return note.bodyText;
  return note.bodyHtml.replace(/<[^>]*>/g, ' ').replace(/&nbsp;/g, ' ');
}

/** Pinned first, then most-recently-updated first (stable for equal keys). */
export function sortNotes(list: readonly Note[]): Note[] {
  return [...list].sort((a, b) => {
    if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
    return b.updatedAt.localeCompare(a.updatedAt);
  });
}

/** Does `note` match the lowercased query across title / tags / body? */
export function noteMatches(note: Note, q: string): boolean {
  if (note.title.toLowerCase().includes(q)) return true;
  if (note.tags.some((t) => t.toLowerCase().includes(q))) return true;
  return noteBodyText(note).toLowerCase().includes(q);
}

// ── Slice ────────────────────────────────────────────────────────────────────

export interface NotesSlice {
  notes: Accessor<Note[]>;
  notesLoading: Accessor<boolean>;
  /** The notes matching the current search + tag filter, sorted (pinned first). */
  filteredNotes: Accessor<Note[]>;
  /** Every distinct tag across the loaded notes (sorted) for the filter UI. */
  noteTags: Accessor<string[]>;
  /** Free-text search over title + tags + body (substring, case-insensitive). */
  noteSearch: Accessor<string>;
  setNoteSearch(q: string): void;
  /** Restrict the list to notes carrying this tag, or `null` for all. */
  noteTagFilter: Accessor<string | null>;
  setNoteTagFilter(tag: string | null): void;

  /** Load the account's notes (pinned first). */
  loadNotes(): Promise<void>;
  /** Create a note; resolves to the created note (with its server id). */
  createNote(input?: NotePatch): Promise<Note>;
  /** Patch a note (title / body / color / tags / pinned / links). */
  updateNote(id: Id, patch: NotePatch): Promise<void>;
  /** Delete a note. */
  deleteNote(id: Id): Promise<void>;
  /** Flip a note's pinned flag. */
  toggleNotePin(id: Id): Promise<void>;
  /** Add a cross-link (dedup by type+id). */
  addNoteLink(id: Id, link: NoteLink): Promise<void>;
  /** Remove the cross-link at `index`. */
  removeNoteLink(id: Id, index: number): Promise<void>;
}

export function createNotesSlice(ctx: SliceContext): NotesSlice {
  const client = ctx.client;
  const [notes, setNotes] = createSignal<Note[]>([]);
  const [notesLoading, setNotesLoading] = createSignal(false);
  const [noteSearch, setNoteSearch] = createSignal('');
  const [noteTagFilter, setNoteTagFilter] = createSignal<string | null>(null);
  const [accountId, setAccountId] = createSignal<string | null>(null);

  const noteTags = createMemo(() => {
    const set = new Set<string>();
    for (const n of notes()) for (const t of n.tags) set.add(t);
    return [...set].sort((a, b) => a.localeCompare(b));
  });

  const filteredNotes = createMemo(() => {
    const q = noteSearch().trim().toLowerCase();
    const tag = noteTagFilter();
    let list = notes();
    if (tag !== null) list = list.filter((n) => n.tags.includes(tag));
    if (q.length > 0) list = list.filter((n) => noteMatches(n, q));
    return sortNotes(list);
  });

  /** Resolve (and cache) the notes account id; `null` when none is available. */
  async function resolveAccount(): Promise<string | null> {
    const cur = accountId();
    if (cur !== null) return cur;
    const session = await client.session();
    const primary = session.primaryAccounts[CAP_NOTES];
    const acct = primary ?? Object.keys(session.accounts)[0] ?? null;
    setAccountId(acct);
    return acct;
  }

  async function loadNotes(): Promise<void> {
    setNotesLoading(true);
    try {
      const acct = await resolveAccount();
      if (acct === null) {
        setNotes([]);
        return;
      }
      const res = await client.jmap(noteQuery(acct));
      const got = responseFor<NoteGetResponse>(res, 'g');
      setNotes(sortNotes(got.list));
    } finally {
      setNotesLoading(false);
    }
  }

  async function createNote(input: NotePatch = {}): Promise<Note> {
    const acct = await resolveAccount();
    if (acct === null) throw new Error('no account available for notes');
    const now = new Date().toISOString();
    const draft: Note = {
      id: '',
      notebookId: DEFAULT_NOTEBOOK_ID,
      title: '',
      tags: [],
      color: DEFAULT_NOTE_COLOR,
      pinned: false,
      bodyHtml: '',
      bodyText: '',
      links: [],
      createdAt: now,
      updatedAt: now,
      ...input,
    };
    const creationId = 'new';
    const { id: _omitId, createdAt: _omitCreated, ...create } = draft;
    const res = await client.jmap(noteSet(acct, { create: { [creationId]: create } }));
    const setRes = responseFor<NoteSetResponse>(res, 's');
    const created = setRes.created?.[creationId];
    const note: Note = { ...draft, id: created?.id ?? localId(), ...created };
    setNotes([note, ...notes()]);
    ctx.broadcastChange?.();
    return note;
  }

  async function updateNote(id: Id, patch: NotePatch): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) throw new Error('no account available for notes');
    const now = new Date().toISOString();
    const full = { ...patch, updatedAt: now };
    setNotes(notes().map((n) => (n.id === id ? { ...n, ...full } : n)));
    await client.jmap(noteSet(acct, { update: { [id]: full } }));
    ctx.broadcastChange?.();
  }

  async function deleteNote(id: Id): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) throw new Error('no account available for notes');
    setNotes(notes().filter((n) => n.id !== id));
    await client.jmap(noteSet(acct, { destroy: [id] }));
    ctx.broadcastChange?.();
  }

  async function toggleNotePin(id: Id): Promise<void> {
    const cur = notes().find((n) => n.id === id);
    if (cur === undefined) return;
    await updateNote(id, { pinned: !cur.pinned });
  }

  async function addNoteLink(id: Id, link: NoteLink): Promise<void> {
    const cur = notes().find((n) => n.id === id);
    if (cur === undefined) return;
    if (cur.links.some((l) => l.type === link.type && l.id === link.id)) return;
    await updateNote(id, { links: [...cur.links, link] });
  }

  async function removeNoteLink(id: Id, index: number): Promise<void> {
    const cur = notes().find((n) => n.id === id);
    if (cur === undefined) return;
    await updateNote(id, { links: cur.links.filter((_, i) => i !== index) });
  }

  return {
    notes,
    notesLoading,
    filteredNotes,
    noteTags,
    noteSearch,
    setNoteSearch,
    noteTagFilter,
    setNoteTagFilter,
    loadNotes,
    createNote,
    updateNote,
    deleteNote,
    toggleNotePin,
    addNoteLink,
    removeNoteLink,
  };
}
