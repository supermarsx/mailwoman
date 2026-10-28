// Pointer-driven reordering for short vertical lists (t40 f): the mailbox
// picker and Settings › Linked mailboxes. One primitive for mouse, pen and
// touch: a mouse drag starts after a few pixels of movement, a touch drag after
// a long press (or at once from the grip, which has `touch-action: none`).
// While dragging, the row lifts and follows the pointer, a drop line marks the
// target slot and the scroll container scrolls near its edges. A press that
// never moved stays a click; one that dragged swallows the click that follows,
// so dropping a mailbox never switches to it.
//
// Like the workspace tab drag (`realtime/workspaceTabDrag.ts`), a drag belongs
// to the account that started it: when the signed-in owner changes mid-drag the
// drop is ignored.
import { createSignal, onCleanup, type Accessor } from 'solid-js';
import { preferenceAccount, preferenceAccountIsCurrent } from '../../api/accountPreferenceLifecycle.ts';

export interface ReorderState {
  from: number;
  /** Insertion slot 0..n (before item `over`; n = after the last). */
  over: number;
}

export interface PointerReorder {
  dragging: Accessor<ReorderState | null>;
  /** Attach to a row's (or grip's) `onPointerDown`. */
  start(event: PointerEvent, index: number, fromGrip: boolean): void;
  /** True once if the last press dragged: call from the row's click handler. */
  consumeClick(): boolean;
}

const MOUSE_SLOP = 5;
const TOUCH_SLOP = 8;
const LONG_PRESS_MS = 350;
const EDGE = 32;

/** Move `list[from]` into insertion slot `slot` (0..n). */
export function moveTo<T>(list: readonly T[], from: number, slot: number): T[] {
  const next = list.slice();
  const [item] = next.splice(from, 1);
  if (item === undefined) return list.slice();
  next.splice(slot > from ? slot - 1 : slot, 0, item);
  return next;
}

export function createPointerReorder(options: {
  /** The rows, in order. */
  rows: () => HTMLElement[];
  /** The element that scrolls (the menu or the page section). */
  scroller?: () => HTMLElement | null | undefined;
  onDrop: (from: number, slot: number) => void;
}): PointerReorder {
  const [dragging, setDragging] = createSignal<ReorderState | null>(null);
  let pending: { id: number; x: number; y: number; index: number; touch: boolean; grip: boolean; timer?: ReturnType<typeof setTimeout> } | null = null;
  let row: HTMLElement | undefined;
  let originY = 0;
  let lastY = 0;
  let swallowClick = false;
  let frame = 0;
  let owner = preferenceAccount();
  const doc = (): Document => row?.ownerDocument ?? document;

  function slotAt(y: number): number {
    const rows = options.rows();
    for (let i = 0; i < rows.length; i++) {
      const rect = rows[i]!.getBoundingClientRect();
      if (y < rect.top + rect.height / 2) return i;
    }
    return rows.length;
  }
  function autoscroll(): void {
    frame = 0;
    const scroller = options.scroller?.();
    if (!dragging() || !scroller) return;
    const rect = scroller.getBoundingClientRect();
    const delta = lastY < rect.top + EDGE ? -8 : lastY > rect.bottom - EDGE ? 8 : 0;
    if (delta !== 0) {
      scroller.scrollTop += delta;
      update(lastY);
      frame = requestAnimationFrame(autoscroll);
    }
  }
  function update(y: number): void {
    const state = dragging();
    if (!state) return;
    lastY = y;
    row?.style.setProperty('transform', `translateY(${y - originY}px)`);
    const over = slotAt(y);
    if (over !== state.over) setDragging({ ...state, over });
    if (!frame) frame = requestAnimationFrame(autoscroll);
  }
  function begin(index: number, y: number): void {
    const rows = options.rows();
    row = rows[index];
    if (!row) return;
    owner = preferenceAccount();
    originY = y;
    lastY = y;
    row.setAttribute('data-dragging', 'true');
    setDragging({ from: index, over: index });
    doc().addEventListener('touchmove', blockScroll, { passive: false });
  }
  function finish(commit: boolean): void {
    const state = dragging();
    if (pending?.timer) clearTimeout(pending.timer);
    pending = null;
    if (frame) cancelAnimationFrame(frame);
    frame = 0;
    row?.style.removeProperty('transform');
    row?.removeAttribute('data-dragging');
    doc().removeEventListener('touchmove', blockScroll);
    row = undefined;
    setDragging(null);
    if (!state) return;
    swallowClick = true;
    setTimeout(() => (swallowClick = false), 0);
    if (commit && (owner === null ? preferenceAccount() === null : preferenceAccountIsCurrent(owner)) && state.over !== state.from && state.over !== state.from + 1) {
      options.onDrop(state.from, state.over);
    }
  }
  const blockScroll = (event: TouchEvent): void => {
    if (dragging()) event.preventDefault();
  };

  function onMove(event: PointerEvent): void {
    if (pending && event.pointerId === pending.id && !dragging()) {
      const moved = Math.hypot(event.clientX - pending.x, event.clientY - pending.y);
      if (pending.touch && !pending.grip) {
        // Moving before the long press means the finger is scrolling.
        if (moved > TOUCH_SLOP) cancel();
        return;
      }
      if (moved > (pending.touch ? TOUCH_SLOP : MOUSE_SLOP)) begin(pending.index, pending.y);
    }
    if (dragging()) {
      event.preventDefault();
      update(event.clientY);
    }
  }
  function onUp(event: PointerEvent): void {
    if (pending && event.pointerId !== pending.id) return;
    detach();
    finish(event.type === 'pointerup');
  }
  function cancel(): void {
    detach();
    if (pending?.timer) clearTimeout(pending.timer);
    pending = null;
  }
  function detach(): void {
    const d = doc();
    d.removeEventListener('pointermove', onMove);
    d.removeEventListener('pointerup', onUp);
    d.removeEventListener('pointercancel', onUp);
  }
  onCleanup(() => {
    detach();
    finish(false);
  });

  return {
    dragging,
    start(event, index, fromGrip) {
      if (event.button !== 0 || dragging()) return;
      const touch = event.pointerType === 'touch' || event.pointerType === 'pen';
      pending = { id: event.pointerId, x: event.clientX, y: event.clientY, index, touch, grip: fromGrip };
      if (fromGrip) event.preventDefault();
      if (touch && !fromGrip) {
        pending.timer = setTimeout(() => {
          if (pending) begin(pending.index, pending.y);
        }, LONG_PRESS_MS);
      }
      const d = (event.currentTarget as HTMLElement | null)?.ownerDocument ?? document;
      d.addEventListener('pointermove', onMove);
      d.addEventListener('pointerup', onUp);
      d.addEventListener('pointercancel', onUp);
    },
    consumeClick() {
      if (!swallowClick) return false;
      swallowClick = false;
      return true;
    },
  };
}
