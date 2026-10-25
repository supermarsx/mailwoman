import { afterEach, describe, it, expect, vi } from 'vitest';
import { setActiveLocale } from '../../i18n/registry.ts';
import {
  parseLocal,
  localToDate,
  dateToLocal,
  dateToCalDate,
  addDays,
  addMonths,
  addYears,
  startOfWeek,
  monthGrid,
  daysInMonth,
  daysFrom,
  dayMinuteSpan,
  minutesOfDay,
  sameDay,
  formatFull,
  formatMonth,
  formatMonthYear,
  formatTime,
  formatWeekday,
  localeWeekStart,
  weekdayNames,
} from './datetime.ts';

describe('calendar datetime helpers', () => {
  it('parses a LocalDateTime and a date-only string', () => {
    expect(parseLocal('2026-07-12T09:30:15')).toEqual({
      year: 2026, month: 7, day: 12, hour: 9, minute: 30, second: 15, dateOnly: false,
    });
    const d = parseLocal('2026-07-12');
    expect(d.dateOnly).toBe(true);
    expect(d.hour).toBe(0);
  });

  it('round-trips Date <-> LocalDateTime', () => {
    const s = '2026-07-12T09:30:00';
    expect(dateToLocal(localToDate(s))).toBe(s);
    expect(dateToCalDate(localToDate(s))).toBe('2026-07-12');
  });

  it('adds calendar units on Y/M/D fields (DST-safe, not epoch ms)', () => {
    const base = new Date(2026, 2, 8, 9, 30); // 8 March 2026, before EU spring-forward
    // Adding 30 days across the DST boundary keeps the 09:30 wall clock.
    const later = addDays(base, 30);
    expect(later.getHours()).toBe(9);
    expect(later.getMinutes()).toBe(30);
    expect(addMonths(base, 1).getMonth()).toBe(3);
    expect(addYears(base, 1).getFullYear()).toBe(2027);
  });

  it('startOfWeek(Monday) lands on Monday', () => {
    const wed = new Date(2026, 6, 15); // 15 July 2026 is a Wednesday
    const mon = startOfWeek(wed, 1);
    expect(mon.getDay()).toBe(1);
    expect(mon.getDate()).toBe(13);
  });

  it('monthGrid is 6x7 and covers the month', () => {
    const grid = monthGrid(2026, 6, 1); // July 2026
    expect(grid).toHaveLength(6);
    expect(grid[0]).toHaveLength(7);
    // Every row starts on Monday.
    for (const week of grid) expect(week[0]!.getDay()).toBe(1);
    expect(daysInMonth(2026, 6)).toBe(31);
  });

  it('daysFrom yields n consecutive day-starts', () => {
    const days = daysFrom(new Date(2026, 6, 12, 15, 0), 3);
    expect(days.map((d) => d.getDate())).toEqual([12, 13, 14]);
    expect(days[0]!.getHours()).toBe(0);
  });

  it('dayMinuteSpan clamps an interval to a single day', () => {
    const day = new Date(2026, 6, 12);
    const span = dayMinuteSpan(new Date(2026, 6, 12, 9, 0), new Date(2026, 6, 12, 10, 30), day);
    expect(span).toEqual({ top: 540, height: 90 });
    // An event on another day yields null.
    expect(dayMinuteSpan(new Date(2026, 6, 13, 9, 0), new Date(2026, 6, 13, 10, 0), day)).toBeNull();
  });

  it('minutesOfDay + sameDay', () => {
    expect(minutesOfDay(new Date(2026, 6, 12, 2, 30))).toBe(150);
    expect(sameDay(new Date(2026, 6, 12, 1), new Date(2026, 6, 12, 23))).toBe(true);
    expect(sameDay(new Date(2026, 6, 12), new Date(2026, 6, 13))).toBe(false);
  });
});

// Display text follows the active app locale (the one `t()` uses), read when the
// function is called, not the browser's locale frozen when the module loaded.
describe('calendar formatting follows the app locale', () => {
  afterEach(() => {
    setActiveLocale('en');
    vi.unstubAllGlobals();
  });

  it('formats month, weekday and full date in the locale picked after load', () => {
    const d = new Date(2026, 9, 25, 14, 5);
    const de = (o: Intl.DateTimeFormatOptions, at: Date = d): string => new Intl.DateTimeFormat('de', o).format(at);
    expect(formatMonth(d)).toBe('October');
    setActiveLocale('de');
    expect(formatMonth(d)).toBe('Oktober');
    expect(formatMonthYear(d)).toBe(de({ month: 'long', year: 'numeric' }));
    expect(formatWeekday(d)).toBe(de({ weekday: 'short' }));
    expect(formatFull(d)).toBe(de({ weekday: 'long', month: 'long', day: 'numeric', year: 'numeric' }));
    expect(formatTime(d)).toBe('14:05');
    expect(weekdayNames(1, 'long')[0]).toBe('Montag');
    expect(weekdayNames(1)).toEqual([...Array(7).keys()].map((i) => de({ weekday: 'short' }, new Date(2023, 0, 2 + i))));
  });

  it('starts the week by the app language, keeping the browser region when the language matches', () => {
    vi.stubGlobal('navigator', { language: 'en-US' });
    expect(localeWeekStart()).toBe(0);
    setActiveLocale('de');
    expect(localeWeekStart()).toBe(1);
    vi.stubGlobal('navigator', { language: 'en-GB' });
    setActiveLocale('en');
    expect(localeWeekStart()).toBe(1);
    expect(localeWeekStart('ar-EG')).toBe(6);
  });
});
