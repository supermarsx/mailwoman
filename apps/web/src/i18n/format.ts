// Locale-aware formatting for dates, times, numbers, byte sizes and lists
// (t32-admin §2.9).
//
// Every function formats in the ACTIVE APP LOCALE (the one `t()` uses), not the
// browser's default locale: a user who picked German in Mailwoman sees German
// dates even on an English browser. Reading the locale goes through the
// registry's reactive signal, so a call inside JSX re-renders when the locale
// changes, exactly like `t()`.
//
// Use these instead of `toLocaleString()` / `toLocaleDateString()` /
// `new Intl.*Format(undefined, …)` anywhere in the app; the hard-coded-string
// check (scripts/i18n/no-hardcoded-strings.mjs) reports those outside this file.
//
// Counted text ("3 messages") stays a Fluent selector in the .ftl file: pass the
// number to `t()` and let the catalog pick the plural form. `pluralCategory` is
// here only for code that must branch on the CLDR category itself.

import { activeLocaleSignal } from './registry.ts';

/** A date-like input: a `Date`, epoch milliseconds, or an ISO-8601 string. */
export type DateInput = Date | number | string;

// Formatters are costly to build; cache one per (locale, kind, options).
const cache = new Map<string, unknown>();

function cached<T>(kind: string, locale: string, options: object | undefined, make: () => T): T {
  const key = `${kind}|${locale}|${options === undefined ? '' : JSON.stringify(options)}`;
  let f = cache.get(key) as T | undefined;
  if (f === undefined) {
    f = make();
    cache.set(key, f);
  }
  return f;
}

/** The locale every formatter uses: the active app locale (reactive). */
export function formatLocale(): string {
  return activeLocaleSignal();
}

function toDate(value: DateInput): Date | null {
  const d = value instanceof Date ? value : new Date(value);
  return Number.isNaN(d.getTime()) ? null : d;
}

function dateTimeFormat(options: Intl.DateTimeFormatOptions): Intl.DateTimeFormat {
  const locale = formatLocale();
  return cached('dt', locale, options, () => new Intl.DateTimeFormat(locale, options));
}

/** Format with explicit `Intl.DateTimeFormat` options. Returns '' for an invalid date. */
export function formatDateWith(value: DateInput, options: Intl.DateTimeFormatOptions): string {
  const d = toDate(value);
  return d === null ? '' : dateTimeFormat(options).format(d);
}

/** A calendar date, e.g. "Oct 23, 2026" in `en`. Returns '' for an invalid date. */
export function formatDate(value: DateInput, options: Intl.DateTimeFormatOptions = { dateStyle: 'medium' }): string {
  return formatDateWith(value, options);
}

/** A time of day, e.g. "11:18 PM" in `en`. Returns '' for an invalid date. */
export function formatTime(value: DateInput, options: Intl.DateTimeFormatOptions = { timeStyle: 'short' }): string {
  return formatDateWith(value, options);
}

/** A date and time, e.g. "Oct 23, 2026, 11:18 PM" in `en`. Returns '' for an invalid date. */
export function formatDateTime(
  value: DateInput,
  options: Intl.DateTimeFormatOptions = { dateStyle: 'medium', timeStyle: 'short' },
): string {
  return formatDateWith(value, options);
}

/** Parts of a formatted date, for callers that lay out the pieces themselves. */
export function formatDateToParts(value: DateInput, options: Intl.DateTimeFormatOptions): Intl.DateTimeFormatPart[] {
  const d = toDate(value);
  return d === null ? [] : dateTimeFormat(options).formatToParts(d);
}

const SECOND = 1000;
const MINUTE = 60 * SECOND;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

/**
 * A time relative to `now`, e.g. "5 minutes ago", "in 2 days", "yesterday" in
 * `en`. The unit is the largest one that keeps the number meaningful (seconds
 * under 45 s, minutes under 45 min, hours under 22 h, days under 26 d, months
 * under 11 months, then years). Returns '' for an invalid date.
 */
export function formatRelative(value: DateInput, now: DateInput = Date.now()): string {
  const d = toDate(value);
  const n = toDate(now);
  if (d === null || n === null) return '';
  const diff = d.getTime() - n.getTime();
  const abs = Math.abs(diff);
  let amount: number;
  let unit: Intl.RelativeTimeFormatUnit;
  if (abs < 45 * SECOND) {
    amount = diff / SECOND;
    unit = 'second';
  } else if (abs < 45 * MINUTE) {
    amount = diff / MINUTE;
    unit = 'minute';
  } else if (abs < 22 * HOUR) {
    amount = diff / HOUR;
    unit = 'hour';
  } else if (abs < 26 * DAY) {
    amount = diff / DAY;
    unit = 'day';
  } else if (abs < 320 * DAY) {
    amount = diff / (30.4375 * DAY);
    unit = 'month';
  } else {
    amount = diff / (365.25 * DAY);
    unit = 'year';
  }
  const locale = formatLocale();
  const rtf = cached('rt', locale, undefined, () => new Intl.RelativeTimeFormat(locale, { numeric: 'auto' }));
  // Math.round(-0.4) is -0; format(-0) reads "0 seconds ago", format(0) "now".
  return rtf.format(Math.round(amount) || 0, unit);
}

function numberFormat(options: Intl.NumberFormatOptions | undefined): Intl.NumberFormat {
  const locale = formatLocale();
  return cached('num', locale, options, () => new Intl.NumberFormat(locale, options));
}

/** A number with the locale's digits and separators, e.g. "12,345.6" in `en`. */
export function formatNumber(value: number, options?: Intl.NumberFormatOptions): string {
  return numberFormat(options).format(value);
}

/** A ratio as a percentage: `formatPercent(0.25)` is "25%" in `en`. */
export function formatPercent(ratio: number, options: Intl.NumberFormatOptions = {}): string {
  return numberFormat({ maximumFractionDigits: 1, ...options, style: 'percent' }).format(ratio);
}

const BYTE_UNITS = ['byte', 'kilobyte', 'megabyte', 'gigabyte', 'terabyte'] as const;

/**
 * A byte count with a localized unit, e.g. "512 byte", "1.4 MB" in `en` or
 * "1,4 MB" in `de`. Steps are 1024 (the convention the UI used before this
 * file); the unit names are the locale's CLDR short forms. At most one decimal
 * place, none for whole bytes.
 */
export function formatBytes(bytes: number, options: { maximumFractionDigits?: number } = {}): string {
  let n = Math.max(0, bytes);
  let i = 0;
  while (n >= 1024 && i < BYTE_UNITS.length - 1) {
    n /= 1024;
    i += 1;
  }
  return numberFormat({
    style: 'unit',
    unit: BYTE_UNITS[i],
    unitDisplay: 'short',
    maximumFractionDigits: i === 0 ? 0 : (options.maximumFractionDigits ?? 1),
  }).format(n);
}

/** The units a duration can be shown in, smallest first. */
export type DurationUnit = 'nanosecond' | 'microsecond' | 'millisecond' | 'second' | 'minute' | 'hour';

// [unit, size in ms, the value at which the next unit takes over].
const DURATION_UNITS: readonly (readonly [DurationUnit, number, number])[] = [
  ['nanosecond', 1e-6, 1000],
  ['microsecond', 1e-3, 1000],
  ['millisecond', 1, 1000],
  ['second', 1000, 60],
  ['minute', 60_000, 60],
  ['hour', 3_600_000, Infinity],
];

// CLDR's short English forms are "sec" and "hr"; the SI symbols read better
// next to "ms" and "μs". Only the unit part of the formatted string is touched.
const UNIT_SYMBOL: Readonly<Record<string, string>> = { sec: 's', hr: 'h', 'µs': 'μs' };

export interface DurationFormatOptions {
  /**
   * The step the source measures in, in ms (1 for whole milliseconds, 0.001 for
   * microseconds). A value below it, zero included, reads "<1 ms" rather than a
   * zero the source could not tell apart from a short non-zero time.
   */
  resolutionMs?: number | null | undefined;
  /** Significant digits, default 3 ("420 μs", "1.25 ms", "37 ms", "2.4 s"). */
  significantDigits?: number;
}

/** Round to `digits` significant digits (whole numbers keep every integer digit). */
function roundSignificant(v: number, digits: number): number {
  if (v === 0) return 0;
  if (v >= 10 ** digits) return Math.round(v);
  return Number(v.toPrecision(digits));
}

function unitIndex(unit: DurationUnit): number {
  return DURATION_UNITS.findIndex(([u]) => u === unit);
}

/** The unit `formatDurationMs` uses for `ms`, after rounding. */
export function durationUnit(ms: number, significantDigits = 3): DurationUnit {
  const abs = Math.abs(ms);
  if (!Number.isFinite(abs) || abs === 0) return 'millisecond';
  let i = DURATION_UNITS.findIndex(([, size, next]) => abs / size < next);
  if (i < 0) i = DURATION_UNITS.length - 1;
  // 999.7 μs rounds to "1000 μs"; show "1 ms" instead.
  while (i < DURATION_UNITS.length - 1 && roundSignificant(abs / DURATION_UNITS[i]![1], significantDigits) >= DURATION_UNITS[i]![2]) i += 1;
  return DURATION_UNITS[i]![0];
}

function renderIn(ms: number, unit: DurationUnit, digits: number): string {
  const [, size] = DURATION_UNITS[unitIndex(unit)]!;
  const v = roundSignificant(Math.abs(ms) / size, digits);
  const signed = ms < 0 && v !== 0 ? -v : v;
  const options: Intl.NumberFormatOptions = {
    style: 'unit',
    unit,
    unitDisplay: 'short',
    ...(v >= 10 ** digits ? { maximumFractionDigits: 0 } : { maximumSignificantDigits: digits }),
  };
  return numberFormat(options)
    .formatToParts(signed)
    .map((part) => (part.type === 'unit' ? (UNIT_SYMBOL[part.value] ?? part.value) : part.value))
    .join('');
}

/**
 * A measured duration given in milliseconds, in the unit that keeps it readable:
 * ns below 1 μs, μs below 1 ms, ms below 1 s, s below 1 min, then min and h
 * ("420 μs", "1.25 ms", "37 ms", "2.4 s", "1.5 h"). Digits follow the active
 * locale. A non-zero value never reads "0 ms"; with `resolutionMs`, a value
 * below the source's step reads "<1 ms" (or "<1 μs"). Returns "—" for NaN or
 * an infinite value.
 */
export function formatDurationMs(ms: number, options: DurationFormatOptions = {}): string {
  if (typeof ms !== 'number' || !Number.isFinite(ms)) return '—';
  const digits = options.significantDigits ?? 3;
  const res = options.resolutionMs;
  if (typeof res === 'number' && res > 0 && Number.isFinite(res) && Math.abs(ms) < res) {
    return `<${renderIn(res, durationUnit(res, digits), digits)}`;
  }
  return renderIn(ms, durationUnit(ms, digits), digits);
}

/** `formatDurationMs` for a value in microseconds. */
export function formatDurationMicros(us: number, options: DurationFormatOptions = {}): string {
  return formatDurationMs(us / 1000, options);
}

/**
 * One formatter for a set of values that share a unit (a chart, a table
 * column): the unit is the one the largest absolute value reads in, so every
 * value in the set is comparable at a glance ("0.42 ms", "12 ms", "380 ms").
 * Small values keep their significant digits, so they never read as zero.
 * The returned function takes an optional per-value resolution that overrides
 * `options.resolutionMs`.
 */
export function durationFormatterFor(
  values: readonly (number | null | undefined)[],
  options: DurationFormatOptions = {},
): (ms: number, resolutionMs?: number | null) => string {
  const digits = options.significantDigits ?? 3;
  const finite = values.filter((v): v is number => typeof v === 'number' && Number.isFinite(v)).map(Math.abs);
  const max = finite.length ? Math.max(...finite) : 0;
  const unit = durationUnit(max, digits);
  return (ms: number, resolutionMs?: number | null): string => {
    if (typeof ms !== 'number' || !Number.isFinite(ms)) return '—';
    const res = resolutionMs ?? options.resolutionMs;
    if (typeof res === 'number' && res > 0 && Number.isFinite(res) && Math.abs(ms) < res) {
      return `<${renderIn(res, durationUnit(res, digits), digits)}`;
    }
    return renderIn(ms, unit, digits);
  };
}

/** A list joined the locale's way: `formatList(['a', 'b', 'c'])` is "a, b, and c" in `en`. */
export function formatList(items: readonly string[], options: Intl.ListFormatOptions = { type: 'conjunction' }): string {
  const locale = formatLocale();
  return cached('list', locale, options, () => new Intl.ListFormat(locale, options)).format(items);
}

/**
 * The name of a language from its BCP-47 code, in the active locale unless
 * `locale` is given: `formatLanguageName('de')` is "German" in `en` and
 * "Deutsch" in `de`. Returns the code itself if the runtime has no name for it.
 */
export function formatLanguageName(code: string, locale: string = formatLocale()): string {
  const names = cached('lang', locale, undefined, () => new Intl.DisplayNames(locale, { type: 'language' }));
  return names.of(code) ?? code;
}

/** The CLDR plural category of `n` in the active locale ('one', 'few', 'other', …). */
export function pluralCategory(n: number, options?: Intl.PluralRulesOptions): Intl.LDMLPluralRule {
  const locale = formatLocale();
  return cached('pl', locale, options, () => new Intl.PluralRules(locale, options)).select(n);
}

/** TEST ONLY: drop every cached formatter. */
export function _resetFormatCache(): void {
  cache.clear();
}
