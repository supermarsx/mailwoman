// Pure `Task/*` request builders + response shapes for the tasks module (plan
// §2.2). These mirror the frozen envelope machinery in `api/jmap.ts` (methodCalls
// array, `#`-result-references, `{accountId,state,list,notFound}` /
// `{created,updated,destroyed,...}` shapes) but for the Mailwoman PIM `Task/*`
// family. No I/O here so they are trivially unit-testable; the slice runs them
// through the shared `Client.jmap` transport (mock until e10).
//
// Task lists are CalDAV VTODO collections (plan §1.4): they arrive as
// `Calendar`-like collections via `Calendar/get`; we project them to `TaskList`
// and keep only the VTODO ones (`component === 'VTODO'`) when the engine tags
// them — the mock omits `component`, so every returned collection shows until
// e10 wires the real engine. See `state/slices/tasks.ts`.

import { request } from '../../api/jmap.ts';
import { CAP_CORE, type Id, type Invocation, type JmapRequest } from '../../api/jmap-types.ts';
import { CAP_CALENDARS, CAP_TASKS, type Calendar, type Task } from '../../api/pim-types.ts';

/** `using` for the task surface: core + tasks, plus calendars for list fetch. */
const TASK_USING = [CAP_CORE, CAP_TASKS, CAP_CALENDARS];

/** A task list = a VTODO CalDAV collection projected from a `Calendar` (§2.1). */
export interface TaskList {
  id: Id;
  name: string;
  color: string;
  order: number;
}

/** `{accountId,state,list,notFound}` for `Task/get` (frozen JMAP get shape). */
export interface TaskGetResponse {
  accountId: Id;
  state: string;
  list: Task[];
  notFound: Id[];
}

/** `{accountId,queryState,ids,...}` for `Task/query` (frozen JMAP query shape). */
export interface TaskQueryResponse {
  accountId: Id;
  queryState: string;
  ids: Id[];
  position: number;
  total?: number;
}

/** `{created,updated,destroyed,notCreated,...}` for `Task/set`. */
export interface TaskSetResponse {
  accountId: Id;
  oldState: string | null;
  newState: string;
  created: Record<string, Partial<Task> & { id: Id }> | null;
  updated: Record<Id, unknown> | null;
  destroyed: Id[] | null;
  notCreated: Record<string, { type: string; description?: string | null }> | null;
  notUpdated: Record<Id, { type: string; description?: string | null }> | null;
  notDestroyed: Record<Id, { type: string; description?: string | null }> | null;
}

/** `Calendar/get` response — the source of VTODO collections (task lists). */
export interface CalendarGetResponse {
  accountId: Id;
  state: string;
  list: Calendar[];
  notFound: Id[];
}

/** Fetch the account's task-list collections (VTODO calendars). */
export function taskListsGet(accountId: Id, callId = 'lists'): JmapRequest {
  return request(TASK_USING, [['Calendar/get', { accountId, ids: null }, callId]]);
}

/**
 * List a task list's tasks in one round-trip: `Task/query` for the ids (filtered
 * to `listId` when given), then `Task/get` for exactly those ids via a JMAP
 * result reference (`#ids` from the query).
 */
export function tasksQueryGet(accountId: Id, listId?: Id, limit = 500): JmapRequest {
  const filter = listId !== undefined ? { listId } : {};
  return request(TASK_USING, [
    ['Task/query', { accountId, filter, limit }, 'q'],
    ['Task/get', { accountId, '#ids': { resultOf: 'q', name: 'Task/query', path: '/ids' } }, 'g'],
  ]);
}

/** The convenience source fields on a `Task/set` create (plan §2.2). */
export interface TaskCreateSource {
  /** mail→task: seed the new task from a message. */
  fromEmail?: { emailId: Id };
  /** event→task: seed the new task from a calendar event. */
  fromEvent?: { eventId: Id };
}

/** A `Task/set` create payload (a partial task + optional convert source). */
export type TaskCreate = Partial<Task> & TaskCreateSource;

/** Build a `Task/set` request (any of create / update / destroy). */
export function taskSet(
  accountId: Id,
  ops: {
    create?: Record<string, TaskCreate>;
    update?: Record<Id, Record<string, unknown>>;
    destroy?: Id[];
  },
  callId = 'set',
): JmapRequest {
  const args: Record<string, unknown> = { accountId };
  if (ops.create !== undefined) args['create'] = ops.create;
  if (ops.update !== undefined) args['update'] = ops.update;
  if (ops.destroy !== undefined) args['destroy'] = ops.destroy;
  const call: Invocation = ['Task/set', args, callId];
  return request(TASK_USING, [call]);
}

/** Project a `Calendar` collection to a `TaskList`, keeping VTODO ones. A `null`
 *  return means "not a task list" (an event calendar, when the engine tags it). */
export function taskListFromCalendar(cal: Calendar & { component?: string }): TaskList | null {
  if (cal.component !== undefined && cal.component !== 'VTODO') return null;
  return { id: cal.id, name: cal.name, color: cal.color, order: cal.order };
}
