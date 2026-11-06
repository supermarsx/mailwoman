// The admin session bar's clock (no markup here).
//
// The server decides when an admin session ends (`crates/mw-server/src/admin_accounts.rs`,
// `crates/mw-store/src/admin_accounts.rs`): it ends when the idle deadline passes
// (every admin request moves it to now + the idle timeout, never past the cap) or
// when the absolute deadline passes (set at sign-in, never moved). This file only
// COUNTS DOWN to those two instants, as the server reports them:
//
//   * nothing here knows 30 minutes or 12 hours: the limits arrive in the answer;
//   * every figure is wall-clock arithmetic (`Date.now() + offset`), so a laptop
//     that slept, or a tab the browser throttled, shows the right time on waking;
//   * `offset` is the server's clock minus this one's at the moment the answer
//     arrived (`serverNow`), so a wrong local clock does not move the deadline;
//   * the countdown does not extend anything: it reads `GET /admin/session/status`
//     with `x-mw-session-peek: 1`, which leaves the deadline alone (without the
//     header every admin request, this one included, moves the idle deadline). Only `extend()` (`GET /admin/session`, an
//     ordinary admin request) moves the idle deadline, and the server still clips
//     that at the absolute cap, which nothing can move.

import { createEffect, createMemo, createSignal, on, onCleanup, type Accessor } from 'solid-js';
import { apiFetch } from '../../../api/http.ts';
import { adminUrl } from '../../../api/adminPath.ts';
import { basePath } from '../../../api/basePath.ts';
import { onAdminActivity } from '../../../api/adminSession.ts';

/** What the server reports about the session (all unix SECONDS). */
export interface SessionTiming {
  idleTimeoutSeconds: number;
  absoluteTimeoutSeconds: number;
  signedInAt: number;
  idleExpiresAt: number;
  absoluteExpiresAt: number;
  expiresAt: number;
  serverNow: number;
}

export type SessionLevel = 'normal' | 'warning' | 'critical' | 'expired';
export type SessionBarMode = 'always' | 'warn-only' | 'hidden';

/** Below this many seconds the bar turns to its warning state. */
export const WARN_SECONDS = 300;
/** Below this many seconds it is critical. */
export const CRITICAL_SECONDS = 60;
/** How long the "expired" message stays before the console returns to sign-in. */
export const EXPIRED_GRACE_MS = 4000;

export const SESSION_BAR_MODES: readonly SessionBarMode[] = ['always', 'warn-only', 'hidden'];
export function parseSessionBarMode(value: unknown): SessionBarMode {
  return SESSION_BAR_MODES.includes(value as SessionBarMode) ? (value as SessionBarMode) : 'always';
}

/** Read a server answer; anything not shaped like timing is `null`. */
export function parseTiming(value: unknown): SessionTiming | null {
  if (typeof value !== 'object' || value === null) return null;
  const o = value as Record<string, unknown>;
  const keys = ['idleTimeoutSeconds', 'absoluteTimeoutSeconds', 'signedInAt', 'idleExpiresAt', 'absoluteExpiresAt', 'expiresAt', 'serverNow'] as const;
  const out: Record<string, number> = {};
  for (const k of keys) {
    const n = o[k];
    if (typeof n !== 'number' || !Number.isFinite(n)) return null;
    out[k] = n;
  }
  return out as unknown as SessionTiming;
}

export function levelFor(seconds: number): SessionLevel {
  if (seconds <= 0) return 'expired';
  if (seconds < CRITICAL_SECONDS) return 'critical';
  if (seconds < WARN_SECONDS) return 'warning';
  return 'normal';
}

const two = (n: number): string => String(n).padStart(2, '0');

/** `28:41`, or `1:05:09` from an hour up. */
export function formatClock(total: number): string {
  const s = Math.max(0, Math.floor(total));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  return h > 0 ? `${h}:${two(m)}:${two(s % 60)}` : `${m}:${two(s % 60)}`;
}

/** Hours and minutes (an hour or more), for a limit that is hours away. */
export function splitLong(total: number): { hours: number; minutes: number } | null {
  const s = Math.max(0, Math.floor(total));
  return s >= 3600 ? { hours: Math.floor(s / 3600), minutes: Math.floor((s % 3600) / 60) } : null;
}

// ── The server client ──────────────────────────────────────────────────────────

export interface SessionBarApi {
  /** `GET /admin/session/status`: the timing, read without refreshing; `null` when the session has ended. */
  status(): Promise<SessionTiming | null>;
  /** `GET /admin/session`: an ordinary admin request, so it moves the idle deadline; `null` when ended. */
  extend(): Promise<SessionTiming | null>;
}

export function createHttpSessionBarApi(base = basePath()): SessionBarApi {
  async function read(path: string, peek: boolean): Promise<SessionTiming | null> {
    // The peek header asks the server to look at the session without moving its idle deadline.
    const res = await apiFetch(adminUrl(path, base), { credentials: 'same-origin', cache: 'no-store', ...(peek ? { headers: { 'x-mw-session-peek': '1' } } : {}) });
    if (res.status === 401) return null;
    if (!res.ok) throw new Error(`GET ${path} failed (${res.status})`);
    const timing = parseTiming(await res.json());
    if (!timing) throw new Error(`GET ${path}: no session timing`);
    return timing;
  }
  return { status: () => read('/session/status', true), extend: () => read('/session', false) };
}

// ── The controller ─────────────────────────────────────────────────────────────

export interface SessionBarOptions {
  api: SessionBarApi;
  /** Called when the session is over (after the grace period, or on the button). */
  onExpired: () => void;
  /** Whether to run at all (the `hidden` preference stops it). */
  enabled?: Accessor<boolean>;
  /** Wall clock in ms; tests pass their own. */
  now?: () => number;
  /** Time in the page (`document.hidden`); tests may override. */
  isHidden?: () => boolean;
}

export type SessionAnnouncement = 'warning' | 'critical' | 'expired' | null;

interface Base {
  timing: SessionTiming;
  /** server clock minus local clock, ms, at the moment the timing arrived */
  offset: number;
  /** the idle deadline as last seen or inferred (seconds) */
  idleAt: number;
}

export function createSessionBar(options: SessionBarOptions) {
  const now = options.now ?? Date.now;
  const hidden = options.isHidden ?? (() => typeof document !== 'undefined' && document.hidden);
  const enabled = options.enabled ?? (() => true);
  const [base, setBase] = createSignal<Base | null>(null);
  const [tick, setTick] = createSignal(now());
  const [ended, setEnded] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [announcement, setAnnouncement] = createSignal<SessionAnnouncement>(null);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let grace: ReturnType<typeof setTimeout> | undefined;
  let generation = 0;
  let disposed = false;

  function apply(timing: SessionTiming): void {
    setEnded(false);
    setBase({ timing, offset: timing.serverNow * 1000 - now(), idleAt: timing.idleExpiresAt });
    setTick(now());
  }

  const serverNowSeconds = (): number => (tick() + (base()?.offset ?? 0)) / 1000;
  const idleLeft = createMemo(() => { const b = base(); return b ? b.idleAt - serverNowSeconds() : null; });
  const absoluteLeft = createMemo(() => { const b = base(); return b ? b.timing.absoluteExpiresAt - serverNowSeconds() : null; });
  /** Seconds until the sooner of the two deadlines. */
  const seconds = createMemo<number | null>(() => {
    const i = idleLeft();
    const a = absoluteLeft();
    if (i === null || a === null) return null;
    if (ended()) return 0;
    return Math.max(0, Math.ceil(Math.min(i, a)));
  });
  /** Which deadline is the sooner. Ties go to the absolute limit, which cannot be extended. */
  const primary = createMemo<'idle' | 'absolute'>(() => {
    const i = idleLeft();
    const a = absoluteLeft();
    return i !== null && a !== null && i < a ? 'idle' : 'absolute';
  });
  const level = createMemo<SessionLevel | null>(() => { const s = seconds(); return s === null ? null : levelFor(s); });
  const idleSeconds = createMemo(() => { const i = idleLeft(); return i === null ? null : Math.max(0, Math.ceil(i)); });
  const absoluteSeconds = createMemo(() => { const a = absoluteLeft(); return a === null ? null : Math.max(0, Math.ceil(a)); });
  /** Fraction of the primary window still left, for the progress sliver. */
  const fraction = createMemo(() => {
    const b = base();
    const s = seconds();
    if (!b || s === null) return 0;
    const window = primary() === 'idle' ? b.timing.idleTimeoutSeconds : b.timing.absoluteTimeoutSeconds;
    return window > 0 ? Math.max(0, Math.min(1, s / window)) : 0;
  });
  /** An extension helps only while the idle deadline is the one that ends the session first. */
  const canExtend = createMemo(() => primary() === 'idle' && level() !== 'expired');

  // Announce a threshold once, when it is crossed from above. Never on a tick.
  createEffect(on(level, (now_, before) => {
    if (now_ === 'warning' || now_ === 'critical' || now_ === 'expired') {
      if (before === now_) return;
      // Skip the announcement that merely re-states a level already announced
      // (critical -> warning cannot happen without an extension, which resets).
      setAnnouncement(now_);
    } else if (now_ === 'normal') setAnnouncement(null);
  }));

  async function resync(): Promise<void> {
    if (disposed || !enabled()) return;
    const mine = ++generation;
    try {
      const timing = await options.api.status();
      if (mine !== generation || disposed) return;
      if (timing === null) setEnded(true);
      else apply(timing);
    } catch {
      // Unreachable server: keep counting on what is known. The local deadline is
      // the server's own rule, so it is as right as it was.
    }
  }

  async function extend(): Promise<void> {
    if (busy() || !canExtend()) return;
    setBusy(true);
    const mine = ++generation;
    try {
      const timing = await options.api.extend();
      if (mine !== generation || disposed) return;
      if (timing === null) setEnded(true);
      else apply(timing);
    } catch {
      // The button stays; the countdown goes on.
    } finally {
      setBusy(false);
    }
  }

  // An admin request the server accepted moved its idle deadline to now + timeout.
  onCleanup(onAdminActivity(() => {
    const b = base();
    if (!b || ended() || !enabled()) return;
    const at = Math.min(Math.floor((now() + b.offset) / 1000) + b.timing.idleTimeoutSeconds, b.timing.absoluteExpiresAt);
    if (at > b.idleAt) setBase({ ...b, idleAt: at });
  }));

  // Reaching zero by the local clock: ask the server once before believing it.
  // Another tab may have used the session, or this clock may have jumped.
  let confirming = false;
  createEffect(on(level, (value) => {
    if (value !== 'expired' || ended() || confirming) return;
    confirming = true;
    void resync().finally(() => { confirming = false; if (level() === 'expired' && !ended() && base()) setEnded(true); });
  }));

  // Entering the warning window: look at the server's deadline once, without extending it.
  createEffect(on(level, (value, before) => {
    if (value === 'warning' && before === 'normal') void resync();
  }));

  createEffect(() => {
    if (ended()) {
      clearTimeout(grace);
      grace = setTimeout(() => options.onExpired(), EXPIRED_GRACE_MS);
    } else clearTimeout(grace);
  });

  // One tick a second, on the second, only while the page is visible and the bar runs.
  function schedule(): void {
    clearTimeout(timer);
    if (disposed || hidden() || !enabled()) return;
    const t = now();
    timer = setTimeout(() => { setTick(now()); schedule(); }, 1000 - (t % 1000) + 5);
  }
  function onVisibility(): void {
    clearTimeout(timer);
    if (hidden()) return;
    setTick(now());
    void resync();
    schedule();
  }
  if (typeof document !== 'undefined') document.addEventListener('visibilitychange', onVisibility);
  createEffect(() => {
    if (enabled()) { void resync(); schedule(); } else clearTimeout(timer);
  });
  onCleanup(() => {
    disposed = true;
    clearTimeout(timer);
    clearTimeout(grace);
    if (typeof document !== 'undefined') document.removeEventListener('visibilitychange', onVisibility);
  });

  return {
    timing: () => base()?.timing ?? null,
    seconds, idleSeconds, absoluteSeconds, primary, level, fraction, canExtend,
    busy, announcement, extend, resync,
    expireNow: () => options.onExpired(),
  };
}

export type SessionBarController = ReturnType<typeof createSessionBar>;
