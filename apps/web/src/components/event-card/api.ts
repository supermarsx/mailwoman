// `CalendarEvent/itip` and `CalendarEvent/respond {emailId}` (t53) as the
// reader's event card calls them. The server reads the message, its sender and
// its calendar part; the card names the message (and optionally one part) only.

import type { Client } from '../../api/client.ts';
import { CAP_CORE } from '../../api/jmap-types.ts';
import { CAP_CALENDARS } from '../../api/pim-types.ts';
import { pimResponse } from '../../modules/calendar/api.ts';
import type { CardEvent, CardParticipant } from './eventInfo.ts';

export type Outcome = 'new' | 'update' | 'current' | 'stale' | 'cancel' | 'reply' |
  'notOrganizer' | 'notAttendee' | 'unknownEvent' | 'unsupported' | 'publish';

export interface ItipItem {
  event: CardEvent;
  eventId: string | null;
  calendarId: string | null;
}

export interface StoredSummary {
  calendarId: string | null;
  status: string | null;
  sequence: number;
  participationStatus: string | null;
  participants?: Record<string, CardParticipant> | null;
}

/** `CalendarEvent/itip` (crates/mw-engine/src/pim/itip_ingest.rs). */
export interface ItipView {
  accountId: string;
  emailId: string;
  /** Upper-cased METHOD; `null` for a plain calendar file. */
  method: string | null;
  sender: string;
  organizer: string | null;
  /** The account's address the invitation lists (REQUEST/CANCEL), if any. */
  me: string | null;
  /** A REPLY's participation status. */
  answer: string | null;
  event: CardEvent;
  items: ItipItem[];
  todos: number;
  eventId: string | null;
  stored: StoredSummary | null;
  outcome: Outcome;
  applied: boolean;
  imported?: string[];
}

export type ResponseAction = 'accept' | 'tentative' | 'decline' | 'counter';

export interface RespondResult {
  accountId: string;
  emailId: string;
  action: ResponseAction;
  participationStatus: string;
  eventId: string | null;
  calendarId: string | null;
  from: string;
  to: string;
  sendResponse: boolean;
  sent: boolean;
}

type Jmap = Pick<Client, 'jmap'>;
const USING = [CAP_CORE, CAP_CALENDARS];

export async function readItip(
  client: Jmap,
  args: { accountId: string; emailId: string; blobId?: string; apply?: boolean; calendarId?: string; uids?: string[] },
): Promise<ItipView> {
  const call: Record<string, unknown> = { accountId: args.accountId, emailId: args.emailId, apply: args.apply === true };
  if (args.blobId) call['blobId'] = args.blobId;
  if (args.calendarId) call['calendarId'] = args.calendarId;
  if (args.uids) call['uids'] = args.uids;
  return pimResponse<ItipView>(await client.jmap({ using: USING, methodCalls: [['CalendarEvent/itip', call, 'itip']] }), 'itip');
}

export async function respondToMessage(
  client: Jmap,
  args: {
    accountId: string; emailId: string; blobId?: string; action: ResponseAction; calendarId?: string;
    comment?: string; sendResponse: boolean; counter?: { start: string; duration: string };
  },
): Promise<RespondResult> {
  const call: Record<string, unknown> = {
    accountId: args.accountId, emailId: args.emailId, action: args.action, sendResponse: args.sendResponse,
  };
  if (args.blobId) call['blobId'] = args.blobId;
  if (args.calendarId) call['calendarId'] = args.calendarId;
  if (args.comment && args.comment.trim()) call['comment'] = args.comment.trim();
  if (args.counter) call['counter'] = args.counter;
  return pimResponse<RespondResult>(await client.jmap({ using: USING, methodCalls: [['CalendarEvent/respond', call, 'respond']] }), 'respond');
}

export async function destroyEvent(client: Jmap, accountId: string, eventId: string): Promise<boolean> {
  const res = pimResponse<{ destroyed?: string[] | null }>(await client.jmap({
    using: USING, methodCalls: [['CalendarEvent/set', { accountId, destroy: [eventId] }, 'destroy']],
  }), 'destroy');
  return (res.destroyed ?? []).includes(eventId);
}

export interface PickableCalendar { id: string; name: string; color: string; role: string | null }

/** The account's own event calendars (not subscribed overlays, not task lists). */
export async function ownCalendars(client: Jmap, accountId: string): Promise<PickableCalendar[]> {
  const res = pimResponse<{ list: Array<Record<string, unknown>> }>(await client.jmap({
    using: USING, methodCalls: [['Calendar/get', { accountId, ids: null }, 'cals']],
  }), 'cals');
  return (res.list ?? [])
    .filter((c) => c['isReadOnlyOverlay'] !== true && c['component'] !== 'VTODO')
    .map((c) => ({ id: String(c['id']), name: String(c['name'] ?? ''), color: String(c['color'] ?? ''), role: (c['role'] as string | null) ?? null }));
}

/** One expanded instance of the account's own events. */
export interface DayInstance {
  eventId: string;
  uid: string;
  title: string;
  calendarId: string;
  start: Date;
  end: Date;
  allDay: boolean;
  status: string;
  freeBusy: string;
}

/** The account's event instances overlapping `[from, to)`. */
export async function instancesBetween(client: Jmap, accountId: string, from: Date, to: Date): Promise<DayInstance[]> {
  // One day of padding: floating and all-day instances come back as wall
  // clocks stamped Z (see modules/calendar/datetime.ts `queryBounds`).
  const pad = 86_400_000;
  const res = pimResponse<{ list: Array<Record<string, unknown>> }>(await client.jmap({
    using: USING,
    methodCalls: [['CalendarEvent/expand', {
      accountId, start: new Date(from.getTime() - pad).toISOString().replace(/\.\d{3}Z$/, 'Z'),
      end: new Date(to.getTime() + pad).toISOString().replace(/\.\d{3}Z$/, 'Z'),
    }, 'x']],
  }), 'x');
  return (res.list ?? []).map((row) => {
    const allDay = row['showWithoutTime'] === true;
    const parse = (s: unknown): Date => {
      const text = String(s ?? '');
      return new Date(allDay ? text.replace(/Z$/, '').slice(0, 10) + 'T00:00:00Z' : text);
    };
    return {
      eventId: String(row['eventId'] ?? row['id'] ?? ''),
      uid: String(row['uid'] ?? ''),
      title: String(row['title'] ?? ''),
      calendarId: String(row['calendarId'] ?? ''),
      start: parse(row['instanceStart']),
      end: parse(row['instanceEnd']),
      allDay,
      status: String(row['status'] ?? 'confirmed'),
      freeBusy: String(row['freeBusyStatus'] ?? 'busy'),
    };
  }).filter((i) => !Number.isNaN(i.start.getTime()) && !Number.isNaN(i.end.getTime()) && i.end > from && i.start < to);
}
