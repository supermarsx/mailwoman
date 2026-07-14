// The nine calendar views (plan §3 e4): day / 3-day / work-week / week share the
// vertical time grid; month + tri-month share the month grid; schedule + agenda
// share the list; year is a 12-mini-month overview. Every view renders the same
// `EventInstance` shape the controller exposes (engine-expanded), so views are
// agnostic to the mock-vs-engine backend. Conflict badges come from the
// controller's `hasConflict` set.
//
// a11y (t8-e2, SPEC §24): the month view is the flagship WAI-ARIA `grid` — a
// `role=grid` of `role=row` weeks and `role=gridcell` days with a single roving
// tabindex, full arrow-key date navigation (RTL-mirrored), and a polite live
// region announcing the focused day + its event count. The time grid's day
// headers form a companion navigable row. All user-controlled event titles are
// bidi-isolated (`<bdi>` for display, `isolate()` for accessible names).

import { For, Match, Show, Switch, createEffect, createMemo, createSignal, type JSX } from 'solid-js';
import { t, isolate } from '../../i18n';
import type { CalendarController } from './controller.ts';
import type { CalendarView, EventInstance } from './types.ts';
import { effectiveDir, inWindow, nextGridDate } from './a11y.ts';
import {
  addDays,
  addMonths,
  dateToCalDate,
  dayMinuteSpan,
  daysFrom,
  daysInMonth,
  formatDayHeader,
  formatFull,
  formatMonth,
  formatMonthYear,
  formatTime,
  formatWeekday,
  isToday,
  localeWeekStart,
  monthGrid,
  sameDay,
  startOfDay,
  startOfMonth,
  startOfWeek,
  weekdayNames,
} from './datetime.ts';
import * as css from './calendar.css.ts';

const HOUR_PX = 48;

export interface ViewProps {
  controller: CalendarController;
  onOpenEvent: (inst: EventInstance) => void;
  /** Open the event editor to create at a focused date (grid Enter on an empty day). */
  onNewAt?: (day: Date) => void;
}

/** The plural "N events" fragment for a day (also used in accessible names). */
function eventsCount(n: number): string {
  return t('calendar-events-count', { count: n });
}

/** Accessible name for one event instance: time + isolated title (or "all day"). */
function eventLabel(inst: EventInstance): string {
  const title = isolate(inst.event.title);
  return inst.allDay
    ? t('calendar-event-allday', { title })
    : t('calendar-event-at', { time: formatTime(inst.start), title });
}

/** The live-region / cell announcement for a focused day. */
function announceDay(controller: CalendarController, day: Date): string {
  return t('calendar-announce', { date: formatFull(day), events: eventsCount(controller.instancesForDay(day).length) });
}

/** How many day-columns each time-grid view shows. */
function dayCount(view: CalendarView): number {
  switch (view) {
    case 'day':
      return 1;
    case '3day':
      return 3;
    case 'work-week':
      return 5;
    default:
      return 7;
  }
}

/** The first day of a time-grid view around the focused date. */
function gridStart(view: CalendarView, focus: Date): Date {
  if (view === 'day' || view === '3day') return startOfDay(focus);
  return startOfWeek(focus, localeWeekStart());
}

function EventChip(props: {
  inst: EventInstance;
  top: number;
  height: number;
  conflict: boolean;
  onOpen: (i: EventInstance) => void;
}): JSX.Element {
  return (
    <button
      type="button"
      class={css.eventBlock}
      style={{ top: `${props.top}px`, height: `${props.height}px`, background: props.inst.color }}
      title={props.inst.event.title}
      aria-label={eventLabel(props.inst)}
      onClick={() => props.onOpen(props.inst)}
    >
      {formatTime(props.inst.start)} <bdi>{props.inst.event.title}</bdi>
      <Show when={props.conflict}>
        <span class={css.conflictBadge} aria-label={t('calendar-conflict')}>{t('calendar-conflict')}</span>
      </Show>
    </button>
  );
}

export function TimeGridView(props: ViewProps): JSX.Element {
  const c = props.controller;
  const view = (): CalendarView => c.view();
  const days = (): Date[] => daysFrom(gridStart(view(), c.focusDate()), dayCount(view()));
  const hours = Array.from({ length: 24 }, (_, h) => h);
  const cols = (): string => `3.5rem repeat(${days().length}, 1fr)`;

  const timed = (day: Date): EventInstance[] => c.instancesForDay(day).filter((i) => !i.allDay);
  const allDay = (day: Date): EventInstance[] => c.instancesForDay(day).filter((i) => i.allDay);

  // Roving date navigation across the day-header row (companion to the month grid).
  const [hdrActive, setHdrActive] = createSignal<Date>(startOfDay(c.focusDate()));
  let headerEl: HTMLDivElement | undefined;
  createEffect(() => {
    const f = startOfDay(c.focusDate());
    setHdrActive((prev) => (sameDay(prev, f) ? prev : f));
  });
  const liveText = createMemo(() => announceDay(c, hdrActive()));

  function focusHeader(day: Date): void {
    headerEl?.querySelector<HTMLElement>(`[data-date="${dateToCalDate(day)}"]`)?.focus();
  }
  function onHeaderKey(e: KeyboardEvent): void {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      c.goToDate(hdrActive());
      return;
    }
    const back = effectiveDir(headerEl) === 'rtl' ? 1 : -1;
    let delta = 0;
    if (e.key === 'ArrowRight') delta = -back;
    else if (e.key === 'ArrowLeft') delta = back;
    else if (e.key === 'Home') delta = -999;
    else if (e.key === 'End') delta = 999;
    else return;
    e.preventDefault();
    const list = days();
    const idx = list.findIndex((d) => sameDay(d, hdrActive()));
    const nextIdx = Math.max(0, Math.min(list.length - 1, (idx < 0 ? 0 : idx) + delta));
    const next = list[nextIdx];
    if (next !== undefined) {
      setHdrActive(startOfDay(next));
      focusHeader(next);
    }
  }

  return (
    <div style={{ display: 'flex', 'flex-direction': 'column', 'min-height': '100%' }}>
      <div class={css.srOnly} aria-live="polite" aria-atomic="true">{liveText()}</div>
      <div ref={headerEl} role="grid" aria-label={t('calendar-time-grid')}>
        {/* grid → row → gridcell (WCAG 1.3.1): the day headers are a single grid
            row; the CSS grid lives on the row so the layout is unchanged. */}
        <div role="row" style={{ display: 'grid', 'grid-template-columns': cols() }}>
          <div class={css.gutter} role="presentation" />
          <For each={days()}>
            {(d) => (
              <div
                role="gridcell"
                data-date={dateToCalDate(d)}
                tabindex={sameDay(d, hdrActive()) ? 0 : -1}
                aria-current={isToday(d) ? 'date' : undefined}
                aria-label={t('calendar-cell', { date: formatFull(d), events: eventsCount(c.instancesForDay(d).length) })}
                class={css.dayHeader}
                onKeyDown={onHeaderKey}
                onFocus={() => setHdrActive(startOfDay(d))}
                onClick={() => c.goToDate(d)}
              >
                <span aria-hidden="true">{formatDayHeader(d)}</span>
              </div>
            )}
          </For>
        </div>
      </div>
      <div style={{ display: 'grid', 'grid-template-columns': cols(), 'border-bottom': '1px solid' }} class={css.allDayRow}>
        <div class={css.gutter} style={{ 'font-size': '0.7rem', 'text-align': 'end', 'padding-inline-end': '4px' }}>
          {t('calendar-all-day')}
        </div>
        <For each={days()}>
          {(d) => (
            <div style={{ 'min-height': '1.5rem', padding: '2px' }}>
              <For each={allDay(d)}>
                {(inst) => (
                  <button
                    type="button"
                    class={css.allDayChip}
                    style={{ background: inst.color }}
                    title={inst.event.title}
                    aria-label={eventLabel(inst)}
                    onClick={() => props.onOpenEvent(inst)}
                  >
                    <bdi>{inst.event.title}</bdi>
                  </button>
                )}
              </For>
            </div>
          )}
        </For>
      </div>
      <div style={{ display: 'grid', 'grid-template-columns': cols(), flex: 1, position: 'relative' }}>
        <div class={css.gutter}>
          <For each={hours}>
            {(h) => (
              <div class={css.hourCell} style={{ height: `${HOUR_PX}px` }}>
                {h === 0 ? '' : `${String(h).padStart(2, '0')}:00`}
              </div>
            )}
          </For>
        </div>
        <For each={days()}>
          {(day) => (
            <div class={css.dayColumn} style={{ position: 'relative' }} data-testid="day-column">
              <For each={hours}>{() => <div class={css.hourLine} style={{ height: `${HOUR_PX}px` }} />}</For>
              <For each={timed(day)}>
                {(inst) => {
                  const span = dayMinuteSpan(inst.start, inst.end, day);
                  return (
                    <Show when={span}>
                      {(s) => (
                        <EventChip
                          inst={inst}
                          top={(s().top / 60) * HOUR_PX}
                          height={(s().height / 60) * HOUR_PX}
                          conflict={c.hasConflict(inst.event.id)}
                          onOpen={props.onOpenEvent}
                        />
                      )}
                    </Show>
                  );
                }}
              </For>
            </div>
          )}
        </For>
      </div>
    </div>
  );
}

/** The interactive month grid (WAI-ARIA `grid`), used by the Month view. */
export function MonthView(props: ViewProps): JSX.Element {
  const c = props.controller;
  const weekStart = localeWeekStart();
  const focus = (): Date => c.focusDate();
  const weeks = (): Date[][] => monthGrid(focus().getFullYear(), focus().getMonth(), weekStart);

  const [active, setActive] = createSignal<Date>(startOfDay(c.focusDate()));
  let gridEl: HTMLDivElement | undefined;
  let wantFocus = false;

  // Realign the roving anchor when the focus date changes externally
  // (Today / prev / next / view switch) — without stealing focus.
  createEffect(() => {
    const f = startOfDay(c.focusDate());
    setActive((prev) => (sameDay(prev, f) ? prev : f));
  });

  // After a month-crossing navigation reloads + re-renders, pull DOM focus onto
  // the active cell so keyboard nav feels continuous across month boundaries.
  createEffect(() => {
    c.instances();
    c.focusDate();
    if (!wantFocus) return;
    wantFocus = false;
    queueMicrotask(() => gridEl?.querySelector<HTMLElement>(`[data-date="${dateToCalDate(active())}"]`)?.focus());
  });

  const liveText = createMemo(() => announceDay(c, active()));

  function focusCell(day: Date): void {
    gridEl?.querySelector<HTMLElement>(`[data-date="${dateToCalDate(day)}"]`)?.focus();
  }

  function openDay(day: Date): void {
    const evs = c.instancesForDay(day);
    if (evs.length > 0) props.onOpenEvent(evs[0]!);
    else props.onNewAt?.(day);
  }

  function onKeyDown(e: KeyboardEvent): void {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      openDay(active());
      return;
    }
    const next = nextGridDate(e.key, active(), { dir: effectiveDir(gridEl), weekStart, shift: e.shiftKey });
    if (next === null) return;
    e.preventDefault();
    const nd = startOfDay(next);
    setActive(nd);
    if (inWindow(nd, c.window())) {
      focusCell(nd);
    } else {
      wantFocus = true;
      c.goToDate(nd);
    }
  }

  return (
    <div style={{ display: 'flex', 'flex-direction': 'column', height: '100%' }}>
      <div class={css.srOnly} aria-live="polite" aria-atomic="true" data-testid="calendar-live">{liveText()}</div>
      <div class={css.weekdayHead} aria-hidden="true">
        <For each={weekdayNames(weekStart)}>{(w) => <div class={css.weekdayCell}>{w}</div>}</For>
      </div>
      <div
        ref={gridEl}
        role="grid"
        aria-label={t('calendar-month-grid', { month: formatMonthYear(focus()) })}
        class={css.monthGridRows}
        onKeyDown={onKeyDown}
      >
        <For each={weeks()}>
          {(week) => (
            <div role="row" class={css.monthRow}>
              <For each={week}>
                {(day) => {
                  const inMonth = (): boolean => day.getMonth() === focus().getMonth();
                  const evs = (): EventInstance[] => c.instancesForDay(day);
                  const isActive = (): boolean => sameDay(day, active());
                  return (
                    <div
                      role="gridcell"
                      data-date={dateToCalDate(day)}
                      tabindex={isActive() ? 0 : -1}
                      aria-selected={isActive()}
                      aria-current={isToday(day) ? 'date' : undefined}
                      aria-label={t('calendar-cell', { date: formatFull(day), events: eventsCount(evs().length) })}
                      class={inMonth() ? css.monthCell : css.monthCellOut}
                      onFocus={() => setActive(startOfDay(day))}
                      onClick={() => {
                        setActive(startOfDay(day));
                        openDay(day);
                      }}
                    >
                      <span class={isToday(day) ? css.dayNumToday : css.dayNum} aria-hidden="true">{day.getDate()}</span>
                      <For each={evs().slice(0, 4)}>
                        {(inst) => (
                          <button
                            type="button"
                            tabindex={-1}
                            class={css.monthEvent}
                            style={{ background: inst.color }}
                            title={inst.event.title}
                            aria-label={eventLabel(inst)}
                            onClick={(e) => {
                              e.stopPropagation();
                              props.onOpenEvent(inst);
                            }}
                          >
                            {inst.allDay ? '' : `${formatTime(inst.start)} `}
                            <bdi>{inst.event.title}</bdi>
                            <Show when={c.hasConflict(inst.event.id)}>
                              <span class={css.conflictBadge} aria-label={t('calendar-conflict')}>!</span>
                            </Show>
                          </button>
                        )}
                      </For>
                      <Show when={evs().length > 4}>
                        <span class={css.dimText}>{t('calendar-more', { count: evs().length - 4 })}</span>
                      </Show>
                    </div>
                  );
                }}
              </For>
            </div>
          )}
        </For>
      </div>
    </div>
  );
}

/** Display-only month grid (no roving nav) — used by the Quarter/tri-month view. */
function MonthGrid(props: { controller: CalendarController; year: number; month0: number; onOpenEvent: (i: EventInstance) => void }): JSX.Element {
  const weekStart = localeWeekStart();
  const weeks = (): Date[][] => monthGrid(props.year, props.month0, weekStart);
  return (
    <div style={{ display: 'flex', 'flex-direction': 'column', height: '100%' }}>
      <div class={css.weekdayHead} aria-hidden="true">
        <For each={weekdayNames(weekStart)}>{(w) => <div class={css.weekdayCell}>{w}</div>}</For>
      </div>
      <div class={css.monthGridEl}>
        <For each={weeks()}>
          {(week) => (
            <For each={week}>
              {(day) => {
                const inMonth = day.getMonth() === props.month0;
                const evs = props.controller.instancesForDay(day);
                return (
                  <div
                    class={inMonth ? css.monthCell : css.monthCellOut}
                    aria-label={t('calendar-cell', { date: formatFull(day), events: eventsCount(evs.length) })}
                  >
                    <span class={isToday(day) ? css.dayNumToday : css.dayNum} aria-hidden="true">{day.getDate()}</span>
                    <For each={evs.slice(0, 4)}>
                      {(inst) => (
                        <button
                          type="button"
                          class={css.monthEvent}
                          style={{ background: inst.color }}
                          title={inst.event.title}
                          aria-label={eventLabel(inst)}
                          onClick={() => props.onOpenEvent(inst)}
                        >
                          {inst.allDay ? '' : `${formatTime(inst.start)} `}
                          <bdi>{inst.event.title}</bdi>
                          <Show when={props.controller.hasConflict(inst.event.id)}>
                            <span class={css.conflictBadge} aria-label={t('calendar-conflict')}>!</span>
                          </Show>
                        </button>
                      )}
                    </For>
                    <Show when={evs.length > 4}>
                      <span class={css.dimText}>{t('calendar-more', { count: evs.length - 4 })}</span>
                    </Show>
                  </div>
                );
              }}
            </For>
          )}
        </For>
      </div>
    </div>
  );
}

export function TriMonthView(props: ViewProps): JSX.Element {
  const months = (): Date[] => {
    const base = startOfMonth(props.controller.focusDate());
    return [addMonths(base, -1), base, addMonths(base, 1)];
  };
  return (
    <div class={css.triMonth}>
      <For each={months()}>
        {(m) => (
          <div>
            <div class={css.miniTitle}>
              {formatMonth(m)} {m.getFullYear()}
            </div>
            <MonthGrid controller={props.controller} year={m.getFullYear()} month0={m.getMonth()} onOpenEvent={props.onOpenEvent} />
          </div>
        )}
      </For>
    </div>
  );
}

/** schedule + agenda: a flat, date-grouped list of upcoming instances. */
export function AgendaView(props: ViewProps): JSX.Element {
  const days = (): Date[] => {
    const start = startOfDay(props.controller.focusDate());
    return Array.from({ length: 30 }, (_, i) => addDays(start, i)).filter(
      (d) => props.controller.instancesForDay(d).length > 0,
    );
  };
  return (
    <div class={css.agenda} role="list" aria-label={t('calendar-view-agenda')}>
      <Show when={days().length > 0} fallback={<p class={css.dimText}>{t('calendar-no-events-30')}</p>}>
        <For each={days()}>
          {(day) => (
            <div class={css.agendaDay} role="listitem">
              <div class={css.agendaDate}>
                {formatWeekday(day)} {day.getDate()}/{day.getMonth() + 1}
              </div>
              <For each={props.controller.instancesForDay(day)}>
                {(inst) => (
                  <button type="button" class={css.agendaRow} aria-label={eventLabel(inst)} onClick={() => props.onOpenEvent(inst)}>
                    <span class={css.agendaTime} aria-hidden="true">
                      {inst.allDay ? t('calendar-all-day-full') : `${formatTime(inst.start)} – ${formatTime(inst.end)}`}
                    </span>
                    <span class={css.agendaDot} style={{ background: inst.color }} aria-hidden="true" />
                    <span><bdi>{inst.event.title}</bdi></span>
                    <Show when={props.controller.hasConflict(inst.event.id)}>
                      <span class={css.conflictBadge} aria-label={t('calendar-conflict')}>{t('calendar-conflict')}</span>
                    </Show>
                  </button>
                )}
              </For>
            </div>
          )}
        </For>
      </Show>
    </div>
  );
}

export function YearView(props: ViewProps): JSX.Element {
  const year = (): number => props.controller.focusDate().getFullYear();
  const months = Array.from({ length: 12 }, (_, m) => m);
  return (
    <div class={css.yearGrid}>
      <For each={months}>
        {(m0) => {
          const first = new Date(year(), m0, 1);
          const total = daysInMonth(year(), m0);
          const days = Array.from({ length: total }, (_, i) => new Date(year(), m0, i + 1));
          return (
            <div class={css.miniMonth}>
              <div class={css.miniTitle}>{formatMonth(first)}</div>
              <div class={css.miniGrid}>
                <For each={days}>
                  {(day) => {
                    const count = props.controller.instancesForDay(day).length;
                    const cls = isToday(day) ? css.miniDayToday : count > 0 ? css.miniDayEvent : css.miniDay;
                    return (
                      <button
                        type="button"
                        class={cls}
                        aria-label={t('calendar-cell', { date: formatFull(day), events: eventsCount(count) })}
                        onClick={() => props.controller.goToDate(day)}
                      >
                        {day.getDate()}
                      </button>
                    );
                  }}
                </For>
              </div>
            </div>
          );
        }}
      </For>
    </div>
  );
}

/** Route the active view to its component. */
export function ActiveView(props: ViewProps): JSX.Element {
  const v = (): CalendarView => props.controller.view();
  const isGrid = (): boolean => v() === 'day' || v() === '3day' || v() === 'work-week' || v() === 'week';
  const isList = (): boolean => v() === 'schedule' || v() === 'agenda';
  return (
    <Switch fallback={<TimeGridView {...props} />}>
      <Match when={isGrid()}>
        <TimeGridView {...props} />
      </Match>
      <Match when={v() === 'month'}>
        <MonthView {...props} />
      </Match>
      <Match when={v() === 'tri-month'}>
        <TriMonthView {...props} />
      </Match>
      <Match when={isList()}>
        <AgendaView {...props} />
      </Match>
      <Match when={v() === 'year'}>
        <YearView {...props} />
      </Match>
    </Switch>
  );
}
