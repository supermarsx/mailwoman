// The nine calendar views (plan §3 e4): day / 3-day / work-week / week share the
// vertical time grid; month + tri-month share the month grid; schedule + agenda
// share the list; year is a 12-mini-month overview. Every view renders the same
// `EventInstance` shape the controller exposes (engine-expanded), so views are
// agnostic to the mock-vs-engine backend. Conflict badges come from the
// controller's `hasConflict` set.

import { For, Match, Show, Switch, type JSX } from 'solid-js';
import type { CalendarController } from './controller.ts';
import type { CalendarView, EventInstance } from './types.ts';
import {
  addDays,
  addMonths,
  dayMinuteSpan,
  daysFrom,
  daysInMonth,
  formatDayHeader,
  formatMonth,
  formatTime,
  formatWeekday,
  isToday,
  monthGrid,
  sameDay,
  startOfDay,
  startOfMonth,
  startOfWeek,
} from './datetime.ts';
import * as css from './calendar.css.ts';

const HOUR_PX = 48;

export interface ViewProps {
  controller: CalendarController;
  onOpenEvent: (inst: EventInstance) => void;
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
  return startOfWeek(focus, 1);
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
      onClick={() => props.onOpen(props.inst)}
    >
      {formatTime(props.inst.start)} {props.inst.event.title}
      <Show when={props.conflict}>
        <span class={css.conflictBadge}>conflict</span>
      </Show>
    </button>
  );
}

export function TimeGridView(props: ViewProps): JSX.Element {
  const view = (): CalendarView => props.controller.view();
  const days = (): Date[] => daysFrom(gridStart(view(), props.controller.focusDate()), dayCount(view()));
  const hours = Array.from({ length: 24 }, (_, h) => h);
  const cols = (): string => `3.5rem repeat(${days().length}, 1fr)`;

  const timed = (day: Date): EventInstance[] => props.controller.instancesForDay(day).filter((i) => !i.allDay);
  const allDay = (day: Date): EventInstance[] => props.controller.instancesForDay(day).filter((i) => i.allDay);

  return (
    <div style={{ display: 'flex', 'flex-direction': 'column', 'min-height': '100%' }}>
      <div style={{ display: 'grid', 'grid-template-columns': cols() }}>
        <div class={css.gutter} />
        <For each={days()}>
          {(d) => (
            <div class={css.dayHeader} aria-current={isToday(d) ? 'date' : undefined}>
              {formatDayHeader(d)}
            </div>
          )}
        </For>
      </div>
      <div style={{ display: 'grid', 'grid-template-columns': cols(), 'border-bottom': '1px solid' }} class={css.allDayRow}>
        <div class={css.gutter} style={{ 'font-size': '0.7rem', 'text-align': 'right', 'padding-right': '4px' }}>
          all-day
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
                    onClick={() => props.onOpenEvent(inst)}
                  >
                    {inst.event.title}
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
                          conflict={props.controller.hasConflict(inst.event.id)}
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

const WEEKDAY_LABELS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

function MonthGrid(props: { controller: CalendarController; year: number; month0: number; onOpenEvent: (i: EventInstance) => void }): JSX.Element {
  const weeks = (): Date[][] => monthGrid(props.year, props.month0, 1);
  return (
    <div style={{ display: 'flex', 'flex-direction': 'column', height: '100%' }}>
      <div class={css.weekdayHead}>
        <For each={WEEKDAY_LABELS}>{(w) => <div class={css.weekdayCell}>{w}</div>}</For>
      </div>
      <div class={css.monthGridEl}>
        <For each={weeks()}>
          {(week) => (
            <For each={week}>
              {(day) => {
                const inMonth = day.getMonth() === props.month0;
                const evs = props.controller.instancesForDay(day);
                return (
                  <div class={inMonth ? css.monthCell : css.monthCellOut}>
                    <span class={isToday(day) ? css.dayNumToday : css.dayNum}>{day.getDate()}</span>
                    <For each={evs.slice(0, 4)}>
                      {(inst) => (
                        <button
                          type="button"
                          class={css.monthEvent}
                          style={{ background: inst.color }}
                          onClick={() => props.onOpenEvent(inst)}
                        >
                          {inst.allDay ? '' : `${formatTime(inst.start)} `}
                          {inst.event.title}
                          <Show when={props.controller.hasConflict(inst.event.id)}>
                            <span class={css.conflictBadge}>!</span>
                          </Show>
                        </button>
                      )}
                    </For>
                    <Show when={evs.length > 4}>
                      <span class={css.dimText}>+{evs.length - 4} more</span>
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

export function MonthView(props: ViewProps): JSX.Element {
  const focus = (): Date => props.controller.focusDate();
  return (
    <MonthGrid
      controller={props.controller}
      year={focus().getFullYear()}
      month0={focus().getMonth()}
      onOpenEvent={props.onOpenEvent}
    />
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
    <div class={css.agenda}>
      <Show when={days().length > 0} fallback={<p class={css.dimText}>No events in the next 30 days.</p>}>
        <For each={days()}>
          {(day) => (
            <div class={css.agendaDay}>
              <div class={css.agendaDate}>
                {formatWeekday(day)} {day.getDate()}/{day.getMonth() + 1}
              </div>
              <For each={props.controller.instancesForDay(day)}>
                {(inst) => (
                  <button type="button" class={css.agendaRow} onClick={() => props.onOpenEvent(inst)}>
                    <span class={css.agendaTime}>
                      {inst.allDay ? 'All day' : `${formatTime(inst.start)} – ${formatTime(inst.end)}`}
                    </span>
                    <span class={css.agendaDot} style={{ background: inst.color }} />
                    <span>{inst.event.title}</span>
                    <Show when={props.controller.hasConflict(inst.event.id)}>
                      <span class={css.conflictBadge}>conflict</span>
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
                    const has = props.controller.instancesForDay(day).length > 0;
                    const cls = isToday(day) ? css.miniDayToday : has ? css.miniDayEvent : css.miniDay;
                    return (
                      <button
                        type="button"
                        class={cls}
                        title={has ? `${props.controller.instancesForDay(day).length} events` : undefined}
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

// Referenced to keep imports honest across view refactors.
void sameDay;
