// Dashboard number and time formatting, in the active app locale through the
// shared `i18n/format.ts`. The server sends Unix seconds; `i18n` takes ms.

import { formatBytes as bytes, formatDateTime as dateTime, formatNumber, formatTime as time, t } from '../../../i18n';

export const formatCount = (n: number): string => formatNumber(n);

/** Bytes in 1024 steps with the locale's unit names ("1.5 GB"). */
export const formatBytes = (n: number): string => bytes(n);

export function formatMs(n: number): string {
  return t('admin-dash-ms', { ms: formatNumber(n, { maximumFractionDigits: n < 10 ? 1 : 0 }) });
}

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
