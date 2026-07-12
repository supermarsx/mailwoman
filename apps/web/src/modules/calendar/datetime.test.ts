import { describe, it, expect } from 'vitest';
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
