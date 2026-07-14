// Accessibility helpers for the calendar grid (t8-e2, SPEC §24 L1371-1372).
//
// The calendar's flagship a11y surface is the WAI-ARIA `grid` date picker: a
// `role=grid` of `role=row` weeks and `role=gridcell` days, a single roving
// tabindex, full arrow-key date navigation, and a polite live region that
// announces the focused day + its event count. These helpers keep that logic
// DOM-light and unit-testable, and — crucially — make arrow-key movement
// direction-aware so the grid mirrors correctly under `dir="rtl"`.

import { addDays, addMonths, startOfDay, startOfWeek } from './datetime.ts';

/**
 * The effective writing direction for an element. Reads the nearest ancestor
 * carrying an explicit `dir` attribute (the shell sets `dir` on `<html>` from
 * the locale), falling back to the document element and finally LTR. Used so the
 * grid swaps ArrowLeft/ArrowRight under RTL without depending on the i18n
 * provider (the PIM modules render provider-free in unit tests).
 */
export function effectiveDir(el?: Element | null): 'ltr' | 'rtl' {
  const fromEl = el?.closest?.('[dir]')?.getAttribute('dir');
  const dir = fromEl ?? (typeof document !== 'undefined' ? document.documentElement.getAttribute('dir') : null);
  return dir === 'rtl' ? 'rtl' : 'ltr';
}

/** The date-grid navigation keys the month grid understands. */
export type GridKey =
  | 'ArrowLeft'
  | 'ArrowRight'
  | 'ArrowUp'
  | 'ArrowDown'
  | 'Home'
  | 'End'
  | 'PageUp'
  | 'PageDown';

/**
 * Resolve a keyboard event to the next focused date on a month grid, or `null`
 * when the key isn't a navigation key. Pure — takes the current focused date,
 * the week-start, and the resolved direction, and returns the target day.
 *
 * - Left/Right move ±1 day, MIRRORED under RTL (Right = previous day).
 * - Up/Down move ±7 days (one week).
 * - Home/End jump to the first/last day of the focused week.
 * - PageUp/PageDown move ±1 month (±1 year with Shift).
 */
export function nextGridDate(
  key: string,
  current: Date,
  opts: { dir: 'ltr' | 'rtl'; weekStart: number; shift?: boolean },
): Date | null {
  const back = opts.dir === 'rtl' ? 1 : -1;
  const fwd = -back;
  switch (key) {
    case 'ArrowRight':
      return addDays(current, fwd);
    case 'ArrowLeft':
      return addDays(current, back);
    case 'ArrowDown':
      return addDays(current, 7);
    case 'ArrowUp':
      return addDays(current, -7);
    case 'Home':
      return startOfWeek(current, opts.weekStart);
    case 'End':
      return addDays(startOfWeek(current, opts.weekStart), 6);
    case 'PageUp':
      return addMonths(current, opts.shift === true ? -12 : -1);
    case 'PageDown':
      return addMonths(current, opts.shift === true ? 12 : 1);
    default:
      return null;
  }
}

/** True when `day` falls inside the half-open [start, end) load window. */
export function inWindow(day: Date, window: { start: Date; end: Date }): boolean {
  const d = startOfDay(day).getTime();
  return d >= startOfDay(window.start).getTime() && d < startOfDay(window.end).getTime();
}
