import { beforeAll, describe, expect, it } from 'vitest';
import { loadCatalog } from '../../i18n';
import {
  describeRecurrence, displayDescription, durationMs, eventRange, formatRange, joinLink, linkSegments,
  organizerTime, safeUrl,
} from './eventInfo.ts';

beforeAll(async () => { await loadCatalog('calendar'); });

describe('event card text', () => {
  it('reads ISO durations', () => {
    expect(durationMs('PT1H30M')).toBe(90 * 60_000);
    expect(durationMs('P1D')).toBe(86_400_000);
    expect(durationMs('P2W')).toBe(14 * 86_400_000);
    expect(durationMs('nonsense')).toBe(0);
  });

  it('places a zoned event at its instant and shows the organizer clock only when it differs', () => {
    const ev = { start: '2026-11-03T10:00:00', timeZone: 'Europe/Berlin', duration: 'PT1H' };
    const range = eventRange(ev, 'Europe/Lisbon')!;
    expect(range.start.toISOString()).toBe('2026-11-03T09:00:00.000Z');
    expect(range.end.toISOString()).toBe('2026-11-03T10:00:00.000Z');
    expect(formatRange(range, 'Europe/Lisbon')).toContain('9:00');
    expect(organizerTime(ev, range, 'Europe/Lisbon')).toContain('Europe/Berlin');
    expect(organizerTime(ev, range, 'Europe/Paris')).toBeNull();
    expect(organizerTime({ ...ev, timeZone: 'UTC' }, range, 'Europe/Lisbon')).toBeNull();
  });

  it('reads an all-day event as dates, never shifted by the viewer zone', () => {
    const range = eventRange({ start: '2026-12-24', showWithoutTime: true, duration: 'P2D' }, 'America/Los_Angeles')!;
    expect(range.allDay).toBe(true);
    const text = formatRange(range, 'America/Los_Angeles');
    expect(text).toContain('24');
    expect(text).toContain('25');
    expect(text).toContain('all day');
  });

  it('describes recurrence in words and never guesses a rule it does not read', () => {
    expect(describeRecurrence({ recurrenceRules: [{ rrule: 'FREQ=WEEKLY;BYDAY=TU;UNTIL=20261203T235959Z' }] }))
      .toMatch(/^Every Tuesday until Dec 3, 2026$/);
    expect(describeRecurrence({ recurrenceRules: [{ rrule: 'FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,WE' }] }))
      .toBe('Every 2 weeks on Monday and Wednesday');
    expect(describeRecurrence({ recurrenceRules: [{ rrule: 'FREQ=DAILY;COUNT=5' }] })).toBe('Every day, 5 times');
    expect(describeRecurrence({ recurrenceRules: [{ rrule: 'FREQ=MONTHLY;BYMONTHDAY=15' }] })).toBe('Every month on day 15');
    expect(describeRecurrence({ recurrenceRules: [{ rrule: 'FREQ=MONTHLY;BYDAY=2TU' }] })).toBe('Repeats on a custom schedule');
    expect(describeRecurrence({ recurrenceRules: [{ rrule: 'FREQ=MONTHLY;BYDAY=TU;BYSETPOS=2' }] })).toBe('Repeats on a custom schedule');
    expect(describeRecurrence({ recurrenceRules: [{ rrule: 'FREQ=WEEKLY;BYDAY=TU' }], excludedRecurrenceDates: ['2026-11-10T10:00:00'] }))
      .toBe('Every Tuesday (1 date skipped)');
    expect(describeRecurrence({})).toBeNull();
  });

  it('finds meeting join links only on the meeting services', () => {
    expect(joinLink({ locations: [{ name: 'https://teams.microsoft.com/l/meetup-join/abc' }] })?.provider).toBe('teams');
    expect(joinLink({ description: 'Join: https://acme.zoom.us/j/123?pwd=x.' })).toEqual({ provider: 'zoom', url: 'https://acme.zoom.us/j/123?pwd=x' });
    expect(joinLink({ description: 'https://meet.google.com/abc-defg-hij' })?.provider).toBe('meet');
    expect(joinLink({ locations: [{ name: 'https://meet.jit.si/Standup' }] })?.provider).toBe('jitsi');
    expect(joinLink({ description: 'https://teams.microsoft.com.evil.example/l/x' })).toBeNull();
    expect(joinLink({ description: 'javascript:alert(1) teams.microsoft.com' })).toBeNull();
  });

  it('only ever links http(s) URLs', () => {
    expect(safeUrl('https://example.com/a')).toBe('https://example.com/a');
    expect(safeUrl('javascript:alert(1)')).toBeNull();
    expect(safeUrl('data:text/html,<b>x</b>')).toBeNull();
    expect(safeUrl('https://exa mple.com')).toBeNull();
    const segs = linkSegments('See <script>x</script> https://example.com/x, javascript:alert(1)');
    expect(segs.filter((s) => s.url).map((s) => s.url)).toEqual(['https://example.com/x']);
    expect(segs.map((s) => s.text).join('')).toBe('See <script>x</script> https://example.com/x, javascript:alert(1)');
  });

  it('bounds and cleans the description', () => {
    const long = displayDescription('a'.repeat(5000));
    expect(long.clipped).toBe(true);
    expect(long.text).toHaveLength(4000);
    expect(displayDescription('line\r\nnext\u0007').text).toBe('line\nnext');
  });
});
