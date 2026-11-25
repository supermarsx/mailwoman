import {alertValue} from '../../api/notificationControls.ts';
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
// At most TOAST_LIMIT ordinary toasts are shown; additional actions wait
// in a bounded queue and receive their full display time when promoted.
// Showing a toast with an `id` that is on screen replaces it in place and
// restarts its clock. Repeated warnings/errors coalesce; completed actions
// without an explicit id are independent, even when their wording matches.
//
// Module state, so a slice can raise a toast without a component in scope;
// timers are plain `setTimeout`s, which fake timers drive in tests.

import { createSignal, untrack, type Accessor } from 'solid-js';

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
  /** Apply notification preferences even when this alert has an Open action. */
  respectPreferences?: boolean;
  /** Showing the same id again replaces the toast on screen instead of adding one. */
  id?: string;
  /** Reserve the single background-job slot; ordinary notifications cannot evict it. */
  persistent?: boolean;
  action?: ToastAction;
  /** A second button beside `action` ("Send now" next to "Undo"). */
  secondaryAction?: ToastAction;
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
  /** Work in progress (t111): a spinner takes the tone icon's place. */
  busy?: boolean;
}

export interface ToastItem {
  readonly id: string;
  readonly persistent: boolean;
  readonly tone: ToastTone;
  readonly message: string;
  readonly action: ToastAction | undefined;
  readonly secondaryAction: ToastAction | undefined;
  readonly duration: number;
  readonly countdown: boolean;
  readonly pausable: boolean;
  readonly testId: string | undefined;
  readonly busy: boolean;
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
export type PauseReason = 'hover' | 'focus' | 'hidden' | 'gesture';

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
const [regions, setRegions] = createSignal(0);
let seq = 0;
const idPrefix=Date.now().toString(36)+Math.random().toString(36).slice(2,8);
let waiting:ToastItem[]=[];
const MAX_WAITING=200;
/** Includes queued notifications so Dismiss all describes the entire stack. */
export function toastCount():number{return items().length+waiting.length;}
export type ToastEvent={kind:'show';item:ToastItem;options:ToastOptions}|{kind:'dismiss';id:string;reason:ToastCloseReason}|{kind:'clear'};
const observers=new Set<(event:ToastEvent)=>void>();
export function observeToastEvents(listener:(event:ToastEvent)=>void):()=>void{observers.add(listener);return()=>observers.delete(listener);}
const emit=(event:ToastEvent)=>{for(const listener of observers)listener(event);};


function defaultDuration(tone: ToastTone, opts: ToastOptions): number {
  if (opts.duration !== undefined) return opts.duration;
  return opts.action !== undefined || opts.persistent || opts.busy ? TOAST_DURATIONS.action : TOAST_DURATIONS[tone]*alertValue('notifications.toasts.durationSeconds',5)/5;
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
  // Raising a notification must not subscribe its caller to the toast list.
  // Resource-error effects would otherwise rerun after their own notification.
  if(!opts.persistent&&!opts.busy&&(opts.respectPreferences||!opts.action&&!opts.secondaryAction&&!opts.countdown)&&(!alertValue('notifications.toasts.enabled',true)||!alertValue(`notifications.toasts.${tone}`,true)))return opts.id??`muted:${++seq}`;
  const list = [...untrack(items),...waiting];
  const id =
    opts.id ?? ((tone==='warning'||tone==='error')?list.find((i) => i.id.startsWith('auto:') && i.tone === tone && i.message === message)?.id:undefined) ?? `auto:${idPrefix}:${++seq}`;
  const item: ToastItem = {
    id,
    persistent: opts.persistent === true,
    tone,
    message,
    action: opts.action,
    secondaryAction: opts.secondaryAction,
    duration: defaultDuration(tone, opts),
    countdown: opts.countdown === true,
    pausable: opts.pausable !== false,
    testId: opts.testId,
    busy: opts.busy === true,
  };
  const old = clocks.get(id);
  if (old !== undefined) clearTimeout(old.handle);
  clocks.set(id, { left: item.duration, startedAt: null, handle: undefined, onClose: opts.onClose });

  const next=list.some(entry=>entry.id===id)?list.map(entry=>entry.id===id?item:entry):[...list,item];
  arrange(next);
  emit({kind:'show',item,options:opts});
  return id;
}

/** Preserve every action until it has had a chance to be read. Timed actions
 * with an external deadline continue expiring even while queued. */
function arrange(all:readonly ToastItem[]):void {
  const visible:ToastItem[]=[],pending:ToastItem[]=[];
  const limit=Math.max(1,Math.min(8,alertValue('notifications.toasts.limit',TOAST_LIMIT)));
  let ordinary=0,jobs=0;
  for(const item of all){
    if(item.persistent?jobs<1:ordinary<limit){visible.push(item);if(item.persistent)jobs++;else ordinary++;}
    else pending.push(item);
  }
  waiting=pending.slice(0,MAX_WAITING);
  setItems(visible);
  for(const item of waiting){if(item.pausable)stopClock(item.id);else startClock(item.id);}
  for(const item of visible)if(!item.pausable||pauses.size===0)startClock(item.id);
  for(const item of pending.slice(MAX_WAITING))close(item.id,'evicted');
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
  const all=[...untrack(items),...waiting];
  if (!all.some((i) => i.id === id)) return;
  arrange(all.filter((i)=>i.id!==id));
  close(id, reason);
  emit({kind:'dismiss',id,reason});
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
    const ids = [...items(),...waiting].map((i) => i.id);
    waiting=[];setItems([]);
    for (const id of ids) close(id, 'cleared');
    emit({kind:'clear'});
  },
};

/** The toasts on screen, oldest first. */
export const toasts: Accessor<readonly ToastItem[]> = items;
/** Bounded shared snapshot includes queued actions as well as visible alerts. */
export const allToasts=():readonly ToastItem[]=>[...items(),...waiting];

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

/** Whether a ToastRegion is mounted, i.e. a toast raised now is seen (reactive). */
export function toastRegionMounted(): boolean {
  return regions() > 0;
}

/** ToastRegion's mount bookkeeping; returns the unmount half. */
export function registerToastRegion(): () => void {
  setRegions((n) => n + 1);
  return () => setRegions((n) => n - 1);
}

// Floating bars the toasts must not cover (t67): the bulk selection bar (t58)
// floats over the bottom of a list, where a 1440 px folder tab puts its right
// end under the bottom inline-end toast column. While one is registered, the
// region lifts its column above any of them it would overlap.
const [obstacles, setObstacles] = createSignal<readonly HTMLElement[]>([]);
/** Elements the toast column stays clear of (reactive). */
export const toastObstacles: Accessor<readonly HTMLElement[]> = obstacles;
/** Keep the toasts clear of `element` while it is shown; returns the unregister. */
export function registerToastObstacle(element: HTMLElement): () => void {
  setObstacles((list) => [...list, element]);
  return () => setObstacles((list) => list.filter((item) => item !== element));
}

/** Test reset: no toasts, no pause reasons. */
export function resetToasts(): void {
  toast.clear();
  pauses.clear();
  setPaused(false);
}
