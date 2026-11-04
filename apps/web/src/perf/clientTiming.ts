// Browser load-time collection for signed-in sessions (admin Observability >
// Client load times). Server side: `crates/mw-server/src/client_timing.rs`.
//
// Once per page load, after the mailbox list has shown rows (so never on the
// sign-in or any anonymous screen), this reads Navigation Timing, paint timing,
// largest contentful paint and the app's own "time to usable", and sends ONE
// small beacon:
//
//   POST /api/client-timing  { device?: 'desktop'|'mobile',
//                              metrics: [{ name, ms }] }
//
// Nothing else is sent: no URL, path, account, address, user agent or clock
// time. The server keeps only counts in fixed histograms.
//
// It sends nothing when the server's `/api/policy` says collection is off, when
// this session is not in the admin-set sample, when Do Not Track or Global
// Privacy Control is on (as the analytics tag does), when the browser lacks
// `performance`, or when anything throws. It never blocks the UI and never throws.

import { apiFetch } from '../api/http.ts';
import { withBase } from '../api/basePath.ts';
import { loadClientPolicy, clientPolicy } from '../api/policy.ts';

export type TimingMetric = 'timeToUsable' | 'ttfb' | 'dcl' | 'load' | 'fcp' | 'lcp' | 'folderOpen';
export interface TimingSample { name: TimingMetric; ms: number }
export interface TimingPayload { device?: 'desktop' | 'mobile'; metrics: TimingSample[] }

/** The server clamps to this; the client does the same so nothing is lost to a 400. */
export const MAX_MS = 120_000;
const ENDPOINT = '/api/client-timing';

export interface Deps {
  /** Milliseconds since navigation start (`performance.now()`). */
  now(): number;
  /** Current performance entries by type; may throw when unsupported. */
  entries(type: string): PerformanceEntry[];
  supported(): boolean;
  /** 0 <= r < 1. */
  random(): number;
  privacySignal(): boolean;
  device(): 'desktop' | 'mobile' | undefined;
  /** Run `fn` once the page has loaded and the main thread is idle. */
  whenSettled(fn: () => void): void;
  /** Run `fn` when the page is being hidden or closed. */
  onHide(fn: () => void): void;
  policy(): Promise<{ enabled: boolean; sampleRatePercent: number }>;
  send(payload: TimingPayload): Promise<void>;
  /** Largest contentful paint (ms) or null. */
  lcp(): number | null;
}

function round(value: unknown): number | null {
  if (typeof value !== 'number' || !Number.isFinite(value) || value < 0) return null;
  return Math.min(MAX_MS, Math.round(value));
}

export function browserDeps(): Deps {
  let lcpMs: number | null = null;
  try {
    if (typeof PerformanceObserver !== 'undefined') {
      const observer = new PerformanceObserver((list) => {
        const last = list.getEntries().at(-1);
        if (last !== undefined) lcpMs = last.startTime;
      });
      observer.observe({ type: 'largest-contentful-paint', buffered: true });
    }
  } catch {
    // LCP is optional.
  }
  return {
    now: () => performance.now(),
    entries: (type) => performance.getEntriesByType(type),
    supported: () => typeof performance !== 'undefined' && typeof performance.now === 'function' && typeof performance.getEntriesByType === 'function',
    random: () => Math.random(),
    privacySignal: () => {
      const nav = navigator as Navigator & { globalPrivacyControl?: boolean };
      return nav.doNotTrack === '1' || nav.globalPrivacyControl === true;
    },
    device: () => {
      const hinted = (navigator as Navigator & { userAgentData?: { mobile?: boolean } }).userAgentData?.mobile;
      if (hinted === true) return 'mobile';
      if (typeof matchMedia !== 'function') return hinted === false ? 'desktop' : undefined;
      return matchMedia('(max-width: 820px)').matches ? 'mobile' : 'desktop';
    },
    whenSettled: (fn) => {
      const idle = (): void => {
        const ric = (globalThis as { requestIdleCallback?: (cb: () => void, o?: { timeout: number }) => number }).requestIdleCallback;
        if (typeof ric === 'function') ric(fn, { timeout: 3000 });
        else setTimeout(fn, 1000);
      };
      if (document.readyState === 'complete') idle();
      else addEventListener('load', () => setTimeout(idle, 0), { once: true });
    },
    onHide: (fn) => addEventListener('pagehide', fn, { once: true }),
    policy: async () => {
      await loadClientPolicy();
      return clientPolicy().clientTiming ?? { enabled: false, sampleRatePercent: 0 };
    },
    send: async (payload) => {
      await apiFetch(withBase(ENDPOINT), {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(payload),
        keepalive: true,
      });
    },
    lcp: () => lcpMs,
  };
}

/** Collect the page-load metrics that exist now. Only whitelisted names, integer ms. */
export function collectLoadMetrics(deps: Deps, usableMs: number | null): TimingSample[] {
  const out: TimingSample[] = [];
  const push = (name: TimingMetric, value: unknown): void => {
    const ms = round(value);
    if (ms !== null) out.push({ name, ms });
  };
  push('timeToUsable', usableMs);
  try {
    const nav = deps.entries('navigation')[0] as PerformanceNavigationTiming | undefined;
    if (nav !== undefined) {
      push('ttfb', nav.responseStart);
      if (nav.domContentLoadedEventEnd > 0) push('dcl', nav.domContentLoadedEventEnd);
      if (nav.loadEventEnd > 0) push('load', nav.loadEventEnd);
    }
  } catch {
    // Navigation Timing unavailable: the other metrics still go.
  }
  try {
    const fcp = deps.entries('paint').find((entry) => entry.name === 'first-contentful-paint');
    if (fcp !== undefined) push('fcp', fcp.startTime);
  } catch {
    // Paint Timing unavailable.
  }
  const lcp = deps.lcp();
  if (lcp !== null) push('lcp', lcp);
  return out;
}

export class ClientTiming {
  private usableMs: number | null = null;
  private sent = false;
  private armed = false;
  private folderStart: number | null = null;
  private folderSawLoading = false;
  private folderDone = false;
  private decision: Promise<boolean> | null = null;

  constructor(private readonly deps: Deps) {}

  /** The list has shown rows for the first time this page load. */
  markUsable(): void {
    try {
      if (this.usableMs !== null || !this.deps.supported()) return;
      this.usableMs = round(this.deps.now());
      if (this.usableMs === null || this.deps.privacySignal()) return;
      // Ask now, so the decision is known when the page is hidden.
      this.decision = this.decide();
      this.deps.onHide(() => void this.flush());
      this.deps.whenSettled(() => void this.flush());
      this.armed = true;
    } catch {
      this.armed = false;
    }
  }

  /** The list's loading state and row count changed (call from an effect). */
  noteList(loading: boolean, rowCount: number): void {
    try {
      if (this.usableMs === null && !loading && rowCount > 0) this.markUsable();
      if (this.folderStart !== null && !this.folderDone) {
        if (loading) this.folderSawLoading = true;
        else if (this.folderSawLoading) this.finishFolder();
      }
    } catch {
      // Measurement must never reach the UI.
    }
  }

  /** The user picked another folder after the first usable list. */
  noteFolderSelected(): void {
    try {
      if (!this.armed || this.folderDone || this.folderStart !== null) return;
      this.folderStart = this.deps.now();
      this.folderSawLoading = false;
    } catch {
      // ignore
    }
  }

  private finishFolder(): void {
    this.folderDone = true;
    const ms = round(this.deps.now() - (this.folderStart ?? 0));
    if (ms !== null) void this.post([{ name: 'folderOpen', ms }]);
  }

  private decide(): Promise<boolean> {
    return this.deps.policy().then(
      (p) => p.enabled && p.sampleRatePercent > 0 && this.deps.random() * 100 < p.sampleRatePercent,
      () => false,
    );
  }

  private async post(metrics: TimingSample[]): Promise<void> {
    try {
      if (metrics.length === 0 || this.decision === null || !(await this.decision)) return;
      const payload: TimingPayload = { metrics };
      const device = this.deps.device();
      if (device !== undefined) payload.device = device;
      await this.deps.send(payload);
    } catch {
      // A lost beacon is fine.
    }
  }

  private async flush(): Promise<void> {
    if (this.sent) return;
    this.sent = true;
    await this.post(collectLoadMetrics(this.deps, this.usableMs));
  }
}

let shared: ClientTiming | null = null;
function instance(): ClientTiming | null {
  try {
    shared ??= new ClientTiming(browserDeps());
    return shared;
  } catch {
    return null;
  }
}

/** The mailbox list's state changed; call from an effect in the list component. */
export function noteMailboxList(loading: boolean, rowCount: number): void {
  instance()?.noteList(loading, rowCount);
}
export function noteFolderSelected(): void {
  instance()?.noteFolderSelected();
}
/** Tests: forget the page's state. */
export function resetClientTiming(): void {
  shared = null;
}
