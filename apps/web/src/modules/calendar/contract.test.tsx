// Calendar client ↔ server contract (26.20 t28-e5).
//
// The client and the engine disagreed on eight calendar contracts and no test
// saw it, because every calendar test ran against `mock.ts`, and `mock.ts`
// implemented the CLIENT's idea of each shape. A test whose oracle is written
// by the same hand as its subject cannot fail.
//
// So the oracle here is the SERVER. Every fixture below is transcribed from a
// Rust handler and says which lines; none is derived from `api.ts`, the
// controller or the mock. Three things are then held to those fixtures:
//   1. the request builders — they send only keys the handler reads;
//   2. the controller — it reads the handler's response (and failure) shape;
//   3. the mock — it accepts and returns what the handler does, and REJECTS the
//      old client-shaped requests, so it can no longer hide a mismatch.
// If a handler changes, change the fixture from the Rust; do not copy the
// client's types into it.

import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import type { Invocation, JmapRequest, JmapResponse } from '../../api/jmap-types.ts';
import type { CalendarEvent } from '../../api/pim-types.ts';
import {
  detectConflicts,
  eventsExpand,
  eventsExport,
  eventsImport,
  eventsQueryByCategory,
  eventsQueryInCalendar,
  freeBusy,
  pimResponse,
} from './api.ts';
import { ConflictResolver } from './ConflictResolver.tsx';
import { conflictsInWindow, createCalendarController, type CalendarController } from './controller.ts';
import { instanceBound, queryBounds } from './datetime.ts';
import { createCalendarFeeds, FeedError, type CalendarFeeds } from './feeds.ts';
import { CalendarApp } from './index.tsx';
import { createMockFeeds, createMockJmap, createMockStore, type MockStore } from './mock.ts';

// ── fixtures transcribed from the server ─────────────────────────────────────

/** The argument keys each handler reads (besides `accountId`). */
const SERVER_READS: Record<string, readonly string[]> = {
  // crates/mw-engine/src/pim/calendars.rs:359-372 — `start`, `end`, `calendarIds`.
  'Calendar/freeBusy': ['start', 'end', 'calendarIds'],
  // crates/mw-engine/src/pim/calendars.rs:392-399 — `start`, `end`.
  'Calendar/detectConflicts': ['start', 'end'],
  // crates/mw-engine/src/pim/events.rs:450-455 — `start`, `end`, `ids` (via `wanted_ids`).
  'CalendarEvent/expand': ['start', 'end', 'ids'],
  // crates/mw-engine/src/pim/events.rs:500,505 — `blob`, `calendarId`.
  'CalendarEvent/import': ['blob', 'calendarId'],
  // crates/mw-engine/src/pim/events.rs:569 — `ids` (via `wanted_ids`, pim/mod.rs:154-161).
  'CalendarEvent/export': ['ids'],
  // crates/mw-engine/src/pim/events.rs:338 — `filter`.
  'CalendarEvent/query': ['filter'],
};

/** The `filter` conditions `event_query_ids` reads — events.rs:339-361. */
const SERVER_QUERY_FILTER_KEYS = ['calendarId', 'inCalendar', 'after', 'before', 'categories', 'category'];

/** A method-level failure — `server_fail`, pim/mod.rs:79-81; the envelope echoes
 *  the METHOD NAME, not `"error"` (jmap.rs:132). */
const SERVER_FAIL = { type: 'serverFail', description: 'Calendar/freeBusy requires start + end (RFC3339 UTC)' };

/** `Calendar/freeBusy` — calendars.rs:379-383; the values are the ones the
 *  engine's own test asserts (crates/mw-engine/tests/pim.rs:306-314). */
const SERVER_FREE_BUSY = {
  accountId: 'acct1',
  list: [{ start: '2026-07-13T09:00:00Z', end: '2026-07-13T10:00:00Z', status: 'busy' }],
};

/** `CalendarEvent/import` — events.rs:516. */
const SERVER_IMPORT = { accountId: 'acct1', imported: ['ev-1a', 'ev-1b'], count: 2 };

/** `CalendarEvent/export` — events.rs:573-584 (CRLF, one VCALENDAR). */
const SERVER_EXPORT = {
  accountId: 'acct1',
  blob: 'BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Mailwoman//EN\r\nBEGIN:VEVENT\r\nUID:imp-1\r\nSUMMARY:Imported\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n',
};

/** `CalendarEvent/query` — `query_response`, pim/mod.rs:141-150. */
const SERVER_QUERY = { accountId: 'acct1', queryState: '7', ids: ['ev-1'], total: 1, position: 0, canCalculateChanges: true };

/** `CalendarEvent/quickAdd` — events.rs:728-738. */
const SERVER_QUICK_ADD = {
  accountId: 'acct1',
  created: { id: 'ev-9' },
  parsed: { title: 'Dentist', start: '2026-07-14T09:00:00', duration: 'PT1H', allDay: false, location: null },
};

/** `Calendar/subscribe` — calendars.rs:291-296. */
const SERVER_SUBSCRIBE = { accountId: 'acct1', created: { id: 'cal-7' }, url: 'https://example.com/team.ics', imported: 3 };

/** `Calendar/refreshSubscription` — calendars.rs:348. */
const SERVER_REFRESH = { accountId: 'acct1', calendarId: 'cal-7', imported: 3 };

/** One `Calendar/get` row — `calendar_row_to_json`, calendars.rs:518-531. */
function serverCalendar(over: Record<string, unknown>): Record<string, unknown> {
  return {
    id: 'cal-1', name: 'Calendar', color: '#3366ff', order: 0, isVisible: true, isSubscribed: true,
    role: 'default', shareWith: [], caldavUrl: null, syncToken: null, isReadOnlyOverlay: false,
    component: 'VEVENT', ...over,
  };
}

/** `Calendar/get` after first access — `seed_default_collections`
 *  (calendars.rs:471-480) seeds an event calendar AND a task list. */
const SERVER_CALENDARS = {
  accountId: 'acct1',
  state: '2',
  list: [serverCalendar({}), serverCalendar({ id: 'list-1', name: 'Tasks', color: '#8855ff', component: 'VTODO' })],
  notFound: [],
};

function master(over: Partial<CalendarEvent> & Pick<CalendarEvent, 'id' | 'title' | 'start'>): CalendarEvent {
  return {
    calendarId: 'cal-1', uid: over.id, description: '', locations: [], timeZone: 'Europe/London', duration: 'PT1H',
    showWithoutTime: false, recurrenceRules: [], recurrenceOverrides: {}, excludedRecurrenceDates: [],
    status: 'confirmed', priority: 0, freeBusyStatus: 'busy', participants: {}, alerts: {}, sequence: 0, etag: null,
    ...over,
  };
}

// ── a backend that answers with the server fixtures ─────────────────────────

interface Harness {
  controller: CalendarController;
  /** Every `[method, args]` the controller sent. */
  sent: Array<[string, Record<string, unknown>]>;
}

/** A controller over canned server bodies (by method name). Unlisted methods
 *  answer with an empty-but-well-formed server body. */
function overServer(bodies: Record<string, unknown>, feeds?: CalendarFeeds): Harness {
  const sent: Array<[string, Record<string, unknown>]> = [];
  const defaults: Record<string, unknown> = {
    'Calendar/get': SERVER_CALENDARS,
    'CalendarEvent/get': { accountId: 'acct1', state: '2', list: [], notFound: [] },
    'CalendarEvent/expand': { accountId: 'acct1', list: [] },
    'Calendar/detectConflicts': { accountId: 'acct1', list: [] },
  };
  const jmap = (body: JmapRequest): Promise<JmapResponse> => {
    const methodResponses = body.methodCalls.map(([name, args, callId]) => {
      sent.push([name, args as Record<string, unknown>]);
      // The engine echoes the method name for results AND failures (jmap.rs:132).
      return [name, bodies[name] ?? defaults[name] ?? { type: 'unknownMethod', description: name }, callId];
    });
    return Promise.resolve({ methodResponses } as unknown as JmapResponse);
  };
  const controller = createCalendarController({
    jmap,
    resolveAccount: () => Promise.resolve('acct1'),
    ...(feeds !== undefined ? { feeds } : {}),
  });
  return { controller, sent };
}

function argsOf(req: JmapRequest): Record<string, unknown> {
  return req.methodCalls[0]![1] as Record<string, unknown>;
}

/** The keys a request sends beyond `accountId`. */
function sentKeys(req: JmapRequest): string[] {
  return Object.keys(argsOf(req)).filter((k) => k !== 'accountId');
}

function lastArgs(h: Harness, method: string): Record<string, unknown> {
  const hit = [...h.sent].reverse().find(([name]) => name === method);
  expect(hit, `${method} was sent`).toBeDefined();
  return hit![1];
}

// ── 1. requests: only what the handler reads ────────────────────────────────

describe('request builders send only the arguments the server reads', () => {
  const cases: Array<[string, JmapRequest]> = [
    ['Calendar/freeBusy', freeBusy('acct1', '2026-07-13T00:00:00Z', '2026-07-14T00:00:00Z')],
    ['Calendar/freeBusy', freeBusy('acct1', '2026-07-13T00:00:00Z', '2026-07-14T00:00:00Z', ['cal-1'])],
    ['Calendar/detectConflicts', detectConflicts('acct1', '2026-07-13T00:00:00Z', '2026-07-14T00:00:00Z')],
    ['CalendarEvent/expand', eventsExpand('acct1', '2026-07-13T00:00:00Z', '2026-07-14T00:00:00Z')],
    ['CalendarEvent/import', eventsImport('acct1', 'cal-1', 'BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n')],
    ['CalendarEvent/export', eventsExport('acct1', ['ev-1'])],
    ['CalendarEvent/export', eventsExport('acct1', null)],
    ['CalendarEvent/query', eventsQueryByCategory('acct1', 'work')],
    ['CalendarEvent/query', eventsQueryInCalendar('acct1', 'cal-1')],
  ];

  it.each(cases)('%s', (method, req) => {
    expect(req.methodCalls[0]![0]).toBe(method);
    const reads = SERVER_READS[method]!;
    for (const key of sentKeys(req)) expect(reads, `${method} does not read "${key}"`).toContain(key);
  });

  it('CalendarEvent/query filters use only conditions the server reads', () => {
    for (const req of [eventsQueryByCategory('acct1', 'work', 'cal-1'), eventsQueryInCalendar('acct1', 'cal-1')]) {
      const filter = argsOf(req)['filter'] as Record<string, unknown>;
      for (const key of Object.keys(filter)) expect(SERVER_QUERY_FILTER_KEYS).toContain(key);
    }
    // The single-calendar condition is a string id (events.rs:339-342), not a list.
    expect((argsOf(eventsQueryInCalendar('acct1', 'cal-1'))['filter'] as Record<string, unknown>)['calendarId']).toBe('cal-1');
  });

  it('CalendarEvent/import carries the document under `blob`', () => {
    expect(argsOf(eventsImport('acct1', 'cal-1', 'DOC'))['blob']).toBe('DOC');
  });

  it('window bounds carry the zone designator the engine parser requires', () => {
    // `DateTime::parse_from_rfc3339` (crates/mw-ics/src/recur.rs:40-44) rejects a bare wall clock.
    const q = queryBounds(new Date(2026, 6, 13), new Date(2026, 6, 14));
    expect(q.start).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/);
    expect(q.end).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/);
    // One day of padding either side: the widest UTC offset is +14:00.
    expect(q.start).toBe('2026-07-12T00:00:00Z');
    expect(q.end).toBe('2026-07-15T00:00:00Z');
  });
});

// ── 2. the controller reads the server's shapes ─────────────────────────────

describe('controller against server-shaped responses', () => {
  it('free/busy: reads `list`, and asks with Z-suffixed bounds and no principals', async () => {
    const h = overServer({ 'Calendar/freeBusy': SERVER_FREE_BUSY });
    const blocks = await h.controller.queryFreeBusy(new Date(2026, 6, 13), new Date(2026, 6, 14));
    expect(blocks).toEqual(SERVER_FREE_BUSY.list);
    const args = lastArgs(h, 'Calendar/freeBusy');
    expect(args['start']).toMatch(/Z$/);
    expect(args['end']).toMatch(/Z$/);
    expect(args).not.toHaveProperty('principals');
  });

  it('import: sends `blob`, resolves to the server count', async () => {
    const h = overServer({ 'CalendarEvent/import': SERVER_IMPORT });
    await expect(h.controller.importIcs('cal-1', 'BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n')).resolves.toBe(2);
    expect(lastArgs(h, 'CalendarEvent/import')['blob']).toBe('BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n');
  });

  it('export: reads `blob`; everything is `ids: null`', async () => {
    const h = overServer({ 'CalendarEvent/export': SERVER_EXPORT });
    await expect(h.controller.exportIcs()).resolves.toBe(SERVER_EXPORT.blob);
    expect(lastArgs(h, 'CalendarEvent/export')['ids']).toBeNull();
  });

  it('export of one calendar resolves its ids first — the server has no calendar filter', async () => {
    const h = overServer({ 'CalendarEvent/export': SERVER_EXPORT, 'CalendarEvent/query': SERVER_QUERY });
    await h.controller.exportIcs({ calendarId: 'cal-1' });
    expect(lastArgs(h, 'CalendarEvent/query')['filter']).toEqual({ calendarId: 'cal-1' });
    expect(lastArgs(h, 'CalendarEvent/export')).toEqual({ accountId: 'acct1', ids: SERVER_QUERY.ids });
  });

  it('quick add: the created id is `created.id`', async () => {
    const h = overServer({ 'CalendarEvent/quickAdd': SERVER_QUICK_ADD });
    await expect(h.controller.quickAdd('Dentist tomorrow 9am')).resolves.toBe('ev-9');
  });

  it('a method-level failure rejects instead of reading fields off the error body', async () => {
    // Precondition: this is not an `error` invocation, so `responseFor` alone
    // would hand the failure body back as a result.
    const res = { methodResponses: [['Calendar/freeBusy', SERVER_FAIL, 'fb']] } as unknown as JmapResponse;
    expect(() => pimResponse(res, 'fb')).toThrow(/serverFail/);

    const h = overServer({
      'Calendar/freeBusy': SERVER_FAIL,
      'CalendarEvent/import': { type: 'serverFail', description: 'ics: parse error' },
      'CalendarEvent/quickAdd': { type: 'serverFail', description: 'CalendarEvent/quickAdd requires a non-empty text' },
    });
    await expect(h.controller.queryFreeBusy(new Date(2026, 6, 13), new Date(2026, 6, 14))).rejects.toThrow(/serverFail/);
    await expect(h.controller.importIcs('cal-1', 'not a calendar')).rejects.toThrow(/serverFail/);
    await expect(h.controller.quickAdd('x')).rejects.toThrow(/serverFail/);
  });

  it('a successful body is not mistaken for a failure', () => {
    const res = { methodResponses: [['Calendar/freeBusy', SERVER_FREE_BUSY, 'fb']] } as unknown as JmapResponse;
    expect(pimResponse(res, 'fb')).toEqual(SERVER_FREE_BUSY);
  });

  it('task lists returned by Calendar/get are not shown as calendars', async () => {
    // Precondition: the server body does contain a VTODO collection.
    expect(SERVER_CALENDARS.list.some((c) => c['component'] === 'VTODO')).toBe(true);
    const h = overServer({});
    await h.controller.load();
    expect(h.controller.calendars().map((c) => c.id)).toEqual(['cal-1']);
  });

  it('subscribe goes to the sync driver, and the created id is `created.id`', async () => {
    const calls: Array<[string, unknown]> = [];
    const feeds = createCalendarFeeds((input, init) => {
      calls.push([input, JSON.parse(String(init?.body))]);
      return Promise.resolve(new Response(JSON.stringify(SERVER_SUBSCRIBE), { status: 200 }));
    });
    const h = overServer({}, feeds);
    await expect(h.controller.subscribeUrl('https://example.com/team.ics', 'Team')).resolves.toBe('cal-7');
    // crates/mw-server/src/import_routes.rs:84 (route), :414-419 (`SubscribeReq {url, name?}`).
    expect(calls).toEqual([['/api/calendar/subscribe', { url: 'https://example.com/team.ics', name: 'Team' }]]);
    // The JMAP method, which fetches nothing, is not what the app calls.
    expect(h.sent.some(([name]) => name === 'Calendar/subscribe')).toBe(false);
  });

  it('refresh posts {calendarId, url} to the sync driver', async () => {
    const calls: Array<[string, unknown]> = [];
    const feeds = createCalendarFeeds((input, init) => {
      calls.push([input, JSON.parse(String(init?.body))]);
      return Promise.resolve(new Response(JSON.stringify(SERVER_REFRESH), { status: 200 }));
    });
    const overlay = serverCalendar({ id: 'cal-7', name: 'Team', role: null, caldavUrl: 'https://example.com/team.ics', isReadOnlyOverlay: true });
    const h = overServer({ 'Calendar/get': { ...SERVER_CALENDARS, list: [...SERVER_CALENDARS.list, overlay] } }, feeds);
    await h.controller.load();
    await h.controller.refreshSubscription('cal-7');
    // crates/mw-server/src/import_routes.rs:85 (route), :421-426 (`RefreshReq {calendarId, url}`).
    expect(calls).toEqual([['/api/calendar/refresh', { calendarId: 'cal-7', url: 'https://example.com/team.ics' }]]);
  });

  it('a refused feed request rejects with the status', async () => {
    // `upstream`, crates/mw-server/src/import_routes.rs:594-597.
    const feeds = createCalendarFeeds(() =>
      Promise.resolve(new Response(JSON.stringify({ error: 'fetch failed' }), { status: 502 })),
    );
    const err = await feeds.subscribe('https://example.com/x.ics').then(
      () => null,
      (e: unknown) => e,
    );
    expect(err).toBeInstanceOf(FeedError);
    expect((err as FeedError).status).toBe(502);
  });
});

// ── 3. occurrence times: instants vs. wall clocks ───────────────────────────

describe('engine occurrence bounds', () => {
  // `local_to_utc`, crates/mw-ics/src/recur.rs:47-63.
  it('a zoned event comes back as a true UTC instant', () => {
    expect(instanceBound('2026-07-13T08:00:00Z', true).getTime()).toBe(Date.UTC(2026, 6, 13, 8, 0, 0));
  });

  it('a floating or all-day event comes back as its wall clock stamped Z', () => {
    const d = instanceBound('2026-07-13T08:00:00Z', false);
    expect([d.getFullYear(), d.getMonth(), d.getDate(), d.getHours()]).toEqual([2026, 6, 13, 8]);
    // An all-day event stays on its own day in every zone.
    expect(instanceBound('2026-07-13T00:00:00Z', false).getDate()).toBe(13);
  });

  it('the controller places a zoned instance at its instant, and an all-day one on its day', async () => {
    const zoned = master({ id: 'ev-z', title: 'Zoned', start: '2026-07-13T09:00:00', timeZone: 'Europe/London' });
    const allDay = master({ id: 'ev-d', title: 'All day', start: '2026-07-13', timeZone: null, showWithoutTime: true, duration: 'P1D' });
    const h = overServer({
      'CalendarEvent/get': { accountId: 'acct1', state: '2', list: [zoned, allDay], notFound: [] },
      // Rows as `event_expand` builds them (events.rs:469-476): 09:00 BST is 08:00Z.
      'CalendarEvent/expand': {
        accountId: 'acct1',
        list: [
          { ...zoned, eventId: 'ev-z', instanceStart: '2026-07-13T08:00:00Z', instanceEnd: '2026-07-13T09:00:00Z' },
          { ...allDay, eventId: 'ev-d', instanceStart: '2026-07-13T00:00:00Z', instanceEnd: '2026-07-14T00:00:00Z' },
        ],
      },
    });
    h.controller.setView('month');
    h.controller.goToDate(new Date(2026, 6, 13));
    await waitFor(() => expect(h.controller.instances().length).toBe(2));
    const byId = new Map(h.controller.instances().map((i) => [i.event.id, i]));
    expect(byId.get('ev-z')!.start.getTime()).toBe(Date.UTC(2026, 6, 13, 8, 0, 0));
    const d = byId.get('ev-d')!.start;
    expect([d.getMonth(), d.getDate(), d.getHours()]).toEqual([6, 13, 0]);
  });

  it('instances outside the view window are dropped (the query is padded by a day)', async () => {
    const ev = master({ id: 'ev-o', title: 'Outside', start: '2026-07-14T09:00:00' });
    const h = overServer({
      'CalendarEvent/get': { accountId: 'acct1', state: '2', list: [ev], notFound: [] },
      'CalendarEvent/expand': {
        accountId: 'acct1',
        list: [{ ...ev, eventId: 'ev-o', instanceStart: '2026-07-14T12:00:00Z', instanceEnd: '2026-07-14T13:00:00Z' }],
      },
    });
    h.controller.setView('day');
    h.controller.goToDate(new Date(2026, 6, 13));
    await waitFor(() => expect(h.controller.masters().length).toBe(1));
    expect(h.controller.instances()).toEqual([]);
  });

  it('a conflict pair with an all-day event is not offered for resolution', () => {
    const a = master({ id: 'a', title: 'A', start: '2026-07-13T09:00:00' });
    const b = master({ id: 'b', title: 'B', start: '2026-07-13T09:30:00' });
    const day = master({ id: 'd', title: 'Holiday', start: '2026-07-13', timeZone: null, showWithoutTime: true, duration: 'P1D' });
    // Pairs as `calendar_detect_conflicts` builds them (calendars.rs:417-426);
    // it does not exclude all-day instances.
    const pairs = [
      { eventA: 'd', eventB: 'a', overlapStart: '2026-07-13T08:00:00Z', overlapEnd: '2026-07-13T09:00:00Z' },
      { eventA: 'a', eventB: 'b', overlapStart: '2026-07-13T08:30:00Z', overlapEnd: '2026-07-13T09:00:00Z' },
    ];
    const w = { start: new Date(2026, 6, 13), end: new Date(2026, 6, 14) };
    const kept = conflictsInWindow(pairs, [a, b, day], w);
    expect(kept.map((p) => [p.a, p.b])).toEqual([['a', 'b']]);
  });
});

// ── 4. the mock is held to the same fixtures ────────────────────────────────

describe('mock.ts mirrors the server', () => {
  async function ask(store: MockStore, name: string, args: Record<string, unknown>): Promise<[string, Record<string, unknown>]> {
    const inv = [name, { accountId: 'acct-mock', ...args }, 'c'] as unknown as Invocation;
    const res = await createMockJmap(store)({ using: [], methodCalls: [inv] } as unknown as JmapRequest);
    const [method, body] = res.methodResponses[0]!;
    return [method, body as Record<string, unknown>];
  }

  const day = (): { start: string; end: string } => {
    const now = new Date();
    return queryBounds(new Date(now.getFullYear(), now.getMonth(), now.getDate()), new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1));
  };

  it('response key sets equal the server fixtures', async () => {
    const store = createMockStore();
    const cases: Array<[string, Record<string, unknown>, Record<string, unknown>]> = [
      ['Calendar/freeBusy', day(), SERVER_FREE_BUSY],
      ['CalendarEvent/import', { blob: 'BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:u1\r\nSUMMARY:One\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n' }, SERVER_IMPORT],
      ['CalendarEvent/export', {}, SERVER_EXPORT],
      ['CalendarEvent/query', { filter: {} }, SERVER_QUERY],
      ['CalendarEvent/quickAdd', { text: 'Dentist' }, SERVER_QUICK_ADD],
      ['Calendar/subscribe', { url: 'webcal://example.com/team.ics' }, SERVER_SUBSCRIBE],
    ];
    for (const [name, args, fixture] of cases) {
      const [method, body] = await ask(store, name, args);
      expect(method).toBe(name);
      expect(Object.keys(body).sort(), name).toEqual(Object.keys(fixture).sort());
    }
    // Row shapes, where the fixture has rows.
    const [, fb] = await ask(store, 'Calendar/freeBusy', day());
    const rows = fb['list'] as Array<Record<string, unknown>>;
    expect(rows.length).toBeGreaterThan(0);
    for (const row of rows) expect(Object.keys(row).sort()).toEqual(Object.keys(SERVER_FREE_BUSY.list[0]!).sort());
    const [, qa] = await ask(store, 'CalendarEvent/quickAdd', { text: 'Dentist' });
    expect(Object.keys(qa['created'] as object)).toEqual(['id']);
    expect(Object.keys(qa['parsed'] as object).sort()).toEqual(Object.keys(SERVER_QUICK_ADD.parsed).sort());
    const [, sub] = await ask(store, 'Calendar/subscribe', { url: 'webcal://example.com/a.ics' });
    expect(Object.keys(sub['created'] as object)).toEqual(['id']);
    expect(sub['url']).toBe('https://example.com/a.ics'); // `normalize_webcal`, calendars.rs:539-547
    const [, cals] = await ask(store, 'Calendar/get', {});
    for (const row of cals['list'] as Array<Record<string, unknown>>) {
      expect(Object.keys(row).sort()).toEqual(Object.keys(serverCalendar({})).sort());
    }
    expect((cals['list'] as Array<Record<string, unknown>>).some((c) => c['component'] === 'VTODO')).toBe(true);
  });

  it('refreshSubscription returns the server keys and requires a blob', async () => {
    const store = createMockStore();
    const [, sub] = await ask(store, 'Calendar/subscribe', { url: 'https://example.com/a.ics' });
    const calendarId = (sub['created'] as { id: string }).id;
    const [, okBody] = await ask(store, 'Calendar/refreshSubscription', { calendarId, blob: 'BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n' });
    expect(Object.keys(okBody).sort()).toEqual(Object.keys(SERVER_REFRESH).sort());
    const [method, noBlob] = await ask(store, 'Calendar/refreshSubscription', { calendarId });
    expect(method).toBe('Calendar/refreshSubscription');
    expect(Object.keys(noBlob).sort()).toEqual(Object.keys(SERVER_FAIL).sort());
  });

  it('failures are {type, description} under the method name, never an `error` invocation', async () => {
    const [method, body] = await ask(createMockStore(), 'Calendar/freeBusy', {});
    expect(method).toBe('Calendar/freeBusy');
    expect(body).toEqual(SERVER_FAIL);
  });

  it('rejects the old client-shaped requests it used to accept', async () => {
    const store = createMockStore();
    const before = store.events.length;
    // `ics` is not read (events.rs:500): the document is empty, nothing is imported.
    const [, imp] = await ask(store, 'CalendarEvent/import', { ics: 'BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n' });
    expect(imp).not.toHaveProperty('imported');
    expect(store.events.length).toBe(before);
    // `principals` is not read, and without `start`/`end` the call fails (calendars.rs:361-363).
    const [, fb] = await ask(store, 'Calendar/freeBusy', { principals: ['me@example.com'] });
    expect(fb).not.toHaveProperty('list');
    // `eventIds` is not read (events.rs:569): everything is exported.
    const [, exp] = await ask(store, 'CalendarEvent/export', { eventIds: ['ev-lunch'] });
    expect(String(exp['blob'])).toContain('SUMMARY:Daily standup');
    // `inCalendars` is not read (events.rs:339-342): the filter is ignored.
    const [, all] = await ask(store, 'CalendarEvent/query', { filter: {} });
    const [, listFiltered] = await ask(store, 'CalendarEvent/query', { filter: { inCalendars: ['cal-work'] } });
    expect(listFiltered['ids']).toEqual(all['ids']);
    const [, one] = await ask(store, 'CalendarEvent/query', { filter: { calendarId: 'cal-work' } });
    expect((one['ids'] as string[]).length).toBeLessThan((all['ids'] as string[]).length);
  });

  it('free/busy merges overlapping intervals and skips free events (aggregate_free_busy)', async () => {
    // crates/mw-ics/src/freebusy.rs:37-74. Lunch 12:00-13:00 (busy) overlaps
    // Design review 12:30-13:30 (tentative) → one busy block 12:00-13:30.
    const store = createMockStore();
    const [, fb] = await ask(store, 'Calendar/freeBusy', day());
    const rows = fb['list'] as Array<{ start: string; end: string; status: string }>;
    const noon = rows.filter((r) => instanceBound(r.start, true).getHours() === 12);
    expect(noon).toHaveLength(1);
    expect(noon[0]!.status).toBe('busy');
    expect(instanceBound(noon[0]!.end, true).getHours()).toBe(13);
    expect(instanceBound(noon[0]!.end, true).getMinutes()).toBe(30);
    // Marking both free removes the block.
    store.events = store.events.map((e) => (e.id === 'ev-lunch' || e.id === 'ev-review' ? { ...e, freeBusyStatus: 'free' as const } : e));
    const [, fb2] = await ask(store, 'Calendar/freeBusy', day());
    const rows2 = fb2['list'] as Array<{ start: string }>;
    expect(rows2.filter((r) => instanceBound(r.start, true).getHours() === 12)).toHaveLength(0);
  });
});

// ── 5. the conflict resolver renders over the server's free/busy shape ──────

describe('conflict resolver free/busy grid', () => {
  function mockController(store: MockStore, over: Partial<{ jmap: (b: JmapRequest) => Promise<JmapResponse> }> = {}): CalendarController {
    return createCalendarController({
      jmap: over.jmap ?? createMockJmap(store),
      resolveAccount: () => Promise.resolve('acct-mock'),
      feeds: createMockFeeds(store),
    });
  }

  it('renders one own-account row for a pair WITH participants, busy where the server says', async () => {
    const store = createMockStore();
    // Precondition: the seeded overlapping pair has participants — the case that
    // threw `TypeError: blocks() is not iterable` at render.
    expect(Object.keys(store.events.find((e) => e.id === 'ev-review')!.participants).length).toBeGreaterThan(0);
    const controller = mockController(store);
    await controller.load();
    expect(controller.conflicts().length).toBeGreaterThan(0);
    render(() => <ConflictResolver controller={controller} onClose={() => {}} />);
    const grid = await screen.findByTestId('freebusy-grid');
    expect(grid.querySelectorAll('tbody tr')).toHaveLength(1);
    await waitFor(() => expect(screen.getByLabelText('Yours at 12:00: Busy')).toBeInTheDocument());
    // 11:00 is before Lunch and after the 09:30 standup.
    expect(screen.getByLabelText('Yours at 11:00: Free')).toBeInTheDocument();
    // Attendee availability is not claimed.
    expect(screen.getByTestId('freebusy-own-only')).toBeInTheDocument();
    expect(screen.queryByText('boss@example.com')).toBeNull();
  });

  it('withholds the grid when the free/busy call fails, rather than showing all-free', async () => {
    const store = createMockStore();
    const real = createMockJmap(store);
    const jmap = (body: JmapRequest): Promise<JmapResponse> =>
      body.methodCalls[0]![0] === 'Calendar/freeBusy'
        ? Promise.resolve({ methodResponses: [['Calendar/freeBusy', SERVER_FAIL, body.methodCalls[0]![2]]] } as unknown as JmapResponse)
        : real(body);
    const controller = mockController(store, { jmap });
    await controller.load();
    render(() => <ConflictResolver controller={controller} onClose={() => {}} />);
    expect(await screen.findByTestId('freebusy-unavailable')).toBeInTheDocument();
    expect(screen.queryByTestId('freebusy-grid')).toBeNull();
  });
});

// ── 6. the shell reports a failed action ────────────────────────────────────

describe('calendar shell feedback', () => {
  it('a feed the server cannot fetch is reported and adds no calendar', async () => {
    const store = createMockStore();
    const controller = createCalendarController({
      jmap: createMockJmap(store),
      resolveAccount: () => Promise.resolve('acct-mock'),
      feeds: createMockFeeds(store, () => null),
    });
    render(() => <CalendarApp controller={controller} />);
    await screen.findByTitle('Design review');
    const before = store.calendars.length;
    fireEvent.input(screen.getByLabelText('Calendar URL'), { target: { value: 'https://example.com/gone.ics' } });
    fireEvent.click(screen.getByRole('button', { name: 'Subscribe' }));
    const alert = await screen.findByTestId('calendar-feedback');
    expect(alert).toHaveAttribute('role', 'alert');
    expect(alert).toHaveTextContent('could not be fetched');
    expect(store.calendars.length).toBe(before);
  });

  it('importing a file reports how many events were created, and they appear', async () => {
    const store = createMockStore();
    const controller = createCalendarController({
      jmap: createMockJmap(store),
      resolveAccount: () => Promise.resolve('acct-mock'),
      feeds: createMockFeeds(store),
    });
    render(() => <CalendarApp controller={controller} />);
    await screen.findByTitle('Design review');
    const ics = 'BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:imp-1\r\nSUMMARY:Imported planning\r\nDTSTART:20260714T120000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n';
    const file = new File([ics], 'plan.ics', { type: 'text/calendar' });
    // jsdom's File has no `text()` in every version; the handler only needs that.
    Object.defineProperty(file, 'text', { value: () => Promise.resolve(ics) });
    const input = screen.getByLabelText('Import calendar file') as HTMLInputElement;
    Object.defineProperty(input, 'files', { value: [file] });
    fireEvent.change(input);
    await waitFor(() => expect(screen.getByTestId('calendar-feedback')).toHaveTextContent('Imported 1 event.'));
    expect(controller.masters().some((m) => m.title === 'Imported planning')).toBe(true);
  });

  it('a file that is not a calendar is reported, and nothing is imported', async () => {
    const store = createMockStore();
    const controller = createCalendarController({
      jmap: createMockJmap(store),
      resolveAccount: () => Promise.resolve('acct-mock'),
      feeds: createMockFeeds(store),
    });
    render(() => <CalendarApp controller={controller} />);
    await screen.findByTitle('Design review');
    const before = store.events.length;
    const file = new File(['hello'], 'notes.txt');
    Object.defineProperty(file, 'text', { value: () => Promise.resolve('hello') });
    const input = screen.getByLabelText('Import calendar file') as HTMLInputElement;
    Object.defineProperty(input, 'files', { value: [file] });
    fireEvent.change(input);
    const alert = await screen.findByTestId('calendar-feedback');
    expect(alert).toHaveAttribute('role', 'alert');
    expect(store.events.length).toBe(before);
  });
});
