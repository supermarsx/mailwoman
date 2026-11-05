import { describe, expect, it } from 'vitest';
import type { BreachClock, BreachSummary } from '../../../api/admin/breaches.ts';
import { SETTING_GROUPS, clockTone, fromLocalInput, needsReason, remaining, splitList, toLocalInput, urgent } from './model.ts';

const clock = (over: Partial<BreachClock>): BreachClock => ({ phase: 'running', awareAt: 0, deadlineAt: 72 * 3600, remainingSecs: null, elapsedSecs: null, notifiedAt: null, ...over });

describe('breach register model', () => {
  it('measures the time to the deadline in whole hours and minutes', () => {
    expect(remaining(72 * 3600, 24 * 3600)).toEqual({ overdue: false, hours: 48, minutes: 0 });
    expect(remaining(72 * 3600, 72 * 3600 + 90 * 60)).toEqual({ overdue: true, hours: 1, minutes: 30 });
    expect(remaining(100, 100)).toEqual({ overdue: false, hours: 0, minutes: 0 });
  });

  it('reads the clock as idle, ok, soon, overdue or stopped', () => {
    expect(clockTone(clock({ phase: 'not-started', deadlineAt: null }), 0)).toBe('idle');
    expect(clockTone(clock({}), 0)).toBe('ok');
    expect(clockTone(clock({}), 50 * 3600)).toBe('soon');
    expect(clockTone(clock({}), 72 * 3600 + 1)).toBe('overdue');
    expect(clockTone(clock({ phase: 'stopped-late' }), 99 * 3600)).toBe('stopped');
    expect(clockTone(clock({ phase: 'not-required' }), 0)).toBe('stopped');
  });

  it('asks for a reason when closing, marking not notifiable or reopening', () => {
    expect(needsReason('assessing', 'closed')).toBe(true);
    expect(needsReason('assessing', 'not-notifiable')).toBe(true);
    expect(needsReason('closed', 'assessing')).toBe(true);
    expect(needsReason('assessing', 'contained')).toBe(false);
  });

  it('round-trips a local date time input', () => {
    const secs = Math.floor(new Date(2026, 10, 5, 9, 30).getTime() / 1000);
    expect(toLocalInput(secs)).toBe('2026-11-05T09:30');
    expect(fromLocalInput('2026-11-05T09:30')).toBe(secs);
    expect(fromLocalInput('')).toBeUndefined();
    expect(fromLocalInput('not a date')).toBeUndefined();
  });

  it('splits lists on commas, semicolons and lines', () => {
    expect(splitList('a@x.test, b@x.test;c@x.test\n\n d@x.test ')).toEqual(['a@x.test', 'b@x.test', 'c@x.test', 'd@x.test']);
  });

  it('counts overdue and soon rows for the banner', () => {
    const row = (c: BreachClock): BreachSummary => ({ clock: c }) as BreachSummary;
    expect(urgent([row(clock({})), row(clock({ deadlineAt: 10 })), row(clock({ deadlineAt: 3600 }))], 100)).toEqual({ overdue: 1, soon: 1 });
  });

  it('lists every breach setting once', () => {
    const keys = SETTING_GROUPS.flatMap((g) => g.keys);
    expect(new Set(keys).size).toBe(keys.length);
    expect(keys.every((k) => k.startsWith('breach.'))).toBe(true);
    expect(keys).toContain('breach.noticeHours');
  });
});
