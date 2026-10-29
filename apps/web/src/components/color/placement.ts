// Where an anchored popover goes (t81): below its trigger when it fits, above
// when it fits there instead, otherwise on the side with more room; shifted
// along the edge so it stays inside the viewport, and clamped into it when it
// is taller than the room on either side (it then scrolls inside). Pure maths,
// so `placement.test.ts` can check every edge; the picker feeds it
// getBoundingClientRect() and the visual viewport.

export interface Box { left: number; top: number; width: number; height: number }
export interface Viewport { left: number; top: number; width: number; height: number }

export interface PlacementOptions {
  /** Space between trigger and popover, px. */
  gap?: number;
  /** Space kept to the viewport edge, px. */
  margin?: number;
  /** Right-to-left: the popover lines up with the trigger's end (right) edge. */
  rtl?: boolean;
}

export interface Placement {
  left: number;
  top: number;
  side: 'below' | 'above';
  /** The most the popover may be tall: its side's room, or the viewport less its margins. */
  maxHeight: number;
}

/** The least room (px) a popover scrolls in before it is laid over its trigger instead. */
export const MIN_ROOM = 160;

export function placePopover(anchor: Box, size: { width: number; height: number }, view: Viewport, options: PlacementOptions = {}): Placement {
  const gap = options.gap ?? 4;
  const margin = options.margin ?? 8;
  const minTop = view.top + margin;
  const maxBottom = view.top + view.height - margin;
  const anchorBottom = anchor.top + anchor.height;
  const below = maxBottom - (anchorBottom + gap);
  const above = anchor.top - gap - minTop;
  const side: Placement['side'] =
    size.height <= below ? 'below' : size.height <= above ? 'above' : below >= above ? 'below' : 'above';
  const room = side === 'below' ? below : above;
  // Too tall for its side: it scrolls inside the room there, as long as that
  // room is usable; otherwise it is clamped into the viewport over the trigger.
  const maxHeight = Math.round(size.height > room && room >= MIN_ROOM ? room : Math.max(0, view.height - 2 * margin));
  const height = Math.min(size.height, maxHeight);
  const wanted = side === 'below' ? anchorBottom + gap : anchor.top - gap - height;
  const top = Math.max(minTop, Math.min(wanted, maxBottom - height));
  const minLeft = view.left + margin;
  const maxLeft = view.left + view.width - margin - size.width;
  const preferred = options.rtl ? anchor.left + anchor.width - size.width : anchor.left;
  const left = maxLeft < minLeft ? minLeft : Math.max(minLeft, Math.min(preferred, maxLeft));
  return { left: Math.round(left), top: Math.round(top), side, maxHeight };
}
