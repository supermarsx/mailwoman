// Insert event: pick an existing event of the user's calendars, or write a new one.
// Inserts an HTML summary block and returns the .ics for the host to attach
// (METHOD:REQUEST with ORGANIZER when there are attendees, else PUBLISH).

import { createMemo, createResource, createSignal, For, Show, type JSX } from 'solid-js';
import { t } from '../../../i18n/index.ts';
import type { CalendarController } from '../../../modules/calendar/controller.ts';
import { expandEvent } from '../../../modules/calendar/recurrence.ts';
import { InsertDialog } from './InsertDialog.tsx';
import { buildEventIcs, eventSummaryHtml, localToInstant, type EventDraft } from './calendarInsert.ts';
import { TimeZoneInput, browserTimeZone } from './TimeZoneInput.tsx';

const pad = (n: number): string => String(n).padStart(2, '0');
const dateOf = (d: Date): string => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
const timeOf = (d: Date): string => `${pad(d.getHours())}:${pad(d.getMinutes())}`;

interface Candidate { key: string; title: string; start: Date; end: Date; location: string; description: string; attendees: string[] }

export function EventDialog(props: {
  existing: boolean;
  controller: CalendarController;
  identity: { name: string; email: string } | null;
  returnTo: HTMLElement | null;
  onClose: () => void;
  onInsert: (html: string, text: string, ics: { method: 'REQUEST' | 'PUBLISH'; text: string }) => void;
}): JSX.Element {
  const soon = new Date(Date.now() + 86_400_000);
  soon.setHours(10, 0, 0, 0);
  const [title, setTitle] = createSignal('');
  const [date, setDate] = createSignal(dateOf(soon));
  const [start, setStart] = createSignal('10:00');
  const [end, setEnd] = createSignal('11:00');
  const [tz, setTz] = createSignal(browserTimeZone());
  const [location, setLocation] = createSignal('');
  const [description, setDescription] = createSignal('');
  const [attendees, setAttendees] = createSignal('');
  const [alsoCreate, setAlsoCreate] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [loaded] = createResource(async () => { if (props.existing) { try { await props.controller.load(); } catch { setError(t('ins-cal-load-failed')); } } return true; });

  const candidates = createMemo((): Candidate[] => {
    if (!props.existing || loaded.loading) return [];
    const now = new Date(), until = new Date(now.getTime() + 90 * 86_400_000);
    const calendars = props.controller.calendars();
    const out: Candidate[] = [];
    for (const master of props.controller.masters()) {
      const color = calendars.find((c) => c.id === master.calendarId)?.color ?? '';
      for (const inst of expandEvent(master, now, until, color)) {
        out.push({
          key: inst.key, title: inst.event.title, start: inst.start, end: inst.end,
          location: master.locations[0]?.name ?? '', description: master.description,
          attendees: Object.values(master.participants).map((p) => p.email ?? '').filter((e) => e !== '' && e.toLowerCase() !== (props.identity?.email ?? '').toLowerCase()),
        });
      }
    }
    return out.sort((a, b) => a.start.getTime() - b.start.getTime()).slice(0, 100);
  });
  const choose = (c: Candidate): void => {
    setTitle(c.title); setDate(dateOf(c.start)); setStart(timeOf(c.start)); setEnd(timeOf(c.end));
    setLocation(c.location); setDescription(c.description); setAttendees(c.attendees.join(', '));
  };
  const attendeeList = (): string[] => attendees().split(/[,;\s]+/).map((a) => a.trim()).filter((a) => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(a));
  const draft = (): EventDraft => {
    const s = localToInstant(date(), start(), tz());
    let e = localToInstant(date(), end(), tz());
    if (e <= s) e = s + 3_600_000;
    return {
      uid: `${crypto.randomUUID()}@mailwoman`, title: title().trim(), start: s, end: e, tz: tz(),
      ...(location().trim() ? { location: location().trim() } : {}), ...(description().trim() ? { description: description().trim() } : {}),
      attendees: attendeeList(), organizer: { name: props.identity?.name ?? '', email: props.identity?.email ?? '' },
    };
  };
  const insert = (): void => {
    const ev = draft();
    if (ev.title === '' || ev.organizer.email === '') return;
    const labels = { when: t('ins-event-when'), where: t('ins-event-where'), who: t('ins-event-who') };
    const html = eventSummaryHtml(ev, labels);
    const text = [ev.title, `${labels.when}: ${new Intl.DateTimeFormat(undefined, { timeZone: ev.tz, dateStyle: 'full', timeStyle: 'short' }).format(new Date(ev.start))} (${ev.tz})`, ev.location ? `${labels.where}: ${ev.location}` : '', ev.attendees.length ? `${labels.who}: ${ev.attendees.join(', ')}` : ''].filter(Boolean).join('\n');
    const ics = buildEventIcs(ev);
    const finish = (): void => props.onInsert(html, text, ics);
    if (!alsoCreate()) { finish(); return; }
    setBusy(true);
    const calendar = props.controller.calendars().find((c) => c.role === 'default') ?? props.controller.calendars()[0];
    if (!calendar) { setBusy(false); setError(t('ins-event-no-calendar')); return; }
    void props.controller.createEvent({
      calendarId: calendar.id, title: ev.title, start: `${date()}T${start()}:00`, timeZone: ev.tz,
      duration: `PT${Math.max(1, Math.round((ev.end - ev.start) / 60_000))}M`,
      ...(ev.location ? { locations: [{ name: ev.location }] } : {}), ...(ev.description ? { description: ev.description } : {}),
    }).then((id) => { if (id === null) setError(t('ins-event-create-failed')); else finish(); }).catch(() => setError(t('ins-event-create-failed'))).finally(() => setBusy(false));
  };
  const fmt = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' });
  return (
    <InsertDialog title={t(props.existing ? 'ins-event-existing' : 'ins-event-new')} testId="insert-event-dialog" returnTo={props.returnTo} busy={busy() || loaded.loading} onClose={props.onClose} onSubmit={insert} submitDisabled={title().trim() === '' || props.identity === null}>
      <Show when={props.existing}>
        <ul role="listbox" aria-label={t('ins-event-existing')} style={{ 'list-style': 'none', margin: '0', padding: '0', 'max-height': '9rem', overflow: 'auto' }}>
          <For each={candidates()} fallback={<li class="mw-insert-warn" style={{ color: 'inherit' }}>{loaded.loading ? t('ins-loading') : t('ins-event-none')}</li>}>{(c) => (
            <li><button type="button" role="option" class="btn btn--ghost" style={{ width: '100%', 'text-align': 'start' }} data-testid="insert-event-pick" aria-selected={title() === c.title && date() === dateOf(c.start)} onClick={() => choose(c)}>{fmt.format(c.start)} · {c.title}</button></li>
          )}</For>
        </ul>
      </Show>
      <label>{t('ins-event-title')}<input data-testid="insert-event-title" value={title()} onInput={(e) => setTitle(e.currentTarget.value)} /></label>
      <div class="mw-insert-row">
        <label>{t('ins-event-date')}<input type="date" data-testid="insert-event-date" value={date()} onInput={(e) => setDate(e.currentTarget.value)} /></label>
        <label>{t('ins-event-start')}<input type="time" data-testid="insert-event-start" value={start()} onInput={(e) => setStart(e.currentTarget.value)} /></label>
        <label>{t('ins-event-end')}<input type="time" data-testid="insert-event-end" value={end()} onInput={(e) => setEnd(e.currentTarget.value)} /></label>
      </div>
      <TimeZoneInput value={tz()} onChange={setTz} testId="insert-event-tz" />
      <label>{t('ins-event-where')}<input data-testid="insert-event-location" value={location()} onInput={(e) => setLocation(e.currentTarget.value)} /></label>
      <label>{t('ins-event-description')}<textarea rows="3" data-testid="insert-event-description" value={description()} onInput={(e) => setDescription(e.currentTarget.value)} /></label>
      <label>{t('ins-event-attendees')}<input data-testid="insert-event-attendees" value={attendees()} placeholder="a@example.org, b@example.org" onInput={(e) => setAttendees(e.currentTarget.value)} /></label>
      <p class="mw-insert-warn" style={{ color: 'inherit' }} data-testid="insert-event-method">{attendeeList().length > 0 ? t('ins-event-method-request') : t('ins-event-method-publish')}</p>
      <label class="mw-insert-check"><input type="checkbox" data-testid="insert-event-also-create" checked={alsoCreate()} onChange={(e) => setAlsoCreate(e.currentTarget.checked)} />{t('ins-event-also-create')}</label>
      <Show when={error()}>{(m) => <p class="mw-insert-warn" role="alert">{m()}</p>}</Show>
    </InsertDialog>
  );
}
