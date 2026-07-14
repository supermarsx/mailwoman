// Pure date/time helpers for the calendar grid (plan §3 e4). DOM-free and
// dependency-free (native `Date` / `Intl`) so they are directly unit-testable.
//
// Time model (plan §1.12): event wall-clock times are `LocalDateTime` strings
// with a separate IANA `timeZone`; the engine does the authoritative TZID-aware
// expansion. In the browser the module renders occurrences the backend already
// resolved, so these helpers work in the viewer's local zone — the same
// discipline `Intl.DateTimeFormat` applies. All grid math is calendar-arithmetic
// (add days/months), never naive millisecond addition, so DST shifts are safe.

/** Parsed components of a `LocalDateTime` / date string. */
export interface DateParts {
  year: number;
  month: number; // 1-12
  day: number;
  hour: number;
  minute: number;
  second: number;
  /** True when the source had no time component (`"2026-07-12"`). */
  dateOnly: boolean;
}

/** Parse a `LocalDateTime` (`"2026-07-12T09:30:00"`) or date (`"2026-07-12"`). */
export function parseLocal(s: string): DateParts {
  const m = /^(\d{4})-(\d{2})-(\d{2})(?:[T ](\d{2}):(\d{2})(?::(\d{2}))?)?/.exec(s);
  if (m === null) {
    // Fall back to a Date parse so we never throw on odd-but-valid input.
    const d = new Date(s);
    return {
      year: d.getFullYear(),
      month: d.getMonth() + 1,
      day: d.getDate(),
      hour: d.getHours(),
      minute: d.getMinutes(),
      second: d.getSeconds(),
      dateOnly: false,
    };
  }
  const [, y, mo, d, h, mi, se] = m;
  const hasTime = h !== undefined;
  return {
    year: Number(y),
    month: Number(mo),
    day: Number(d),
    hour: hasTime ? Number(h) : 0,
    minute: hasTime && mi !== undefined ? Number(mi) : 0,
    second: hasTime && se !== undefined ? Number(se) : 0,
    dateOnly: !hasTime,
  };
}

/** A local `Date` from parsed parts (viewer-zone wall clock). */
export function partsToDate(p: DateParts): Date {
  return new Date(p.year, p.month - 1, p.day, p.hour, p.minute, p.second, 0);
}

/** A local `Date` from a `LocalDateTime` / date string. */
export function localToDate(s: string): Date {
  return partsToDate(parseLocal(s));
}

/** Serialize a `Date` back to a `LocalDateTime` (`"2026-07-12T09:30:00"`). */
export function dateToLocal(d: Date): string {
  return (
    `${pad(d.getFullYear(), 4)}-${pad(d.getMonth() + 1, 2)}-${pad(d.getDate(), 2)}` +
    `T${pad(d.getHours(), 2)}:${pad(d.getMinutes(), 2)}:${pad(d.getSeconds(), 2)}`
  );
}

/** Serialize the date portion only (`"2026-07-12"`). */
export function dateToCalDate(d: Date): string {
  return `${pad(d.getFullYear(), 4)}-${pad(d.getMonth() + 1, 2)}-${pad(d.getDate(), 2)}`;
}

function pad(n: number, width: number): string {
  return String(n).padStart(width, '0');
}

// ── Calendar arithmetic (DST-safe: operate on Y/M/D fields, not epoch ms) ─────

export function startOfDay(d: Date): Date {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate(), 0, 0, 0, 0);
}

export function endOfDay(d: Date): Date {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate(), 23, 59, 59, 999);
}

export function addDays(d: Date, n: number): Date {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate() + n, d.getHours(), d.getMinutes(), d.getSeconds());
}

export function addMonths(d: Date, n: number): Date {
  return new Date(d.getFullYear(), d.getMonth() + n, d.getDate(), d.getHours(), d.getMinutes(), d.getSeconds());
}

export function addYears(d: Date, n: number): Date {
  return new Date(d.getFullYear() + n, d.getMonth(), d.getDate(), d.getHours(), d.getMinutes(), d.getSeconds());
}

export function startOfMonth(d: Date): Date {
  return new Date(d.getFullYear(), d.getMonth(), 1, 0, 0, 0, 0);
}

export function daysInMonth(year: number, month0: number): number {
  return new Date(year, month0 + 1, 0).getDate();
}

/** `weekStartsOn`: 0 = Sunday … 6 = Saturday. */
export function startOfWeek(d: Date, weekStartsOn: number = 1): Date {
  const day = d.getDay();
  const diff = (day - weekStartsOn + 7) % 7;
  return addDays(startOfDay(d), -diff);
}

/**
 * The locale-correct first day of the week (0 = Sunday … 6 = Saturday), read
 * from the viewer's locale via `Intl.Locale`'s week info (e.g. Monday for most of
 * Europe, Sunday for en-US, Saturday for many MENA locales). Falls back to Monday
 * when the runtime doesn't expose week info. The calendar grid + the controller's
 * load window both consult this so they always agree on where a week begins.
 */
export function localeWeekStart(locale?: string): number {
  try {
    const tag = locale ?? (typeof navigator !== 'undefined' ? navigator.language : 'en');
    const loc = new Intl.Locale(tag) as Intl.Locale & {
      getWeekInfo?: () => { firstDay: number };
      weekInfo?: { firstDay: number };
    };
    const info = loc.getWeekInfo?.() ?? loc.weekInfo;
    // Intl week info uses ISO firstDay 1..7 (Mon..Sun); JS getDay() is 0..6
    // (Sun..Sat), so Sunday (7) maps to 0.
    if (info !== undefined && typeof info.firstDay === 'number') return info.firstDay % 7;
  } catch {
    /* Intl.Locale unsupported / bad tag — fall through to Monday. */
  }
  return 1;
}

export function sameDay(a: Date, b: Date): boolean {
  return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
}

export function isToday(d: Date, now: Date = new Date()): boolean {
  return sameDay(d, now);
}

export function isWeekend(d: Date): boolean {
  const day = d.getDay();
  return day === 0 || day === 6;
}

/** `n` consecutive days starting at `startOfDay(start)`. */
export function daysFrom(start: Date, n: number): Date[] {
  const base = startOfDay(start);
  const out: Date[] = [];
  for (let i = 0; i < n; i += 1) out.push(addDays(base, i));
  return out;
}

/**
 * A 6-row x 7-col month grid of day starts covering `year`/`month0`, padded with
 * the trailing days of the previous month and leading days of the next so every
 * cell is filled (the standard month-view layout).
 */
export function monthGrid(year: number, month0: number, weekStartsOn: number = 1): Date[][] {
  const first = new Date(year, month0, 1);
  const gridStart = startOfWeek(first, weekStartsOn);
  const weeks: Date[][] = [];
  for (let w = 0; w < 6; w += 1) {
    const row: Date[] = [];
    for (let d = 0; d < 7; d += 1) row.push(addDays(gridStart, w * 7 + d));
    weeks.push(row);
  }
  return weeks;
}

// ── Formatting (Intl, viewer locale) ─────────────────────────────────────────

const timeFmt = new Intl.DateTimeFormat(undefined, { hour: 'numeric', minute: '2-digit' });
const dayHeaderFmt = new Intl.DateTimeFormat(undefined, { weekday: 'short', day: 'numeric' });
const weekdayFmt = new Intl.DateTimeFormat(undefined, { weekday: 'short' });
const monthYearFmt = new Intl.DateTimeFormat(undefined, { month: 'long', year: 'numeric' });
const monthFmt = new Intl.DateTimeFormat(undefined, { month: 'long' });
const fullFmt = new Intl.DateTimeFormat(undefined, { weekday: 'long', month: 'long', day: 'numeric', year: 'numeric' });

export function formatTime(d: Date): string {
  return timeFmt.format(d);
}
export function formatDayHeader(d: Date): string {
  return dayHeaderFmt.format(d);
}
export function formatWeekday(d: Date): string {
  return weekdayFmt.format(d);
}
export function formatMonthYear(d: Date): string {
  return monthYearFmt.format(d);
}
export function formatMonth(d: Date): string {
  return monthFmt.format(d);
}
export function formatFull(d: Date): string {
  return fullFmt.format(d);
}

/**
 * Localized weekday header labels in display order for a grid that begins on
 * `weekStart` (0 = Sunday … 6 = Saturday). Derived from `Intl` so the month
 * header reads correctly in every locale. `2023-01-01` is a known Sunday anchor.
 */
export function weekdayNames(weekStart = 1, style: 'short' | 'long' = 'short'): string[] {
  const fmt = style === 'long' ? new Intl.DateTimeFormat(undefined, { weekday: 'long' }) : weekdayFmt;
  const sunday = new Date(2023, 0, 1);
  const out: string[] = [];
  for (let i = 0; i < 7; i += 1) out.push(fmt.format(addDays(sunday, weekStart + i)));
  return out;
}

/** Minutes since local midnight — the vertical position key for time grids. */
export function minutesOfDay(d: Date): number {
  return d.getHours() * 60 + d.getMinutes();
}

/** Clamp a [start,end] interval to a single day's [00:00, 24:00) in minutes. */
export function dayMinuteSpan(instStart: Date, instEnd: Date, day: Date): { top: number; height: number } | null {
  const dayStart = startOfDay(day);
  const dayEnd = addDays(dayStart, 1);
  if (instEnd <= dayStart || instStart >= dayEnd) return null;
  const startMin = instStart <= dayStart ? 0 : minutesOfDay(instStart);
  const endMin = instEnd >= dayEnd ? 24 * 60 : minutesOfDay(instEnd) || 24 * 60;
  return { top: startMin, height: Math.max(15, endMin - startMin) };
}
