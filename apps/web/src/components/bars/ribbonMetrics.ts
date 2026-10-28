// Ribbon row metrics in px (t38 §2.2). Kept out of ribbonGroup.css.ts, which
// may export only class names and plain values.

/** Row height on a fine pointer and on a coarse pointer (the 44 px touch rule), and the row gap. */
export const RIBBON_ROW_FINE = 28;
export const RIBBON_ROW_COARSE = 44;
export const RIBBON_GAP = 2;
export const RIBBON_CAPTION = 16;
export const RIBBON_PAD = 2;
export const RIBBON_SCROLL_RESERVE = 10;

/** The fixed height of a group's controls area: 88 px fine, 136 px coarse. */
export function ribbonControlsHeight(row: number = RIBBON_ROW_FINE, gap: number = RIBBON_GAP): number {
  return 3 * row + 2 * gap;
}
