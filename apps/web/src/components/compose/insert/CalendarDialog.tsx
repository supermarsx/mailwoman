// Insert calendar: availability (free slots), agenda, or a month snapshot of the
// user's calendars. Only free time, or titles/times per the visibility option.

import { createMemo, createResource, createSignal, For, Show, type JSX } from 'solid-js';
import { t } from '../../../i18n/index.ts';
import type { CalendarController } from '../../../modules/calendar/controller.ts';
import { expandEvent } from '../../../modules/calendar/recurrence.ts';
import { InsertDialog } from './InsertDialog.tsx';
import { agendaHtml, availabilityHtml, snapshotHtml, zonedToUtc, type AgendaEvent, type Interval, type Visibility } from './calendarInsert.ts';
import { TimeZoneInput, browserTimeZone } from './TimeZoneInput.tsx';
import { loadCatalog } from '../../../i18n/catalog.ts';
// i18n: the ids this file shows live in these catalogs, which load with it.
void loadCatalog('compose-insert');

export type CalendarKind = 'availability' | 'agenda' | 'snapshot';

const isoDay = (d: Date): string => `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;

export function CalendarDialog(props: { mode: CalendarKind; controller: CalendarController; returnTo: HTMLElement | null; onClose: () => void; onInsert: (html: string, text: string) => void }): JSX.Element {
  const today = new Date();
  const [from, setFrom] = createSignal(isoDay(today));
  const [to, setTo] = createSignal(isoDay(new Date(today.getTime() + 6 * 86_400_000)));
  const [month, setMonth] = createSignal(`${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}`);
  const [tz, setTz] = createSignal(browserTimeZone());
  const [visibility, setVisibility] = createSignal<Visibility>('busy');
  const [calendarId, setCalendarId] = createSignal('');
  const [workStart, setWorkStart] = createSignal(9);
  const [workEnd, setWorkEnd] = createSignal(17);
  const [error, setError] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [loaded] = createResource(async () => { try { await props.controller.load(); } catch { setError(t('ins-cal-load-failed')); } return true; });

  const range = createMemo((): [number, number] => {
    if (props.mode === 'snapshot') {
      const [y, m] = month().split('-').map(Number) as [number, number];
      return [zonedToUtc(y, m, 1, 0, 0, tz()), zonedToUtc(m === 12 ? y + 1 : y, m === 12 ? 1 : m + 1, 1, 0, 0, tz())];
    }
    const [fy, fm, fd] = from().split('-').map(Number) as [number, number, number];
    const [ty, tm, td] = to().split('-').map(Number) as [number, number, number];
    return [zonedToUtc(fy, fm, fd, 0, 0, tz()), zonedToUtc(ty, tm, td, 23, 59, tz()) + 60_000];
  });
  const events = (): AgendaEvent[] => {
    const [s, e] = range();
    const calendars = props.controller.calendars();
    const out: AgendaEvent[] = [];
    for (const master of props.controller.masters()) {
      if (calendarId() !== '' && master.calendarId !== calendarId()) continue;
      const color = calendars.find((c) => c.id === master.calendarId)?.color ?? '';
      for (const inst of expandEvent(master, new Date(s), new Date(e), color)) {
        const loc = master.locations[0];
        out.push({ start: inst.start.toISOString(), end: inst.end.toISOString(), title: inst.event.title, allDay: inst.allDay, ...(loc?.name ? { location: loc.name } : {}) });
      }
    }
    return out;
  };
  const labels = () => ({ day: t('ins-cal-day'), free: t('ins-cal-free'), none: t('ins-cal-none'), note: t('ins-cal-note', { tz: tz() }), busy: t('ins-cal-busy'), allDay: t('ins-cal-allday'), empty: t('ins-cal-empty') });
  const build = async (): Promise<{ html: string; text: string }> => {
    const l = labels();
    if (props.mode === 'availability') {
      const [s, e] = range();
      const blocks = await props.controller.queryFreeBusy(new Date(s), new Date(e));
      const busyIntervals: Interval[] = blocks.flatMap((b) => ('start' in b ? [{ start: b.start, end: b.end }] : []));
      const html = availabilityHtml(busyIntervals, { from: from(), to: to(), tz: tz(), workStart: workStart(), workEnd: workEnd(), labels: l });
      return { html, text: html.replace(/<\/tr>/g, '\n').replace(/<\/td>/g, '  ').replace(/<\/p>/g, '\n').replace(/<[^>]+>/g, '').replace(/&amp;/g, '&').trim() };
    }
    if (props.mode === 'agenda') {
      const html = agendaHtml(events(), { tz: tz(), visibility: visibility(), labels: l });
      return { html, text: html.replace(/<\/tr>/g, '\n').replace(/<\/td>/g, '  ').replace(/<\/p>/g, '\n').replace(/<[^>]+>/g, '').trim() };
    }
    const [y, m] = month().split('-').map(Number) as [number, number];
    const html = snapshotHtml(events(), { year: y, month: m, tz: tz(), visibility: visibility(), labels: { note: l.note } });
    return { html, text: html.replace(/<\/tr>/g, '\n').replace(/<\/t[dh]>/g, '\t').replace(/<[^>]+>/g, '').trim() };
  };
  const insert = (): void => {
    setBusy(true);
    setError(null);
    void build().then((out) => props.onInsert(out.html, out.text)).catch(() => setError(t('ins-cal-load-failed'))).finally(() => setBusy(false));
  };
  const title = (): string => t(`ins-cal-${props.mode}`);
  return (
    <InsertDialog title={title()} testId="insert-calendar-dialog" returnTo={props.returnTo} busy={busy() || loaded.loading} onClose={props.onClose} onSubmit={insert}>
      <Show when={props.mode !== 'snapshot'} fallback={<label>{t('ins-cal-month')}<input type="month" data-testid="insert-cal-month" value={month()} onInput={(e) => setMonth(e.currentTarget.value)} /></label>}>
        <div class="mw-insert-row">
          <label>{t('ins-cal-from')}<input type="date" data-testid="insert-cal-from" value={from()} onInput={(e) => setFrom(e.currentTarget.value)} /></label>
          <label>{t('ins-cal-to')}<input type="date" data-testid="insert-cal-to" value={to()} onInput={(e) => setTo(e.currentTarget.value)} /></label>
        </div>
      </Show>
      <TimeZoneInput value={tz()} onChange={setTz} testId="insert-cal-tz" />
      <Show when={props.mode === 'availability'}>
        <div class="mw-insert-row">
          <label>{t('ins-cal-work-start')}<input type="number" min="0" max="23" data-testid="insert-cal-work-start" value={workStart()} onInput={(e) => setWorkStart(Math.max(0, Math.min(23, Number(e.currentTarget.value) || 0)))} /></label>
          <label>{t('ins-cal-work-end')}<input type="number" min="1" max="24" data-testid="insert-cal-work-end" value={workEnd()} onInput={(e) => setWorkEnd(Math.max(1, Math.min(24, Number(e.currentTarget.value) || 24)))} /></label>
        </div>
        <p class="mw-insert-note">{t('ins-cal-availability-note')}</p>
      </Show>
      <Show when={props.mode !== 'availability'}>
        <div class="mw-insert-row">
          <label>{t('ins-cal-calendar')}
            <select data-testid="insert-cal-calendar" value={calendarId()} onChange={(e) => setCalendarId(e.currentTarget.value)}>
              <option value="">{t('ins-cal-all')}</option>
              <For each={props.controller.calendars()}>{(c) => <option value={c.id}>{c.name}</option>}</For>
            </select>
          </label>
          <label>{t('ins-cal-visibility')}
            <select data-testid="insert-cal-visibility" value={visibility()} onChange={(e) => setVisibility(e.currentTarget.value as Visibility)}>
              <option value="busy">{t('ins-cal-vis-busy')}</option>
              <option value="titles">{t('ins-cal-vis-titles')}</option>
            </select>
          </label>
        </div>
      </Show>
      <Show when={error()}>{(m) => <p class="mw-insert-warn" role="alert">{m()}</p>}</Show>
    </InsertDialog>
  );
}
