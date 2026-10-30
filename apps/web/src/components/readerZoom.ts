// Message-body zoom for the reading pane (t59).
//
// The level is a per-user server preference (`reader.zoom`, a whole percent the
// registry bounds to 50-200) written through the settings API, so it carries the
// storage authority like every other personal preference and applies to every
// message, in every reader: the mail pane, message tabs and detached windows.
// An administrator can set the default or lock it like any `reader.*` key.
//
// Only the message body scales. The level is applied from outside the body
// frame (ReaderZoomBox, t79): one custom property on the frame's box lays the
// frame out at 1/level and scales it back up, so text reflows and images scale
// as with a zoom on the document, while the document itself never changes. A
// step therefore does not reload the body (no blank frame, no image refetch) and
// its scroll position stays put. Until t79 the level was a rule in the frame
// document's own stylesheet, so every step rebuilt and reloaded it. The sandbox
// and the app CSP stay as they are. The app UI never zooms.
//
// Shortcuts (Ctrl/Cmd + `=` / `-` / `0`, configurable in Keyboard & display) are
// actions in the app's one dispatcher. They answer only while focus is inside
// this reader; anywhere else they decline, so the browser keeps its own page
// zoom in the message list, the composer and the rest of the app.
//
// Ctrl/Cmd + wheel over the body: the body is a script-free sandboxed document,
// and its wheel and key events never reach the app. While Ctrl or Cmd is held
// in the app document the frame stops taking pointer events (a CSS class, not
// an inline style), so the wheel lands on the reader, which zooms the body and
// keeps the browser from zooming the page. With focus inside the body itself
// the browser keeps its own shortcuts: nothing outside the sandbox sees them.

import { createSignal, onCleanup, onMount } from 'solid-js';
import { loadSettings, settings, settingsLoadStatus, SettingsApiError, writeSetting } from '../api/settings.ts';
import { onPageHide, pendingWrite } from '../api/unloadFlush.ts';
import { newIdempotencyKey } from '../api/recovery.ts';
import { observePreferenceAccount, preferenceAccount, preferenceAccountIsCurrent, type PreferenceAccount } from '../api/accountPreferenceLifecycle.ts';
import { useActions } from '../keymap/index.ts';
import * as css from './readerZoom.css.ts';

export const ZOOM_SETTING = 'reader.zoom';
export const ZOOM_STEPS = [50, 67, 75, 90, 100, 110, 125, 150, 175, 200] as const;
export const DEFAULT_ZOOM = 100;
export const MIN_ZOOM = ZOOM_STEPS[0];
export const MAX_ZOOM = ZOOM_STEPS[ZOOM_STEPS.length - 1]!;

/** The step nearest to `value`; anything that is not a finite number is 100%. */
export function snapZoom(value: unknown): number {
  const n = typeof value === 'number' ? value : typeof value === 'string' && value.trim() !== '' ? Number(value) : NaN;
  if (!Number.isFinite(n)) return DEFAULT_ZOOM;
  let best: number = ZOOM_STEPS[0];
  for (const step of ZOOM_STEPS) if (Math.abs(step - n) < Math.abs(best - n)) best = step;
  return best;
}

/** The next step above (`direction` 1) or below (-1) `level`, clamped at the ends. */
export function stepZoom(level: number, direction: 1 | -1): number {
  const current = snapZoom(level);
  const index = ZOOM_STEPS.indexOf(current as (typeof ZOOM_STEPS)[number]);
  return ZOOM_STEPS[Math.max(0, Math.min(ZOOM_STEPS.length - 1, index + direction))]!;
}

// The level the user just chose, shown at once while its write is in flight
// (also journaled under the authenticated storage authority until acknowledged).
// Signed-out choices remain session-only. `null` means "what the settings map says".
const [chosen, setChosen] = createSignal<number | null>(null);

/** The effective zoom level in percent (reactive). */
export function readerZoom(): number {
  return chosen() ?? snapZoom(settings()[ZOOM_SETTING]?.value ?? DEFAULT_ZOOM);
}

/** An administrator holds the level: the controls show it but cannot change it. */
export function readerZoomLocked(): boolean {
  return settings()[ZOOM_SETTING]?.locked === true;
}

const CHANNEL = 'mw-reader-zoom';
let channel: BroadcastChannel | null | undefined;
function peers(): BroadcastChannel | null {
  if (channel !== undefined) return channel;
  try {
    channel = typeof BroadcastChannel === 'function' ? new BroadcastChannel(CHANNEL) : null;
    // Another window of this app (a detached reader) stored a new level: read
    // it back from the server, so the account the window is signed in to decides.
    channel?.addEventListener('message', () => void loadSettings({ refresh: true }));
  } catch {
    channel = null;
  }
  return channel;
}

// The level on screen follows every step at once; the stored preference does
// not need to (t79). One write is in flight at a time and only the latest level
// is sent. During a burst (steps less than BURST_GAP ms apart: a Ctrl+wheel
// spin, a held shortcut) writes also start at least WRITE_SPACING ms apart; a
// step on its own is written at once. A step still waiting is sent when the
// page goes away.
const WRITE_SPACING = 400;
const BURST_GAP = 150;
let lastStep = 0;
let bursting = false;
let writing = false;
let queued: number | null = null;
let lastStart = 0;
let timer: ReturnType<typeof setTimeout> | undefined;
let unloadHooked = false;
let epoch = 0;
let intent: PendingZoom | null = null;
let intentPersisted = false;
interface PendingZoom { token: string; next: number; base: number; inflight: number | null }
const validStep = (value: unknown): value is number => typeof value === 'number' && (ZOOM_STEPS as readonly number[]).includes(value);
function parsePending(raw: unknown): PendingZoom | null {
  if (!raw || typeof raw !== 'object') return null;
  const p = raw as Partial<PendingZoom>;
  return typeof p.token === 'string' && p.token.length > 0 && p.token.length <= 100 && validStep(p.next) && validStep(p.base) && (p.inflight === null || validStep(p.inflight)) ? p as PendingZoom : null;
}
const pending = (owner: PreferenceAccount | null) => pendingWrite(owner?.storageAuthority ? `mw.reader-zoom:v1:${owner.storageAuthority}` : '', parsePending);
function persist(): void {
  const slot = pending(preferenceAccount());
  slot.write(intent);
  intentPersisted = intent !== null && slot.read()?.token === intent.token;
}
function forgetIntent(): void {
  const slot = pending(preferenceAccount());
  if (intent && slot.read()?.token === intent.token) slot.write(null);
  intent = null;
  intentPersisted = false;
}

const storedZoom = (): number => snapZoom(settings()[ZOOM_SETTING]?.value ?? DEFAULT_ZOOM);

function pump(): void {
  if (writing || queued === null) return;
  const saved = pending(preferenceAccount()).read();
  if (readerZoomLocked() || (intent && (saved ? saved.token !== intent.token : intentPersisted))) {
    queued = null;
    setChosen(null);
    forgetIntent();
    return;
  }
  const wait = bursting ? lastStart + WRITE_SPACING - Date.now() : 0;
  if (wait > 0) {
    timer ??= setTimeout(() => { timer = undefined; pump(); }, wait);
    return;
  }
  const next = queued;
  queued = null;
  if (next === storedZoom()) {
    if (chosen() === next) setChosen(null);
    forgetIntent();
    return;
  }
  const currentEpoch = epoch;
  const owner = preferenceAccount();
  const current = () => currentEpoch === epoch && (owner ? preferenceAccountIsCurrent(owner) : preferenceAccount() === null);
  const token = intent?.token;
  if (intent) { intent = { ...intent, inflight: next }; persist(); }
  writing = true;
  lastStart = Date.now();
  peers();
  writeSetting(ZOOM_SETTING, next).then(
    () => {
      if (!current()) return;
      if (intent && pending(owner).read()?.token === intent.token) {
        if (intent.token === token) forgetIntent();
        else { intent = { ...intent, base: next, inflight: null }; persist(); }
      }
      if (chosen() === next && queued === null) setChosen(null);
      try { channel?.postMessage({ type: 'zoom' }); } catch { /* no peers */ }
    },
    (e: unknown) => {
      if (!current()) return;
      // A lock that arrived since the map loaded wins; transient failures keep
      // the latest durable intent for the next authenticated settings load.
      if (e instanceof SettingsApiError && e.code === 'setting_locked') {
        queued = null;
        setChosen(null);
        forgetIntent();
      }
    },
  ).finally(() => { if (current()) { writing = false; pump(); } });
}

function store(next: number): void {
  intent = { token: newIdempotencyKey(), next, base: intent?.base ?? storedZoom(), inflight: intent?.inflight ?? null };
  persist(); // Save before starting transport: pagehide may never finish a request.
  queued = next;
  const now = Date.now();
  bursting = now - lastStep < BURST_GAP;
  lastStep = now;
  if (!bursting && timer !== undefined) { clearTimeout(timer); timer = undefined; }
  if (!unloadHooked) {
    unloadHooked = true;
    onPageHide(() => { clearTimeout(timer); timer = undefined; lastStart = 0; bursting = false; pump(); });
  }
  pump();
}

/** Set the level (snapped to a step) and store it as the user's preference. */
export function setReaderZoom(level: number): void {
  if (readerZoomLocked()) return;
  const next = snapZoom(level);
  if (next === readerZoom()) return;
  setChosen(next);
  store(next);
}

export const zoomIn = (): void => setReaderZoom(stepZoom(readerZoom(), 1));
export const zoomOut = (): void => setReaderZoom(stepZoom(readerZoom(), -1));
export const resetZoom = (): void => setReaderZoom(DEFAULT_ZOOM);

/** Forget the session-only level (sign-out, tests). */
export function clearReaderZoom(): void {
  epoch++;
  writing = false;
  intent = null;
  intentPersisted = false;
  setChosen(null);
  queued = null;
  clearTimeout(timer);
  timer = undefined;
  lastStart = 0;
  lastStep = 0;
  bursting = false;
}

observePreferenceAccount(owner => {
  clearReaderZoom();
  if (!owner?.storageAuthority || !pending(owner).read()) return;
  // Defer until the account transition has cleared the previous settings map.
  queueMicrotask(() => {
    if (!preferenceAccountIsCurrent(owner)) return;
    void loadSettings({ refresh: true }).then(() => {
      if (!preferenceAccountIsCurrent(owner) || settingsLoadStatus() !== 'ready' || intent) return;
      const slot = pending(owner);
      const saved = slot.read();
      if (!saved) return;
      const server = storedZoom();
      if (readerZoomLocked() || server === saved.next || (server !== saved.base && server !== saved.inflight)) {
        slot.write(null);
        return;
      }
      intent = saved;
      intentPersisted = true;
      setChosen(saved.next);
      queued = saved.next;
      pump();
    });
  });
});

/** Wheel distance (CSS px) for one zoom step; a mouse notch is ~100, a pinch less. */
const WHEEL_STEP = 50;

/** Wire the zoom shortcuts and Ctrl/Cmd + wheel to one reader. `root` is the
 *  reader element; `active` is false while the reader is a background tab. */
export function useReaderZoom(opts: { root: () => HTMLElement | undefined; active: () => boolean }): void {
  const inReader = (e: KeyboardEvent): boolean => {
    const root = opts.root();
    if (!root || !opts.active()) return false;
    const target = e.target instanceof Node ? e.target : null;
    const focused = root.ownerDocument.activeElement;
    return (target !== null && root.contains(target)) || (focused !== null && root.contains(focused));
  };
  const run = (action: () => void) => (e: KeyboardEvent): boolean => {
    if (!inReader(e)) return false;
    if (!e.repeat || action !== resetZoom) action();
    return true;
  };
  useActions({ zoomIn: run(zoomIn), zoomOut: run(zoomOut), zoomReset: run(resetZoom) });

  onMount(() => {
    const root = opts.root();
    if (!root) return;
    const doc = root.ownerDocument;
    const win = doc.defaultView ?? window;
    let held = false;
    const hold = (on: boolean): void => {
      const next = on && opts.active();
      if (next === held) return;
      held = next;
      root.classList.toggle(css.gesture, next);
    };
    const keys = (e: KeyboardEvent): void => {
      if (e.key === 'Control' || e.key === 'Meta') hold(e.type === 'keydown');
      else hold(e.ctrlKey || e.metaKey);
    };
    const release = (): void => hold(false);
    // A keyup can be missed (the key was released in another window): any
    // pointer movement in the app document says whether it is still held.
    const pointer = (e: PointerEvent | MouseEvent): void => { if (held && !e.ctrlKey && !e.metaKey) hold(false); };

    let distance = 0;
    let idle: ReturnType<typeof setTimeout> | undefined;
    const overBody = (e: WheelEvent): boolean => {
      for (const frame of Array.from(root.querySelectorAll<HTMLIFrameElement>('iframe.reader__frame'))) {
        const r = frame.getBoundingClientRect();
        if (e.clientX >= r.left && e.clientX <= r.right && e.clientY >= r.top && e.clientY <= r.bottom) return true;
      }
      return false;
    };
    const wheel = (e: WheelEvent): void => {
      if (!(e.ctrlKey || e.metaKey) || !opts.active() || !overBody(e)) return;
      e.preventDefault();
      const scale = e.deltaMode === 1 ? 16 : e.deltaMode === 2 ? 400 : 1;
      distance += e.deltaY * scale;
      clearTimeout(idle);
      idle = setTimeout(() => { distance = 0; }, 400);
      if (Math.abs(distance) < WHEEL_STEP) return;
      if (distance < 0) zoomIn(); else zoomOut();
      distance = 0;
    };

    doc.addEventListener('keydown', keys, true);
    doc.addEventListener('keyup', keys, true);
    doc.addEventListener('pointermove', pointer, true);
    win.addEventListener('blur', release);
    doc.addEventListener('visibilitychange', release);
    root.addEventListener('wheel', wheel, { passive: false });
    onCleanup(() => {
      clearTimeout(idle);
      hold(false);
      doc.removeEventListener('keydown', keys, true);
      doc.removeEventListener('keyup', keys, true);
      doc.removeEventListener('pointermove', pointer, true);
      win.removeEventListener('blur', release);
      doc.removeEventListener('visibilitychange', release);
      root.removeEventListener('wheel', wheel);
    });
  });
}
