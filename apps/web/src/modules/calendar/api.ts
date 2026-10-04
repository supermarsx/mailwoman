// Pure `Calendar/*` + `CalendarEvent/*` request builders + response shapes for
// the calendar module (plan §2.2). These mirror the frozen envelope machinery in
// `api/jmap.ts` (methodCalls array, `#`-result-references, `{accountId,state,
// list,notFound}` / `{created,updated,destroyed,...}` shapes) but for the
// Mailwoman PIM calendar family. No I/O here so they are trivially unit-testable;
// the slice runs them through the shared `Client.jmap` transport.
//
// The SERVER is the source of truth for every shape in this file. Each request
// builder and response type cites the handler it mirrors in
// `crates/mw-engine/src/pim/{calendars,events}.rs`; `mock.ts` cites the same
// lines, and `contract.test.ts` pins both against fixtures taken from them.

import { request, responseFor } from '../../api/jmap.ts';
import { CAP_CORE, type Id, type Invocation, type JmapRequest, type JmapResponse } from '../../api/jmap-types.ts';
import { CAP_CALENDARS, type Calendar, type CalendarEvent } from '../../api/pim-types.ts';

/** `using` for the calendar surface: core + calendars. */
const CAL_USING = [CAP_CORE, CAP_CALENDARS];

// ── Response shapes (frozen JMAP get/query/set shapes) ───────────────────────

export interface CalendarGetResponse {
  accountId: Id;
  state: string;
  list: Calendar[];
  notFound: Id[];
}

export interface CalendarSetResponse {
  accountId: Id;
  oldState: string | null;
  newState: string;
  created: Record<string, Partial<Calendar> & { id: Id }> | null;
  updated: Record<Id, unknown> | null;
  destroyed: Id[] | null;
  notCreated: Record<string, { type: string; description?: string | null }> | null;
  notUpdated: Record<Id, { type: string; description?: string | null }> | null;
  notDestroyed: Record<Id, { type: string; description?: string | null }> | null;
}

export interface EventGetResponse {
  accountId: Id;
  state: string;
  list: CalendarEvent[];
  notFound: Id[];
}

export interface EventQueryResponse {
  accountId: Id;
  queryState: string;
  ids: Id[];
  position: number;
  total?: number;
}

export interface EventSetResponse {
  accountId: Id;
  oldState: string | null;
  newState: string;
  created: Record<string, Partial<CalendarEvent> & { id: Id }> | null;
  updated: Record<Id, unknown> | null;
  destroyed: Id[] | null;
  notCreated: Record<string, { type: string; description?: string | null }> | null;
  notUpdated: Record<Id, { type: string; description?: string | null }> | null;
  notDestroyed: Record<Id, { type: string; description?: string | null }> | null;
}

/**
 * One expanded, dated instance from `CalendarEvent/expand` (`event_expand`,
 * `events.rs:449-480`). The engine returns these in the response `list` (each
 * row also carries the master's projection — `id`/`calendarId`/`title`/…; the
 * controller reads only the id + occurrence bounds and joins to the masters
 * fetched separately). `instanceStart` / `instanceEnd` are RFC3339 UTC: a true
 * instant for an event with a `timeZone`, and the wall clock stamped `Z` for a
 * floating or all-day one (`local_to_utc`, `mw-ics/src/recur.rs:47-63`).
 * `instanceBound` in `datetime.ts` decodes the two cases.
 */
export interface ExpandedInstance {
  eventId: Id;
  /** Occurrence start. */
  instanceStart: string;
  /** Occurrence end. */
  instanceEnd: string;
}

export interface EventExpandResponse {
  accountId: Id;
  /** Concrete instances overlapping the window (frozen `list` envelope). */
  list: ExpandedInstance[];
}

/** One overlapping-instance pair from `Calendar/detectConflicts`. */
export interface ConflictPairResponse {
  eventA: Id;
  eventB: Id;
  /** Overlap window bounds (RFC3339 UTC / `LocalDateTime`). */
  overlapStart: string;
  overlapEnd: string;
}

export interface DetectConflictsResponse {
  accountId: Id;
  /** Overlapping pairs (frozen `list` envelope). */
  list: ConflictPairResponse[];
}

/**
 * One merged busy interval from `Calendar/freeBusy` (`calendar_free_busy`,
 * `calendars.rs:377-383`). The engine aggregates the ACCOUNT'S OWN events, so a
 * block names no principal: it is the signed-in user's busy time. `start` /
 * `end` are RFC3339 UTC in the same two encodings as `ExpandedInstance`, and
 * a merged block does not say which encoding its contributors used.
 */
export interface FreeBusyBlock {
  start: string;
  end: string;
  status: 'busy' | 'tentative';
}

export interface FreeBusyResponse {
  accountId: Id;
  /** Merged busy intervals (frozen `list` envelope). */
  list: FreeBusyBlock[];
}

/** A `CalendarEvent/export` result (`event_export`, `events.rs:584`). */
export interface EventExportResponse {
  accountId: Id;
  /** One VCALENDAR document holding every exported VEVENT. */
  blob: string;
}

/** A `CalendarEvent/quickAdd` result (`event_quick_add`, `events.rs:728-738`).
 *  Empty text is a method-level `serverFail`, not `created: null`. */
export interface EventQuickAddResponse {
  accountId: Id;
  created: { id: Id };
  /** What the engine's parser understood, for echoing back. */
  parsed: {
    title: string;
    start: string | null;
    duration: string;
    allDay: boolean;
    location: string | null;
  };
}

/** A `Calendar/subscribe` result (`calendar_subscribe`, `calendars.rs:291-296`). */
export interface CalendarSubscribeResponse {
  accountId: Id;
  /** The created read-only overlay calendar. */
  created: { id: Id };
  /** The stored feed URL (`webcal://` normalized to `https://`). */
  url: string;
  /** Events imported from the fetched feed body. */
  imported: number;
}

/** A `Calendar/refreshSubscription` result (`calendars.rs:348`). */
export interface CalendarRefreshResponse {
  accountId: Id;
  calendarId: Id;
  /** Events in the overlay after the re-import. */
  imported: number;
}

/** A `CalendarEvent/import` result (`event_import`, `events.rs:516`). An event
 *  that fails to persist is skipped server-side and is simply absent here. */
export interface EventImportResponse {
  accountId: Id;
  /** Ids of the events created. */
  imported: Id[];
  count: number;
}

/**
 * Read one PIM method response, throwing on a method-level failure.
 *
 * The engine does not send PIM failures as an RFC 8620 `error` invocation: it
 * echoes the method name with a `{type, description}` body (`server_fail`,
 * `pim/mod.rs:79-81`; envelope `jmap.rs:132`). `responseFor` therefore returns
 * that body as if it were a result, and the caller reads `undefined` fields
 * off it. No successful PIM response carries a top-level `type`, and every one
 * carries `accountId`, so that pair identifies a failure.
 */
export function pimResponse<T>(res: JmapResponse, callId: string): T {
  const body = responseFor<Record<string, unknown>>(res, callId);
  if (typeof body['type'] === 'string' && body['accountId'] === undefined) {
    const description = typeof body['description'] === 'string' ? body['description'] : '';
    throw new Error(`JMAP method error: ${body['type']} ${description}`.trim());
  }
  return body as T;
}

// ── Request builders ─────────────────────────────────────────────────────────

/** Fetch the account's calendars. */
export function calendarsGet(accountId: Id, callId = 'cals'): JmapRequest {
  return request(CAL_USING, [['Calendar/get', { accountId, ids: null }, callId]]);
}

/** Create / update / destroy calendars (visibility, color, order, sharing). */
export function calendarSet(
  accountId: Id,
  ops: {
    create?: Record<string, Partial<Calendar>>;
    update?: Record<Id, Record<string, unknown>>;
    destroy?: Id[];
  },
  callId = 'set',
): JmapRequest {
  const args: Record<string, unknown> = { accountId };
  if (ops.create !== undefined) args['create'] = ops.create;
  if (ops.update !== undefined) args['update'] = ops.update;
  if (ops.destroy !== undefined) args['destroy'] = ops.destroy;
  return request(CAL_USING, [['Calendar/set', args, callId]]);
}

/**
 * Expand every event in the account over `[start, end)` (RFC3339 UTC) into
 * concrete instances. `event_expand` (`events.rs:449-461`) reads `start`, `end`
 * and an optional `ids`; it has no calendar filter, so none is sent — the
 * controller narrows to the visible calendars itself.
 */
export function eventsExpand(accountId: Id, start: string, end: string, callId = 'x'): JmapRequest {
  return request(CAL_USING, [['CalendarEvent/expand', { accountId, start, end }, callId]]);
}

/**
 * Fetch every event master in the account (unfiltered by window) in one call —
 * `CalendarEvent/get` with `ids: null`. Used for the master lookup the editor
 * needs: `CalendarEvent/expand` only returns per-instance rows (whose `start` is
 * the occurrence, not the series DTSTART), so the true masters come from here.
 */
export function eventsGetAll(accountId: Id, callId = 'g'): JmapRequest {
  return request(CAL_USING, [['CalendarEvent/get', { accountId, ids: null }, callId]]);
}

/**
 * Query the ids of the events carrying `category` (P4), optionally within one
 * calendar. `event_query_ids` (`events.rs:337-361`) reads `filter.calendarId` —
 * a single id, there is no multi-calendar condition — and `filter.categories`,
 * matched case-insensitively against any of the event's categories
 * (`filter_events_by_categories`, `events.rs:748-777`). The handler does not
 * page, so no `limit` is sent. The controller intersects the id set with the
 * expanded window.
 */
export function eventsQueryByCategory(
  accountId: Id,
  category: string,
  calendarId?: Id,
  callId = 'q',
): JmapRequest {
  const filter: Record<string, unknown> = { categories: [category] };
  if (calendarId !== undefined) filter['calendarId'] = calendarId;
  return request(CAL_USING, [['CalendarEvent/query', { accountId, filter }, callId]]);
}

/** Query the ids of every event in one calendar (`filter.calendarId`, `events.rs:339-342`). */
export function eventsQueryInCalendar(accountId: Id, calendarId: Id, callId = 'q'): JmapRequest {
  return request(CAL_USING, [['CalendarEvent/query', { accountId, filter: { calendarId } }, callId]]);
}

/** Build a `CalendarEvent/set` request (create / update / destroy). */
export function eventSet(
  accountId: Id,
  ops: {
    create?: Record<string, Partial<CalendarEvent>>;
    update?: Record<Id, Record<string, unknown>>;
    destroy?: Id[];
  },
  callId = 'set',
): JmapRequest {
  const args: Record<string, unknown> = { accountId };
  if (ops.create !== undefined) args['create'] = ops.create;
  if (ops.update !== undefined) args['update'] = ops.update;
  if (ops.destroy !== undefined) args['destroy'] = ops.destroy;
  const call: Invocation = ['CalendarEvent/set', args, callId];
  return request(CAL_USING, [call]);
}

/** The iTIP response action (plan §2.6). */
export type RespondAction = 'accept' | 'decline' | 'tentative' | 'counter';

/**
 * Respond to an invite (iTIP REPLY / COUNTER). Updates the local
 * `participationStatus`, bumps `sequence`, and (engine-side) emits the iMIP
 * reply to the organizer via `MailSubmitter` (plan §2.6).
 */
export function eventRespond(
  accountId: Id,
  eventId: Id,
  action: RespondAction,
  counter?: { start: string; duration: string },
  callId = 'respond',
): JmapRequest {
  const args: Record<string, unknown> = { accountId, eventId, action };
  if (counter !== undefined) args['counter'] = counter;
  return request(CAL_USING, [['CalendarEvent/respond', args, callId]]);
}

/**
 * Detect overlapping instances across the whole account in a window.
 * `calendar_detect_conflicts` (`calendars.rs:391-403`) reads only `start` and
 * `end`; it has no calendar filter, so none is sent.
 */
export function detectConflicts(
  accountId: Id,
  start: string,
  end: string,
  callId = 'conflicts',
): JmapRequest {
  return request(CAL_USING, [['Calendar/detectConflicts', { accountId, start, end }, callId]]);
}

/**
 * Query the account's own merged busy intervals over `[start, end)`.
 * `calendar_free_busy` (`calendars.rs:359-372`) reads `start` and `end` — both
 * REQUIRED to be RFC3339 with a zone designator — plus an optional
 * `calendarIds`. It reads no `principals`: it cannot report on anyone else.
 */
export function freeBusy(
  accountId: Id,
  start: string,
  end: string,
  calendarIds?: Id[],
  callId = 'fb',
): JmapRequest {
  const args: Record<string, unknown> = { accountId, start, end };
  if (calendarIds !== undefined) args['calendarIds'] = calendarIds;
  return request(CAL_USING, [['Calendar/freeBusy', args, callId]]);
}

/**
 * Import an ICS / `.hol` document into a calendar. `event_import`
 * (`events.rs:500-505`) reads the document from `blob`.
 */
export function eventsImport(
  accountId: Id,
  calendarId: Id,
  blob: string,
  callId = 'import',
): JmapRequest {
  return request(CAL_USING, [['CalendarEvent/import', { accountId, calendarId, blob }, callId]]);
}

/**
 * Export events to one ICS document. `event_export` (`events.rs:568-572`) reads
 * `ids`; anything that is not an array — `null` included — exports every event
 * in the account. It has no calendar filter: to export one calendar the caller
 * passes that calendar's event ids.
 */
export function eventsExport(accountId: Id, ids: Id[] | null, callId = 'export'): JmapRequest {
  return request(CAL_USING, [['CalendarEvent/export', { accountId, ids }, callId]]);
}

/**
 * Create an event from a natural-language line (P3). The engine's `quickAdd`
 * parser (`event_quick_add`, `events.rs:682-742`) turns e.g. "Lunch with Sam
 * Friday 1pm" into a dated event on `calendarId` (defaulting server-side to the
 * primary calendar when omitted). Empty `text` is a method-level failure.
 */
export function eventQuickAdd(
  accountId: Id,
  text: string,
  calendarId?: Id,
  callId = 'qa',
): JmapRequest {
  const args: Record<string, unknown> = { accountId, text };
  if (calendarId !== undefined) args['calendarId'] = calendarId;
  return request(CAL_USING, [['CalendarEvent/quickAdd', args, callId]]);
}

// `Calendar/subscribe` and `Calendar/refreshSubscription` have no request
// builder here on purpose. The engine methods fetch nothing — they import a feed
// body handed to them as `blob` (`calendar_subscribe` / `calendar_refresh_subscription`,
// `calendars.rs:234-349`) — so the app reaches them only through the server's
// sync driver, which does the fetch: see `feeds.ts`.
