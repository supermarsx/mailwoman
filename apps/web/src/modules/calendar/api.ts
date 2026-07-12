// Pure `Calendar/*` + `CalendarEvent/*` request builders + response shapes for
// the calendar module (plan §2.2). These mirror the frozen envelope machinery in
// `api/jmap.ts` (methodCalls array, `#`-result-references, `{accountId,state,
// list,notFound}` / `{created,updated,destroyed,...}` shapes) but for the
// Mailwoman PIM calendar family. No I/O here so they are trivially unit-testable;
// the slice runs them through the shared `Client.jmap` transport (mock until e10).

import { request } from '../../api/jmap.ts';
import { CAP_CORE, type Id, type Invocation, type JmapRequest } from '../../api/jmap-types.ts';
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

/** One expanded, dated instance the engine returns from `CalendarEvent/expand`. */
export interface ExpandedInstance {
  eventId: Id;
  /** Occurrence start as a `LocalDateTime`. */
  start: string;
  /** Occurrence end as a `LocalDateTime`. */
  end: string;
  recurring: boolean;
}

export interface EventExpandResponse {
  accountId: Id;
  /** The masters covering the window (for editing). */
  list: CalendarEvent[];
  /** Concrete instances overlapping the window. */
  instances: ExpandedInstance[];
}

/** One overlapping-instance pair from `Calendar/detectConflicts`. */
export interface ConflictPairResponse {
  a: string;
  b: string;
}

export interface DetectConflictsResponse {
  accountId: Id;
  conflicts: ConflictPairResponse[];
}

/** One busy block from `Calendar/freeBusy`. */
export interface FreeBusyBlock {
  principal: string;
  start: string;
  end: string;
  status: 'busy' | 'tentative';
}

export interface FreeBusyResponse {
  accountId: Id;
  blocks: FreeBusyBlock[];
}

/** A `CalendarEvent/export` result — an ICS blob (plan §2.2). */
export interface EventExportResponse {
  accountId: Id;
  ics: string;
}

/** A `CalendarEvent/import` result — the created event ids. */
export interface EventImportResponse {
  accountId: Id;
  created: Id[];
  notCreated: Array<{ index: number; reason: string }>;
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
 * Expand a window of events across the given calendars in one round-trip. The
 * engine (`mw-ics` + `rrule`) returns both the masters (for editing) and the
 * concrete instances overlapping `[start, end)` (plan §2.1/§2.2).
 */
export function eventsExpand(
  accountId: Id,
  calendarIds: Id[],
  start: string,
  end: string,
  callId = 'x',
): JmapRequest {
  return request(CAL_USING, [
    ['CalendarEvent/expand', { accountId, calendarIds, start, end }, callId],
  ]);
}

/**
 * Fetch a page of events by query, then hydrate exactly those ids via a JMAP
 * result reference — used where the caller wants raw masters, not expansion.
 */
export function eventsQueryGet(accountId: Id, calendarIds: Id[], limit = 1000): JmapRequest {
  return request(CAL_USING, [
    ['CalendarEvent/query', { accountId, filter: { inCalendars: calendarIds }, limit }, 'q'],
    ['CalendarEvent/get', { accountId, '#ids': { resultOf: 'q', name: 'CalendarEvent/query', path: '/ids' } }, 'g'],
  ]);
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

/** Detect overlapping instances across visible calendars in a window. */
export function detectConflicts(
  accountId: Id,
  calendarIds: Id[],
  start: string,
  end: string,
  callId = 'conflicts',
): JmapRequest {
  return request(CAL_USING, [
    ['Calendar/detectConflicts', { accountId, calendarIds, start, end }, callId],
  ]);
}

/** Query busy blocks for principals over a window (the free/busy picker). */
export function freeBusy(
  accountId: Id,
  principals: string[],
  start: string,
  end: string,
  callId = 'fb',
): JmapRequest {
  return request(CAL_USING, [['Calendar/freeBusy', { accountId, principals, start, end }, callId]]);
}

/** Import an ICS / `.hol` blob into a calendar (plan §2.2). */
export function eventsImport(
  accountId: Id,
  calendarId: Id,
  ics: string,
  callId = 'import',
): JmapRequest {
  return request(CAL_USING, [['CalendarEvent/import', { accountId, calendarId, ics }, callId]]);
}

/** Export a set of events (or a whole calendar) to an ICS blob (plan §2.2). */
export function eventsExport(
  accountId: Id,
  opts: { calendarId?: Id; eventIds?: Id[] },
  callId = 'export',
): JmapRequest {
  return request(CAL_USING, [['CalendarEvent/export', { accountId, ...opts }, callId]]);
}
