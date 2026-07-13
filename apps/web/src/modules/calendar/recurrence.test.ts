import { describe, it, expect } from 'vitest';
import type { CalendarEvent } from '../../api/pim-types.ts';
import {
  parseDuration,
  formatDuration,
  parseRule,
  ruleToJson,
  describeRule,
  expandEvent,
  firstRule,
  type RecurrenceRule,
} from './recurrence.ts';

function mkEvent(over: Partial<CalendarEvent>): CalendarEvent {
  return {
    id: 'e', calendarId: 'c', uid: 'e', title: 'T', description: '', locations: [],
    start: '2026-07-13T09:00:00', timeZone: 'Europe/London', duration: 'PT1H', showWithoutTime: false,
    recurrenceRules: [], recurrenceOverrides: {}, excludedRecurrenceDates: [], status: 'confirmed',
    priority: 0, freeBusyStatus: 'busy', participants: {}, alerts: {}, sequence: 0, etag: null, ...over,
  };
}

describe('ISO 8601 duration', () => {
  it('round-trips common durations', () => {
    expect(parseDuration('PT1H')).toBe(3600000);
    expect(parseDuration('PT1H30M')).toBe(5400000);
    expect(parseDuration('P1D')).toBe(86400000);
    expect(formatDuration(5400000)).toBe('PT1H30M');
    expect(formatDuration(1800000)).toBe('PT30M');
    expect(formatDuration(86400000)).toBe('P1D');
    expect(formatDuration(0)).toBe('PT0S');
  });
});

describe('rule <-> json + describe', () => {
  it('narrows and serializes a weekly rule', () => {
    const raw = { frequency: 'weekly', interval: 2, byDay: ['mo', 'we'], count: 6 };
    const rule = parseRule(raw);
    expect(rule).toEqual({ frequency: 'weekly', interval: 2, byDay: ['mo', 'we'], count: 6 });
    // Serializes to the engine's RFC5545 `{ rrule }` shape (mw-ics reads this),
    // and parses back to the same typed model.
    expect(ruleToJson(rule!)).toEqual({ rrule: 'FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,WE;COUNT=6' });
    expect(parseRule(ruleToJson(rule!))).toEqual(rule);
  });

  it('parses the engine `{ rrule }` shape (with an UNTIL bound)', () => {
    expect(parseRule({ rrule: 'FREQ=DAILY;UNTIL=20260715T000000Z' })).toEqual({
      frequency: 'daily',
      until: '2026-07-15T00:00:00',
    });
  });

  it('rejects an unknown frequency', () => {
    expect(parseRule({ frequency: 'hourly' })).toBeNull();
    expect(parseRule({ rrule: 'FREQ=HOURLY' })).toBeNull();
  });

  it('describes a rule in prose', () => {
    const rule: RecurrenceRule = { frequency: 'weekly', interval: 2, byDay: ['we', 'mo'] };
    expect(describeRule(rule)).toBe('Every 2 weeks on Mon, Wed');
    expect(describeRule({ frequency: 'daily', count: 5 })).toBe('Every day, 5 times');
  });

  it('firstRule reads the first stored rule', () => {
    const ev = mkEvent({ recurrenceRules: [{ frequency: 'daily' }] });
    expect(firstRule(ev)?.frequency).toBe('daily');
    expect(firstRule(mkEvent({}))).toBeNull();
  });
});

const W_START = new Date(2026, 6, 1);
const W_END = new Date(2026, 8, 30);

describe('expandEvent', () => {
  it('yields a single instance for a non-recurring event in-window', () => {
    const ev = mkEvent({ start: '2026-07-13T09:00:00' });
    const out = expandEvent(ev, W_START, W_END, '#000');
    expect(out).toHaveLength(1);
    expect(out[0]!.start.getHours()).toBe(9);
  });

  it('expands a daily COUNT rule', () => {
    const ev = mkEvent({ start: '2026-07-13T09:00:00', recurrenceRules: [{ frequency: 'daily', count: 3 }] });
    const out = expandEvent(ev, new Date(2026, 6, 13), new Date(2026, 6, 20), '#000');
    expect(out).toHaveLength(3);
    expect(out.map((i) => i.start.getDate())).toEqual([13, 14, 15]);
  });

  it('expands a weekly BYDAY COUNT rule', () => {
    const ev = mkEvent({ start: '2026-07-13T09:00:00', recurrenceRules: [{ frequency: 'weekly', byDay: ['mo', 'we'], count: 4 }] });
    const out = expandEvent(ev, new Date(2026, 6, 13), new Date(2026, 6, 31), '#000');
    expect(out).toHaveLength(4);
    for (const inst of out) {
      expect([1, 3]).toContain(inst.start.getDay());
      expect(inst.start.getHours()).toBe(9);
    }
  });

  it('expands a monthly BYMONTHDAY rule', () => {
    const ev = mkEvent({ start: '2026-07-13T09:00:00', recurrenceRules: [{ frequency: 'monthly', byMonthDay: [15], count: 2 }] });
    const out = expandEvent(ev, W_START, W_END, '#000');
    expect(out.map((i) => `${i.start.getMonth()}-${i.start.getDate()}`)).toEqual(['6-15', '7-15']);
  });

  it('expands a yearly rule', () => {
    const ev = mkEvent({ start: '2026-07-13T09:00:00', recurrenceRules: [{ frequency: 'yearly', count: 2 }] });
    const out = expandEvent(ev, new Date(2026, 0, 1), new Date(2028, 0, 1), '#000');
    expect(out.map((i) => i.start.getFullYear())).toEqual([2026, 2027]);
  });

  it('skips EXDATE occurrences', () => {
    const ev = mkEvent({
      start: '2026-07-13T09:00:00',
      recurrenceRules: [{ frequency: 'daily', count: 3 }],
      excludedRecurrenceDates: ['2026-07-14'],
    });
    const out = expandEvent(ev, new Date(2026, 6, 13), new Date(2026, 6, 20), '#000');
    expect(out.map((i) => i.start.getDate())).toEqual([13, 15]);
  });

  it('honors an UNTIL bound', () => {
    const ev = mkEvent({ start: '2026-07-13T09:00:00', recurrenceRules: [{ frequency: 'daily', until: '2026-07-15T00:00:00' }] });
    const out = expandEvent(ev, new Date(2026, 6, 13), new Date(2026, 6, 31), '#000');
    expect(out.map((i) => i.start.getDate())).toEqual([13, 14]);
  });

  it('keeps the wall-clock time stable across a DST boundary (weekly)', () => {
    // EU spring-forward is 29 March 2026; a weekly Monday 09:30 event must stay
    // 09:30 on every occurrence (client renders wall-clock; engine owns TZID).
    const ev = mkEvent({ start: '2026-03-23T09:30:00', duration: 'PT1H', recurrenceRules: [{ frequency: 'weekly', byDay: ['mo'], count: 3 }] });
    const out = expandEvent(ev, new Date(2026, 2, 1), new Date(2026, 4, 1), '#000');
    expect(out).toHaveLength(3);
    for (const inst of out) {
      expect(inst.start.getDay()).toBe(1);
      expect(inst.start.getHours()).toBe(9);
      expect(inst.start.getMinutes()).toBe(30);
    }
  });
});
