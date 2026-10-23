import { describe, it, expect, afterEach } from 'vitest';
import { createRoot, createEffect } from 'solid-js';
import { setActiveLocale } from './registry.ts';
import {
  formatBytes,
  formatDate,
  formatDateTime,
  formatDateWith,
  formatList,
  formatNumber,
  formatPercent,
  formatRelative,
  formatTime,
  pluralCategory,
} from './format.ts';

// A fixed instant, formatted in UTC so the expectations do not depend on the
// machine's time zone.
const WHEN = Date.UTC(2026, 9, 23, 21, 5, 0);
const UTC = { timeZone: 'UTC' } as const;

afterEach(() => setActiveLocale('en'));

describe('format.ts uses the active app locale', () => {
  it('formats dates in en, then de after a locale switch', () => {
    expect(formatDate(WHEN, { dateStyle: 'medium', ...UTC })).toBe('Oct 23, 2026');
    setActiveLocale('de');
    expect(formatDate(WHEN, { dateStyle: 'medium', ...UTC })).toBe('23.10.2026');
  });

  it('formats times and date-times', () => {
    expect(formatTime(WHEN, { timeStyle: 'short', ...UTC })).toMatch(/^9:05\sPM$/);
    expect(formatDateTime(WHEN, { dateStyle: 'medium', timeStyle: 'short', ...UTC })).toMatch(
      /^Oct 23, 2026, 9:05\sPM$/,
    );
    setActiveLocale('de');
    expect(formatTime(WHEN, { timeStyle: 'short', ...UTC })).toBe('21:05');
  });

  it('accepts Date, epoch ms and ISO strings, and returns "" for an invalid date', () => {
    const opts = { year: 'numeric', ...UTC } as const;
    expect(formatDateWith(new Date(WHEN), opts)).toBe('2026');
    expect(formatDateWith('2026-10-23T21:05:00Z', opts)).toBe('2026');
    expect(formatDateWith('not a date', opts)).toBe('');
    expect(formatDate(Number.NaN)).toBe('');
  });

  it('formats relative times with the right unit and direction', () => {
    const now = WHEN;
    expect(formatRelative(now - 5 * 60_000, now)).toBe('5 minutes ago');
    expect(formatRelative(now + 2 * 86_400_000, now)).toBe('in 2 days');
    expect(formatRelative(now - 86_400_000, now)).toBe('yesterday');
    expect(formatRelative(now, now)).toBe('now');
    expect(formatRelative(now - 3 * 3_600_000, now)).toBe('3 hours ago');
    expect(formatRelative(now - 400 * 86_400_000, now)).toBe('last year');
    setActiveLocale('de');
    expect(formatRelative(now - 5 * 60_000, now)).toBe('vor 5 Minuten');
  });

  it('formats numbers and percentages with the locale separators', () => {
    expect(formatNumber(12345.6)).toBe('12,345.6');
    expect(formatPercent(0.25)).toBe('25%');
    setActiveLocale('de');
    expect(formatNumber(12345.6)).toBe('12.345,6');
    expect(formatPercent(0.125)).toMatch(/^12,5\s%$/);
  });

  it('formats byte sizes in 1024 steps with localized unit names', () => {
    expect(formatBytes(512)).toBe('512 byte');
    expect(formatBytes(2048)).toBe('2 kB');
    expect(formatBytes(1_500_000)).toBe('1.4 MB');
    expect(formatBytes(3 * 1024 ** 3)).toBe('3 GB');
    expect(formatBytes(-1)).toBe('0 byte');
    setActiveLocale('de');
    expect(formatBytes(1_500_000)).toBe('1,4 MB');
    setActiveLocale('ru');
    expect(formatBytes(1_500_000)).toBe('1,4 МБ');
  });

  it('joins lists the locale way', () => {
    expect(formatList(['a', 'b', 'c'])).toBe('a, b, and c');
    setActiveLocale('de');
    expect(formatList(['a', 'b', 'c'])).toBe('a, b und c');
  });

  it('reports CLDR plural categories per locale', () => {
    expect(pluralCategory(1)).toBe('one');
    expect(pluralCategory(2)).toBe('other');
    setActiveLocale('ar');
    expect(pluralCategory(0)).toBe('zero');
    expect(pluralCategory(2)).toBe('two');
    expect(pluralCategory(5)).toBe('few');
    expect(pluralCategory(11)).toBe('many');
    setActiveLocale('ja');
    expect(pluralCategory(1)).toBe('other');
  });

  it('re-runs a reactive reader when the locale changes', () => {
    const seen: string[] = [];
    const dispose = createRoot((d) => {
      createEffect(() => seen.push(formatNumber(1234.5)));
      return d;
    });
    setActiveLocale('de');
    dispose();
    expect(seen).toEqual(['1,234.5', '1.234,5']);
  });
});
