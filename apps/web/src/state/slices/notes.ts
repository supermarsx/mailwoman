import { toastTarget } from '../../components/toast/context.ts';
// Notes store slice (plan §2.5, §3 e6 — Web Notes module). Owns the `Note/*`
// surface for the web client: load, create, edit, pin, color, tag, delete, plus
// the title/tag + body-substring search and the tag filter. Disjoint file — no
// `store.ts` collision with the other PIM slices (same discipline as V2).
//
// Bodies transit the envelope in the clear (same-origin cookie-authed channel);
// they are sealed only at rest server-side (plan §1.6) — so the CLIENT holds
// plaintext HTML and is responsible for allowlist-sanitizing the editor output
// before it is sent (see `modules/notes/sanitize.ts`). Mock-backed until e10.

import { createSignal, createMemo, batch, type Accessor } from 'solid-js';
import { CAP_CORE, type Id, type Invocation, type JmapRequest } from '../../api/jmap-types.ts';
import { responseFor } from '../../api/jmap.ts';
import { CAP_NOTES, type Note, type NoteLink } from '../../api/pim-types.ts';
import { t } from '../../i18n';
import type { SliceContext } from './context.ts';
import { LocalWrites, keepUnchanged, sameValue } from './localWrites.ts';

// A save asks the transport to finish even if the page closes or reloads
// right after it starts (t32-h11); see `jmap` in `api/client.ts`.
const SAVE = { keepalive: true } as const;

/** The default notebook every account has until multi-notebook lands (e8). */
export const DEFAULT_NOTEBOOK_ID = 'default';

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

/** The fields a user edits; a change to one of them made elsewhere is worth a notice
 *  (timestamps and the server-derived `bodyText` are not). */
const EDITED_FIELDS = ['notebookId', 'title', 'tags', 'color', 'pinned', 'bodyHtml', 'links'] as const;

function sameContent(a: Note, b: Note): boolean {
  return EDITED_FIELDS.every((k) => sameValue(a[k], b[k]));
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
  /** How many reloads brought a change to note `id` made on another device or
   *  session (0 when none has); compare against an earlier reading. */
  noteChangedElsewhere(id: Id): number;
  /** Whether a reload found note `id` gone without this client deleting it. */
  noteDeletedElsewhere(id: Id): boolean;
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

  // A load answered after this client wrote can predate the write; `writes`
  // keeps the local version of such notes (see `localWrites.ts`), so a note
  // just created or a title just typed is not undone by a pushed reload, and
  // `keepUnchanged` keeps the old object for every unchanged note (t32-h11).
  const writes = new LocalWrites<Note>();
  const [changedElsewhere, setChangedElsewhere] = createSignal<ReadonlyMap<Id, number>>(new Map());
  const [deletedElsewhere, setDeletedElsewhere] = createSignal<ReadonlySet<Id>>(new Set());

  function applyLoad(loaded: Note[], sentAt: number): void {
    const prev = notes();
    const merged = writes.merge(prev, loaded, sentAt);
    const next = keepUnchanged(prev, sortNotes(merged.rows));
    const prevById = new Map(prev.map((n) => [n.id, n]));
    const changed = next.filter((n) => {
      const old = prevById.get(n.id);
      return old !== undefined && !merged.local.has(n.id) && !sameContent(old, n);
    });
    const kept = new Set(next.map((n) => n.id));
    const gone = prev.filter((n) => !kept.has(n.id)).map((n) => n.id);
    batch(() => {
      if (changed.length > 0) {
        setChangedElsewhere((m) => {
          const out = new Map(m);
          for (const n of changed) out.set(n.id, (out.get(n.id) ?? 0) + 1);
          return out;
        });
      }
      if (gone.length > 0) setDeletedElsewhere((s) => new Set([...s, ...gone]));
      setNotes(next);
    });
  }

  // Each `loadNotes()` takes a ticket when it is called. Calls overlap (opening
  // Notes starts two loads), and one that started earlier can answer last; only
  // the call holding the latest ticket writes state and ends the loading state,
  // so the older answer is dropped instead of replacing the newer one (as
  // `loadTicket` in `modules/calendar/controller.ts`).
  let loadTicket = 0;

  async function loadNotes(): Promise<void> {
    const ticket = ++loadTicket;
    const stale = (): boolean => ticket !== loadTicket;
    setNotesLoading(true);
    try {
      const acct = await resolveAccount();
      if (stale()) return;
      if (acct === null) {
        setNotes([]);
        return;
      }
      const sentAt = writes.loadSent();
      const res = await client.jmap(noteQuery(acct));
      if (stale()) return;
      applyLoad(responseFor<NoteGetResponse>(res, 'g').list, sentAt);
    } finally {
      // The latest call ends the loading state; a dropped one leaves it alone.
      if (!stale()) setNotesLoading(false);
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
    let created: Partial<Note> | undefined;
    try {
      const res = await client.jmap(noteSet(acct, { create: { [creationId]: create } }), SAVE);
      created = responseFor<NoteSetResponse>(res, 's').created?.[creationId] ?? undefined;
    } catch (err) {
      ctx.showToast('error', t('notes-create-failed'));
      throw err;
    }
    // A refused create adds nothing: the note would vanish on the next load.
    if (created?.id === undefined) {
      ctx.showToast('error', t('notes-create-failed'));
      throw new Error('the server did not create the note');
    }
    const note: Note = { ...draft, ...created, id: created.id };
    // The server holds it now; a load sent before this answer may not.
    writes.begin(note.id, 'create')();
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
    const answered = writes.begin(id, 'update', full);
    try {
      await client.jmap(noteSet(acct, { update: { [id]: full } }), SAVE);
    } finally {
      answered();
    }
    ctx.broadcastChange?.();
  }

  async function deleteNote(id: Id): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) throw new Error('no account available for notes');
    setNotes(notes().filter((n) => n.id !== id));
    const answered = writes.begin(id, 'destroy');
    try {
      await client.jmap(noteSet(acct, { destroy: [id] }), SAVE);
    } finally {
      answered();
    }
    ctx.broadcastChange?.();
  }

  async function toggleNotePin(id: Id): Promise<void> {
    const cur = notes().find((n) => n.id === id);
    if (cur === undefined) return;
    const acct = await resolveAccount();
    if (acct === null) return;
    const pinned = !cur.pinned;
    const updatedAt = new Date().toISOString();
    const answered = writes.begin(id, 'update', { pinned, updatedAt });
    try {
      const response = await client.jmap(noteSet(acct, { update: { [id]: { pinned, updatedAt } } }), SAVE);
      const result = responseFor<NoteSetResponse>(response, 's');
      if (!Object.prototype.hasOwnProperty.call(result.updated ?? {}, id)) throw new Error('note pin was refused');
      setNotes(notes().map((note) => note.id === id ? { ...note, pinned, updatedAt } : note));
      ctx.broadcastChange?.();
      ctx.showToast('success', t(pinned ? 'notes-toast-pinned' : 'notes-toast-unpinned', { target: toastTarget(cur.title, t('notes-untitled')) }));
    } catch {
      ctx.showToast('error', t('notes-toast-pin-failed', { target: toastTarget(cur.title, t('notes-untitled')) }));
    } finally {
      answered();
    }
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
    noteChangedElsewhere: (id: Id) => changedElsewhere().get(id) ?? 0,
    noteDeletedElsewhere: (id: Id) => deletedElsewhere().has(id),
    createNote,
    updateNote,
    deleteNote,
    toggleNotePin,
    addNoteLink,
    removeNoteLink,
  };
}
