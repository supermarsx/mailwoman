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

/**
 * How many row slots the list spans, given what is loaded and how big the query
 * actually is.
 *
 * ⚠ The two inputs are in DIFFERENT UNITS and that is the whole subtlety.
 * `loadedRows` counts VISUAL rows — a collapsed conversation of nine messages is
 * one row. `queryTotal` counts MESSAGES, because that is what the server counted;
 * it cannot know how the client will group them. So the loaded region is measured
 * in visual rows and the unloaded tail in messages, and `loadedThrough` (the query
 * index the loaded page reaches) is what joins them.
 *
 * The tail is therefore an UPPER BOUND: every unloaded message gets a slot, and
 * some of those messages will fold into an existing conversation when they arrive,
 * so the count can shrink as pages load. That is honest — the alternative is
 * describing a 20 000-message folder as 50 rows, which is what made `aria-setsize`
 * announce "1 of 50" and gave a 20 000-message folder a 3 600px scrollbar.
 *
 * `queryTotal === null` means the server did not (or could not) count. Then the
 * only defensible answer is what is loaded, which is exactly the pre-paging
 * behaviour — never a guess.
 */
export function projectedRowCount(
  loadedRows: number,
  queryTotal: number | null,
  loadedThrough: number,
): number {
  if (queryTotal === null) return loadedRows;
  return loadedRows + Math.max(0, queryTotal - loadedThrough);
}

/** A window split into the slots that have rows and the slots that do not yet. */
export interface WindowSplit {
  /** `[loadedStart, loadedEnd)` — indices backed by a real row. */
  loadedStart: number;
  loadedEnd: number;
  /** `[pendingStart, pendingEnd)` — indices inside the query but not yet fetched. */
  pendingStart: number;
  pendingEnd: number;
}

/**
 * Divide a window at `loadedCount`, the number of slots that actually have rows.
 *
 * Both halves are returned rather than just the boundary so the caller cannot get
 * the arithmetic subtly wrong in one of them — an off-by-one in the pending half
 * renders a placeholder ON TOP of a real row at the same offset, which looks like
 * a flicker rather than like a bug.
 */
export function splitWindow(win: Window, loadedCount: number): WindowSplit {
  const bound = Math.max(0, loadedCount);
  return {
    loadedStart: Math.min(win.startIndex, bound),
    loadedEnd: Math.min(win.endIndex, bound),
    pendingStart: Math.max(win.startIndex, bound),
    pendingEnd: Math.max(win.endIndex, bound),
  };
}

/**
 * Value equality for two windows.
 *
 * `computeWindow` returns a fresh object on every call, so a memo over it
 * notifies on every scroll *event* — including the many events that leave the
 * mounted slice exactly where it was (a 5px scroll inside one row, or the tail
 * of a momentum flick). Used as the memo's `equals` comparator, this confines
 * downstream work to the events that actually move the window.
 */
export function sameWindow(a: Window, b: Window): boolean {
  return (
    a.startIndex === b.startIndex &&
    a.endIndex === b.endIndex &&
    a.offsetY === b.offsetY &&
    a.totalHeight === b.totalHeight
  );
}
