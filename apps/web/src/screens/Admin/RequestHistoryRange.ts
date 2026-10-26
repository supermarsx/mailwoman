/** Unix seconds throughout; custom inputs deliberately use the administrator's local time. */
export interface HistoryRange { from: number; to: number }
export const HISTORY_PRESETS = [3600, 21600, 86400, 604800, 2592000] as const;

export function relativeHistoryRange(seconds: number, now = Date.now()): HistoryRange {
  const to = Math.floor(now / 1000);
  return { from: to - seconds, to };
}

export function historyLocalInput(seconds: number): string {
  const date = new Date(seconds * 1000);
  const pad = (n: number): string => String(n).padStart(2, '0');
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export type HistoryRangeError = 'invalid' | 'order' | 'future' | 'limit';
export function customHistoryRange(from: string, to: string, maxSeconds: number, now = Date.now()):
  { range: HistoryRange; error?: never } | { error: HistoryRangeError; range?: never } {
  const parse = (value: string): number => {
    if (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/.test(value)) return NaN;
    const seconds = new Date(value).getTime() / 1000;
    // Reject normalized impossible dates (including missing local DST times).
    return Number.isFinite(seconds) && historyLocalInput(seconds) === value ? seconds : NaN;
  };
  const range = { from: parse(from), to: parse(to) };
  if (!Number.isFinite(range.from) || !Number.isFinite(range.to)) return { error: 'invalid' };
  if (range.from >= range.to) return { error: 'order' };
  if (range.to > Math.floor(now / 1000)) return { error: 'future' };
  if (range.to - range.from > maxSeconds) return { error: 'limit' };
  return { range };
}
