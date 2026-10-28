import { onCleanup } from 'solid-js';

/** Pixels per line for `WheelEvent.DOM_DELTA_LINE` (Firefox's mouse wheel): Chromium's 100 px notch is 3 lines. */
const LINE_PX = 33;

/** The wheel's vertical travel in pixels, from pixel, line or page units. */
function verticalPixels(event: WheelEvent, element: HTMLElement): number {
  if (event.deltaMode === 1) return event.deltaY * LINE_PX;
  if (event.deltaMode === 2) return event.deltaY * element.clientWidth;
  return event.deltaY;
}

/**
 * Lets a vertical mouse wheel scroll a horizontal strip (a row of tabs) sideways.
 *
 * The wheel is taken only while the strip overflows and can still move in that direction, so the page
 * scrolls on as normal at either end and over a strip that fits. A horizontal gesture (trackpad deltaX),
 * Shift+wheel (the browser already scrolls sideways) and Ctrl+wheel (zoom) are left to the browser.
 * Returns the function that removes the listener.
 */
export function bindHorizontalWheel(element: HTMLElement): () => void {
  const onWheel = (event: WheelEvent): void => {
    if (event.defaultPrevented || event.shiftKey || event.ctrlKey || event.metaKey) return;
    if (event.deltaY === 0 || Math.abs(event.deltaX) >= Math.abs(event.deltaY)) return;
    const max = element.scrollWidth - element.clientWidth;
    if (max <= 0) return;
    // Right-to-left strips start at scrollLeft 0 and run negative toward the inline end.
    const rtl = getComputedStyle(element).direction === 'rtl';
    const position = rtl ? -element.scrollLeft : element.scrollLeft;
    const delta = verticalPixels(event, element);
    if (delta > 0 ? position >= max - 1 : position <= 0) return;
    event.preventDefault();
    const next = Math.min(max, Math.max(0, position + delta));
    element.scrollLeft = rtl ? -next || 0 : next;
  };
  element.addEventListener('wheel', onWheel, { passive: false });
  return () => element.removeEventListener('wheel', onWheel);
}

/** `ref` form of {@link bindHorizontalWheel}: the listener goes when the owning component is disposed. */
export function horizontalWheel(element: HTMLElement): void {
  onCleanup(bindHorizontalWheel(element));
}
