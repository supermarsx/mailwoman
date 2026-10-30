// t92: multi-day events are bars across the days they cover (time-grid all-day
// row and month weeks), cut at week edges, stacked in lanes with "+N more";
// free / tentative / cancelled events each look different from a busy one and
// say their state in the accessible name. Pure layout first, then the views over
// a stub controller exposing only what they read.

import { describe, it, expect } from 'vitest';
import { render, screen, within } from '@solidjs/testing-library';
import { MonthView, TimeGridView } from './views.tsx';
import { layoutRowSpans, spansDays } from './layout.ts';
import type { CalendarController } from './controller.ts';
import type { CalendarView, EventInstance } from './types.ts';
import type { CalendarEvent } from '../../api/pim-types.ts';

const d = (day: number, h = 0, m = 0): Date => new Date(2026, 9, day, h, m);

function inst(id: string, title: string, start: Date, end: Date, over: Partial<CalendarEvent> = {}, color = '#1e3a8a'): EventInstance {
  const event = { id, calendarId: 'cal', title, status: 'confirmed', freeBusyStatus: 'busy', ...over } as CalendarEvent;
  return { key: `${id}:${start.getTime()}`, event, start, end, allDay: event.showWithoutTime === true, recurring: false, color };
}
const allDay = (id: string, title: string, from: number, toExclusive: number, over: Partial<CalendarEvent> = {}): EventInstance =>
  inst(id, title, d(from), d(toExclusive), { showWithoutTime: true, ...over });

function stub(list: EventInstance[], view: CalendarView, focus: Date): CalendarController & { views: CalendarView[] } {
  const views: CalendarView[] = [];
  return {
    views,
    view: () => view,
    focusDate: () => focus,
    instances: () => list,
    window: () => ({ start: d(1), end: d(31) }),
    instancesForDay: (day: Date) => {
      const s = new Date(day.getFullYear(), day.getMonth(), day.getDate());
      const e = new Date(s.getFullYear(), s.getMonth(), s.getDate() + 1);
      return list.filter((i) => i.start < e && i.end > s).sort((a, b) => a.start.getTime() - b.start.getTime());
    },
    hasConflict: () => false,
    goToDate: () => undefined,
    setView: (v: CalendarView) => views.push(v),
  } as unknown as CalendarController & { views: CalendarView[] };
}

// The week of Mon 26 Oct 2026 .. Sun 1 Nov 2026.
const week = Array.from({ length: 7 }, (_, i) => new Date(2026, 9, 26 + i));

describe('layoutRowSpans', () => {
  it('covers the days an event touches, with an exclusive all-day end', () => {
    const r = layoutRowSpans([allDay('a', 'A', 27, 30)], week, 3);
    expect(r.bars.map((b) => [b.startCol, b.span, b.lane, b.continuesBefore, b.continuesAfter])).toEqual([[1, 3, 0, false, false]]);
  });

  it('cuts an event at the week edges and flags both cuts', () => {
    const prev = Array.from({ length: 7 }, (_, i) => new Date(2026, 9, 19 + i));
    const ev = [allDay('long', 'Long', 23, 28)];
    const first = layoutRowSpans(ev, prev, 3).bars[0]!;
    const second = layoutRowSpans(ev, week, 3).bars[0]!;
    expect([first.startCol, first.span, first.continuesBefore, first.continuesAfter]).toEqual([4, 3, false, true]);
    expect([second.startCol, second.span, second.continuesBefore, second.continuesAfter]).toEqual([0, 2, true, false]);
    const across = layoutRowSpans([allDay('x', 'X', 20, 40)], week, 3).bars[0]!;
    expect([across.startCol, across.span, across.continuesBefore, across.continuesAfter]).toEqual([0, 7, true, true]);
  });

  it('stacks overlapping bars in lanes and reuses a lane once it is free', () => {
    const r = layoutRowSpans([allDay('a', 'A', 26, 29), allDay('b', 'B', 27, 28), allDay('c', 'C', 29, 31)], week, 3);
    const lane = Object.fromEntries(r.bars.map((b) => [b.item.event.id, b.lane]));
    expect(lane).toEqual({ a: 0, b: 1, c: 0 });
    expect(r.lanes).toBe(2);
  });

  it('counts bars past the lane limit per covered day ("+N more")', () => {
    const r = layoutRowSpans(
      [allDay('a', 'A', 26, 30), allDay('b', 'B', 26, 30), allDay('c', 'C', 27, 29), allDay('e', 'E', 28, 29)],
      week,
      2,
    );
    expect(r.bars.map((b) => b.item.event.id).sort()).toEqual(['a', 'b']);
    expect(r.hidden).toEqual([0, 1, 2, 0, 0, 0, 0]);
  });

  it('tells a multi-day event from a single-day one', () => {
    expect(spansDays(allDay('one', 'One', 27, 28))).toBe(false);
    expect(spansDays(allDay('two', 'Two', 27, 29))).toBe(true);
    expect(spansDays(inst('night', 'Night', d(27, 22), d(28, 2)))).toBe(true);
    expect(spansDays(inst('day', 'Day', d(27, 9), d(27, 17)))).toBe(false);
  });
});

describe('TimeGridView all-day bars', () => {
  it('draws one bar across the days it covers, sized to the span', () => {
    render(() => <TimeGridView controller={stub([allDay('c', 'Conference', 27, 30)], 'week', d(28))} onOpenEvent={() => undefined} />);
    const bars = screen.getAllByTestId('span-bar');
    expect(bars).toHaveLength(1);
    const bar = bars[0]!;
    expect(bar.dataset['span']).toBe('3');
    expect(bar.style.getPropertyValue('inline-size')).toMatch(/^calc\(3 \* \(100% \+ var\(--[^)]+\)\) - var\(--[^)]+\)\)$/);
    // The bar lives in its first day's cell.
    expect(bar.closest('[data-testid="all-day-cell"]')?.getAttribute('data-date')).toBe('2026-10-27');
    expect(bar.getAttribute('aria-label')).toContain('Conference');
  });

  it('says "+N more" past three lanes and opens that day', () => {
    const list = ['a', 'b', 'c', 'e'].map((id) => allDay(id, id.toUpperCase(), 27, 29));
    const c = stub(list, 'week', d(28));
    render(() => <TimeGridView controller={c} onOpenEvent={() => undefined} />);
    expect(screen.getAllByTestId('span-bar')).toHaveLength(3);
    const more = screen.getAllByTestId('span-more');
    expect(more.map((m) => m.textContent)).toEqual(['+1 more', '+1 more']);
    more[0]!.click();
    expect(c.views).toEqual(['day']);
  });
});

describe('MonthView spanning bars', () => {
  it('splits a bar at the week boundary: one bar per week row', () => {
    render(() => <MonthView controller={stub([allDay('t', 'Trip', 22, 28)], 'month', d(15))} onOpenEvent={() => undefined} />);
    const bars = screen.getAllByTestId('span-bar');
    // Whatever day the locale starts its week on, the six days split into one
    // bar ending at the first row's edge and one starting the next row.
    expect(bars.map((b) => b.dataset['continues'])).toEqual(['after', 'before']);
    const cellOf = (b: HTMLElement): HTMLElement => b.closest<HTMLElement>('[role="gridcell"]')!;
    expect(cellOf(bars[0]!).getAttribute('data-date')).toBe('2026-10-22');
    const row = cellOf(bars[0]!).parentElement!;
    const firstCol = [...row.children].indexOf(cellOf(bars[0]!));
    expect(Number(bars[0]!.dataset['span'])).toBe(7 - firstCol);
    expect(cellOf(bars[1]!).parentElement!.firstElementChild).toBe(cellOf(bars[1]!));
    expect(Number(bars[0]!.dataset['span']) + Number(bars[1]!.dataset['span'])).toBe(6);
    // The cut bar still names the whole range.
    expect(bars[1]!.getAttribute('aria-label')).toMatch(/Trip.*22.*27/s);
  });

  it('keeps single-day events as chips beside the bars and counts the overflow', () => {
    const list = [
      allDay('a', 'A', 27, 29),
      allDay('b', 'B', 27, 29),
      ...['x', 'y', 'z'].map((id, i) => inst(id, id.toUpperCase(), d(27, 9 + i), d(27, 10 + i))),
    ];
    render(() => <MonthView controller={stub(list, 'month', d(15))} onOpenEvent={() => undefined} />);
    const cell = document.querySelector<HTMLElement>('[role="gridcell"][data-date="2026-10-27"]')!;
    expect(within(cell).getAllByTestId('span-bar')).toHaveLength(2);
    // Four rows in all: two lanes of bars, then two of the three chips.
    expect(within(cell).getByTitle('X')).toBeTruthy();
    expect(within(cell).getByTitle('Y')).toBeTruthy();
    expect(within(cell).queryByTitle('Z')).toBeNull();
    expect(within(cell).getByTestId('month-more').textContent).toBe('+1 more');
  });
});

describe('free / tentative / cancelled', () => {
  it('outlines a free event, hatches a tentative one, strikes a cancelled one, and says so', () => {
    const list = [
      inst('f', 'Focus', d(27, 9), d(27, 10), { freeBusyStatus: 'free' }, '#00897b'),
      inst('t', 'Maybe', d(27, 11), d(27, 12), { status: 'tentative' }),
      inst('x', 'Dropped', d(27, 13), d(27, 14), { status: 'cancelled' }),
      inst('b', 'Firm', d(27, 15), d(27, 16)),
    ];
    render(() => <TimeGridView controller={stub(list, 'day', d(27))} onOpenEvent={() => undefined} />);
    const free = screen.getByTitle('Focus');
    expect(free.dataset['busy']).toBe('free');
    expect(free.style.getPropertyValue('border-color')).not.toBe('');
    expect(free.style.getPropertyValue('background-color')).toBe('');
    expect(free.getAttribute('aria-label')).toMatch(/shows as free/i);

    const maybe = screen.getByTitle('Maybe');
    expect(maybe.dataset['status']).toBe('tentative');
    expect(within(maybe).getByTestId('event-hatch').querySelector('pattern')).not.toBeNull();
    expect(maybe.getAttribute('aria-label')).toMatch(/tentative/i);

    const dropped = screen.getByTitle('Dropped');
    expect(dropped.dataset['status']).toBe('cancelled');
    expect(dropped.getAttribute('aria-label')).toMatch(/cancelled/i);

    const firm = screen.getByTitle('Firm');
    expect(firm.dataset['busy']).toBeUndefined();
    expect(firm.dataset['status']).toBeUndefined();
    expect(firm.querySelector('[data-testid="event-hatch"]')).toBeNull();
    expect(firm.getAttribute('aria-label')).not.toMatch(/tentative|cancelled|free/i);
  });

  it('marks the states on month chips and bars too', () => {
    const list = [
      allDay('t', 'Offsite', 27, 29, { status: 'tentative', freeBusyStatus: 'free' }),
      inst('x', 'Dropped', d(28, 9), d(28, 10), { status: 'cancelled' }),
    ];
    render(() => <MonthView controller={stub(list, 'month', d(15))} onOpenEvent={() => undefined} />);
    const bar = screen.getByTestId('span-bar');
    expect(bar.dataset['status']).toBe('tentative');
    expect(bar.dataset['busy']).toBe('free');
    expect(bar.getAttribute('aria-label')).toMatch(/tentative, shows as free/i);
    expect(screen.getByTitle('Dropped').getAttribute('aria-label')).toMatch(/cancelled/i);
  });
});
