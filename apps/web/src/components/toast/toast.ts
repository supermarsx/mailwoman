// The one toast store for the webmail and the admin console (t32 f39, user
// request 2026-10-25: "saved alerts and all other messages should be popped up
// in a neat toast message that auto dismisses after a while but is also user
// dismissable").
//
// Every transient message goes through `toast.success|info|warning|error`:
// "Saved", "Deleted", "Sent", copy done, background failures, Undo/Retry
// offers. `<ToastRegion>` (mounted once per app shell) renders the list.
// Not toasts: field validation errors (they stay at the field), blocking error
// states and persistent banners (maintenance, offline, locked).
//
// Timing (coordinator defaults): success/info 5 s, warning 8 s, error 10 s,
// any toast with an action 10 s. The clock of every toast stops while the
// region is hovered or holds focus and while the window is hidden, and starts
// again with the time that was left. A toast created with `pausable: false`
// keeps running through all of that: its deadline is set elsewhere (the
// server's undo-send hold ends whether or not the pointer rests on it).
//
// At most TOAST_LIMIT toasts are shown; a new one pushes the oldest out.
// Showing a toast with an `id` that is on screen replaces it in place and
// restarts its clock; without an `id`, the same tone and text count as the
// same toast, so a failure repeated five times shows once.
//
// Module state, so a slice can raise a toast without a component in scope;
// timers are plain `setTimeout`s, which fake timers drive in tests.

import { createSignal, type Accessor } from 'solid-js';

export type ToastTone = 'success' | 'info' | 'warning' | 'error';

export interface ToastAction {
  /** The button text ("Undo", "Retry", "Reload"). */
  label: string;
  /** Runs on click; the toast closes first. */
  run: () => void | Promise<void>;
}

/** Why a toast left the screen. */
export type ToastCloseReason = 'timeout' | 'dismiss' | 'action' | 'evicted' | 'cleared';

export interface ToastOptions {
  /** Showing the same id again replaces the toast on screen instead of adding one. */
  id?: string;
  action?: ToastAction;
  /** Milliseconds on screen. Defaults by tone (TOAST_DURATIONS); `Infinity` stays until closed. */
  duration?: number;
  /** Show the seconds left beside the message (undo windows). */
  countdown?: boolean;
  /** `false`: hover, focus and a hidden window do not stop the clock. Default `true`. */
  pausable?: boolean;
  /** Called once when the toast leaves the screen (not when it is replaced by the same id). */
  onClose?: (reason: ToastCloseReason) => void;
  /** `data-testid` of the toast element (default `toast`). */
  testId?: string;
}

export interface ToastItem {
  readonly id: string;
  readonly tone: ToastTone;
  readonly message: string;
  readonly action: ToastAction | undefined;
  readonly duration: number;
  readonly countdown: boolean;
  readonly pausable: boolean;
  readonly testId: string | undefined;
}

export const TOAST_DURATIONS = {
  success: 5_000,
  info: 5_000,
  warning: 8_000,
  error: 10_000,
  /** Any toast with an action button. */
  action: 10_000,
} as const;

/** The most toasts on screen at once. */
export const TOAST_LIMIT = 3;

/** What stops the clocks: pointer over the region, focus inside it, a hidden window. */
export type PauseReason = 'hover' | 'focus' | 'hidden';

interface Clock {
  /** Milliseconds left when the clock last stopped (or was set). */
  left: number;
  /** `Date.now()` when the clock last started; null while stopped. */
  startedAt: number | null;
  handle: ReturnType<typeof setTimeout> | undefined;
  onClose: ToastOptions['onClose'];
}

const [items, setItems] = createSignal<readonly ToastItem[]>([]);
const clocks = new Map<string, Clock>();
const pauses = new Set<PauseReason>();
const [paused, setPaused] = createSignal(false);
let seq = 0;

function defaultDuration(tone: ToastTone, opts: ToastOptions): number {
  if (opts.duration !== undefined) return opts.duration;
  return opts.action !== undefined ? TOAST_DURATIONS.action : TOAST_DURATIONS[tone];
}

function stopClock(id: string): void {
  const c = clocks.get(id);
  if (c === undefined || c.startedAt === null) return;
  clearTimeout(c.handle);
  c.left = Math.max(0, c.left - (Date.now() - c.startedAt));
  c.startedAt = null;
  c.handle = undefined;
}

function startClock(id: string): void {
  const c = clocks.get(id);
  if (c === undefined || c.startedAt !== null || !Number.isFinite(c.left)) return;
  c.startedAt = Date.now();
  c.handle = setTimeout(() => dismiss(id, 'timeout'), c.left);
}

function isPausable(id: string): boolean {
  return items().find((i) => i.id === id)?.pausable !== false;
}

function show(tone: ToastTone, message: string, opts: ToastOptions = {}): string {
  const list = items();
  const id =
    opts.id ?? list.find((i) => i.id.startsWith('auto:') && i.tone === tone && i.message === message)?.id ?? `auto:${++seq}`;
  const item: ToastItem = {
    id,
    tone,
    message,
    action: opts.action,
    duration: defaultDuration(tone, opts),
    countdown: opts.countdown === true,
    pausable: opts.pausable !== false,
    testId: opts.testId,
  };
  const old = clocks.get(id);
  if (old !== undefined) clearTimeout(old.handle);
  clocks.set(id, { left: item.duration, startedAt: null, handle: undefined, onClose: opts.onClose });

  let evicted: ToastItem[] = [];
  if (list.some((i) => i.id === id)) {
    setItems(list.map((i) => (i.id === id ? item : i)));
  } else {
    const next = [...list, item];
    evicted = next.slice(0, Math.max(0, next.length - TOAST_LIMIT));
    setItems(next.slice(evicted.length));
  }
  if (!item.pausable || pauses.size === 0) startClock(id);
  for (const e of evicted) close(e.id, 'evicted');
  return id;
}

function close(id: string, reason: ToastCloseReason): void {
  const c = clocks.get(id);
  if (c === undefined) return;
  clearTimeout(c.handle);
  clocks.delete(id);
  c.onClose?.(reason);
}

/** Take a toast off the screen. Unknown ids are ignored. */
function dismiss(id: string, reason: ToastCloseReason = 'dismiss'): void {
  if (!items().some((i) => i.id === id)) return;
  setItems(items().filter((i) => i.id !== id));
  close(id, reason);
}

/** The shared toast API. Each call returns the toast's id. */
export const toast = {
  success: (message: string, opts?: ToastOptions): string => show('success', message, opts),
  info: (message: string, opts?: ToastOptions): string => show('info', message, opts),
  warning: (message: string, opts?: ToastOptions): string => show('warning', message, opts),
  error: (message: string, opts?: ToastOptions): string => show('error', message, opts),
  /** Any tone by name (callers that pick the tone at run time). */
  show,
  dismiss,
  /** Close the newest toast (Escape). Returns whether one was closed. */
  dismissNewest(): boolean {
    const last = items().at(-1);
    if (last === undefined) return false;
    dismiss(last.id);
    return true;
  },
  /** Close every toast (sign-out, tests). */
  clear(): void {
    const ids = items().map((i) => i.id);
    setItems([]);
    for (const id of ids) close(id, 'cleared');
  },
};

/** The toasts on screen, oldest first. */
export const toasts: Accessor<readonly ToastItem[]> = items;

/** Stop (`on`) or restart (`!on`) the clocks for one reason; they run while no reason holds. */
export function setToastPause(reason: PauseReason, on: boolean): void {
  const was = pauses.size > 0;
  if (on) pauses.add(reason);
  else pauses.delete(reason);
  const now = pauses.size > 0;
  if (was === now) return;
  setPaused(now);
  for (const { id } of items()) {
    if (!isPausable(id)) continue;
    if (now) stopClock(id);
    else startClock(id);
  }
}

/** Whether any reason holds the clocks (reactive). */
export const toastsPaused: Accessor<boolean> = paused;

/** Milliseconds before the toast closes on its own (`Infinity` for one that stays). */
export function toastTimeLeft(id: string): number {
  const c = clocks.get(id);
  if (c === undefined) return 0;
  if (c.startedAt === null) return c.left;
  return Math.max(0, c.left - (Date.now() - c.startedAt));
}

/** Test reset: no toasts, no pause reasons. */
export function resetToasts(): void {
  toast.clear();
  pauses.clear();
  setPaused(false);
}
