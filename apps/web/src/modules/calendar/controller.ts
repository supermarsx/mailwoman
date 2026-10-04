// The calendar module's reactive controller (plan §3 e4). It owns view state
// (current view + focused date), the loaded calendars + expanded event
// instances for the visible window, conflict detection, and every mutation
// (event CRUD, invite responses, calendar visibility/color, ICS import/export).
//
// It runs over a `CalendarBackend` — a `jmap`-shaped transport + account
// resolver — so it is identical against the in-memory mock (tests) and the real
// engine surface (`state/slices/calendar.ts`). All engine calls go through the
// `Calendar/*` / `CalendarEvent/*` builders in `api.ts`, and every response is
// read with `pimResponse`, which turns the engine's method-level failure body
// into a thrown error instead of a result with missing fields.

import { batch, createMemo, createSignal, type Accessor } from 'solid-js';
import type { Id, JmapRequest, JmapResponse } from '../../api/jmap-types.ts';
import type { Calendar, CalendarEvent } from '../../api/pim-types.ts';
import {
  calendarSet,
  calendarsGet,
  detectConflicts,
  eventQuickAdd,
  eventRespond,
  eventSet,
  eventsExpand,
  eventsExport,
  eventsGetAll,
  eventsImport,
  eventsQueryByCategory,
  eventsQueryInCalendar,
  freeBusy,
  pimResponse,
  type CalendarGetResponse,
  type CalendarSetResponse,
  type ConflictPairResponse,
  type DetectConflictsResponse,
  type EventExpandResponse,
  type EventExportResponse,
  type EventGetResponse,
  type EventImportResponse,
  type EventQueryResponse,
  type EventQuickAddResponse,
  type EventSetResponse,
  type ExpandedInstance,
  type FreeBusyBlock,
  type FreeBusyResponse,
  type RespondAction,
} from './api.ts';
import {
  addDays,
  addMonths,
  dateToLocal,
  instanceBound,
  isZonedTimed,
  localeWeekStart,
  queryBounds,
  startOfDay,
  startOfMonth,
  startOfWeek,
} from './datetime.ts';
import { createCalendarFeeds, type CalendarFeeds } from './feeds.ts';
import type { CalendarRow, CalendarView, ConflictPair, EventAttachment, EventInstance } from './types.ts';

/** The transport the controller runs over (mock or the real engine). */
export interface CalendarBackend {
  jmap(body: JmapRequest): Promise<JmapResponse>;
  /** Resolve the account id (from the session), cached by the caller. */
  resolveAccount(): Promise<Id | null>;
  /**
   * Resolve the account identity — the session `username`, which is the address
   * the engine keys the user's own participant entry by.
   */
  resolveIdentity?(): Promise<string | null>;
  /**
   * The feed-subscription client. Omitted by the app, which then talks to the
   * server's sync-driver routes (`feeds.ts`); tests pass the mock's.
   */
  feeds?: CalendarFeeds;
}

/** The fields an event editor supplies on create/edit (a subset of the event). */
export interface EventDraft {
  calendarId: Id;
  title: string;
  description?: string;
  start: string;
  timeZone?: string | null;
  duration?: string;
  showWithoutTime?: boolean;
  locations?: Array<{ name: string }>;
  recurrenceRules?: Array<Record<string, unknown>>;
  excludedRecurrenceDates?: string[];
  status?: CalendarEvent['status'];
  freeBusyStatus?: CalendarEvent['freeBusyStatus'];
  participants?: CalendarEvent['participants'];
  alerts?: CalendarEvent['alerts'];
  /** Free-form category tags (P4). */
  categories?: string[];
  /** Blob/URI attachments (P5). */
  attachments?: EventAttachment[];
}

/** The inclusive-start / exclusive-end window a view displays. */
export interface ViewWindow {
  start: Date;
  end: Date;
}

/** The reactive surface the views + editor consume. */
export interface CalendarController {
  // ── state ──
  calendars: Accessor<Calendar[]>;
  masters: Accessor<CalendarEvent[]>;
  instances: Accessor<EventInstance[]>;
  view: Accessor<CalendarView>;
  focusDate: Accessor<Date>;
  loading: Accessor<boolean>;
  error: Accessor<string | null>;
  /** Ids of events with at least one overlapping instance in the window. */
  conflictEventIds: Accessor<Set<Id>>;
  /** The overlapping instance pairs in the window (for the resolver). */
  conflicts: Accessor<ConflictPair[]>;
  /** The active category filter (P4), or `null` when unfiltered. */
  categoryFilter: Accessor<string | null>;
  /** The account identity (known once `load()` has run), or `null`. */
  identity: Accessor<string | null>;

  // ── derived ──
  visibleCalendars: Accessor<Calendar[]>;
  visibleInstances: Accessor<EventInstance[]>;
  window: Accessor<ViewWindow>;
  masterById(id: Id): CalendarEvent | undefined;
  instancesForDay(day: Date): EventInstance[];
  hasConflict(eventId: Id): boolean;

  // ── navigation ──
  setView(v: CalendarView): void;
  goToday(): void;
  goPrev(): void;
  goNext(): void;
  goToDate(d: Date): void;

  // ── data ──
  load(): Promise<void>;

  // ── category filter (P4) ──
  /** Restrict the view to events carrying `category` (or clear with `null`). */
  setCategoryFilter(category: string | null): void;

  // ── event mutations ──
  /** Create an event. Rejects when the engine refuses it (`notCreated`). */
  createEvent(draft: EventDraft): Promise<Id | null>;
  /** Update an event. Rejects when the engine refuses it (`notUpdated`). */
  updateEvent(id: Id, patch: Partial<CalendarEvent>): Promise<void>;
  deleteEvent(id: Id): Promise<void>;
  respond(eventId: Id, action: RespondAction, counter?: { start: string; duration: string }): Promise<void>;
  /** Create an event from a natural-language line (P3). */
  quickAdd(text: string): Promise<Id | null>;

  // ── calendar mutations ──
  toggleCalendar(id: Id): Promise<void>;
  setCalendarColor(id: Id, color: string): Promise<void>;
  createCalendar(name: string, color: string): Promise<Id | null>;
  deleteCalendar(id: Id): Promise<void>;
  shareCalendar(id: Id, principal: string, access: 'read' | 'readWrite'): Promise<void>;
  /** Remove a principal's share grant from a calendar (P1). */
  unshareCalendar(id: Id, principal: string): Promise<void>;
  /**
   * Subscribe to an external ICS/webcal URL as a read-only overlay (P6). The
   * server fetches the feed; rejects when it cannot.
   */
  subscribeUrl(url: string, name?: string): Promise<Id | null>;
  /** Re-fetch a subscription calendar's feed and replace its events (P6). */
  refreshSubscription(calendarId: Id): Promise<void>;

  // ── ics / free-busy ──
  /** Import an ICS / `.hol` document; resolves to the number of events created. */
  importIcs(calendarId: Id, ics: string): Promise<number>;
  /** Export to one ICS document: the given events, one calendar's, or (default) all. */
  exportIcs(opts?: { calendarId?: Id; eventIds?: Id[] }): Promise<string>;
  /**
   * The signed-in account's own busy intervals overlapping the local range
   * `[start, end)`. The engine reports on no one else's calendars.
   */
  queryFreeBusy(start: Date, end: Date): Promise<FreeBusyBlock[]>;
}

/** Compute the [start,end) window a view needs expanded around `focus`. */
export function windowFor(view: CalendarView, focus: Date): ViewWindow {
  const day = startOfDay(focus);
  const ws = localeWeekStart();
  switch (view) {
    case 'day':
      return { start: day, end: addDays(day, 1) };
    case '3day':
      return { start: day, end: addDays(day, 3) };
    case 'work-week': {
      const mon = startOfWeek(focus, ws);
      return { start: mon, end: addDays(mon, 5) };
    }
    case 'week': {
      const start = startOfWeek(focus, ws);
      return { start, end: addDays(start, 7) };
    }
    case 'month': {
      const gridStart = startOfWeek(startOfMonth(focus), ws);
      return { start: gridStart, end: addDays(gridStart, 42) };
    }
    case 'tri-month': {
      const start = startOfMonth(addMonths(focus, -1));
      return { start, end: startOfMonth(addMonths(focus, 2)) };
    }
    case 'schedule':
    case 'agenda':
      return { start: day, end: addDays(day, 30) };
    case 'year':
      return { start: new Date(focus.getFullYear(), 0, 1), end: new Date(focus.getFullYear() + 1, 0, 1) };
    default:
      return { start: day, end: addDays(day, 1) };
  }
}

/**
 * Turn the engine's conflict pairs into the resolver's, for the window `w`.
 *
 * `Calendar/detectConflicts` pairs every two overlapping instances in the
 * account, all-day ones included, and stamps each pair with RFC3339 UTC bounds
 * (`calendar_detect_conflicts`, `crates/mw-engine/src/pim/calendars.rs:404-429`).
 * Three things happen here:
 *   - a pair whose master is not loaded is dropped;
 *   - a pair with an all-day event is dropped — an all-day entry (a birthday, a
 *     holiday) overlaps everything on its day and is not a double-booking the
 *     resolver's reschedule/shorten actions apply to;
 *   - the overlap bounds are decoded to viewer-local `LocalDateTime`s and pairs
 *     outside `w` (the query is padded — `queryBounds`) are dropped. Both bounds
 *     are taken from instance columns, so they are decoded by the later event's
 *     encoding (`overlapStart` is its start).
 */
export function conflictsInWindow(
  pairs: ConflictPairResponse[],
  masters: CalendarEvent[],
  w: ViewWindow,
): ConflictPair[] {
  const byId = new Map(masters.map((m) => [m.id, m]));
  const out: ConflictPair[] = [];
  for (const p of pairs) {
    const a = byId.get(p.eventA);
    const b = byId.get(p.eventB);
    if (a === undefined || b === undefined) continue;
    if (a.showWithoutTime || b.showWithoutTime) continue;
    const zoned = isZonedTimed(b);
    const start = instanceBound(p.overlapStart, zoned);
    const end = instanceBound(p.overlapEnd, zoned);
    if (end <= w.start || start >= w.end) continue;
    out.push({ a: p.eventA, b: p.eventB, overlapStart: dateToLocal(start), overlapEnd: dateToLocal(end) });
  }
  return out;
}

/**
 * Throw for a per-item `SetError` (`notCreated[key]` / `notUpdated[id]`). The
 * engine omits the `notX` maps when nothing failed (`SetOutcome::into_response`,
 * `crates/mw-engine/src/pim/mod.rs:103-122`), so an absent entry is success.
 */
function throwIfRefused(err: { type: string; description?: string | null } | undefined): void {
  if (err === undefined) return;
  throw new Error(`event not saved: ${err.type} ${err.description ?? ''}`.trim());
}

/** The step a prev/next navigation applies for a view. */
function navigate(view: CalendarView, focus: Date, dir: -1 | 1): Date {
  switch (view) {
    case 'day':
      return addDays(focus, dir);
    case '3day':
      return addDays(focus, dir * 3);
    case 'work-week':
    case 'week':
      return addDays(focus, dir * 7);
    case 'month':
      return addMonths(focus, dir);
    case 'tri-month':
      return addMonths(focus, dir * 3);
    case 'schedule':
    case 'agenda':
      return addDays(focus, dir * 30);
    case 'year':
      return new Date(focus.getFullYear() + dir, focus.getMonth(), focus.getDate());
    default:
      return addDays(focus, dir);
  }
}

export function createCalendarController(backend: CalendarBackend): CalendarController {
  const feeds = backend.feeds ?? createCalendarFeeds();
  const [calendars, setCalendars] = createSignal<Calendar[]>([]);
  const [masters, setMasters] = createSignal<CalendarEvent[]>([]);
  const [instances, setInstances] = createSignal<EventInstance[]>([]);
  const [view, setView] = createSignal<CalendarView>('week');
  const [focusDate, setFocusDate] = createSignal<Date>(startOfDay(new Date()));
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [conflictEventIds, setConflictEventIds] = createSignal<Set<Id>>(new Set());
  const [conflicts, setConflicts] = createSignal<ConflictPair[]>([]);
  const [categoryFilter, setCategoryFilterSig] = createSignal<string | null>(null);
  const [identity, setIdentity] = createSignal<string | null>(null);

  const window = createMemo<ViewWindow>(() => windowFor(view(), focusDate()));

  const visibleCalendars = createMemo(() => calendars().filter((c) => c.isVisible));

  const visibleInstances = createMemo(() => {
    const visible = new Set(visibleCalendars().map((c) => c.id));
    return instances().filter((i) => visible.has(i.event.calendarId));
  });

  function masterById(id: Id): CalendarEvent | undefined {
    return masters().find((m) => m.id === id);
  }

  function instancesForDay(dayInput: Date): EventInstance[] {
    const s = startOfDay(dayInput);
    const e = addDays(s, 1);
    return visibleInstances()
      .filter((i) => i.start < e && i.end > s)
      .sort((a, b) => a.start.getTime() - b.start.getTime());
  }

  function hasConflict(eventId: Id): boolean {
    return conflictEventIds().has(eventId);
  }

  /**
   * Join the engine's expanded instances onto the loaded masters + colors,
   * decoding each bound by its master's encoding and keeping only the instances
   * that overlap the view's window `w` (the query is padded — `queryBounds`).
   */
  function buildInstances(allMasters: CalendarEvent[], expanded: ExpandedInstance[], w: ViewWindow): EventInstance[] {
    const byId = new Map(allMasters.map((m) => [m.id, m]));
    const colorByCal = new Map(calendars().map((c) => [c.id, c.color]));
    const out: EventInstance[] = [];
    for (const inst of expanded) {
      const master = byId.get(inst.eventId);
      if (master === undefined) continue;
      const zoned = isZonedTimed(master);
      const start = instanceBound(inst.instanceStart, zoned);
      const end = instanceBound(inst.instanceEnd, zoned);
      if (end <= w.start || start >= w.end) continue;
      out.push({
        key: `${inst.eventId}:${start.getTime()}`,
        event: master,
        start,
        end,
        allDay: master.showWithoutTime,
        recurring: (master.recurrenceRules?.length ?? 0) > 0,
        color: colorByCal.get(master.calendarId) ?? '#3b82f6',
      });
    }
    return out.sort((a, b) => a.start.getTime() - b.start.getTime());
  }

  async function load(): Promise<void> {
    const acct = await backend.resolveAccount();
    if (acct === null) return;
    setLoading(true);
    setError(null);
    try {
      if (identity() === null && backend.resolveIdentity !== undefined) {
        setIdentity(await backend.resolveIdentity());
      }
      const calRes = await backend.jmap(calendarsGet(acct));
      // `Calendar/get` lists the account's task lists next to its event
      // calendars; only the latter belong in this module (see `CalendarRow`).
      const cals = pimResponse<CalendarGetResponse>(calRes, 'cals').list.filter(
        (c) => (c as CalendarRow).component !== 'VTODO',
      );
      setCalendars([...cals].sort((a, b) => a.order - b.order || a.name.localeCompare(b.name)));

      const w = window();
      const q = queryBounds(w.start, w.end);

      const expRes = await backend.jmap(eventsExpand(acct, q.start, q.end));
      const expanded = pimResponse<EventExpandResponse>(expRes, 'x').list;
      const getRes = await backend.jmap(eventsGetAll(acct));
      const allMasters = pimResponse<EventGetResponse>(getRes, 'g').list;
      const conRes = await backend.jmap(detectConflicts(acct, q.start, q.end));
      const conflicts = conflictsInWindow(pimResponse<DetectConflictsResponse>(conRes, 'conflicts').list, allMasters, w);

      // P4: when a category filter is active, ask the engine which event ids carry
      // it (`CalendarEvent/query` `categories` condition) and narrow the masters +
      // expanded instances to that set. Clearing the filter skips the query.
      const cat = categoryFilter();
      let masterList = allMasters;
      let instanceRows = expanded;
      if (cat !== null && cat !== '') {
        const qRes = await backend.jmap(eventsQueryByCategory(acct, cat));
        const matched = new Set(pimResponse<EventQueryResponse>(qRes, 'q').ids);
        masterList = allMasters.filter((m) => matched.has(m.id));
        instanceRows = expanded.filter((i) => matched.has(i.eventId));
      }

      batch(() => {
        setMasters(masterList);
        setInstances(buildInstances(masterList, instanceRows, w));
        const ids = new Set<Id>();
        for (const p of conflicts) {
          ids.add(p.a);
          ids.add(p.b);
        }
        setConflictEventIds(ids);
        setConflicts(conflicts);
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'failed to load calendar');
    } finally {
      setLoading(false);
    }
  }

  // ── navigation ──
  function goToday(): void {
    setFocusDate(startOfDay(new Date()));
    void load();
  }
  function goPrev(): void {
    setFocusDate((f) => navigate(view(), f, -1));
    void load();
  }
  function goNext(): void {
    setFocusDate((f) => navigate(view(), f, 1));
    void load();
  }
  function goToDate(d: Date): void {
    setFocusDate(startOfDay(d));
    void load();
  }
  function changeView(v: CalendarView): void {
    setView(v);
    void load();
  }

  // ── event mutations ──
  async function createEvent(draft: EventDraft): Promise<Id | null> {
    const acct = await backend.resolveAccount();
    if (acct === null) return null;
    // Built as the frozen shape plus the additive P4/P5 fields (categories /
    // attachments) the engine accepts on set; the cast keeps the frozen
    // `CalendarEvent` type untouched (see `CalendarEventExt`).
    const create: Partial<CalendarEvent> & { categories?: string[]; attachments?: EventAttachment[] } = {
      calendarId: draft.calendarId,
      title: draft.title,
      description: draft.description ?? '',
      start: draft.start,
      timeZone: draft.timeZone ?? null,
      duration: draft.duration ?? 'PT1H',
      showWithoutTime: draft.showWithoutTime ?? false,
      locations: draft.locations ?? [],
      recurrenceRules: draft.recurrenceRules ?? [],
      excludedRecurrenceDates: draft.excludedRecurrenceDates ?? [],
      status: draft.status ?? 'confirmed',
      freeBusyStatus: draft.freeBusyStatus ?? 'busy',
      participants: draft.participants ?? {},
      alerts: draft.alerts ?? {},
      categories: draft.categories ?? [],
      attachments: draft.attachments ?? [],
    };
    const res = await backend.jmap(eventSet(acct, { create: { new: create } }));
    const set = pimResponse<EventSetResponse>(res, 'set');
    throwIfRefused(set.notCreated?.['new']);
    const id = set.created?.['new']?.id ?? null;
    await load();
    return id;
  }

  async function updateEvent(id: Id, patch: Partial<CalendarEvent>): Promise<void> {
    const acct = await backend.resolveAccount();
    if (acct === null) return;
    const res = await backend.jmap(eventSet(acct, { update: { [id]: { ...patch } } }));
    throwIfRefused(pimResponse<EventSetResponse>(res, 'set').notUpdated?.[id]);
    await load();
  }

  async function deleteEvent(id: Id): Promise<void> {
    const acct = await backend.resolveAccount();
    if (acct === null) return;
    await backend.jmap(eventSet(acct, { destroy: [id] }));
    await load();
  }

  async function respond(
    eventId: Id,
    action: RespondAction,
    counter?: { start: string; duration: string },
  ): Promise<void> {
    const acct = await backend.resolveAccount();
    if (acct === null) return;
    await backend.jmap(eventRespond(acct, eventId, action, counter));
    await load();
  }

  async function quickAdd(text: string): Promise<Id | null> {
    const trimmed = text.trim();
    if (trimmed === '') return null;
    const acct = await backend.resolveAccount();
    if (acct === null) return null;
    // Default the target to the first visible calendar (else the first calendar);
    // the engine falls back to the primary calendar when none is passed.
    const target = visibleCalendars()[0]?.id ?? calendars()[0]?.id;
    const res = await backend.jmap(eventQuickAdd(acct, trimmed, target));
    const id = pimResponse<EventQuickAddResponse>(res, 'qa').created.id;
    await load();
    return id;
  }

  // ── category filter (P4) ──
  function setCategoryFilter(category: string | null): void {
    const next = category === null || category.trim() === '' ? null : category.trim();
    setCategoryFilterSig(next);
    void load();
  }

  // ── calendar mutations ──
  async function toggleCalendar(id: Id): Promise<void> {
    const acct = await backend.resolveAccount();
    if (acct === null) return;
    const cal = calendars().find((c) => c.id === id);
    if (cal === undefined) return;
    // Optimistic flip so the overlay toggles instantly; reload reconciles.
    setCalendars((cs) => cs.map((c) => (c.id === id ? { ...c, isVisible: !c.isVisible } : c)));
    await backend.jmap(calendarSet(acct, { update: { [id]: { isVisible: !cal.isVisible } } }));
  }

  async function setCalendarColor(id: Id, color: string): Promise<void> {
    const acct = await backend.resolveAccount();
    if (acct === null) return;
    setCalendars((cs) => cs.map((c) => (c.id === id ? { ...c, color } : c)));
    await backend.jmap(calendarSet(acct, { update: { [id]: { color } } }));
    await load();
  }

  async function createCalendar(name: string, color: string): Promise<Id | null> {
    const acct = await backend.resolveAccount();
    if (acct === null) return null;
    const res = await backend.jmap(
      calendarSet(acct, { create: { new: { name, color, isVisible: true, isSubscribed: true } } }),
    );
    const set = pimResponse<CalendarSetResponse>(res, 'set');
    const id = set.created?.['new']?.id ?? null;
    await load();
    return id;
  }

  async function deleteCalendar(id: Id): Promise<void> {
    const acct = await backend.resolveAccount();
    if (acct === null) return;
    await backend.jmap(calendarSet(acct, { destroy: [id] }));
    await load();
  }

  async function shareCalendar(id: Id, principal: string, access: 'read' | 'readWrite'): Promise<void> {
    const acct = await backend.resolveAccount();
    if (acct === null) return;
    const cal = calendars().find((c) => c.id === id);
    if (cal === undefined) return;
    const shareWith = [...cal.shareWith.filter((s) => s.principal !== principal), { principal, access }];
    await backend.jmap(calendarSet(acct, { update: { [id]: { shareWith } } }));
    await load();
  }

  async function unshareCalendar(id: Id, principal: string): Promise<void> {
    const acct = await backend.resolveAccount();
    if (acct === null) return;
    const cal = calendars().find((c) => c.id === id);
    if (cal === undefined) return;
    const shareWith = cal.shareWith.filter((s) => s.principal !== principal);
    await backend.jmap(calendarSet(acct, { update: { [id]: { shareWith } } }));
    await load();
  }

  // ── subscriptions (P6) ──
  async function subscribeUrl(url: string, name?: string): Promise<Id | null> {
    const trimmed = url.trim();
    if (trimmed === '') return null;
    const id = (await feeds.subscribe(trimmed, name)).created.id;
    await load();
    return id;
  }

  async function refreshSubscription(calendarId: Id): Promise<void> {
    // The sync driver re-fetches the URL the overlay was registered with.
    const url = calendars().find((c) => c.id === calendarId)?.caldavUrl ?? null;
    if (url === null) return;
    await feeds.refresh(calendarId, url);
    await load();
  }

  // ── ics / free-busy ──
  async function importIcs(calendarId: Id, ics: string): Promise<number> {
    const acct = await backend.resolveAccount();
    if (acct === null) return 0;
    const res = await backend.jmap(eventsImport(acct, calendarId, ics));
    const imp = pimResponse<EventImportResponse>(res, 'import');
    await load();
    return imp.count;
  }

  async function exportIcs(opts: { calendarId?: Id; eventIds?: Id[] } = {}): Promise<string> {
    const acct = await backend.resolveAccount();
    if (acct === null) return '';
    // `CalendarEvent/export` selects by event id only, so one calendar's export
    // is that calendar's ids, asked for with the single-calendar query filter.
    let ids: Id[] | null = opts.eventIds ?? null;
    if (opts.calendarId !== undefined) {
      const qRes = await backend.jmap(eventsQueryInCalendar(acct, opts.calendarId));
      const inCal = pimResponse<EventQueryResponse>(qRes, 'q').ids;
      ids = ids === null ? inCal : ids.filter((id) => inCal.includes(id));
    }
    const res = await backend.jmap(eventsExport(acct, ids));
    return pimResponse<EventExportResponse>(res, 'export').blob;
  }

  async function queryFreeBusy(start: Date, end: Date): Promise<FreeBusyBlock[]> {
    const acct = await backend.resolveAccount();
    if (acct === null) return [];
    const q = queryBounds(start, end);
    const res = await backend.jmap(freeBusy(acct, q.start, q.end));
    return pimResponse<FreeBusyResponse>(res, 'fb').list;
  }

  return {
    calendars,
    masters,
    instances,
    view,
    focusDate,
    loading,
    error,
    conflictEventIds,
    conflicts,
    categoryFilter,
    identity,
    visibleCalendars,
    visibleInstances,
    window,
    masterById,
    instancesForDay,
    hasConflict,
    setView: changeView,
    goToday,
    goPrev,
    goNext,
    goToDate,
    load,
    setCategoryFilter,
    createEvent,
    updateEvent,
    deleteEvent,
    respond,
    quickAdd,
    toggleCalendar,
    setCalendarColor,
    createCalendar,
    deleteCalendar,
    shareCalendar,
    unshareCalendar,
    subscribeUrl,
    refreshSubscription,
    importIcs,
    exportIcs,
    queryFreeBusy,
  };
}
