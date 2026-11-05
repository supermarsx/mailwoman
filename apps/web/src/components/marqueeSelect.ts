// Marquee (rubber-band) selection controller, framework-free so any list can
// reuse it (the folder tree now, the message list later).
//
// Press on empty space and drag: a translucent band is drawn and every item
// whose box it touches is selected live. Plain drag replaces the selection,
// Ctrl/Cmd adds, Shift extends to the covering range. Escape (or a cancelled
// pointer) restores the selection from before the drag; a click on empty space
// without dragging clears it. The scroll area scrolls while the pointer is near
// its top or bottom edge. Touch is left alone entirely so the pane scrolls as
// usual. Hit-testing uses element geometry in scroll-content coordinates, so
// hidden (display:none) items drop out and long or nested lists work the same.
import { autoScrollSpeed, boxBetween, clampBox, intersects, modeFor, passedThreshold, resolveSelection, type Box, type MarqueeMode, type Point } from './marqueeGeometry.ts';
import * as css from './marqueeSelect.css.ts';

export interface MarqueeItem { id: string; el: HTMLElement }
export interface MarqueeOptions {
  /** Receives pointerdown; empty space inside it starts a drag. */
  container: HTMLElement;
  /** The element that scrolls (default: the container). */
  scroller?: HTMLElement;
  /** The selectable items now, in display order. Items with no box (hidden) are skipped. */
  items: () => readonly MarqueeItem[];
  /** The current selection, read at drag start. */
  selection: () => ReadonlySet<string>;
  /** A new selection while dragging, and on cancel (restoring the old one). */
  onChange: (ids: ReadonlySet<string>) => void;
  /** A click on empty space without a drag (default: clear the selection). */
  onEmptyClick?: (event: PointerEvent) => void;
  /** Pointerdowns on these never start a marquee. Items themselves are always excluded. */
  ignore?: string;
  /** Pixels the pointer must travel before a drag begins. */
  threshold?: number;
  edge?: number;
  maxSpeed?: number;
}

const DEFAULT_IGNORE = 'button, a, input, select, textarea, label, summary, [role="button"], [role="menuitem"], [contenteditable], [draggable="true"], [data-marquee-ignore]';

export function attachMarquee(options: MarqueeOptions): () => void {
  const { container, scroller = container } = options;
  const threshold = options.threshold ?? 4;
  const ignore = options.ignore ?? DEFAULT_IGNORE;
  let pointerId = -1;
  let start: Point | null = null;      // in scroll-content coordinates
  let startClient: Point | null = null;
  let last: Point = { x: 0, y: 0 };    // pointer, client coordinates
  let mode: MarqueeMode = 'replace';
  let base: ReadonlySet<string> = new Set();
  let dragging = false;
  let frame = 0;
  let bandEl: HTMLElement | null = null;
  let modifiers = { ctrlKey: false, metaKey: false, shiftKey: false };

  const origin = (): Point => {
    const box = scroller.getBoundingClientRect();
    return { x: box.left + scroller.clientLeft - scroller.scrollLeft, y: box.top + scroller.clientTop - scroller.scrollTop };
  };
  const toContent = (point: Point): Point => { const o = origin(); return { x: point.x - o.x, y: point.y - o.y }; };
  const viewport = (): Box => {
    const box = scroller.getBoundingClientRect();
    return { left: box.left + scroller.clientLeft, top: box.top + scroller.clientTop, right: box.left + scroller.clientLeft + scroller.clientWidth, bottom: box.top + scroller.clientTop + scroller.clientHeight };
  };

  /** True when the press landed on the scroller's own scrollbar gutter (either side, so RTL works). */
  function onScrollbar(event: PointerEvent): boolean {
    if (event.target !== scroller) return false;
    const view = viewport();
    return event.clientX < view.left || event.clientX > view.right || event.clientY < view.top || event.clientY > view.bottom;
  }

  const prevent = (event: Event): void => event.preventDefault();

  function begin(event: PointerEvent): void {
    if (event.button !== 0 || event.pointerType === 'touch' || dragging || start) return;
    const target = event.target as Element | null;
    if (!target || !container.contains(target)) return;
    if (target.closest(ignore) || options.items().some(item => item.el.contains(target))) return;
    if (onScrollbar(event)) return;
    pointerId = event.pointerId;
    startClient = { x: event.clientX, y: event.clientY };
    start = toContent(startClient);
    last = { ...startClient };
    modifiers = { ctrlKey: event.ctrlKey, metaKey: event.metaKey, shiftKey: event.shiftKey };
    mode = modeFor(event);
    base = new Set(options.selection());
    document.addEventListener('pointermove', move);
    document.addEventListener('pointerup', up);
    document.addEventListener('pointercancel', cancelPointer);
    document.addEventListener('keydown', key, true);
  }

  function activate(): void {
    dragging = true;
    bandEl = document.createElement('div');
    bandEl.className = css.band;
    bandEl.setAttribute('aria-hidden', 'true');
    bandEl.setAttribute('data-marquee-band', '');
    document.body.appendChild(bandEl);
    container.classList.add(css.dragging);
    try { container.setPointerCapture?.(pointerId); } catch { /* the pointer may already be gone */ }
    container.addEventListener('selectstart', prevent);
    container.addEventListener('dragstart', prevent);
  }

  function move(event: PointerEvent): void {
    if (event.pointerId !== pointerId || !startClient) return;
    last = { x: event.clientX, y: event.clientY };
    if (!dragging) {
      if (!passedThreshold(startClient, last, threshold)) return;
      activate();
    }
    schedule();
  }

  function schedule(): void {
    if (frame) return;
    frame = requestAnimationFrame(tick);
  }

  function tick(): void {
    frame = 0;
    if (!dragging || !start) return;
    const view = viewport();
    const speed = autoScrollSpeed(last.y, view.top, view.bottom, options.edge, options.maxSpeed);
    if (speed !== 0) {
      const before = scroller.scrollTop;
      scroller.scrollTop = before + speed;
      if (scroller.scrollTop !== before) schedule();
    }
    update();
  }

  function update(): void {
    if (!start) return;
    const o = origin();
    const content = boxBetween(start, toContent(last));
    const hit = new Set<string>(), order: string[] = [];
    for (const item of options.items()) {
      const r = item.el.getBoundingClientRect();
      if (r.width === 0 && r.height === 0) continue; // hidden or collapsed: not selectable
      order.push(item.id);
      const box: Box = { left: r.left - o.x, top: r.top - o.y, right: r.right - o.x, bottom: r.bottom - o.y };
      if (intersects(content, box)) hit.add(item.id);
    }
    options.onChange(resolveSelection(mode, order, base, hit));
    if (bandEl) {
      const shown = clampBox({ left: content.left + o.x, top: content.top + o.y, right: content.right + o.x, bottom: content.bottom + o.y }, viewport());
      bandEl.hidden = !shown;
      if (shown) Object.assign(bandEl.style, { left: `${shown.left}px`, top: `${shown.top}px`, width: `${shown.right - shown.left}px`, height: `${shown.bottom - shown.top}px` });
    }
  }

  function finish(): void {
    document.removeEventListener('pointermove', move);
    document.removeEventListener('pointerup', up);
    document.removeEventListener('pointercancel', cancelPointer);
    document.removeEventListener('keydown', key, true);
    if (frame) { cancelAnimationFrame(frame); frame = 0; }
    if (dragging) {
      container.classList.remove(css.dragging);
      container.removeEventListener('selectstart', prevent);
      container.removeEventListener('dragstart', prevent);
      try { container.releasePointerCapture?.(pointerId); } catch { /* already released */ }
    }
    bandEl?.remove(); bandEl = null;
    dragging = false; start = null; startClient = null; pointerId = -1;
  }

  function up(event: PointerEvent): void {
    if (event.pointerId !== pointerId) return;
    const wasDrag = dragging;
    if (wasDrag) { last = { x: event.clientX, y: event.clientY }; update(); }
    finish();
    // A plain click on empty space clears; with a modifier it leaves the selection alone.
    if (!wasDrag && !modifiers.ctrlKey && !modifiers.metaKey && !modifiers.shiftKey) (options.onEmptyClick ?? (() => options.onChange(new Set())))(event);
  }

  function cancelPointer(event: PointerEvent): void {
    if (event.pointerId !== pointerId) return;
    cancel();
  }

  function cancel(): void {
    const restore = dragging;
    finish();
    if (restore) options.onChange(new Set(base));
  }

  function key(event: KeyboardEvent): void {
    if (event.key !== 'Escape') return;
    event.preventDefault(); event.stopPropagation();
    cancel();
  }

  container.addEventListener('pointerdown', begin);
  return () => {
    container.removeEventListener('pointerdown', begin);
    if (start) cancel();
  };
}
