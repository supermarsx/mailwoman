// The calendar module's reactive controller (plan §3 e4). It owns view state
// (current view + focused date), the loaded calendars + expanded event
// instances for the visible window, conflict detection, and every mutation
// (event CRUD, invite responses, calendar visibility/color, ICS import/export).
//
// It runs over a `CalendarBackend` — a `jmap`-shaped transport + account
// resolver — so it is identical against the in-memory mock (default, until e10)
// and the real engine surface (`mw-server`, wired by e10). All engine calls go
// through the frozen `Calendar/*` / `CalendarEvent/*` builders in `api.ts`.

import { batch, createMemo, createSignal, type Accessor } from 'solid-js';
import type { Id, JmapRequest, JmapResponse } from '../../api/jmap-types.ts';
import { responseFor } from '../../api/jmap.ts';
import type { Calendar, CalendarEvent } from '../../api/pim-types.ts';
import {
  calendarSet,
  calendarsGet,
  detectConflicts,
  eventRespond,
  eventSet,
  eventsExpand,
  eventsExport,
  eventsGetAll,
  eventsImport,
  freeBusy,
  type CalendarGetResponse,
  type CalendarSetResponse,
  type DetectConflictsResponse,
  type EventExpandResponse,
  type EventExportResponse,
  type EventGetResponse,
  type EventImportResponse,
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
  localeWeekStart,
  localToDate,
  startOfDay,
  startOfMonth,
  startOfWeek,
} from './datetime.ts';
import type { CalendarView, EventInstance } from './types.ts';

/** The transport the controller runs over (mock or the real engine). */
export interface CalendarBackend {
  jmap(body: JmapRequest): Promise<JmapResponse>;
  /** Resolve the account id (from the session), cached by the caller. */
  resolveAccount(): Promise<Id | null>;
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

  // ── event mutations ──
  createEvent(draft: EventDraft): Promise<Id | null>;
  updateEvent(id: Id, patch: Partial<CalendarEvent>): Promise<void>;
  deleteEvent(id: Id): Promise<void>;
  respond(eventId: Id, action: RespondAction, counter?: { start: string; duration: string }): Promise<void>;

  // ── calendar mutations ──
  toggleCalendar(id: Id): Promise<void>;
  setCalendarColor(id: Id, color: string): Promise<void>;
  createCalendar(name: string, color: string): Promise<Id | null>;
  deleteCalendar(id: Id): Promise<void>;
  shareCalendar(id: Id, principal: string, access: 'read' | 'readWrite'): Promise<void>;

  // ── ics / free-busy ──
  importIcs(calendarId: Id, ics: string): Promise<number>;
  exportIcs(opts: { calendarId?: Id; eventIds?: Id[] }): Promise<string>;
  queryFreeBusy(principals: string[], start: string, end: string): Promise<FreeBusyBlock[]>;
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
  const [calendars, setCalendars] = createSignal<Calendar[]>([]);
  const [masters, setMasters] = createSignal<CalendarEvent[]>([]);
  const [instances, setInstances] = createSignal<EventInstance[]>([]);
  const [view, setView] = createSignal<CalendarView>('week');
  const [focusDate, setFocusDate] = createSignal<Date>(startOfDay(new Date()));
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [conflictEventIds, setConflictEventIds] = createSignal<Set<Id>>(new Set());

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

  /** Join the engine's expanded instances onto the loaded masters + colors. */
  function buildInstances(allMasters: CalendarEvent[], expanded: ExpandedInstance[]): EventInstance[] {
    const byId = new Map(allMasters.map((m) => [m.id, m]));
    const colorByCal = new Map(calendars().map((c) => [c.id, c.color]));
    const out: EventInstance[] = [];
    for (const inst of expanded) {
      const master = byId.get(inst.eventId);
      if (master === undefined) continue;
      const start = localToDate(inst.instanceStart);
      const end = localToDate(inst.instanceEnd);
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
      const calRes = await backend.jmap(calendarsGet(acct));
      const cals = responseFor<CalendarGetResponse>(calRes, 'cals').list;
      setCalendars([...cals].sort((a, b) => a.order - b.order || a.name.localeCompare(b.name)));

      const w = window();
      const calIds = cals.map((c) => c.id);
      // The engine parses the expand/conflict window bounds with
      // `DateTime::parse_from_rfc3339`, which REQUIRES a zone designator — so send
      // the window wall-clock with a trailing `Z`. The module's naive wall-clock
      // time model (plan §1.12) is preserved: `localToDate` + the mock ignore the
      // suffix, so instances still render in the viewer's local zone.
      const startL = `${dateToLocal(w.start)}Z`;
      const endL = `${dateToLocal(w.end)}Z`;

      const expRes = await backend.jmap(eventsExpand(acct, calIds, startL, endL));
      const expanded = responseFor<EventExpandResponse>(expRes, 'x').list;
      const getRes = await backend.jmap(eventsGetAll(acct));
      const allMasters = responseFor<EventGetResponse>(getRes, 'g').list;
      const conRes = await backend.jmap(detectConflicts(acct, calIds, startL, endL));
      const conflicts = responseFor<DetectConflictsResponse>(conRes, 'conflicts').list;

      batch(() => {
        setMasters(allMasters);
        setInstances(buildInstances(allMasters, expanded));
        const ids = new Set<Id>();
        for (const p of conflicts) {
          ids.add(p.eventA);
          ids.add(p.eventB);
        }
        setConflictEventIds(ids);
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
    const create: Partial<CalendarEvent> = {
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
    };
    const res = await backend.jmap(eventSet(acct, { create: { new: create } }));
    const set = responseFor<EventSetResponse>(res, 'set');
    const id = set.created?.['new']?.id ?? null;
    await load();
    return id;
  }

  async function updateEvent(id: Id, patch: Partial<CalendarEvent>): Promise<void> {
    const acct = await backend.resolveAccount();
    if (acct === null) return;
    await backend.jmap(eventSet(acct, { update: { [id]: { ...patch } } }));
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
    const set = responseFor<CalendarSetResponse>(res, 'set');
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

  // ── ics / free-busy ──
  async function importIcs(calendarId: Id, ics: string): Promise<number> {
    const acct = await backend.resolveAccount();
    if (acct === null) return 0;
    const res = await backend.jmap(eventsImport(acct, calendarId, ics));
    const imp = responseFor<EventImportResponse>(res, 'import');
    await load();
    return imp.created.length;
  }

  async function exportIcs(opts: { calendarId?: Id; eventIds?: Id[] }): Promise<string> {
    const acct = await backend.resolveAccount();
    if (acct === null) return '';
    const res = await backend.jmap(eventsExport(acct, opts));
    return responseFor<EventExportResponse>(res, 'export').ics;
  }

  async function queryFreeBusy(principals: string[], start: string, end: string): Promise<FreeBusyBlock[]> {
    const acct = await backend.resolveAccount();
    if (acct === null) return [];
    const res = await backend.jmap(freeBusy(acct, principals, start, end));
    return responseFor<FreeBusyResponse>(res, 'fb').blocks;
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
    createEvent,
    updateEvent,
    deleteEvent,
    respond,
    toggleCalendar,
    setCalendarColor,
    createCalendar,
    deleteCalendar,
    shareCalendar,
    importIcs,
    exportIcs,
    queryFreeBusy,
  };
}
