// In-memory mock backend for the calendar module. It implements the
// `Calendar/*` + `CalendarEvent/*` families over the same `JmapResponse`
// envelope the engine speaks, so the controller + views can be driven without a
// server. Tests drive this directly; the app itself runs over `client.jmap`
// (`state/slices/calendar.ts`).
//
// EVERY HANDLER HERE MIRRORS A SERVER HANDLER and says which one. The argument
// names it reads, the keys it returns and its failure shape are copied from
// `crates/mw-engine/src/pim/{calendars,events}.rs`, not from what `api.ts` or
// the controller would find convenient: a mock that implements the client's
// idea of the contract passes every test while the app fails against the real
// server (which is how `Calendar/freeBusy` shipped reading a `blocks` key the
// engine never sent). `contract.test.ts` holds this file to fixtures taken
// from the server code. When a handler changes there, change it here.

import type { JmapRequest, JmapResponse, JmapSession, Invocation } from '../../api/jmap-types.ts';
import type { Calendar, CalendarEvent } from '../../api/pim-types.ts';
import { dateToCalDate, dateToLocal, isZonedTimed, localToDate } from './datetime.ts';
import { expandEvent } from './recurrence.ts';
import type {
  CalendarGetResponse,
  CalendarRefreshResponse,
  CalendarSubscribeResponse,
  DetectConflictsResponse,
  EventExpandResponse,
  EventExportResponse,
  EventGetResponse,
  EventImportResponse,
  EventQueryResponse,
  EventQuickAddResponse,
  ExpandedInstance,
  FreeBusyBlock,
  FreeBusyResponse,
  ConflictPairResponse,
} from './api.ts';
import { FeedError, type CalendarFeeds } from './feeds.ts';
import type { CalendarEventExt, CalendarRow } from './types.ts';

const ACCOUNT = 'acct-mock';

/**
 * The seeded collections: a personal (default) calendar, a work calendar in a
 * second color, and the default task list. The engine seeds a VTODO list next to
 * the default calendar and `Calendar/get` returns both, told apart by
 * `component` (`seed_default_collections` + `calendar_row_to_json`,
 * `crates/mw-engine/src/pim/calendars.rs:471-480,518-531`).
 */
export function seedCalendars(): CalendarRow[] {
  return [
    {
      id: 'cal-personal',
      name: 'Personal',
      color: '#3b82f6',
      order: 0,
      isVisible: true,
      isSubscribed: true,
      role: 'default',
      shareWith: [],
      caldavUrl: null,
      syncToken: null,
      isReadOnlyOverlay: false,
      component: 'VEVENT',
    },
    {
      id: 'cal-work',
      name: 'Work',
      color: '#ef4444',
      order: 1,
      isVisible: true,
      isSubscribed: true,
      role: null,
      shareWith: [{ principal: 'team@example.com', access: 'read' }],
      caldavUrl: 'https://dav.example.com/cal/work',
      syncToken: 'sync-1',
      isReadOnlyOverlay: false,
      component: 'VEVENT',
    },
    {
      id: 'list-tasks',
      name: 'Tasks',
      color: '#8855ff',
      order: 0,
      isVisible: true,
      isSubscribed: true,
      role: 'default',
      shareWith: [],
      caldavUrl: null,
      syncToken: null,
      isReadOnlyOverlay: false,
      component: 'VTODO',
    },
  ];
}

function baseEvent(over: Partial<CalendarEvent> & Pick<CalendarEvent, 'id' | 'calendarId' | 'title' | 'start'>): CalendarEvent {
  return {
    uid: over.id,
    description: '',
    locations: [],
    timeZone: 'Europe/London',
    duration: 'PT1H',
    showWithoutTime: false,
    recurrenceRules: [],
    recurrenceOverrides: {},
    excludedRecurrenceDates: [],
    status: 'confirmed',
    priority: 0,
    freeBusyStatus: 'busy',
    participants: {},
    alerts: {},
    sequence: 0,
    etag: null,
    ...over,
  };
}

/** A seeded event set anchored on a supplied "today" so views have content. */
export function seedEvents(today = new Date()): CalendarEvent[] {
  const y = today.getFullYear();
  const m = today.getMonth();
  const d = today.getDate();
  const at = (day: number, h: number, mi = 0): string =>
    dateToLocal(new Date(y, m, day, h, mi, 0));
  return [
    baseEvent({ id: 'ev-standup', calendarId: 'cal-work', title: 'Daily standup', start: at(d, 9, 30), duration: 'PT30M',
      recurrenceRules: [{ frequency: 'weekly', byDay: ['mo', 'tu', 'we', 'th', 'fr'] }] }),
    baseEvent({ id: 'ev-lunch', calendarId: 'cal-personal', title: 'Lunch', start: at(d, 12, 0), duration: 'PT1H' }),
    baseEvent({ id: 'ev-review', calendarId: 'cal-work', title: 'Design review', start: at(d, 12, 30), duration: 'PT1H',
      participants: {
        me: { name: 'Me', email: 'me@example.com', role: 'attendee', participationStatus: 'needs-action', expectReply: true },
        org: { name: 'Organizer', email: 'boss@example.com', role: 'owner', participationStatus: 'accepted', expectReply: false },
      },
      status: 'tentative' }),
    baseEvent({ id: 'ev-oneon', calendarId: 'cal-work', title: '1:1', start: at(d + 1, 15, 0), duration: 'PT30M' }),
    baseEvent({ id: 'ev-allday', calendarId: 'cal-personal', title: 'Conference', start: dateToLocal(new Date(y, m, d + 2)).slice(0, 10),
      showWithoutTime: true, duration: 'P1D' }),
    baseEvent({ id: 'ev-birthday', calendarId: 'cal-personal', title: 'Birthday', start: dateToLocal(new Date(y, m, 15)).slice(0, 10),
      showWithoutTime: true, duration: 'P1D', recurrenceRules: [{ frequency: 'yearly' }] }),
  ];
}

/** Mutable in-memory state the mock dispatches over. */
export interface MockStore {
  calendars: CalendarRow[];
  events: CalendarEvent[];
}

export function createMockStore(today = new Date()): MockStore {
  return { calendars: seedCalendars(), events: seedEvents(today) };
}

function colorFor(store: MockStore, calendarId: string): string {
  return store.calendars.find((c) => c.id === calendarId)?.color ?? '#3b82f6';
}

let idSeq = 1000;
function nextId(prefix: string): string {
  idSeq += 1;
  return `${prefix}-${idSeq}`;
}

function ok(callId: string, name: string, args: unknown): Invocation {
  return [name, args, callId] as unknown as Invocation;
}

/**
 * A method-level failure, exactly as the engine sends one: the method name is
 * echoed with a `{type, description}` body — NOT an RFC 8620 `error` invocation
 * (`server_fail`, `crates/mw-engine/src/pim/mod.rs:79-81`; envelope
 * `crates/mw-engine/src/jmap.rs:132`).
 */
function fail(callId: string, name: string, description: string): Invocation {
  return ok(callId, name, { type: 'serverFail', description });
}

/**
 * One occurrence bound in the engine's wire form (`local_to_utc`,
 * `crates/mw-ics/src/recur.rs:47-63`): RFC3339 UTC, a true instant for an event
 * with a `timeZone` and the wall clock stamped `Z` for a floating or all-day
 * one. The mock has no zone database, so it takes a zoned event's wall clock to
 * be in the viewer's zone — which is the zone the editor stamps on every event
 * it creates.
 */
function wire(d: Date, ev: CalendarEvent): string {
  return isZonedTimed(ev) ? d.toISOString().replace(/\.\d{3}Z$/, 'Z') : `${dateToLocal(d)}Z`;
}

/** The mock's event calendars — `Calendar/get` also lists VTODO task lists. */
function eventCalendarIds(store: MockStore): string[] {
  return store.calendars.filter((c) => (c as CalendarRow).component !== 'VTODO').map((c) => c.id);
}

/** Every event as the engine walks them: per event calendar, in store order. */
function allEvents(store: MockStore): CalendarEvent[] {
  const cals = eventCalendarIds(store);
  return store.events.filter((e) => cals.includes(e.calendarId));
}

/** `wanted_ids` (`pim/mod.rs:154-161`): an `ids` ARRAY restricts; anything else means all. */
function wantedIds(args: Record<string, unknown>): string[] | null {
  const ids = args['ids'];
  return Array.isArray(ids) ? ids.filter((x): x is string => typeof x === 'string') : null;
}

/** A naive stand-in for `mw-ics` parsing: one event per VEVENT block. */
function parseIcsEvents(blob: string): Array<{ uid?: string; title?: string; start?: string; allDay: boolean }> {
  const out: Array<{ uid?: string; title?: string; start?: string; allDay: boolean }> = [];
  for (const block of blob.split('BEGIN:VEVENT').slice(1)) {
    const body = block.split('END:VEVENT')[0] ?? '';
    const prop = (name: string): string | undefined =>
      new RegExp(`^${name}(?:;[^:\\r\\n]*)?:(.*)$`, 'm').exec(body)?.[1]?.trim();
    const dt = prop('DTSTART');
    const m = dt === undefined ? null : /^(\d{4})(\d{2})(\d{2})(?:T(\d{2})(\d{2})(\d{2}))?/.exec(dt);
    const ev: { uid?: string; title?: string; start?: string; allDay: boolean } = { allDay: m !== null && m[4] === undefined };
    const uid = prop('UID');
    const title = prop('SUMMARY');
    if (uid !== undefined) ev.uid = uid;
    if (title !== undefined) ev.title = title;
    if (m !== null) {
      ev.start = m[4] === undefined ? `${m[1]}-${m[2]}-${m[3]}` : `${m[1]}-${m[2]}-${m[3]}T${m[4]}:${m[5]}:${m[6]}`;
    }
    out.push(ev);
  }
  return out;
}

/** Persist parsed events into a calendar, returning the created ids
 *  (`persist_parsed_events`, `events.rs:524-559`). */
function persistParsed(store: MockStore, calendarId: string, blob: string): string[] {
  const created: string[] = [];
  parseIcsEvents(blob).forEach((p, i) => {
    const id = nextId('ev');
    store.events.push(
      baseEvent({
        id,
        calendarId,
        title: p.title ?? `Imported ${i + 1}`,
        start: p.start ?? dateToLocal(new Date()),
        ...(p.uid !== undefined ? { uid: p.uid } : {}),
        ...(p.allDay ? { showWithoutTime: true, duration: 'P1D', timeZone: null } : {}),
      }),
    );
    created.push(id);
  });
  return created;
}

/** The engine's PIM set-response envelope (`SetOutcome::into_response`, `pim/mod.rs:103-122`):
 *  `created` / `updated` / `destroyed` are always present; the `notX` maps are
 *  omitted when empty. */
function setResponse(created: Record<string, unknown>, updated: Record<string, unknown>, destroyed: string[]): unknown {
  return { accountId: ACCOUNT, oldState: '1', newState: '2', created, updated, destroyed };
}

/** Dispatch one method call against the store, returning its `Invocation`. */
function dispatch(store: MockStore, call: Invocation): Invocation {
  const [name, rawArgs, callId] = call;
  const args = rawArgs as Record<string, unknown>;
  switch (name) {
    case 'Calendar/get': {
      // `calendar_get` (`calendars.rs:19-52`): every collection of the account,
      // task lists included, each row carrying its `component`.
      const res: CalendarGetResponse = { accountId: ACCOUNT, state: '1', list: store.calendars, notFound: [] };
      return ok(callId, name, res);
    }
    case 'Calendar/set': {
      // `calendar_set` (`calendars.rs:56-117`).
      const created: Record<string, Partial<Calendar> & { id: string }> = {};
      const updated: Record<string, unknown> = {};
      const destroyed: string[] = [];
      for (const [key, val] of Object.entries((args['create'] as Record<string, Partial<Calendar>>) ?? {})) {
        const id = nextId('cal');
        const cal: CalendarRow = {
          id, name: val.name ?? 'Calendar', color: val.color ?? '#3366ff', order: val.order ?? 0,
          isVisible: val.isVisible ?? true, isSubscribed: true, role: val.role ?? null,
          shareWith: val.shareWith ?? [], caldavUrl: val.caldavUrl ?? null, syncToken: null,
          isReadOnlyOverlay: val.isReadOnlyOverlay ?? false,
          component: (val as Partial<CalendarRow>).component ?? 'VEVENT',
        };
        store.calendars.push(cal);
        created[key] = { id };
      }
      for (const [id, patch] of Object.entries((args['update'] as Record<string, Partial<Calendar>>) ?? {})) {
        store.calendars = store.calendars.map((c) => (c.id === id ? { ...c, ...patch } : c));
        updated[id] = null;
      }
      for (const id of (args['destroy'] as string[]) ?? []) {
        store.calendars = store.calendars.filter((c) => c.id !== id);
        destroyed.push(id);
      }
      return ok(callId, name, setResponse(created, updated, destroyed));
    }
    case 'CalendarEvent/get': {
      // `event_get` (`events.rs:24-46`).
      const ids = wantedIds(args);
      const all = allEvents(store);
      const list = ids === null ? all : all.filter((e) => ids.includes(e.id));
      const notFound = ids === null ? [] : ids.filter((id) => !all.some((e) => e.id === id));
      const res: EventGetResponse = { accountId: ACCOUNT, state: '1', list, notFound };
      return ok(callId, name, res);
    }
    case 'CalendarEvent/expand': {
      // `event_expand` (`events.rs:449-480`): `start` + `end` are required, `ids`
      // is the only filter (there is no calendar filter), and each row is the
      // master projection plus `eventId` / `instanceStart` / `instanceEnd`.
      if (typeof args['start'] !== 'string' || typeof args['end'] !== 'string') {
        return fail(callId, name, 'CalendarEvent/expand requires start + end (RFC3339 UTC)');
      }
      const start = localToDate(args['start']);
      const end = localToDate(args['end']);
      const ids = wantedIds(args);
      const masters = allEvents(store).filter((e) => ids === null || ids.includes(e.id));
      const list: ExpandedInstance[] = [];
      for (const ev of masters) {
        for (const inst of expandEvent(ev, start, end, colorFor(store, ev.calendarId))) {
          list.push({ ...ev, eventId: ev.id, instanceStart: wire(inst.start, ev), instanceEnd: wire(inst.end, ev) });
        }
      }
      const res: EventExpandResponse = { accountId: ACCOUNT, list };
      return ok(callId, name, res);
    }
    case 'CalendarEvent/set': {
      // `event_set` (`events.rs:50-106`).
      const created: Record<string, Partial<CalendarEvent> & { id: string }> = {};
      const updated: Record<string, unknown> = {};
      const destroyed: string[] = [];
      for (const [key, val] of Object.entries((args['create'] as Record<string, Partial<CalendarEvent>>) ?? {})) {
        const id = nextId('ev');
        const ev = baseEvent({
          id,
          calendarId: val.calendarId ?? store.calendars[0]!.id,
          title: val.title ?? '(no title)',
          start: val.start ?? dateToLocal(new Date()),
          ...val,
        });
        store.events.push(ev);
        created[key] = { id };
      }
      for (const [id, patch] of Object.entries((args['update'] as Record<string, Partial<CalendarEvent>>) ?? {})) {
        store.events = store.events.map((e) => (e.id === id ? { ...e, ...patch, sequence: e.sequence + 1 } : e));
        updated[id] = null;
      }
      for (const id of (args['destroy'] as string[]) ?? []) {
        store.events = store.events.filter((e) => e.id !== id);
        destroyed.push(id);
      }
      return ok(callId, name, setResponse(created, updated, destroyed));
    }
    case 'CalendarEvent/respond': {
      // `event_respond` (`events.rs:589-674`) updates the participant keyed by
      // the account identity and returns the reloaded event under `updated`.
      // The mock's own participant is the seeded `me` key.
      const eventId = args['eventId'] as string;
      const action = args['action'] as string;
      const statusMap: Record<string, CalendarEvent['participants'][string]['participationStatus']> = {
        accept: 'accepted', decline: 'declined', tentative: 'tentative', counter: 'tentative',
      };
      if (!store.events.some((e) => e.id === eventId)) return fail(callId, name, `unknown event ${eventId}`);
      if (statusMap[action] === undefined) return fail(callId, name, `unknown respond action ${action}`);
      store.events = store.events.map((e) => {
        if (e.id !== eventId) return e;
        const participants = { ...e.participants };
        if (participants['me'] !== undefined) {
          participants['me'] = { ...participants['me'], participationStatus: statusMap[action]! };
        }
        return { ...e, participants, sequence: e.sequence + 1 };
      });
      return ok(callId, name, { accountId: ACCOUNT, updated: store.events.find((e) => e.id === eventId) ?? null });
    }
    case 'Calendar/detectConflicts': {
      // `calendar_detect_conflicts` (`calendars.rs:391-430`): reads only `start`
      // and `end` (no calendar filter, and all-day instances are not excluded);
      // walks the instances in start order and pairs each with the later ones
      // that begin before it ends. `overlapStart` is the later instance's start.
      const start = localToDate((args['start'] as string | undefined) ?? '2000-01-01T00:00:00Z');
      const end = localToDate((args['end'] as string | undefined) ?? '2100-01-01T00:00:00Z');
      const flat = allEvents(store)
        .flatMap((e) => expandEvent(e, start, end, '#000'))
        .map((inst) => ({ id: inst.event.id, start: wire(inst.start, inst.event), end: wire(inst.end, inst.event) }))
        .sort((a, b) => (a.start < b.start ? -1 : a.start > b.start ? 1 : 0));
      const list: ConflictPairResponse[] = [];
      for (let i = 0; i < flat.length; i += 1) {
        for (let j = i + 1; j < flat.length; j += 1) {
          const a = flat[i]!;
          const b = flat[j]!;
          if (a.id === b.id) continue;
          if (b.start >= a.end) break;
          list.push({ eventA: a.id, eventB: b.id, overlapStart: b.start, overlapEnd: a.end < b.end ? a.end : b.end });
        }
      }
      const res: DetectConflictsResponse = { accountId: ACCOUNT, list };
      return ok(callId, name, res);
    }
    case 'Calendar/freeBusy': {
      // `calendar_free_busy` (`calendars.rs:353-387`) + `aggregate_free_busy`
      // (`crates/mw-ics/src/freebusy.rs:31-76`): `start` + `end` are required;
      // the optional `calendarIds` is the only filter (`principals` is not read);
      // the account's own events are expanded, `freeBusyStatus: "free"` ones
      // skipped, and overlapping or touching intervals merged — a merged block
      // is `tentative` only when every contributor was. Rows are
      // `{start, end, status}` under `list`, with no principal.
      if (typeof args['start'] !== 'string' || typeof args['end'] !== 'string') {
        return fail(callId, name, 'Calendar/freeBusy requires start + end (RFC3339 UTC)');
      }
      const start = localToDate(args['start']);
      const end = localToDate(args['end']);
      const wanted = Array.isArray(args['calendarIds']) ? (args['calendarIds'] as string[]) : null;
      const raw: Array<{ start: string; end: string; tentative: boolean }> = [];
      for (const ev of allEvents(store)) {
        if (wanted !== null && !wanted.includes(ev.calendarId)) continue;
        if (ev.freeBusyStatus === 'free') continue;
        for (const inst of expandEvent(ev, start, end, '#000')) {
          raw.push({ start: wire(inst.start, ev), end: wire(inst.end, ev), tentative: ev.status === 'tentative' });
        }
      }
      raw.sort((a, b) => (a.start < b.start ? -1 : a.start > b.start ? 1 : 0));
      const list: FreeBusyBlock[] = [];
      for (const r of raw) {
        const last = list[list.length - 1];
        if (last !== undefined && r.start <= last.end) {
          if (r.end > last.end) last.end = r.end;
          if (!r.tentative) last.status = 'busy';
        } else {
          list.push({ start: r.start, end: r.end, status: r.tentative ? 'tentative' : 'busy' });
        }
      }
      const res: FreeBusyResponse = { accountId: ACCOUNT, list };
      return ok(callId, name, res);
    }
    case 'CalendarEvent/import': {
      // `event_import` (`events.rs:494-517`): the document is read from `blob`;
      // the target defaults to the default event calendar; the result is
      // `{imported: [ids], count}`. A blob that is neither a VCALENDAR nor a
      // `.hol` pack is a parse failure (`parse_calendar_blob`, `events.rs:938-955`).
      const blob = typeof args['blob'] === 'string' ? args['blob'] : '';
      if (!blob.includes('BEGIN:VCALENDAR')) return fail(callId, name, 'ics: not a calendar document');
      const calendarId = (args['calendarId'] as string | undefined) ?? eventCalendarIds(store)[0]!;
      const imported = persistParsed(store, calendarId, blob);
      const res: EventImportResponse = { accountId: ACCOUNT, imported, count: imported.length };
      return ok(callId, name, res);
    }
    case 'CalendarEvent/query': {
      // `event_query_ids` (`events.rs:337-409`): `filter.calendarId` (or
      // `inCalendar`) is ONE calendar id; `filter.categories` (or `category`)
      // keeps events sharing ANY wanted category, compared case-insensitively
      // (`filter_events_by_categories`, `events.rs:748-777`). Envelope:
      // `query_response`, `pim/mod.rs:141-150`.
      const filter = (args['filter'] as Record<string, unknown> | undefined) ?? {};
      const cal = filter['calendarId'] ?? filter['inCalendar'];
      const calendarId = typeof cal === 'string' ? cal : null;
      const wantCats = (
        Array.isArray(filter['categories'])
          ? (filter['categories'] as unknown[]).filter((x): x is string => typeof x === 'string')
          : typeof filter['category'] === 'string'
            ? [filter['category']]
            : []
      ).map((s) => s.toLowerCase());
      const ids = allEvents(store)
        .filter((e) => calendarId === null || e.calendarId === calendarId)
        .filter((e) => {
          if (wantCats.length === 0) return true;
          const have = ((e as CalendarEventExt).categories ?? []).map((s) => s.toLowerCase());
          return have.some((c) => wantCats.includes(c));
        })
        .map((e) => e.id);
      const res: EventQueryResponse & { canCalculateChanges: boolean } = {
        accountId: ACCOUNT, queryState: '1', ids, total: ids.length, position: 0, canCalculateChanges: true,
      };
      return ok(callId, name, res);
    }
    case 'CalendarEvent/quickAdd': {
      // `event_quick_add` (`events.rs:682-742`): empty text is a failure; success
      // is `{created: {id}, parsed: {...}}`. The mock does not parse the phrase —
      // it takes the "no day/time recognized" branch (`events.rs:709-719`): the
      // whole text as the title of an all-day event today.
      const text = String(args['text'] ?? '').trim();
      if (text === '') return fail(callId, name, 'CalendarEvent/quickAdd requires a non-empty text');
      const id = nextId('ev');
      store.events.push(
        baseEvent({
          id,
          calendarId: (args['calendarId'] as string | undefined) ?? eventCalendarIds(store)[0]!,
          title: text,
          start: dateToCalDate(new Date()),
          showWithoutTime: true,
          duration: 'P1D',
          timeZone: null,
        }),
      );
      const res: EventQuickAddResponse = {
        accountId: ACCOUNT,
        created: { id },
        parsed: { title: text, start: null, duration: 'PT1H', allDay: false, location: null },
      };
      return ok(callId, name, res);
    }
    case 'Calendar/subscribe': {
      // `calendar_subscribe` (`calendars.rs:234-297`): `url` is required;
      // `webcal(s)://` is stored as `https://`; the overlay is populated only
      // from a supplied `blob` (the engine fetches nothing); the result is
      // `{created: {id}, url, imported}`.
      const url = typeof args['url'] === 'string' ? args['url'].trim() : '';
      if (url === '') return fail(callId, name, 'Calendar/subscribe requires a non-empty url');
      const normalized = url.replace(/^webcals?:\/\//, 'https://');
      const id = nextId('cal');
      const cal: CalendarRow = {
        id,
        name: typeof args['name'] === 'string' && args['name'] !== '' ? args['name'] : 'Subscription',
        color: (args['color'] as string | undefined) ?? '#3366ff',
        order: 0,
        isVisible: true,
        isSubscribed: true,
        role: null,
        shareWith: [],
        caldavUrl: normalized,
        syncToken: null,
        isReadOnlyOverlay: true,
        component: 'VEVENT',
      };
      store.calendars.push(cal);
      const imported = typeof args['blob'] === 'string' ? persistParsed(store, id, args['blob']).length : 0;
      const res: CalendarSubscribeResponse = { accountId: ACCOUNT, created: { id }, url: normalized, imported };
      return ok(callId, name, res);
    }
    case 'Calendar/refreshSubscription': {
      // `calendar_refresh_subscription` (`calendars.rs:301-349`): `calendarId`
      // and a non-empty `blob` are required, the target must be an overlay, and
      // its events are REPLACED by the parsed blob. Result: `{calendarId, imported}`.
      const calendarId = typeof args['calendarId'] === 'string' ? args['calendarId'] : null;
      if (calendarId === null) return fail(callId, name, 'Calendar/refreshSubscription requires calendarId');
      const blob = typeof args['blob'] === 'string' ? args['blob'] : '';
      if (blob === '') return fail(callId, name, 'Calendar/refreshSubscription requires the fetched blob');
      const cal = store.calendars.find((c) => c.id === calendarId);
      if (cal === undefined) return fail(callId, name, `unknown calendar ${calendarId}`);
      if (!cal.isReadOnlyOverlay) return fail(callId, name, 'target calendar is not a subscription overlay');
      store.events = store.events.filter((e) => e.calendarId !== calendarId);
      const imported = persistParsed(store, calendarId, blob).length;
      const res: CalendarRefreshResponse = { accountId: ACCOUNT, calendarId, imported };
      return ok(callId, name, res);
    }
    case 'CalendarEvent/export': {
      // `event_export` (`events.rs:568-585`): `ids` (an array) selects events,
      // anything else exports all of them; there is no calendar filter; the
      // document comes back under `blob`, CRLF-delimited.
      const ids = wantedIds(args);
      const events = ids === null ? allEvents(store) : ids.flatMap((id) => store.events.filter((e) => e.id === id));
      let blob = 'BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Mailwoman//EN\r\n';
      for (const e of events) blob += `BEGIN:VEVENT\r\nUID:${e.uid}\r\nSUMMARY:${e.title}\r\nEND:VEVENT\r\n`;
      blob += 'END:VCALENDAR\r\n';
      const res: EventExportResponse = { accountId: ACCOUNT, blob };
      return ok(callId, name, res);
    }
    default:
      // `dispatch_pim` (`pim/dispatch.rs:115-118`).
      return ok(callId, name, { type: 'unknownMethod', description: `engine does not implement PIM method ${name}` });
  }
}

/** A minimal session advertising the calendar capability + the mock account. */
export function mockSession(): JmapSession {
  return {
    capabilities: {},
    accounts: { [ACCOUNT]: { name: 'Mock', isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
    primaryAccounts: { 'urn:mailwoman:calendars': ACCOUNT },
    username: 'mock@example.com',
    apiUrl: '/jmap/api',
    downloadUrl: '',
    uploadUrl: '',
    eventSourceUrl: '',
    state: '0',
  } as unknown as JmapSession;
}

/**
 * A `jmap`-compatible handler over an in-memory store. Wrap it with
 * `mockCalendarClient()` for the slice, or call it directly in tests.
 */
export function createMockJmap(store: MockStore): (body: JmapRequest) => Promise<JmapResponse> {
  return (body: JmapRequest): Promise<JmapResponse> =>
    Promise.resolve({ methodResponses: body.methodCalls.map((c) => dispatch(store, c)) } as JmapResponse);
}

/**
 * The mock of the server's webcal sync driver (`feeds.ts`;
 * `crates/mw-server/src/import_routes.rs:432-476,525-546`): "fetch" the feed
 * body, forward it to the engine method as `blob`, and turn a method-level
 * failure into a non-2xx. `fetchFeed` stands in for the network; returning
 * `null` is a failed fetch.
 */
export function createMockFeeds(
  store: MockStore,
  fetchFeed: (url: string) => string | null = () => 'BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n',
): CalendarFeeds {
  function forward<T>(method: string, args: Record<string, unknown>): T {
    const body = dispatch(store, [method, { accountId: ACCOUNT, ...args }, 'cal0'] as unknown as Invocation)[1] as Record<string, unknown>;
    if (typeof body['type'] === 'string') throw new FeedError(502, `${method} failed: ${body['type']}`);
    return body as T;
  }
  function fetched(url: string): string {
    const blob = fetchFeed(url);
    if (blob === null) throw new FeedError(502, `fetch failed: ${url}`);
    return blob;
  }
  return {
    subscribe: (url, name) =>
      Promise.resolve().then(() =>
        forward<CalendarSubscribeResponse>('Calendar/subscribe', { url, blob: fetched(url), ...(name !== undefined && name !== '' ? { name } : {}) }),
      ),
    refresh: (calendarId, url) =>
      Promise.resolve().then(() =>
        forward<CalendarRefreshResponse>('Calendar/refreshSubscription', { calendarId, blob: fetched(url) }),
      ),
  };
}

/** A minimal `Client`-shaped object backed by the in-memory mock, for the slice. */
export function mockCalendarClient(store: MockStore = createMockStore()): {
  jmap: (body: JmapRequest) => Promise<JmapResponse>;
  session: () => Promise<JmapSession>;
} {
  const jmap = createMockJmap(store);
  return { jmap, session: () => Promise.resolve(mockSession()) };
}
