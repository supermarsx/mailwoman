// The Outlook-style mini day agenda beside an event card (t53): the account's
// own calendar for the event's day, a few hours either side, the event drawn
// over it, and every own event that overlaps it marked as a conflict. Uses the
// calendar module's column layout and chip ink so it reads like the day view.

import { createResource, For, Show, type JSX } from 'solid-js';
import type { Client } from '../../api/client.ts';
import { formatDateWith, formatList, t } from '../../i18n';
import { wallTimeCandidates } from '../../i18n/timezones.ts';
import { inkOn, layoutColumns } from '../../modules/calendar/layout.ts';
import { instancesBetween, type DayInstance, type PickableCalendar } from './api.ts';
import { offsetMinutes, type EventRange } from './eventInfo.ts';
import * as css from './eventCard.css.ts';

const HOUR_REM = 1.75;
const GUTTER = '2.6rem';

/** Minutes since local midnight of `d` in `zone`. */
function localMinutes(d: Date, zone: string): number {
  const utc = Math.floor(d.getTime() / 60_000);
  return (((utc + offsetMinutes(zone, d)) % 1440) + 1440) % 1440;
}

/** The instant of local midnight of the day `d` falls on, in `zone`, and the next. */
export function dayBounds(d: Date, zone: string): { from: Date; to: Date; ymd: string } {
  const ymd = formatDateWith(d, { timeZone: zone, year: 'numeric', month: '2-digit', day: '2-digit' });
  const parts = new Intl.DateTimeFormat('en-CA', { timeZone: zone, year: 'numeric', month: '2-digit', day: '2-digit' }).format(d);
  const from = wallTimeCandidates(`${parts}T00:00:00`, zone)[0] ?? new Date(`${parts}T00:00:00Z`);
  const next = new Date(from.getTime() + 26 * 3_600_000);
  const nextParts = new Intl.DateTimeFormat('en-CA', { timeZone: zone, year: 'numeric', month: '2-digit', day: '2-digit' }).format(next);
  const to = wallTimeCandidates(`${nextParts}T00:00:00`, zone)[0] ?? new Date(from.getTime() + 86_400_000);
  return { from, to, ymd };
}

/** Own instances that overlap the event: busy, not cancelled, not the event itself. */
export function conflictsWith(range: EventRange, uid: string | undefined, own: DayInstance[]): DayInstance[] {
  if (range.allDay) return [];
  return own.filter((i) => !i.allDay && i.uid !== uid && i.status !== 'cancelled' && i.freeBusy !== 'free'
    && i.start < range.end && i.end > range.start);
}

export function DayAgenda(props: {
  accountId: string;
  client: Pick<Client, 'jmap'>;
  range: EventRange;
  uid?: string | undefined;
  zone: string;
  calendars: PickableCalendar[];
}): JSX.Element {
  const day = (): { from: Date; to: Date; ymd: string } => {
    if (props.range.allDay) {
      const ymd = props.range.start.toISOString().slice(0, 10);
      const from = wallTimeCandidates(`${ymd}T00:00:00`, props.zone)[0] ?? props.range.start;
      return { from, to: new Date(from.getTime() + 86_400_000), ymd };
    }
    return dayBounds(props.range.start, props.zone);
  };
  const [own] = createResource(
    () => ({ accountId: props.accountId, from: day().from.getTime(), to: day().to.getTime() }),
    async ({ accountId, from, to }) => instancesBetween(props.client, accountId, new Date(from), new Date(to)),
  );
  const colour = (calendarId: string): string => props.calendars.find((c) => c.id === calendarId)?.color || '#3b82f6';
  // The hours shown: an hour before the event to an hour after, at least four.
  const hours = (): { first: number; last: number } => {
    if (props.range.allDay) return { first: 8, last: 18 };
    const s = localMinutes(props.range.start, props.zone);
    const sameDay = props.range.end.getTime() - day().from.getTime() < 86_400_000;
    const e = sameDay ? localMinutes(props.range.end, props.zone) || 1440 : 1440;
    let first = Math.max(0, Math.floor(s / 60) - 1);
    let last = Math.min(24, Math.ceil(e / 60) + 1);
    while (last - first < 4) { if (first > 0) first -= 1; if (last - first < 4 && last < 24) last += 1; if (first === 0 && last === 24) break; }
    return { first, last };
  };
  const visible = (): DayInstance[] => (own() ?? []).filter((i) => i.uid !== props.uid && i.status !== 'cancelled');
  const timed = (): DayInstance[] => visible().filter((i) => !i.allDay && i.end.getTime() - i.start.getTime() < 86_400_000);
  const allDay = (): DayInstance[] => visible().filter((i) => i.allDay || i.end.getTime() - i.start.getTime() >= 86_400_000);
  const conflicts = (): DayInstance[] => conflictsWith(props.range, props.uid, own() ?? []);
  // Minutes from the window start, clamped to the window.
  const pos = (d: Date): number => {
    const fromDay = (d.getTime() - day().from.getTime()) / 60_000;
    return Math.min(Math.max(fromDay, hours().first * 60), hours().last * 60) - hours().first * 60;
  };
  const blocks = (): Array<{ inst: DayInstance | null; top: number; height: number; col: number; cols: number }> => {
    const items: Array<{ inst: DayInstance | null; start: number; end: number }> = timed()
      .map((inst) => ({ inst, start: pos(inst.start), end: pos(inst.end) }))
      .filter((b) => b.end > b.start);
    if (!props.range.allDay) items.push({ inst: null, start: pos(props.range.start), end: Math.max(pos(props.range.end), pos(props.range.start) + 15) });
    const slots = layoutColumns(items);
    return items.map((b, n) => ({ inst: b.inst, top: (b.start / 60) * HOUR_REM, height: Math.max(0.9, ((b.end - b.start) / 60) * HOUR_REM), col: slots[n]!.col, cols: slots[n]!.cols }));
  };
  const hourLabel = (h: number): string => formatDateWith(new Date(day().from.getTime() + h * 3_600_000), { hour: 'numeric', timeZone: props.zone });
  return (
    <section class={css.agenda} aria-label={t('calendar-card-agenda', { day: formatDateWith(props.range.allDay ? props.range.start : props.range.start, { weekday: 'long', day: 'numeric', month: 'long', timeZone: props.range.allDay ? 'UTC' : props.zone }) })}>
      <p class={css.agendaHead}>
        <span>{formatDateWith(props.range.start, { weekday: 'short', day: 'numeric', month: 'short', timeZone: props.range.allDay ? 'UTC' : props.zone })}</span>
        <span>{t('calendar-card-agenda-title')}</span>
      </p>
      <Show when={own.error}><p role="status" class={css.dim}>{t('calendar-card-agenda-unavailable')}</p></Show>
      <Show when={!own.error}>
        <Show when={allDay().length > 0}>
          <div class={css.agendaAllDay}>
            <For each={allDay()}>{(i) => <span class={css.agendaChip} style={{ 'background-color': colour(i.calendarId), color: inkOn(colour(i.calendarId)) }} dir="auto">{i.title || t('calendar-untitled')}</span>}</For>
          </div>
        </Show>
        <div class={css.agendaGrid} style={{ 'block-size': `${(hours().last - hours().first) * HOUR_REM}rem` }}>
          <For each={Array.from({ length: hours().last - hours().first }, (_, n) => hours().first + n)}>
            {(h) => <div class={css.agendaHour} aria-hidden="true">{hourLabel(h)}</div>}
          </For>
          <For each={blocks()}>{(b) => {
            const width = `calc((100% - ${GUTTER}) / ${b.cols})`;
            const style: JSX.CSSProperties = {
              top: `${b.top}rem`, 'block-size': `${b.height}rem`,
              'inset-inline-start': `calc(${GUTTER} + (100% - ${GUTTER}) * ${b.col} / ${b.cols})`, 'inline-size': width,
            };
            if (b.inst === null) {
              return <div class={`${css.agendaBlock} ${css.agendaInvite}`} style={style} data-agenda="invite">{t('calendar-card-agenda-this-event')}</div>;
            }
            const conflict = conflicts().some((c) => c.eventId === b.inst!.eventId && c.start.getTime() === b.inst!.start.getTime());
            const c = colour(b.inst.calendarId);
            return (
              <div class={`${css.agendaBlock} ${conflict ? css.agendaConflict : ''}`} style={{ ...style, 'background-color': c, color: inkOn(c) }}
                data-agenda={conflict ? 'conflict' : 'event'} title={b.inst.title} dir="auto">
                {b.inst.title || t('calendar-untitled')}
              </div>
            );
          }}</For>
        </div>
        <Show when={!own.loading && !props.range.allDay}>
          <p class={`${css.agendaSummary} ${conflicts().length > 0 ? css.agendaSummaryConflict : ''}`} role="status">
            {conflicts().length > 0
              ? t('calendar-card-conflicts', { count: conflicts().length, titles: formatList([...new Set(conflicts().map((c) => c.title || t('calendar-untitled')))]) })
              : t('calendar-card-no-conflicts')}
          </p>
        </Show>
      </Show>
    </section>
  );
}
