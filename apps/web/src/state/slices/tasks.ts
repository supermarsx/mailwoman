// Tasks store slice (plan §2.5, §3 e5). Task lists (CalDAV VTODO collections),
// task create/edit (due/start/priority/recurrence/percent-complete/subtasks via
// parentId), the My Day / Today view, complete/reopen, drag-reorder, and the
// mail→task / event→task convert entry points — all over the frozen `Task/*`
// surface (plan §2.2), mock-backed until e10 swaps in the real engine.
//
// Disjoint file — no `store.ts` collision with the other PIM slices (same slice
// discipline as V2). `store.ts` spreads whatever this factory returns into
// `AppState`, so the interface below is additive and self-contained.

import { createSignal, createMemo, batch, type Accessor } from 'solid-js';
import type { Id, JmapResponse } from '../../api/jmap-types.ts';
import { responseFor } from '../../api/jmap.ts';
import type { Task } from '../../api/pim-types.ts';
import {
  taskListFromCalendar,
  taskListsGet,
  taskSet,
  tasksQueryGet,
  type CalendarGetResponse,
  type TaskCreate,
  type TaskGetResponse,
  type TaskList,
  type TaskSetResponse,
} from '../../modules/tasks/api.ts';
import type { SliceContext } from './context.ts';
import { LocalWrites, keepUnchanged } from './localWrites.ts';

// A save asks the transport to finish even if the page closes or reloads
// right after it starts (t32-h11); see `jmap` in `api/client.ts`.
const SAVE = { keepalive: true } as const;

export type { TaskList } from '../../modules/tasks/api.ts';

/** The fields a task editor supplies on create/edit (a subset of `Task`). */
export interface TaskDraft {
  listId: Id;
  title: string;
  description?: string;
  start?: string | null;
  due?: string | null;
  timeZone?: string | null;
  priority?: number;
  percentComplete?: number;
  status?: Task['status'];
  recurrenceRules?: Array<Record<string, unknown>>;
  parentId?: Id | null;
  myDayDate?: string | null;
}

/** The tasks portion of `AppState` (accessors + actions). */
export interface TasksSlice {
  /** Every loaded task across the selected list (roots + subtasks). */
  tasks: Accessor<Task[]>;
  /** The account's task lists (VTODO collections). */
  taskLists: Accessor<TaskList[]>;
  /** The currently-selected list id, or `null` for "all lists" / My Day. */
  selectedListId: Accessor<Id | null>;
  /** True while a load is in flight. */
  tasksLoading: Accessor<boolean>;

  // ── derived views ──
  /** Root tasks (no parent) of the selected list, in load order. */
  rootTasks: Accessor<Task[]>;
  /** Subtasks of a given parent task, in load order. */
  subtasksOf(parentId: Id): Task[];
  /** Whether a reload of the same list found `id` gone: deleted on another
   *  device or session, not by this client and not by picking another list. */
  taskDeletedElsewhere(id: Id): boolean;
  /** Today's working set — the My Day filter (plan §2.2), reorderable. */
  myDayTasks: Accessor<Task[]>;

  // ── actions ──
  /** Load the account's task lists + the selected list's tasks (e10 → engine). */
  loadTasks(): Promise<void>;
  /** Focus a list (or `null` for all); reloads its tasks. */
  selectList(id: Id | null): Promise<void>;
  /** Create a task from an editor draft; returns the new id (or null on failure). */
  createTask(draft: TaskDraft): Promise<Id | null>;
  /** Patch an existing task (title/dates/priority/status/…). */
  updateTask(id: Id, patch: Partial<Task>): Promise<void>;
  /** Toggle a task between completed and needs-action (complete / reopen). */
  toggleComplete(id: Id): Promise<void>;
  /** Delete a task (and detach its subtasks locally). */
  deleteTask(id: Id): Promise<void>;
  /** Pin / unpin a task to My Day for `date` (default today). */
  setMyDay(id: Id, on: boolean, date?: string): Promise<void>;
  /** Reorder the My Day list to the given id order (client-side, drag-reorder). */
  reorderMyDay(orderedIds: Id[]): void;
  /** mail→task: create a task seeded from a message (plan §2.2 `fromEmail`). */
  convertMailToTask(emailId: Id, listId: Id, over?: Partial<TaskDraft>): Promise<Id | null>;
  /** event→task: create a task seeded from an event (plan §2.2 `fromEvent`). */
  convertEventToTask(eventId: Id, listId: Id, over?: Partial<TaskDraft>): Promise<Id | null>;
}

/** The date-only (`YYYY-MM-DD`) portion of a local date-time or date. */
export function dateOf(value: string): string {
  return value.slice(0, 10);
}

/** Today's date as a `YYYY-MM-DD` string in the local zone. */
export function todayDate(now: Date = new Date()): string {
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, '0');
  const d = String(now.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

/** Whether a task is "done" (completed or cancelled) — hidden from active views. */
export function isDone(task: Task): boolean {
  return task.status === 'completed' || task.status === 'cancelled';
}

/**
 * The My Day / Today membership test (plan §2.2): a task belongs to today when
 * it is explicitly pinned to today (`myDayDate === today`), or its due date is
 * today-or-overdue, or its start date is today-or-earlier. Completed/cancelled
 * tasks are excluded from the active working set.
 */
export function isMyDay(task: Task, today: string): boolean {
  if (isDone(task)) return false;
  if (task.myDayDate !== null && task.myDayDate === today) return true;
  if (task.due !== null && dateOf(task.due) <= today) return true;
  if (task.start !== null && dateOf(task.start) <= today) return true;
  return false;
}

export function createTasksSlice(ctx: SliceContext): TasksSlice {
  const { client, showToast } = ctx;

  const [tasks, setTasks] = createSignal<Task[]>([]);
  const [taskLists, setTaskLists] = createSignal<TaskList[]>([]);
  const [selectedListId, setSelectedListId] = createSignal<Id | null>(null);
  const [tasksLoading, setTasksLoading] = createSignal(false);
  // Client-side My Day ordering (drag-reorder). Tasks carry no order field in
  // the frozen shape, so ordering is a view concern persisted locally; e10 may
  // later back it with an engine field. Ids not present fall to the end.
  const [myDayOrder, setMyDayOrder] = createSignal<Id[]>([]);

  const accountId = (): string | null => client === undefined ? null : (currentAccount ?? null);

  // The tasks module resolves the account from the session on first load; we
  // cache it so subsequent mutations reuse it without another round-trip.
  let currentAccount: string | null = null;

  async function resolveAccount(): Promise<string | null> {
    if (currentAccount !== null) return currentAccount;
    const session = await client.session();
    const acct = session.primaryAccounts['urn:mailwoman:tasks'] ?? Object.keys(session.accounts)[0] ?? null;
    currentAccount = acct;
    return acct;
  }

  function patchTask(id: Id, patch: Partial<Task>): void {
    setTasks((ts) => ts.map((t) => (t.id === id ? { ...t, ...patch } : t)));
  }

  // A load answered after this client wrote can predate the write; `writes`
  // keeps the local version of such rows (see `localWrites.ts`), and
  // `applyLoad` keeps the old object for every unchanged row, so an open edit
  // form is not re-created by a pushed reload (t32-h11).
  const writes = new LocalWrites<Task>();
  // The list the shown rows were loaded for (`undefined` = all lists), so a row
  // that left the view because another list was picked is not taken for one
  // deleted elsewhere.
  let shownScope: Id | undefined | null = null;
  const [deletedElsewhere, setDeletedElsewhere] = createSignal<ReadonlySet<Id>>(new Set());

  function applyLoad(loaded: Task[], scope: Id | undefined, sentAt: number): void {
    const prev = tasks();
    const inScope = (t: Task): boolean => scope === undefined || t.listId === scope;
    const next = keepUnchanged(prev, writes.merge(prev, loaded, sentAt, inScope).rows);
    batch(() => {
      if (scope === shownScope) {
        const kept = new Set(next.map((t) => t.id));
        const gone = prev.filter((t) => !kept.has(t.id)).map((t) => t.id);
        if (gone.length > 0) setDeletedElsewhere((s) => new Set([...s, ...gone]));
      } else {
        setDeletedElsewhere(new Set<Id>());
      }
      shownScope = scope;
      setTasks(next);
    });
  }

  // A Task/get answer without a `list` (a stubbed or partial backend) is not
  // an empty list: taking it for one would mark every shown task deleted
  // elsewhere. The rows shown stay as they are.
  function applyAnswer(res: JmapResponse, scope: Id | undefined, sentAt: number): void {
    const list = responseFor<TaskGetResponse>(res, 'g').list as Task[] | undefined;
    if (Array.isArray(list)) applyLoad(list, scope, sentAt);
  }

  // Each `loadTasks()` / `selectList()` takes a ticket when it is called. Calls
  // overlap (opening Tasks starts two loads; a list click starts another), and
  // one that started earlier can answer last; only the call holding the latest
  // ticket writes state and ends the loading state, so the older answer is
  // dropped instead of replacing the newer one (as `loadTicket` in
  // `modules/calendar/controller.ts`).
  let loadTicket = 0;
  // Only `loadTasks()` fetches the lists, so a `selectList()` started meanwhile
  // must not drop them: the lists answer has its own ticket.
  let listsTicket = 0;

  async function loadTasks(): Promise<void> {
    const ticket = ++loadTicket;
    const stale = (): boolean => ticket !== loadTicket;
    const listsTurn = ++listsTicket;
    const acct = await resolveAccount();
    if (acct === null) return;
    if (!stale()) setTasksLoading(true);
    try {
      const listRes = await client.jmap(taskListsGet(acct));
      // A reply without a list (a stubbed or partial backend) is no task lists, not a crash.
      const cals = responseFor<CalendarGetResponse>(listRes, 'lists').list ?? [];
      const lists = cals
        .map((c) => taskListFromCalendar(c))
        .filter((l): l is TaskList => l !== null)
        .sort((a, b) => a.order - b.order || a.name.localeCompare(b.name));
      if (listsTurn === listsTicket) setTaskLists(lists);
      if (stale()) return;
      if (selectedListId() === null && lists.length > 0) {
        // Default focus is "all lists" (null); leave selection as-is.
      }
      const listId = selectedListId() ?? undefined;
      const sentAt = writes.loadSent();
      const taskRes = await client.jmap(tasksQueryGet(acct, listId));
      if (stale()) return;
      applyAnswer(taskRes, listId, sentAt);
    } finally {
      // The latest call ends the loading state; a dropped one leaves it alone.
      if (!stale()) setTasksLoading(false);
    }
  }

  async function selectList(id: Id | null): Promise<void> {
    const ticket = ++loadTicket;
    const stale = (): boolean => ticket !== loadTicket;
    setSelectedListId(id);
    const acct = await resolveAccount();
    if (acct === null || stale()) return;
    setTasksLoading(true);
    try {
      const sentAt = writes.loadSent();
      const res = await client.jmap(tasksQueryGet(acct, id ?? undefined));
      if (stale()) return;
      applyAnswer(res, id ?? undefined, sentAt);
    } finally {
      if (!stale()) setTasksLoading(false);
    }
  }

  function createResultId(res: JmapResponse): Id | null {
    const set = responseFor<TaskSetResponse>(res, 'set');
    if (set.notCreated?.['new'] !== undefined) {
      showToast('error', `Task rejected: ${set.notCreated['new'].type}`);
      return null;
    }
    return set.created?.['new']?.id ?? null;
  }

  async function doCreate(create: TaskCreate): Promise<Id | null> {
    const acct = await resolveAccount();
    if (acct === null) return null;
    const res = await client.jmap(taskSet(acct, { create: { new: create } }), SAVE);
    const id = createResultId(res);
    ctx.broadcastChange?.();
    // Reflect the create locally so the UI updates without a full reload; the
    // engine echoes the canonical row on the next load/push.
    if (id !== null) {
      const created = responseFor<TaskSetResponse>(res, 'set').created?.['new'];
      const optimistic: Task = {
        id,
        listId: create.listId ?? selectedListId() ?? '',
        uid: created?.uid ?? id,
        title: create.title ?? '',
        description: create.description ?? '',
        start: create.start ?? null,
        due: create.due ?? null,
        timeZone: create.timeZone ?? null,
        priority: create.priority ?? 0,
        percentComplete: create.percentComplete ?? 0,
        status: create.status ?? 'needs-action',
        progress: create.progress ?? '',
        recurrenceRules: create.recurrenceRules ?? [],
        parentId: create.parentId ?? null,
        myDayDate: create.myDayDate ?? null,
        etag: created?.etag ?? null,
        ...created,
      };
      // The server holds it now; a load sent before this answer may not.
      writes.begin(id, 'create')();
      setTasks((ts) => [...ts, optimistic]);
    }
    return id;
  }

  async function createTask(draft: TaskDraft): Promise<Id | null> {
    return doCreate({ ...draft });
  }

  async function updateTask(id: Id, patch: Partial<Task>): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) return;
    patchTask(id, patch);
    const answered = writes.begin(id, 'update', patch);
    try {
      await client.jmap(taskSet(acct, { update: { [id]: { ...patch } } }), SAVE);
    } finally {
      answered();
    }
    ctx.broadcastChange?.();
  }

  async function toggleComplete(id: Id): Promise<void> {
    const t = tasks().find((x) => x.id === id);
    if (t === undefined) return;
    const completing = !isDone(t);
    await updateTask(id, {
      status: completing ? 'completed' : 'needs-action',
      percentComplete: completing ? 100 : 0,
    });
  }

  async function deleteTask(id: Id): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) return;
    // Detach subtasks locally (they become roots) and drop the task.
    const answered = [
      writes.begin(id, 'destroy'),
      ...tasks()
        .filter((t) => t.parentId === id)
        .map((t) => writes.begin(t.id, 'update', { parentId: null })),
    ];
    batch(() => {
      setTasks((ts) =>
        ts
          .filter((t) => t.id !== id)
          .map((t) => (t.parentId === id ? { ...t, parentId: null } : t)),
      );
      setMyDayOrder((o) => o.filter((x) => x !== id));
    });
    try {
      await client.jmap(taskSet(acct, { destroy: [id] }), SAVE);
    } finally {
      for (const done of answered) done();
    }
    ctx.broadcastChange?.();
  }

  async function setMyDay(id: Id, on: boolean, date?: string): Promise<void> {
    const target = on ? (date ?? todayDate()) : null;
    await updateTask(id, { myDayDate: target });
    if (on) {
      // Newly-added My Day tasks land at the end of the drag order.
      setMyDayOrder((o) => (o.includes(id) ? o : [...o, id]));
    } else {
      setMyDayOrder((o) => o.filter((x) => x !== id));
    }
  }

  function reorderMyDay(orderedIds: Id[]): void {
    setMyDayOrder([...orderedIds]);
  }

  async function convertMailToTask(emailId: Id, listId: Id, over?: Partial<TaskDraft>): Promise<Id | null> {
    return doCreate({ listId, title: '', ...over, fromEmail: { emailId } });
  }

  async function convertEventToTask(eventId: Id, listId: Id, over?: Partial<TaskDraft>): Promise<Id | null> {
    return doCreate({ listId, title: '', ...over, fromEvent: { eventId } });
  }

  // ── derived views ──
  const rootTasks = createMemo<Task[]>(() => tasks().filter((t) => t.parentId === null));

  function subtasksOf(parentId: Id): Task[] {
    return tasks().filter((t) => t.parentId === parentId);
  }

  function taskDeletedElsewhere(id: Id): boolean {
    return deletedElsewhere().has(id);
  }

  const myDayTasks = createMemo<Task[]>(() => {
    const today = todayDate();
    const members = tasks().filter((t) => isMyDay(t, today));
    const order = myDayOrder();
    if (order.length === 0) return members;
    const rank = new Map(order.map((id, i) => [id, i]));
    return [...members].sort((a, b) => {
      const ra = rank.get(a.id) ?? Number.MAX_SAFE_INTEGER;
      const rb = rank.get(b.id) ?? Number.MAX_SAFE_INTEGER;
      return ra - rb;
    });
  });

  // Reference `accountId` so the unused-var lint is satisfied while keeping the
  // helper available for e10's engine-account wiring.
  void accountId;

  return {
    tasks,
    taskLists,
    selectedListId,
    tasksLoading,
    rootTasks,
    subtasksOf,
    taskDeletedElsewhere,
    myDayTasks,
    loadTasks,
    selectList,
    createTask,
    updateTask,
    toggleComplete,
    deleteTask,
    setMyDay,
    reorderMyDay,
    convertMailToTask,
    convertEventToTask,
  };
}
