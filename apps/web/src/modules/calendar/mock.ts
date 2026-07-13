// In-memory mock backend for the calendar module (plan §3 e4: "consumes the
// frozen surface against a mock until integration (e10)"). It implements the
// §2.2 `Calendar/*` + `CalendarEvent/*` families over the same `JmapResponse`
// envelope the real engine (e8) will speak, so the slice + views are agnostic to
// whether they run against the mock or the engine. Tests drive this directly;
// e10 swaps it for `client.jmap` against `mw-server`.

import type { JmapRequest, JmapResponse, JmapSession, Invocation } from '../../api/jmap-types.ts';
import type { Calendar, CalendarEvent } from '../../api/pim-types.ts';
import { dateToLocal, localToDate } from './datetime.ts';
import { expandEvent } from './recurrence.ts';
import type {
  CalendarGetResponse,
  CalendarSetResponse,
  DetectConflictsResponse,
  EventExpandResponse,
  EventExportResponse,
  EventGetResponse,
  EventImportResponse,
  EventSetResponse,
  ExpandedInstance,
  FreeBusyResponse,
  ConflictPairResponse,
} from './api.ts';

const ACCOUNT = 'acct-mock';

/** Two seeded calendars: a personal (default) + a work overlay in a second color. */
export function seedCalendars(): Calendar[] {
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
  calendars: Calendar[];
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

/** Dispatch one method call against the store, returning its `Invocation`. */
function dispatch(store: MockStore, call: Invocation): Invocation {
  const [name, rawArgs, callId] = call;
  const args = rawArgs as Record<string, unknown>;
  switch (name) {
    case 'Calendar/get': {
      const res: CalendarGetResponse = { accountId: ACCOUNT, state: '1', list: store.calendars, notFound: [] };
      return ok(callId, name, res);
    }
    case 'Calendar/set': {
      const created: Record<string, Partial<Calendar> & { id: string }> = {};
      const updated: Record<string, unknown> = {};
      const destroyed: string[] = [];
      for (const [key, val] of Object.entries((args['create'] as Record<string, Partial<Calendar>>) ?? {})) {
        const id = nextId('cal');
        const cal: Calendar = {
          id, name: val.name ?? 'Calendar', color: val.color ?? '#3b82f6', order: val.order ?? store.calendars.length,
          isVisible: val.isVisible ?? true, isSubscribed: val.isSubscribed ?? true, role: val.role ?? null,
          shareWith: val.shareWith ?? [], caldavUrl: val.caldavUrl ?? null, syncToken: null,
          isReadOnlyOverlay: val.isReadOnlyOverlay ?? false,
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
      const res: CalendarSetResponse = {
        accountId: ACCOUNT, oldState: '1', newState: '2',
        created: Object.keys(created).length ? created : null,
        updated: Object.keys(updated).length ? updated : null,
        destroyed: destroyed.length ? destroyed : null,
        notCreated: null, notUpdated: null, notDestroyed: null,
      };
      return ok(callId, name, res);
    }
    case 'CalendarEvent/get': {
      const ids = args['ids'] as string[] | null;
      const list = ids === null ? store.events : store.events.filter((e) => ids.includes(e.id));
      const res: EventGetResponse = { accountId: ACCOUNT, state: '1', list, notFound: [] };
      return ok(callId, name, res);
    }
    case 'CalendarEvent/expand': {
      // Mirror the engine: return the expanded instances in `list` (each row also
      // carries the master's fields; the controller reads only id + bounds).
      const calendarIds = (args['calendarIds'] as string[]) ?? store.calendars.map((c) => c.id);
      const start = localToDate(args['start'] as string);
      const end = localToDate(args['end'] as string);
      const masters = store.events.filter((e) => calendarIds.includes(e.calendarId));
      const list: ExpandedInstance[] = [];
      for (const ev of masters) {
        for (const inst of expandEvent(ev, start, end, colorFor(store, ev.calendarId))) {
          list.push({ eventId: ev.id, instanceStart: dateToLocal(inst.start), instanceEnd: dateToLocal(inst.end) });
        }
      }
      const res: EventExpandResponse = { accountId: ACCOUNT, list };
      return ok(callId, name, res);
    }
    case 'CalendarEvent/set': {
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
      const res: EventSetResponse = {
        accountId: ACCOUNT, oldState: '1', newState: '2',
        created: Object.keys(created).length ? created : null,
        updated: Object.keys(updated).length ? updated : null,
        destroyed: destroyed.length ? destroyed : null,
        notCreated: null, notUpdated: null, notDestroyed: null,
      };
      return ok(callId, name, res);
    }
    case 'CalendarEvent/respond': {
      const eventId = args['eventId'] as string;
      const action = args['action'] as string;
      const statusMap: Record<string, CalendarEvent['participants'][string]['participationStatus']> = {
        accept: 'accepted', decline: 'declined', tentative: 'tentative', counter: 'tentative',
      };
      store.events = store.events.map((e) => {
        if (e.id !== eventId) return e;
        const participants = { ...e.participants };
        if (participants['me'] !== undefined) {
          participants['me'] = { ...participants['me'], participationStatus: statusMap[action] ?? 'needs-action' };
        }
        return { ...e, participants, sequence: e.sequence + 1 };
      });
      return ok(callId, name, { accountId: ACCOUNT, updated: { [eventId]: null } });
    }
    case 'Calendar/detectConflicts': {
      const calendarIds = (args['calendarIds'] as string[]) ?? store.calendars.map((c) => c.id);
      const start = localToDate(args['start'] as string);
      const end = localToDate(args['end'] as string);
      const flat = store.events
        .filter((e) => calendarIds.includes(e.calendarId) && !e.showWithoutTime)
        .flatMap((e) => expandEvent(e, start, end, '#000'));
      const list: ConflictPairResponse[] = [];
      for (let i = 0; i < flat.length; i += 1) {
        for (let j = i + 1; j < flat.length; j += 1) {
          const a = flat[i]!;
          const b = flat[j]!;
          if (a.event.id === b.event.id) continue;
          if (a.start < b.end && b.start < a.end) {
            const overlapStart = a.start > b.start ? a.start : b.start;
            const overlapEnd = a.end < b.end ? a.end : b.end;
            list.push({
              eventA: a.event.id,
              eventB: b.event.id,
              overlapStart: dateToLocal(overlapStart),
              overlapEnd: dateToLocal(overlapEnd),
            });
          }
        }
      }
      const res: DetectConflictsResponse = { accountId: ACCOUNT, list };
      return ok(callId, name, res);
    }
    case 'Calendar/freeBusy': {
      const res: FreeBusyResponse = { accountId: ACCOUNT, blocks: [] };
      return ok(callId, name, res);
    }
    case 'CalendarEvent/import': {
      const created: string[] = [];
      const raw = String(args['ics'] ?? '');
      // Minimal count of VEVENT blocks; the real engine parses via mw-ics.
      const count = (raw.match(/BEGIN:VEVENT/g) ?? []).length;
      for (let i = 0; i < count; i += 1) {
        const id = nextId('ev');
        store.events.push(baseEvent({
          id, calendarId: (args['calendarId'] as string) ?? store.calendars[0]!.id,
          title: `Imported ${i + 1}`, start: dateToLocal(new Date()),
        }));
        created.push(id);
      }
      const res: EventImportResponse = { accountId: ACCOUNT, created, notCreated: [] };
      return ok(callId, name, res);
    }
    case 'CalendarEvent/export': {
      const ids = args['eventIds'] as string[] | undefined;
      const calendarId = args['calendarId'] as string | undefined;
      const events = store.events.filter(
        (e) => (ids === undefined || ids.includes(e.id)) && (calendarId === undefined || e.calendarId === calendarId),
      );
      const body = events
        .map((e) => `BEGIN:VEVENT\nUID:${e.uid}\nSUMMARY:${e.title}\nEND:VEVENT`)
        .join('\n');
      const res: EventExportResponse = {
        accountId: ACCOUNT,
        ics: `BEGIN:VCALENDAR\nVERSION:2.0\nPRODID:-//Mailwoman//EN\n${body}\nEND:VCALENDAR`,
      };
      return ok(callId, name, res);
    }
    default:
      return ['error', { type: 'unknownMethod', description: name }, callId] as unknown as Invocation;
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

/** A minimal `Client`-shaped object backed by the in-memory mock, for the slice. */
export function mockCalendarClient(store: MockStore = createMockStore()): {
  jmap: (body: JmapRequest) => Promise<JmapResponse>;
  session: () => Promise<JmapSession>;
} {
  const jmap = createMockJmap(store);
  return { jmap, session: () => Promise.resolve(mockSession()) };
}
