/**
 * Touch and pen drag for lists, shared by the folder tree and the Pinned group.
 *
 * HTML5 drag and drop does not fire for a finger, so a touch drag is built from
 * pointer events: hold a row for `LONG_PRESS_MS` without moving and it lifts (a
 * visual cue only, no vibration); from then on the finger drags it and the page
 * does not scroll. Moving before the hold elapses is a scroll and cancels the
 * press. The mouse keeps using native drag and drop, so neither path can double
 * fire. The model that decides what a drop means stays with the caller.
 */
export const LONG_PRESS_MS = 350;
export const SLOP_PX = 8;

export interface TouchDragOptions {
  /** The hold elapsed on this row. Return false to refuse (a locked folder). */
  begin: (id: string) => boolean;
  /** The finger moved while dragging; `under` is the element at that point. */
  move: (x: number, y: number, under: Element | null) => void;
  /** The finger lifted over `under`. */
  finish: (x: number, y: number, under: Element | null) => void;
  /** The system cancelled the gesture, or Escape was pressed. */
  cancel: () => void;
  /** Text shown beside the finger. */
  label?: (id: string) => string;
  delay?: number;
}

export function createTouchDrag(options: TouchDragOptions) {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let pending: { id: string; x: number; y: number; pointer: number; el: HTMLElement } | null = null;
  let active: { id: string; el: HTMLElement; pointer: number; doc: Document } | null = null;
  let ghost: HTMLElement | undefined;

  const stopTouchScroll = (event: Event): void => { if (event.cancelable) event.preventDefault(); };
  const swallow = (event: Event): void => { event.preventDefault(); event.stopPropagation(); };
  const escape = (event: KeyboardEvent): void => { if (event.key === 'Escape') { options.cancel(); clean(); } };

  function clean(): void {
    if (timer) clearTimeout(timer);
    timer = undefined;
    pending = null;
    if (active) {
      const { doc, el, pointer } = active;
      doc.removeEventListener('touchmove', stopTouchScroll);
      doc.removeEventListener('keydown', escape, true);
      try { el.releasePointerCapture(pointer); } catch { /* already released */ }
      // The browser's own long-press menu can arrive just after the drop.
      setTimeout(() => { doc.removeEventListener('contextmenu', swallow, true); }, 600);
    }
    ghost?.remove();
    ghost = undefined;
    active = null;
  }

  function lift(): void {
    const p = pending;
    if (!p) return;
    timer = undefined;
    if (!options.begin(p.id)) { pending = null; return; }
    const doc = p.el.ownerDocument;
    active = { id: p.id, el: p.el, pointer: p.pointer, doc };
    pending = null;
    try { p.el.setPointerCapture(p.pointer); } catch { /* the pointer is gone */ }
    // A scroll that has not started yet can still be refused.
    doc.addEventListener('touchmove', stopTouchScroll, { passive: false });
    doc.addEventListener('contextmenu', swallow, true);
    doc.addEventListener('keydown', escape, true);
    const text = options.label?.(p.id);
    if (text) {
      ghost = doc.createElement('div');
      ghost.className = 'folder-touch-ghost';
      ghost.setAttribute('aria-hidden', 'true');
      ghost.textContent = text;
      ghost.style.left = `${p.x}px`;
      ghost.style.top = `${p.y}px`;
      doc.body.appendChild(ghost);
    }
  }

  function props(id: string) {
    return {
      onPointerDown: (event: PointerEvent): void => {
        if (event.pointerType === 'mouse' || !event.isPrimary || event.button > 0) return;
        clean();
        pending = { id, x: event.clientX, y: event.clientY, pointer: event.pointerId, el: event.currentTarget as HTMLElement };
        timer = setTimeout(lift, options.delay ?? LONG_PRESS_MS);
      },
      onPointerMove: (event: PointerEvent): void => {
        if (pending) {
          if (Math.hypot(event.clientX - pending.x, event.clientY - pending.y) > SLOP_PX) clean();
          return;
        }
        if (!active || event.pointerId !== active.pointer) return;
        if (ghost) { ghost.style.left = `${event.clientX}px`; ghost.style.top = `${event.clientY}px`; }
        const under = active.doc.elementFromPoint(event.clientX, event.clientY);
        options.move(event.clientX, event.clientY, under);
      },
      onPointerUp: (event: PointerEvent): void => {
        if (!active) { clean(); return; }
        const doc = active.doc;
        const under = doc.elementFromPoint(event.clientX, event.clientY);
        clean();
        options.finish(event.clientX, event.clientY, under);
      },
      onPointerCancel: (): void => {
        if (active) options.cancel();
        clean();
      },
    };
  }
  return {
    props,
    dispose: clean,
  };
}
export type TouchDrag = ReturnType<typeof createTouchDrag>;
