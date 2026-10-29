// The Events view's row hint (t78): a calendar mark, what the message does
// (an invitation, an update, a cancellation, a reply, a counter-proposal or a
// plain event) and when the event starts. It sits at the start of the row's
// preview line, inside that line's fixed height, so an Events row is exactly
// as tall as any other row. The data is the server's `mailwomanCalendar`,
// read from the search index at ingest: no message body is fetched for it.

import { Show, type JSX } from 'solid-js';
import type { CalendarHint } from '../api/jmap-types.ts';
import { formatDateWith, t } from '../i18n/index.ts';
import { Icon } from './Icon.tsx';
import { messageTimeZone } from './TimeHelpers.tsx';
import * as css from './eventHint.css.ts';

export type EventState = 'invitation' | 'updated' | 'cancelled' | 'reply' | 'counter' | 'event';

/** What the message does, from its iTIP METHOD (RFC 5546) and the event's SEQUENCE/STATUS. */
export function eventState(hint: CalendarHint): EventState {
  const method = (hint.method ?? '').toUpperCase();
  if (method === 'CANCEL' || hint.status === 'cancelled') return 'cancelled';
  if (method === 'REPLY') return 'reply';
  if (method === 'COUNTER') return 'counter';
  if (method === 'REQUEST') return (hint.sequence ?? 0) > 0 ? 'updated' : 'invitation';
  return 'event';
}

/** When the event starts, in the reader's locale and display time zone (the
 *  one the list's dates use, `time.displayZone`); '' when unknown.
 *  An all-day event is a date, never shifted by a time zone. */
export function eventWhen(hint: CalendarHint, now: Date = new Date()): string {
  const start = hint.start ?? '';
  // The year only when it is not this one: an old calendar file must not read as this year's.
  const year = (y: number): Intl.DateTimeFormatOptions => (y !== now.getFullYear() ? { year: 'numeric' } : {});
  if (hint.showWithoutTime === true || /^\d{4}-\d{2}-\d{2}$/.test(start)) {
    const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(start);
    if (m === null) return '';
    return formatDateWith(Date.UTC(Number(m[1]), Number(m[2]) - 1, Number(m[3])), { weekday: 'short', day: 'numeric', month: 'short', ...year(Number(m[1])), timeZone: 'UTC' });
  }
  const startYear = typeof hint.startUtc === 'number' ? new Date(hint.startUtc * 1000).getUTCFullYear() : Number(start.slice(0, 4));
  const options: Intl.DateTimeFormatOptions = { weekday: 'short', day: 'numeric', month: 'short', ...year(startYear), hour: '2-digit', minute: '2-digit' };
  if (typeof hint.startUtc === 'number') return formatDateWith(hint.startUtc * 1000, { ...options, timeZone: messageTimeZone() });
  // No zone the server could resolve: show the wall time as written.
  return /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}/.test(start) ? formatDateWith(start, options) : '';
}

/** The hint for one row; nothing when the message carries no calendar content. */
export function EventHint(props: { hint: CalendarHint | null | undefined }): JSX.Element {
  return (
    <Show when={props.hint}>
      {(hint) => {
        const state = () => eventState(hint());
        const when = () => eventWhen(hint());
        const label = () => t(`mail-event-state-${state()}`);
        return (
          <span class={css.hint} data-testid="row-event-hint" data-event-state={state()} title={when() ? t('mail-event-hint', { state: label(), when: when() }) : label()}>
            <span class={css.icon} aria-hidden="true"><Icon name="calendar" /></span>
            <span>{label()}</span>
            <Show when={when()}>{(text) => <span>· {text()}</span>}</Show>
          </span>
        );
      }}
    </Show>
  );
}
