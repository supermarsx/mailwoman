// Event create/edit dialog (plan §3 e4): title / calendar / start / duration /
// all-day / location / description, a recurrence editor (common set), reminders
// (VALARM offsets), attendees, a free/busy status picker + a free/busy peek, and
// — when the event is an invite addressed to the user — accept / decline /
// tentative / counter controls (iTIP, plan §2.6). Conflicts for the edited event
// surface inline. All mutations go through the controller.

import { For, Show, createMemo, createSignal, type JSX } from 'solid-js';
import type { CalendarEvent, Participant } from '../../api/pim-types.ts';
import type { CalendarController, EventDraft } from './controller.ts';
import { dateToLocal, localToDate } from './datetime.ts';
import {
  describeRule,
  firstRule,
  formatDuration,
  parseDuration,
  ruleToJson,
  WEEKDAY_LABEL,
  WEEKDAYS,
  type RecurrenceRule,
  type Weekday,
} from './recurrence.ts';
import * as css from './calendar.css.ts';

export interface EventEditorProps {
  controller: CalendarController;
  /** The master being edited, or `null` to create. */
  event: CalendarEvent | null;
  /** Pre-fill the start when creating from a slot (defaults to now). */
  defaultStart?: Date;
  onClose: () => void;
}

/** `LocalDateTime` → the value an `<input type=datetime-local>` expects. */
function toInputDateTime(local: string): string {
  return local.slice(0, 16);
}
function toInputDate(local: string): string {
  return local.slice(0, 10);
}

interface AttendeeRow {
  name: string;
  email: string;
}

export function EventEditor(props: EventEditorProps): JSX.Element {
  const ev = props.event;
  const startDate = ev !== null ? localToDate(ev.start) : (props.defaultStart ?? new Date());

  const [calendarId, setCalendarId] = createSignal(ev?.calendarId ?? props.controller.visibleCalendars()[0]?.id ?? props.controller.calendars()[0]?.id ?? '');
  const [title, setTitle] = createSignal(ev?.title ?? '');
  const [description, setDescription] = createSignal(ev?.description ?? '');
  const [location, setLocation] = createSignal(ev?.locations[0]?.name ?? '');
  const [allDay, setAllDay] = createSignal(ev?.showWithoutTime ?? false);
  const [start, setStart] = createSignal(ev !== null ? ev.start : dateToLocal(startDate));
  const [durationMin, setDurationMin] = createSignal(
    ev !== null ? Math.max(15, Math.round(parseDuration(ev.duration) / 60000)) : 60,
  );
  const [freeBusyStatus, setFreeBusyStatus] = createSignal<CalendarEvent['freeBusyStatus']>(ev?.freeBusyStatus ?? 'busy');
  const [status, setStatus] = createSignal<CalendarEvent['status']>(ev?.status ?? 'confirmed');

  // ── recurrence ──
  const [recurs, setRecurs] = createSignal((ev !== null && firstRule(ev) !== null));
  const initialRule = ev !== null ? firstRule(ev) : null;
  const [freq, setFreq] = createSignal<RecurrenceRule['frequency']>(initialRule?.frequency ?? 'weekly');
  const [interval, setInterval] = createSignal(initialRule?.interval ?? 1);
  const [byDay, setByDay] = createSignal<Weekday[]>(initialRule?.byDay ?? []);
  const [endMode, setEndMode] = createSignal<'never' | 'count' | 'until'>(
    initialRule?.count !== undefined ? 'count' : initialRule?.until !== undefined ? 'until' : 'never',
  );
  const [count, setCount] = createSignal(initialRule?.count ?? 10);
  const [until, setUntil] = createSignal(initialRule?.until ?? '');

  const currentRule = createMemo<RecurrenceRule | null>(() => {
    if (!recurs()) return null;
    const rule: RecurrenceRule = { frequency: freq() };
    if (interval() > 1) rule.interval = interval();
    if (freq() === 'weekly' && byDay().length > 0) rule.byDay = byDay();
    if (endMode() === 'count') rule.count = count();
    else if (endMode() === 'until' && until() !== '') rule.until = until();
    return rule;
  });

  // ── reminders (offset minutes before start) ──
  const initialReminders = ev !== null ? extractReminders(ev) : [];
  const [reminders, setReminders] = createSignal<number[]>(initialReminders);

  // ── attendees ──
  const initialAttendees = ev !== null ? extractAttendees(ev) : [];
  const [attendees, setAttendees] = createSignal<AttendeeRow[]>(initialAttendees);
  const [newAttendee, setNewAttendee] = createSignal('');

  // ── invite state (is this event addressed to me and awaiting a reply?) ──
  const myParticipation = createMemo<Participant | null>(() => (ev !== null ? (ev.participants['me'] ?? null) : null));
  const isInvite = createMemo(() => myParticipation() !== null);

  const [counterOpen, setCounterOpen] = createSignal(false);
  const [counterStart, setCounterStart] = createSignal(ev !== null ? toInputDateTime(ev.start) : '');

  // ── conflict peek for the edited instance's day ──
  const conflictCount = createMemo(() => {
    if (ev === null) return 0;
    return props.controller
      .instancesForDay(localToDate(ev.start))
      .filter((i) => i.event.id === ev.id && props.controller.hasConflict(i.event.id)).length;
  });

  function toggleByDay(d: Weekday): void {
    setByDay((cur) => (cur.includes(d) ? cur.filter((x) => x !== d) : [...cur, d]));
  }

  function addAttendee(): void {
    const raw = newAttendee().trim();
    if (raw === '') return;
    setAttendees((a) => [...a, { name: raw.split('@')[0] ?? raw, email: raw }]);
    setNewAttendee('');
  }

  function buildDraft(): EventDraft {
    const s = allDay() ? toInputDate(start()) : start();
    const duration = allDay() ? 'P1D' : formatDuration(durationMin() * 60000);
    const rule = currentRule();
    const participants: CalendarEvent['participants'] = {};
    if (ev !== null && ev.participants['me'] !== undefined) participants['me'] = ev.participants['me'];
    attendees().forEach((a, i) => {
      participants[`a${i}`] = {
        name: a.name,
        email: a.email,
        role: 'attendee',
        participationStatus: 'needs-action',
        expectReply: true,
      };
    });
    const alerts: CalendarEvent['alerts'] = {};
    reminders().forEach((min, i) => {
      alerts[`r${i}`] = { trigger: { offset: `-PT${min}M` }, action: 'display' };
    });
    return {
      calendarId: calendarId(),
      title: title() === '' ? '(no title)' : title(),
      description: description(),
      start: s,
      timeZone: allDay() ? null : (ev?.timeZone ?? Intl.DateTimeFormat().resolvedOptions().timeZone),
      duration,
      showWithoutTime: allDay(),
      locations: location() === '' ? [] : [{ name: location() }],
      recurrenceRules: rule !== null ? [ruleToJson(rule)] : [],
      excludedRecurrenceDates: ev?.excludedRecurrenceDates ?? [],
      status: status(),
      freeBusyStatus: freeBusyStatus(),
      participants,
      alerts,
    };
  }

  async function save(): Promise<void> {
    const draft = buildDraft();
    if (ev === null) {
      await props.controller.createEvent(draft);
    } else {
      await props.controller.updateEvent(ev.id, {
        calendarId: draft.calendarId,
        title: draft.title,
        description: draft.description ?? '',
        start: draft.start,
        timeZone: draft.timeZone ?? null,
        duration: draft.duration ?? 'PT1H',
        showWithoutTime: draft.showWithoutTime ?? false,
        locations: draft.locations ?? [],
        recurrenceRules: draft.recurrenceRules ?? [],
        status: draft.status ?? 'confirmed',
        freeBusyStatus: draft.freeBusyStatus ?? 'busy',
        participants: draft.participants ?? {},
        alerts: draft.alerts ?? {},
      });
    }
    props.onClose();
  }

  async function remove(): Promise<void> {
    if (ev !== null) await props.controller.deleteEvent(ev.id);
    props.onClose();
  }

  async function respond(action: 'accept' | 'decline' | 'tentative'): Promise<void> {
    if (ev === null) return;
    await props.controller.respond(ev.id, action);
    props.onClose();
  }

  async function sendCounter(): Promise<void> {
    if (ev === null || counterStart() === '') return;
    await props.controller.respond(ev.id, 'counter', {
      start: counterStart(),
      duration: formatDuration(durationMin() * 60000),
    });
    props.onClose();
  }

  return (
    <div class={css.dialogBackdrop} onClick={(e) => e.target === e.currentTarget && props.onClose()}>
      <div class={css.dialog} role="dialog" aria-label={ev === null ? 'New event' : 'Edit event'} aria-modal="true">
        <h2 style={{ margin: 0, 'font-size': '1.1rem' }}>{ev === null ? 'New event' : 'Edit event'}</h2>

        <Show when={isInvite()}>
          <div class={css.inviteBar} role="group" aria-label="Invitation">
            <span>
              You're invited ({myParticipation()?.participationStatus}).
            </span>
            <button type="button" class={css.primaryButton} onClick={() => void respond('accept')}>
              Accept
            </button>
            <button type="button" class={css.button} onClick={() => void respond('tentative')}>
              Tentative
            </button>
            <button type="button" class={css.button} onClick={() => void respond('decline')}>
              Decline
            </button>
            <button type="button" class={css.button} onClick={() => setCounterOpen((v) => !v)}>
              Counter…
            </button>
          </div>
        </Show>
        <Show when={counterOpen()}>
          <div class={css.row}>
            <label class={css.label}>Propose new start</label>
            <input
              class={css.input}
              type="datetime-local"
              value={counterStart()}
              onInput={(e) => setCounterStart(e.currentTarget.value)}
            />
            <button type="button" class={css.primaryButton} onClick={() => void sendCounter()}>
              Send counter
            </button>
          </div>
        </Show>

        <Show when={conflictCount() > 0}>
          <p class={css.dangerText}>This event overlaps {conflictCount()} other event(s).</p>
        </Show>

        <div class={css.field}>
          <label class={css.label} for="ev-title">Title</label>
          <input id="ev-title" class={css.input} value={title()} onInput={(e) => setTitle(e.currentTarget.value)} />
        </div>

        <div class={css.row}>
          <div class={css.field} style={{ flex: 1 }}>
            <label class={css.label} for="ev-cal">Calendar</label>
            <select id="ev-cal" class={css.input} value={calendarId()} onChange={(e) => setCalendarId(e.currentTarget.value)}>
              <For each={props.controller.calendars()}>
                {(c) => <option value={c.id}>{c.name}</option>}
              </For>
            </select>
          </div>
          <label class={css.chip}>
            <input type="checkbox" checked={allDay()} onChange={(e) => setAllDay(e.currentTarget.checked)} /> All day
          </label>
        </div>

        <div class={css.row}>
          <div class={css.field} style={{ flex: 1 }}>
            <label class={css.label} for="ev-start">Start</label>
            <Show
              when={!allDay()}
              fallback={
                <input
                  id="ev-start"
                  class={css.input}
                  type="date"
                  value={toInputDate(start())}
                  onInput={(e) => setStart(`${e.currentTarget.value}T00:00:00`)}
                />
              }
            >
              <input
                id="ev-start"
                class={css.input}
                type="datetime-local"
                value={toInputDateTime(start())}
                onInput={(e) => setStart(`${e.currentTarget.value}:00`)}
              />
            </Show>
          </div>
          <Show when={!allDay()}>
            <div class={css.field}>
              <label class={css.label} for="ev-dur">Duration (min)</label>
              <input
                id="ev-dur"
                class={css.input}
                type="number"
                min="15"
                step="15"
                style={{ width: '6rem' }}
                value={durationMin()}
                onInput={(e) => setDurationMin(Math.max(15, Number(e.currentTarget.value) || 15))}
              />
            </div>
          </Show>
        </div>

        <div class={css.field}>
          <label class={css.label} for="ev-loc">Location</label>
          <input id="ev-loc" class={css.input} value={location()} onInput={(e) => setLocation(e.currentTarget.value)} />
        </div>

        <div class={css.row}>
          <div class={css.field}>
            <label class={css.label} for="ev-fb">Shows as</label>
            <select id="ev-fb" class={css.input} value={freeBusyStatus()} onChange={(e) => setFreeBusyStatus(e.currentTarget.value as CalendarEvent['freeBusyStatus'])}>
              <option value="busy">Busy</option>
              <option value="free">Free</option>
            </select>
          </div>
          <div class={css.field}>
            <label class={css.label} for="ev-status">Status</label>
            <select id="ev-status" class={css.input} value={status()} onChange={(e) => setStatus(e.currentTarget.value as CalendarEvent['status'])}>
              <option value="confirmed">Confirmed</option>
              <option value="tentative">Tentative</option>
              <option value="cancelled">Cancelled</option>
            </select>
          </div>
        </div>

        {/* ── recurrence ── */}
        <div class={css.field}>
          <label class={css.chip}>
            <input type="checkbox" checked={recurs()} onChange={(e) => setRecurs(e.currentTarget.checked)} /> Repeats
          </label>
          <Show when={recurs()}>
            <div class={css.row}>
              <select class={css.input} value={freq()} onChange={(e) => setFreq(e.currentTarget.value as RecurrenceRule['frequency'])} aria-label="Frequency">
                <option value="daily">Daily</option>
                <option value="weekly">Weekly</option>
                <option value="monthly">Monthly</option>
                <option value="yearly">Yearly</option>
              </select>
              <label class={css.label}>every</label>
              <input class={css.input} type="number" min="1" style={{ width: '4rem' }} value={interval()} onInput={(e) => setInterval(Math.max(1, Number(e.currentTarget.value) || 1))} aria-label="Interval" />
            </div>
            <Show when={freq() === 'weekly'}>
              <div class={css.row}>
                <For each={WEEKDAYS}>
                  {(d) => (
                    <label class={css.chip}>
                      <input type="checkbox" checked={byDay().includes(d)} onChange={() => toggleByDay(d)} /> {WEEKDAY_LABEL[d]}
                    </label>
                  )}
                </For>
              </div>
            </Show>
            <div class={css.row}>
              <label class={css.label}>Ends</label>
              <select class={css.input} value={endMode()} onChange={(e) => setEndMode(e.currentTarget.value as 'never' | 'count' | 'until')} aria-label="End mode">
                <option value="never">Never</option>
                <option value="count">After N</option>
                <option value="until">On date</option>
              </select>
              <Show when={endMode() === 'count'}>
                <input class={css.input} type="number" min="1" style={{ width: '5rem' }} value={count()} onInput={(e) => setCount(Math.max(1, Number(e.currentTarget.value) || 1))} aria-label="Occurrences" />
              </Show>
              <Show when={endMode() === 'until'}>
                <input class={css.input} type="date" value={until().slice(0, 10)} onInput={(e) => setUntil(`${e.currentTarget.value}T00:00:00`)} aria-label="Until date" />
              </Show>
            </div>
            <Show when={currentRule() !== null}>
              <p class={css.dimText}>{describeRule(currentRule()!)}</p>
            </Show>
          </Show>
        </div>

        {/* ── reminders ── */}
        <div class={css.field}>
          <label class={css.label}>Reminders</label>
          <div class={css.row}>
            <For each={reminders()}>
              {(min, i) => (
                <span class={css.chip}>
                  {min}m before
                  <button type="button" class={css.button} aria-label={`Remove reminder ${min}`} onClick={() => setReminders((r) => r.filter((_, j) => j !== i()))}>
                    ×
                  </button>
                </span>
              )}
            </For>
            <select class={css.input} onChange={(e) => { const v = Number(e.currentTarget.value); if (v > 0) setReminders((r) => [...r, v]); e.currentTarget.value = ''; }} aria-label="Add reminder">
              <option value="">+ reminder</option>
              <option value="5">5 min</option>
              <option value="15">15 min</option>
              <option value="30">30 min</option>
              <option value="60">1 hour</option>
              <option value="1440">1 day</option>
            </select>
          </div>
        </div>

        {/* ── attendees ── */}
        <div class={css.field}>
          <label class={css.label}>Attendees</label>
          <div class={css.row}>
            <For each={attendees()}>
              {(a, i) => (
                <span class={css.chip}>
                  {a.email}
                  <button type="button" class={css.button} aria-label={`Remove ${a.email}`} onClick={() => setAttendees((cur) => cur.filter((_, j) => j !== i()))}>
                    ×
                  </button>
                </span>
              )}
            </For>
          </div>
          <div class={css.row}>
            <input
              class={css.input}
              placeholder="name@example.com"
              value={newAttendee()}
              onInput={(e) => setNewAttendee(e.currentTarget.value)}
              onKeyDown={(e) => e.key === 'Enter' && addAttendee()}
              aria-label="Add attendee"
            />
            <button type="button" class={css.button} onClick={addAttendee}>Add</button>
          </div>
        </div>

        <div class={css.field}>
          <label class={css.label} for="ev-desc">Notes</label>
          <textarea id="ev-desc" class={css.input} rows="3" value={description()} onInput={(e) => setDescription(e.currentTarget.value)} />
        </div>

        <div class={css.dialogActions}>
          <Show when={ev !== null}>
            <button type="button" class={css.button} onClick={() => void remove()}>Delete</button>
          </Show>
          <span class={css.spacer} />
          <button type="button" class={css.button} onClick={props.onClose}>Cancel</button>
          <button type="button" class={css.primaryButton} onClick={() => void save()}>Save</button>
        </div>
      </div>
    </div>
  );
}

/** Pull reminder offsets (minutes before start) out of an event's VALARMs. */
export function extractReminders(ev: CalendarEvent): number[] {
  const out: number[] = [];
  for (const alert of Object.values(ev.alerts)) {
    const offset = (alert.trigger as { offset?: unknown }).offset;
    if (typeof offset === 'string') {
      const m = /^-?PT(?:(\d+)H)?(?:(\d+)M)?$/.exec(offset);
      if (m !== null) out.push(Number(m[1] ?? 0) * 60 + Number(m[2] ?? 0));
    }
  }
  return out;
}

/** Pull the non-self attendees out of an event's participants map. */
export function extractAttendees(ev: CalendarEvent): AttendeeRow[] {
  return Object.entries(ev.participants)
    .filter(([id]) => id !== 'me')
    .map(([, p]) => ({ name: p.name, email: p.email }));
}
