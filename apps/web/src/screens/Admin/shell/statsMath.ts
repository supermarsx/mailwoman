// Pure helpers behind the statistics tiles and charts (t106). No DOM, no i18n
// catalogue: callers format the numbers these return.

/** Sum of the measured values; `null` when nothing in the series was measured (never zero). */
export function sumKnown(values: readonly (number | null | undefined)[]): number | null {
  let total: number | null = null;
  for (const v of values) if (typeof v === 'number' && Number.isFinite(v)) total = (total ?? 0) + v;
  return total;
}

/** Largest measured value; `null` when nothing was measured. */
export function maxKnown(values: readonly (number | null | undefined)[]): number | null {
  let max: number | null = null;
  for (const v of values) if (typeof v === 'number' && Number.isFinite(v) && (max === null || v > max)) max = v;
  return max;
}

export type DeltaDirection = 'up' | 'down' | 'same';
/**
 * The change from `previous` to `current`. `ratio` is the relative change
 * (0.12 = 12 % up), or `null` when the previous value is 0 and a ratio would be
 * infinite; then `absolute` carries the change.
 */
export interface Delta { direction: DeltaDirection; ratio: number | null; absolute: number }

export function deltaOf(current: number | null | undefined, previous: number | null | undefined): Delta | null {
  if (typeof current !== 'number' || typeof previous !== 'number' || !Number.isFinite(current) || !Number.isFinite(previous)) return null;
  const absolute = current - previous;
  const direction: DeltaDirection = absolute > 0 ? 'up' : absolute < 0 ? 'down' : 'same';
  const ratio = previous === 0 ? (absolute === 0 ? 0 : null) : absolute / Math.abs(previous);
  return { direction, ratio, absolute };
}

/** Whether a change reads as good, bad or neutral, given whether "up" is good for this figure. */
export function deltaTone(delta: Delta, upIsGood: boolean | undefined): 'good' | 'bad' | 'flat' {
  if (delta.direction === 'same' || upIsGood === undefined) return 'flat';
  return (delta.direction === 'up') === upIsGood ? 'good' : 'bad';
}

/**
 * The window just before `range`, of the same length, for "vs previous period".
 * Unix seconds.
 */
export function previousWindow(range: { from: number; to: number }): { from: number; to: number } {
  const span = Math.max(0, range.to - range.from);
  return { from: range.from - span, to: range.from };
}

/** A previous window counts for a comparison only when the server observed most of it. */
export function comparable(coverage: { observedSeconds: number; requestedSeconds: number } | undefined): boolean {
  return !!coverage && coverage.requestedSeconds > 0 && coverage.observedSeconds >= coverage.requestedSeconds * 0.95;
}

/** A "nice" axis maximum at or above `max` (1, 2, 2.5, 5 × 10ⁿ), so tick labels read as round numbers. */
export function niceMax(max: number): number {
  if (!Number.isFinite(max) || max <= 0) return 1;
  const exp = Math.floor(Math.log10(max));
  const base = 10 ** exp;
  for (const step of [1, 2, 2.5, 5, 10]) if (max <= step * base + 1e-9 * base) return step * base;
  return 10 * base;
}
