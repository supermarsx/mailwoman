// Pure layout helpers for the calendar time grid (Outlook-style rendering):
// which instances belong in the all-day row, side-by-side columns for
// concurrent timed events, and a readable ink colour on a calendar's colour.

import type { EventInstance } from './types.ts';

const DAY_MS = 24 * 60 * 60 * 1000;

/** All-day events and timed events lasting a full day or more go in the
 *  all-day row; only shorter timed events are placed in the hourly grid. */
export function inAllDayRow(inst: Pick<EventInstance, 'allDay' | 'start' | 'end'>): boolean {
  return inst.allDay || inst.end.getTime() - inst.start.getTime() >= DAY_MS;
}

export interface ColumnSlot {
  /** Zero-based column within the overlap cluster. */
  col: number;
  /** Number of columns in the cluster this item belongs to. */
  cols: number;
}

/**
 * Assign each interval a column so concurrent items sit side by side. Items are
 * grouped into clusters of transitively overlapping intervals; within a cluster
 * each item takes the first column whose previous item has ended, and every item
 * of the cluster shares the cluster's column count. Returned slots are in the
 * input order. Intervals are half-open: one ending at 10:00 does not overlap one
 * starting at 10:00.
 */
export function layoutColumns(items: ReadonlyArray<{ start: number; end: number }>): ColumnSlot[] {
  const order = items
    .map((it, i) => ({ i, start: it.start, end: Math.max(it.end, it.start + 1) }))
    .sort((a, b) => a.start - b.start || b.end - a.end);
  const out: ColumnSlot[] = items.map(() => ({ col: 0, cols: 1 }));
  let cluster: number[] = [];
  let colEnds: number[] = [];
  let clusterEnd = -Infinity;
  const flush = (): void => {
    for (const i of cluster) out[i]!.cols = colEnds.length;
    cluster = [];
    colEnds = [];
  };
  for (const it of order) {
    if (it.start >= clusterEnd) flush();
    let col = colEnds.findIndex((end) => end <= it.start);
    if (col < 0) {
      col = colEnds.length;
      colEnds.push(it.end);
    } else {
      colEnds[col] = it.end;
    }
    out[it.i]!.col = col;
    cluster.push(it.i);
    clusterEnd = Math.max(clusterEnd, it.end);
  }
  flush();
  return out;
}

/** Dark or light ink for text on a calendar colour (WCAG relative luminance).
 *  Non-hex colours keep the light ink the chips used before. */
export function inkOn(color: string): string {
  const m = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(color.trim());
  if (m === null) return '#fff';
  let hex = m[1]!;
  if (hex.length === 3) hex = [...hex].map((ch) => ch + ch).join('');
  const lin = (v: number): number => {
    const s = v / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  const r = lin(parseInt(hex.slice(0, 2), 16));
  const g = lin(parseInt(hex.slice(2, 4), 16));
  const b = lin(parseInt(hex.slice(4, 6), 16));
  const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  // Contrast against white = 1.05/(L+0.05); against #111 = (L+0.05)/0.0556.
  return 1.05 / (lum + 0.05) >= (lum + 0.05) / 0.0556 ? '#fff' : '#111';
}

/** One event's bar across a run of consecutive day columns (t92). */
export interface SpanBar<T> {
  item: T;
  /** First column the bar covers (0 = the row's first day). */
  startCol: number;
  /** Number of columns the bar covers (>= 1). */
  span: number;
  /** Stacking lane within the row (0 = top). */
  lane: number;
  /** The event started before the row's first day (the bar is cut there). */
  continuesBefore: boolean;
  /** The event ends after the row's last day. */
  continuesAfter: boolean;
}

export interface RowSpans<T> {
  /** Bars that fit in `maxLanes`, in lane then column order. */
  bars: SpanBar<T>[];
  /** Lanes the visible bars use (0..maxLanes). */
  lanes: number;
  /** Per column: how many events of this row are not shown there ("+N more"). */
  hidden: number[];
}

/**
 * Lay out events as bars over one row of consecutive days (a week row of the
 * month grid, or the all-day row of the time grid). Each event covers the
 * columns of the days it touches (end exclusive: an all-day event ending at
 * midnight does not cover the next day), cut at the row's edges, so an event
 * running across a week boundary becomes one bar per week with the
 * `continues*` flags set. Lanes are greedy: earlier start first, longer bar
 * first, so a long event keeps one lane across the row. A bar that needs a lane
 * beyond `maxLanes` is hidden, and every column it covers counts it in `hidden`.
 * Column order is logical (column 0 is the first day); right-to-left placement is
 * the stylesheet's (logical insets), so the same layout serves both directions.
 */
export function layoutRowSpans<T extends { start: Date; end: Date }>(
  items: readonly T[],
  days: readonly Date[],
  maxLanes: number,
): RowSpans<T> {
  const n = days.length;
  const hidden = Array.from({ length: n }, () => 0);
  if (n === 0) return { bars: [], lanes: 0, hidden };
  const dayStart = days.map((d) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime());
  const rowEnd = new Date(days[n - 1]!.getFullYear(), days[n - 1]!.getMonth(), days[n - 1]!.getDate() + 1).getTime();
  const colOf = (t: number): number => {
    let col = 0;
    for (let i = 0; i < n; i++) if (dayStart[i]! <= t) col = i;
    return col;
  };
  const placed = items.flatMap((item) => {
    const s = item.start.getTime();
    // A zero-length event still covers the day it is on.
    const e = Math.max(item.end.getTime(), s + 1);
    if (e <= dayStart[0]! || s >= rowEnd) return [];
    const startCol = s < dayStart[0]! ? 0 : colOf(s);
    const endCol = e > rowEnd ? n - 1 : colOf(e - 1);
    return [{ item, s, startCol, span: endCol - startCol + 1, continuesBefore: s < dayStart[0]!, continuesAfter: e > rowEnd }];
  });
  placed.sort((a, b) => a.startCol - b.startCol || b.span - a.span || a.s - b.s);
  const laneEnds: number[] = [];
  const bars: SpanBar<T>[] = [];
  for (const p of placed) {
    let lane = laneEnds.findIndex((end) => end < p.startCol);
    if (lane < 0) lane = laneEnds.length;
    laneEnds[lane] = p.startCol + p.span - 1;
    if (lane >= maxLanes) {
      for (let c = p.startCol; c < p.startCol + p.span; c++) hidden[c]! += 1;
      continue;
    }
    bars.push({ item: p.item, startCol: p.startCol, span: p.span, lane, continuesBefore: p.continuesBefore, continuesAfter: p.continuesAfter });
  }
  bars.sort((a, b) => a.lane - b.lane || a.startCol - b.startCol);
  const lanes = bars.reduce((m, b) => Math.max(m, b.lane + 1), 0);
  return { bars, lanes, hidden };
}

/** True when an instance covers more than one calendar day (end exclusive). */
export function spansDays(inst: Pick<EventInstance, 'start' | 'end'>): boolean {
  const s = inst.start;
  const last = new Date(Math.max(inst.end.getTime() - 1, s.getTime()));
  return s.getFullYear() !== last.getFullYear() || s.getMonth() !== last.getMonth() || s.getDate() !== last.getDate();
}
