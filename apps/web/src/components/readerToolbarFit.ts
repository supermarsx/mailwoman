// Which reader actions fit on the one action row (t57). Pure, so the overflow
// rule is tested without a layout engine; ReaderToolbar.tsx feeds it measured
// widths.
//
// Every action shows inline when the whole row fits. Otherwise the "More"
// button takes its place at the end and actions are kept in priority order
// (lower number = more important) until the next one would not fit; that one
// and every less important action fold into the menu. The rule stops at the
// first action that does not fit rather than skipping ahead to a smaller one,
// so a less important action is never shown while a more important one is
// folded.

export interface FitItem {
  id: string;
  /** Lower is more important. Ties keep their row order. */
  priority: number;
  /** Measured border-box width in CSS pixels. */
  width: number;
}

/** The ids to fold into "More". `available` is the row's content width, `gap`
 *  the space between two items, `more` the width of the More button. */
export function foldActions(items: readonly FitItem[], available: number, gap: number, more: number): Set<string> {
  const total = items.reduce((sum, item) => sum + item.width, 0) + gap * Math.max(0, items.length - 1);
  if (total <= available + 0.5) return new Set();
  const ranked = items.map((item, index) => ({ item, index })).sort((a, b) => a.item.priority - b.item.priority || a.index - b.index);
  const folded = new Set<string>();
  let used = more;
  let stopped = false;
  for (const { item } of ranked) {
    if (!stopped && used + gap + item.width <= available + 0.5) {
      used += gap + item.width;
      continue;
    }
    stopped = true;
    folded.add(item.id);
  }
  return folded;
}
