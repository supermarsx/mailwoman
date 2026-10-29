// Dashboard number and time formatting, in the active app locale through the
// shared `i18n/format.ts`. The server sends Unix seconds; `i18n` takes ms.

import { formatBytes as bytes, formatDateTime as dateTime, durationFormatterFor, formatDurationMs, formatNumber, formatTime as time, t } from '../../../i18n';

export const formatCount = (n: number): string => formatNumber(n);

/** Bytes in 1024 steps with the locale's unit names ("1.5 GB"). */
export const formatBytes = (n: number): string => bytes(n);

/**
 * A measured time in ms, in the unit that keeps it readable ("420 μs",
 * "1.25 ms", "2.4 s"); `resolutionMs` is the source's step, below which the
 * value reads "<1 μs" or "<1 ms".
 */
export const formatMs = (n: number, resolutionMs?: number | null): string => formatDurationMs(n, { resolutionMs });

/**
 * A time the server sends as whole ms (`…Ms`, truncated) and, from 26.20 on,
 * also as exact μs (`…Us`). The μs field wins; a bare whole-ms 0 reads "<1 ms"
 * rather than "0 ms", since the server truncated a sub-ms measurement.
 */
export function formatMeasured(ms: number | null | undefined, us?: number | null): string {
  if (typeof us === 'number' && Number.isFinite(us)) return formatDurationMs(us / 1000, { resolutionMs: 0.001 });
  if (typeof ms !== 'number') return '—';
  return formatDurationMs(ms, { resolutionMs: 1 });
}

/** One ms formatter for a chart series: every value reads in the unit of the largest. */
export const formatMsSet = (values: readonly (number | null)[]): ((n: number, resolutionMs?: number | null) => string) =>
  durationFormatterFor(values);

/** "3 d 4 h", "4 h 12 min", "12 min", "40 s". */
export function formatDuration(secs: number): string {
  const d = Math.floor(secs / 86400);
  const h = Math.floor((secs % 86400) / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (d > 0) return t('admin-dash-duration-dh', { d, h });
  if (h > 0) return t('admin-dash-duration-hm', { h, m });
  if (m > 0) return t('admin-dash-duration-m', { m });
  return t('admin-dash-duration-s', { s: Math.max(0, Math.floor(secs)) });
}

export const formatTime = (unixSecs: number): string => time(unixSecs * 1000);

export const formatDateTime = (unixSecs: number): string => dateTime(unixSecs * 1000);
