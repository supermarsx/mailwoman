import type { JSX } from 'solid-js';
import type { Client } from '../api/client.ts';
import type { Email } from '../api/jmap-types.ts';
import { EventCard } from './event-card/EventCard.tsx';
import type { ItipView } from './event-card/api.ts';

/** The `CalendarEvent/itip` answer the card renders (see event-card/api.ts). */
export type Invitation = ItipView;

const CALENDAR_TYPES = new Set(['text/calendar', 'application/ics', 'text/x-vcalendar']);

/** Whether a part is calendar data: by its MIME type, or an `.ics` file sent as
 *  something generic. The server reads the part and decides; a file that only
 *  claims the name shows no card. */
export function isCalendarPart(part: { type?: string | null; name?: string | null }): boolean {
  const type = part.type?.split(';', 1)[0]?.trim().toLowerCase() ?? '';
  if (CALENDAR_TYPES.has(type)) return true;
  return /\.(?:ics|ical|ifb)$/i.test(part.name?.trim() ?? '') && (type === '' || type === 'application/octet-stream' || type === 'text/plain');
}

/** Calendar alternatives, attached invitations and `.ics` files. */
export function hasCalendarPart(email: Email): boolean {
  const parts = email as Email & { attachments?: Array<{ type?: string; name?: string | null }> };
  return [...(parts.attachments ?? []), ...(email.textBody ?? []), ...(email.htmlBody ?? [])]
    .some((part) => isCalendarPart(part as { type?: string; name?: string | null }));
}

/** The reader's event card for a message (t53: Outlook-style; see EventCard). */
export function InviteCard(props: { accountId: string; emailId: string; client?: Pick<Client, 'jmap'> }): JSX.Element {
  return <EventCard accountId={props.accountId} emailId={props.emailId} {...(props.client ? { client: props.client } : {})} />;
}
