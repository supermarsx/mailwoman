// Fixed-row-height windowing math for the virtualized message list (§1.5, §23:
// a 100k-row list must stay at 60 fps by only mounting the visible slice). Pure
// and DOM-free so it is directly unit-testable; `MessageList` feeds it the live
// scrollTop/viewport height and renders `[startIndex, endIndex)` at `offsetY`
// inside a `totalHeight`-tall spacer.

export interface Window {
  /** First row index to mount (inclusive). */
  startIndex: number;
  /** One past the last row index to mount (exclusive). */
  endIndex: number;
  /** Translate offset (px) of the mounted slice from the top of the scroller. */
  offsetY: number;
  /** Full scroll height (px) so the scrollbar reflects the whole list. */
  totalHeight: number;
}

/**
 * Compute which rows to mount for a fixed-height virtual list.
 * `overscan` rows are mounted above and below the viewport so a fast flick does
 * not flash blank rows.
 */
export function computeWindow(
  scrollTop: number,
  viewportHeight: number,
  rowHeight: number,
  count: number,
  overscan = 6,
): Window {
  const totalHeight = count * rowHeight;
  if (count === 0 || rowHeight <= 0 || viewportHeight <= 0) {
    return { startIndex: 0, endIndex: 0, offsetY: 0, totalHeight };
  }
  const clampedTop = Math.max(0, Math.min(scrollTop, Math.max(0, totalHeight - viewportHeight)));
  const first = Math.floor(clampedTop / rowHeight);
  const visibleCount = Math.ceil(viewportHeight / rowHeight);
  const startIndex = Math.max(0, first - overscan);
  const endIndex = Math.min(count, first + visibleCount + overscan);
  return { startIndex, endIndex, offsetY: startIndex * rowHeight, totalHeight };
}
