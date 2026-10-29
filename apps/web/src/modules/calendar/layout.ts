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
