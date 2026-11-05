// Pure geometry for marquee (rubber-band) selection. No DOM access: the
// controller in marqueeSelect.ts feeds it numbers.
export interface Point {x: number; y: number}
export interface Box {left: number; top: number; right: number; bottom: number}

/** The box spanned by two corner points, whichever way the pointer dragged. */
export function boxBetween(a: Point, b: Point): Box {
  return {left: Math.min(a.x, b.x), top: Math.min(a.y, b.y), right: Math.max(a.x, b.x), bottom: Math.max(a.y, b.y)};
}

/** True when the boxes share area or touch an edge; empty (zero-size) boxes never intersect. */
export function intersects(a: Box, b: Box): boolean {
  if (a.right <= a.left && a.bottom <= a.top) return false;
  if (b.right <= b.left || b.bottom <= b.top) return false;
  return a.left <= b.right && a.right >= b.left && a.top <= b.bottom && a.bottom >= b.top;
}

/** Clamp `box` to `bounds`; null when nothing is left. */
export function clampBox(box: Box, bounds: Box): Box | null {
  const left = Math.max(box.left, bounds.left), top = Math.max(box.top, bounds.top);
  const right = Math.min(box.right, bounds.right), bottom = Math.min(box.bottom, bounds.bottom);
  return right < left || bottom < top ? null : {left, top, right, bottom};
}

/** Whether the pointer moved far enough from where it went down to count as a drag. */
export function passedThreshold(from: Point, to: Point, threshold: number): boolean {
  return Math.hypot(to.x - from.x, to.y - from.y) >= threshold;
}

/**
 * Auto-scroll speed in px per frame for a pointer at `pos` within a scroll
 * viewport spanning [start, end]. Zero in the middle; inside `edge` px of
 * either end the speed grows quadratically to `max`, and keeps growing (up to
 * 2x) once the pointer is dragged past the edge. Negative scrolls towards start.
 */
export function autoScrollSpeed(pos: number, start: number, end: number, edge = 40, max = 18): number {
  const zone = Math.min(edge, Math.max(0, (end - start) / 2));
  if (zone <= 0) return 0;
  const curve = (depth: number) => {
    const r = Math.min(depth / zone, 2);
    return depth <= 0 ? 0 : Math.max(1, Math.round(max * (r <= 1 ? r * r : r)));
  };
  if (pos < start + zone) return -curve(start + zone - pos);
  if (pos > end - zone) return curve(pos - (end - zone));
  return 0;
}

/**
 * Shift-drag: grow `base` plus `hit` into the contiguous run of `order` that
 * covers both, as file managers extend a range.
 */
export function extendSpan(order: readonly string[], base: ReadonlySet<string>, hit: ReadonlySet<string>): Set<string> {
  let first = -1, last = -1;
  order.forEach((id, index) => {
    if (base.has(id) || hit.has(id)) { if (first < 0) first = index; last = index; }
  });
  const out = new Set<string>();
  // Ids outside `order` (rows scrolled away or hidden) stay as they were.
  for (const id of base) if (!order.includes(id)) out.add(id);
  if (first >= 0) for (let i = first; i <= last; i++) out.add(order[i]!);
  return out;
}

export type MarqueeMode = 'replace' | 'add' | 'extend';
/** Plain drag replaces, Ctrl/Cmd adds, Shift extends (Shift wins over Ctrl). */
export function modeFor(event: {ctrlKey: boolean; metaKey: boolean; shiftKey: boolean}): MarqueeMode {
  return event.shiftKey ? 'extend' : event.ctrlKey || event.metaKey ? 'add' : 'replace';
}

/** The selection a drag in `mode` produces from the selection at drag start (`base`) and the rows under the band (`hit`). */
export function resolveSelection(mode: MarqueeMode, order: readonly string[], base: ReadonlySet<string>, hit: ReadonlySet<string>): Set<string> {
  if (mode === 'replace') return new Set(hit);
  if (mode === 'add') return new Set([...base, ...hit]);
  return extendSpan(order, base, hit);
}
