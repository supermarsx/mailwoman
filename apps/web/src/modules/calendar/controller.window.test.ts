// `windowFor` — the [start,end) range each calendar view expands (t19-e12, 26.19).
//
// This is the function that decides how much recurrence expansion every view
// asks the backend for. Get it wrong and the symptom is not a crash: it is
// events quietly missing from the edge of a view, or a month grid that expands
// six weeks of recurrences it never renders. `controller.ts` had coverage from
// the component specs but this pure entry point had none of its own.
//
// Assertions are written to be LOCALE-ROBUST. `windowFor` resolves the first
// day of the week through `localeWeekStart()`, which reads the runtime's locale,
// so a hardcoded "weeks start on Monday" expectation would pass on this machine
// and fail on a runner configured differently. The week-based views are
// therefore asserted against `localeWeekStart()` itself plus invariants
// (length, alignment, containment); the views that do not consult it are
// asserted against exact dates.

import { describe, it, expect } from 'vitest';
import { windowFor } from './controller.ts';
import { localeWeekStart, startOfDay } from './datetime.ts';
import type { CalendarView } from './types.ts';

/** Local midnight on a given Y-M-D — `windowFor` works in local time. */
const D = (y: number, m: number, d: number, h = 0): Date => new Date(y, m - 1, d, h);

const days = (w: { start: Date; end: Date }): number =>
  Math.round((w.end.getTime() - w.start.getTime()) / 86_400_000);

const ALL_VIEWS: CalendarView[] = [
  'day',
  '3day',
  'work-week',
  'week',
  'month',
  'tri-month',
  'schedule',
  'agenda',
  'year',
];

describe('windowFor — invariants that hold for every view', () => {
  // A Thursday, mid-month, mid-year: far enough from every boundary that a
  // window has room to be wrong in either direction.
  const focus = D(2026, 8, 13, 15);

  it.each(ALL_VIEWS)('%s starts at local midnight', (view) => {
    const w = windowFor(view, focus);
    expect(w.start.getHours()).toBe(0);
    expect(w.start.getMinutes()).toBe(0);
    expect(w.start.getSeconds()).toBe(0);
    expect(w.start.getMilliseconds()).toBe(0);
  });

  it.each(ALL_VIEWS)('%s is a non-empty half-open range containing the focus day', (view) => {
    const w = windowFor(view, focus);
    expect(w.end.getTime()).toBeGreaterThan(w.start.getTime());
    // The day the user is looking at must be inside the range it expands, or
    // the view renders events it never asked for.
    expect(w.start.getTime()).toBeLessThanOrEqual(startOfDay(focus).getTime());
    expect(w.end.getTime()).toBeGreaterThan(startOfDay(focus).getTime());
  });

  it.each(ALL_VIEWS)('%s ignores the time of day within the focus date', (view) => {
    // The focus carries a wall-clock time; the window must not shift with it,
    // or navigating at 23:00 would land on a different range than at 09:00.
    const early = windowFor(view, D(2026, 8, 13, 0));
    const late = windowFor(view, D(2026, 8, 13, 23));
    expect(early.start.getTime()).toBe(late.start.getTime());
    expect(early.end.getTime()).toBe(late.end.getTime());
  });
});

describe('windowFor — the views that do not consult the locale', () => {
  const focus = D(2026, 8, 13, 15);

  it('day is exactly the focus day', () => {
    const w = windowFor('day', focus);
    expect(w.start).toEqual(D(2026, 8, 13));
    expect(w.end).toEqual(D(2026, 8, 14));
  });

  it('3day starts at the focus day and runs three days forward, not centred', () => {
    const w = windowFor('3day', focus);
    expect(w.start).toEqual(D(2026, 8, 13));
    expect(w.end).toEqual(D(2026, 8, 16));
  });

  it('schedule and agenda both span 30 days from the focus day', () => {
    for (const view of ['schedule', 'agenda'] as const) {
      const w = windowFor(view, focus);
      expect(w.start).toEqual(D(2026, 8, 13));
      expect(days(w)).toBe(30);
    }
  });

  it('tri-month spans the previous, current and next month', () => {
    const w = windowFor('tri-month', focus);
    expect(w.start).toEqual(D(2026, 7, 1));
    expect(w.end).toEqual(D(2026, 10, 1));
  });

  it('year is the whole calendar year of the focus', () => {
    const w = windowFor('year', focus);
    expect(w.start).toEqual(D(2026, 1, 1));
    expect(w.end).toEqual(D(2027, 1, 1));
  });
});

describe('windowFor — the week-aligned views', () => {
  const focus = D(2026, 8, 13, 15);
  const weekStart = localeWeekStart();

  it('week is seven days aligned to the locale’s first day', () => {
    const w = windowFor('week', focus);
    expect(days(w)).toBe(7);
    expect(w.start.getDay()).toBe(weekStart);
  });

  it('work-week is the first five days of the same week as week', () => {
    const week = windowFor('week', focus);
    const work = windowFor('work-week', focus);
    // Same anchor, shorter span — a work week that anchored differently would
    // show a different set of days than the week view it sits beside.
    expect(work.start.getTime()).toBe(week.start.getTime());
    expect(days(work)).toBe(5);
  });

  it('month expands a full 42-cell grid, not just the month', () => {
    // Six rows of seven. The grid deliberately over-expands so the leading and
    // trailing days borrowed from the neighbouring months carry their events.
    const w = windowFor('month', focus);
    expect(days(w)).toBe(42);
    expect(w.start.getDay()).toBe(weekStart);
    // The grid starts on or before the 1st of the focus month.
    expect(w.start.getTime()).toBeLessThanOrEqual(D(2026, 8, 1).getTime());
    // And covers every day of it.
    expect(w.end.getTime()).toBeGreaterThan(D(2026, 8, 31).getTime());
  });

  it('month covers a 31-day month that starts on the locale’s first weekday', () => {
    // The tightest case for a 42-cell grid: when the 1st IS the grid start,
    // a 31-day month plus the leading zero days still has to fit.
    for (let m = 1; m <= 12; m += 1) {
      const w = windowFor('month', D(2026, m, 15));
      const lastDay = new Date(2026, m, 0); // day 0 of next month = last of this
      expect(w.start.getTime()).toBeLessThanOrEqual(D(2026, m, 1).getTime());
      expect(w.end.getTime()).toBeGreaterThan(startOfDay(lastDay).getTime());
    }
  });
});

describe('windowFor — boundaries', () => {
  it('handles a focus on the first and last day of a month', () => {
    for (const focus of [D(2026, 8, 1), D(2026, 8, 31)]) {
      const w = windowFor('month', focus);
      expect(w.start.getTime()).toBeLessThanOrEqual(startOfDay(focus).getTime());
      expect(w.end.getTime()).toBeGreaterThan(startOfDay(focus).getTime());
    }
  });

  it('crosses a year boundary without wrapping', () => {
    const w = windowFor('month', D(2026, 12, 28));
    expect(w.start.getFullYear()).toBe(2026);
    expect(w.end.getFullYear()).toBe(2027);
    expect(days(w)).toBe(42);
  });

  it('spans a leap day in February 2028', () => {
    const w = windowFor('month', D(2028, 2, 15));
    expect(w.start.getTime()).toBeLessThanOrEqual(D(2028, 2, 1).getTime());
    expect(w.end.getTime()).toBeGreaterThan(D(2028, 2, 29).getTime());
  });

  it('tri-month around January reaches back into the previous year', () => {
    const w = windowFor('tri-month', D(2026, 1, 15));
    expect(w.start).toEqual(D(2025, 12, 1));
    expect(w.end).toEqual(D(2026, 3, 1));
  });

  it('tri-month around December reaches into the next year', () => {
    const w = windowFor('tri-month', D(2026, 12, 15));
    expect(w.start).toEqual(D(2026, 11, 1));
    expect(w.end).toEqual(D(2027, 2, 1));
  });

  it('falls back to a single day for an unrecognised view', () => {
    // The view name can arrive from persisted preferences, so a value written
    // by a newer build must degrade to something renderable rather than to an
    // empty or infinite range.
    const w = windowFor('not-a-view' as CalendarView, D(2026, 8, 13, 15));
    expect(w.start).toEqual(D(2026, 8, 13));
    expect(w.end).toEqual(D(2026, 8, 14));
  });
});
