// Client-side error reporting: one small module that turns anything the app
// catches (an error boundary, window.onerror, an unhandled rejection, a failed
// lazy chunk, a worker error, a CSP violation, optionally console.error) into a
// scrubbed report, de-duplicates it, and posts it to the server's observability
// (`POST /api/observability/client-errors`, shown under Admin > Observability >
// Client errors with symbolicated stack traces).
//
// What a report contains: the error's class name, a scrubbed message, the raw
// JavaScript stack frames of the bundle (paths only: no origin, query string or
// hash), the name of the boundary or surface, the route with ids replaced by
// `:id`, the build id, a coarse browser/OS/viewport class, the locale and a few
// flags. What it never contains: a user id, address, IP, token, message content,
// attachment or file name, or a URL with a query.
//
// Rules this module keeps:
//  * It never throws and never blocks the UI: every entry point is wrapped.
//  * It never reports its own failures (a re-entrancy guard, no console output,
//    and a circuit breaker after repeated transport failures).
//  * Nothing is stored in the browser: no cookie, no local or session storage.
//  * The same fingerprint within 60 s is sent once; later repeats are folded into
//    one follow-up report with a count.

import { apiFetch } from '../api/http.ts';
import { withBase } from '../api/basePath.ts';

export type ErrorKind = 'boundary' | 'window.onerror' | 'unhandledrejection' | 'chunk' | 'worker' | 'csp' | 'console';

export interface ClientErrorReport {
  id: string;
  /** Epoch milliseconds on the client; the server stamps its own time. */
  time: number;
  kind: ErrorKind;
  name: string;
  message: string;
  stack: string;
  componentStack: string;
  boundary: string;
  route: string;
  build: string;
  fingerprint: string;
  count: number;
  /** Random per page load, held in memory only: counts affected sessions. */
  visit: string;
  uaFamily: string;
  uaVersion: string;
  osFamily: string;
  viewport: string;
  locale: string;
  online: boolean;
  reducedMotion: boolean;
  theme: string;
}

export type ReportStatus = 'pending' | 'reported' | 'off' | 'failed';

export interface ReportHandle {
  /** Eight hex characters a person can quote to support. */
  readonly ref: string;
  status(): ReportStatus;
  subscribe(listener: (status: ReportStatus) => void): () => void;
  /** The scrubbed report as plain text, for pasting into a ticket. */
  details(): string;
}

export interface ClientErrorPolicy {
  enabled: boolean;
  sampleRate: number;
  maxPerSession: number;
  captureConsole: boolean;
  captureCsp: boolean;
}

const DEFAULT_POLICY: ClientErrorPolicy = { enabled: true, sampleRate: 1, maxPerSession: 20, captureConsole: false, captureCsp: true };

export const LIMITS = {
  message: 300,
  stack: 8 * 1024,
  component: 600,
  /** The server refuses a report larger than this. */
  report: 16 * 1024,
  batch: 20,
  dedupeMs: 60_000,
  flushMs: 1500,
  breakerFailures: 3,
  breakerMs: 5 * 60_000,
  policyMs: 10 * 60_000,
  queue: 40,
  consolePerMinute: 5,
} as const;

// ---- scrubbing ---------------------------------------------------------------

const EMAIL = /[A-Za-z0-9._%+'=/-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/g;
const ORIGIN = /\b(?:https?|wss?|blob|file):\/\/[^/\s)'"]*/g;
const FILE_NAME = /[^\s/\\'"()<>:]+\.(?:pdf|docx?|xlsx?|pptx?|csv|txt|rtf|odt|ods|odp|png|jpe?g|gif|webp|heic|svg|zip|rar|7z|gz|tar|eml|msg|ics|vcf|mp3|mp4|mov|wav|pgp|asc)\b/gi;
const BEARER = /\b(?:Bearer|Basic)\s+[A-Za-z0-9._~+/=-]+/gi;
const LONG_TOKEN = /[A-Za-z0-9_+=-]{24,}/g;

/** Remove `?query` and `#hash` from every path- or URL-like word. */
export function stripQueries(text: string): string {
  return text.replace(/(\S*?)([?#])([^\s)'"]*)/g, (whole, head: string, mark: string, tail: string) => {
    const pathish = head.includes('/') || /^\(?[a-z][a-z0-9+.-]*:/i.test(head);
    if (!pathish && !(mark === '?' && tail.includes('='))) return whole;
    // Keep a `:line:col` location that follows the removed part.
    const loc = /(:\d+(?::\d+)?\)?)$/.exec(tail);
    return head + (loc?.[1] ?? '');
  });
}

function hasDigitAndLetter(s: string): boolean {
  return /\d/.test(s) && /[A-Za-z]/.test(s);
}

/** A message: no addresses, tokens, file names, queries or hashes; bounded. */
export function scrubMessage(text: string, max: number = LIMITS.message): string {
  let out = String(text);
  out = out.replace(EMAIL, '[address]').replace(BEARER, '[token]').replace(ORIGIN, '');
  out = stripQueries(out);
  out = out.replace(FILE_NAME, '[file]');
  out = out.replace(LONG_TOKEN, (m) => (hasDigitAndLetter(m) ? '[token]' : m));
  return out.length > max ? `${out.slice(0, max)}…` : out;
}

/**
 * A stack: origins, queries and hashes removed, addresses removed. Long hashed
 * chunk names are kept (they look like tokens and are exactly what the server
 * needs to find the right source map).
 */
export function scrubStack(stack: string, max: number = LIMITS.stack): string {
  const lines = String(stack)
    .split('\n')
    .slice(0, 60)
    .map((line) => stripQueries(line.replace(ORIGIN, '').replace(EMAIL, '[address]').trimEnd()));
  const out = lines.join('\n');
  return out.length > max ? `${out.slice(0, max)}…` : out;
}

/** `#/mail/INBOX/m-0a1b2c3d4e5f?q=x` becomes `/mail/INBOX/:id`. No query, no hash. */
export function routeTemplate(route: string): string {
  const path = route.replace(/^\/?#/, '').split(/[?#]/)[0] ?? '';
  const segments = path
    .split('/')
    .filter((s) => s !== '')
    .slice(0, 8)
    .map((s) => {
      let seg: string;
      try {
        seg = decodeURIComponent(s);
      } catch {
        seg = s;
      }
      const clean = seg.replace(/[^A-Za-z0-9_:-]/g, '').slice(0, 40);
      const digits = (clean.match(/\d/g) ?? []).length;
      if (clean !== seg || clean.length > 16 || digits >= 3 || (digits >= 1 && clean.length >= 8)) return ':id';
      return clean;
    });
  return `/${segments.join('/')}`;
}

// ---- classification ------------------------------------------------------------

export function uaClass(ua: string): { family: string; version: string; os: string } {
  const pick = (re: RegExp): string => re.exec(ua)?.[1]?.split('.')[0] ?? '';
  let family = 'Other';
  let version = '';
  if (/Edg(?:e|A|iOS)?\//.test(ua)) { family = 'Edge'; version = pick(/Edg(?:e|A|iOS)?\/([\d.]+)/); }
  else if (/OPR\/|Opera/.test(ua)) { family = 'Opera'; version = pick(/OPR\/([\d.]+)/); }
  else if (/Firefox\/|FxiOS\//.test(ua)) { family = 'Firefox'; version = pick(/(?:Firefox|FxiOS)\/([\d.]+)/); }
  else if (/Chrome\/|CriOS\//.test(ua)) { family = 'Chrome'; version = pick(/(?:Chrome|CriOS)\/([\d.]+)/); }
  else if (/Safari\//.test(ua) && /Version\//.test(ua)) { family = 'Safari'; version = pick(/Version\/([\d.]+)/); }
  let os = 'Other';
  if (/Android/.test(ua)) os = 'Android';
  else if (/iPhone|iPad|iPod/.test(ua)) os = 'iOS';
  else if (/CrOS/.test(ua)) os = 'ChromeOS';
  else if (/Windows/.test(ua)) os = 'Windows';
  else if (/Mac OS X|Macintosh/.test(ua)) os = 'macOS';
  else if (/Linux|X11/.test(ua)) os = 'Linux';
  return { family, version, os };
}

export function viewportClass(width: number): string {
  return width < 600 ? 'narrow' : width < 1024 ? 'medium' : 'wide';
}

/** Chunk base name without its content hash: `/assets/Reader-AbC123de.js` -> `Reader`. */
function chunkOf(frame: string): string {
  const m = /([A-Za-z0-9_.]+?)(?:-[A-Za-z0-9_-]{6,12})?\.m?js\b/.exec(frame);
  return m?.[1] ?? '';
}

/** 53-bit string hash (cyrb53), hex. Not cryptographic: grouping only. */
export function hash53(text: string): string {
  let h1 = 0xdeadbeef;
  let h2 = 0x41c6ce57;
  for (let i = 0; i < text.length; i++) {
    const ch = text.charCodeAt(i);
    h1 = Math.imul(h1 ^ ch, 2654435761);
    h2 = Math.imul(h2 ^ ch, 1597334677);
  }
  h1 = Math.imul(h1 ^ (h1 >>> 16), 2246822507) ^ Math.imul(h2 ^ (h2 >>> 13), 3266489909);
  h2 = Math.imul(h2 ^ (h2 >>> 16), 2246822507) ^ Math.imul(h1 ^ (h1 >>> 13), 3266489909);
  return (4294967296 * (2097151 & h2) + (h1 >>> 0)).toString(16).padStart(14, '0');
}

/**
 * Groups reports of one defect across builds: error name, the message with
 * numbers and quoted text removed, the boundary, and the chunk names of the top
 * frames. Function names and line numbers are left out because they change with
 * every minified build, which would split one defect into one group per build.
 */
export function fingerprintOf(kind: string, name: string, message: string, stack: string, boundary: string): string {
  const normMessage = message.replace(/\d+/g, '#').replace(/(["'`]).*?\1/g, '$1$1').toLowerCase().slice(0, 120);
  const chunks = stack
    .split('\n')
    .map(chunkOf)
    .filter((c) => c !== '')
    .slice(0, 3);
  return hash53([kind, name, normMessage, boundary, ...chunks].join('|'));
}

// ---- normalising an error --------------------------------------------------------

export interface Context {
  route: string;
  build: string;
  userAgent: string;
  width: number;
  locale: string;
  online: boolean;
  reducedMotion: boolean;
  theme: string;
}

export function uuid(): string {
  try {
    if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') return crypto.randomUUID();
  } catch { /* fall through */ }
  const hex = (n: number): string => Array.from({ length: n }, () => Math.floor(Math.random() * 16).toString(16)).join('');
  return `${hex(8)}-${hex(4)}-4${hex(3)}-a${hex(3)}-${hex(12)}`;
}

function describe(input: unknown): { name: string; message: string; stack: string } {
  if (input instanceof Error) return { name: input.name || 'Error', message: input.message, stack: input.stack ?? '' };
  if (typeof input === 'object' && input !== null) {
    const o = input as { name?: unknown; message?: unknown; stack?: unknown };
    return {
      name: typeof o.name === 'string' && o.name !== '' ? o.name : 'NonError',
      message: typeof o.message === 'string' ? o.message : '',
      stack: typeof o.stack === 'string' ? o.stack : '',
    };
  }
  return { name: 'NonError', message: typeof input === 'string' ? input : '', stack: '' };
}

/** Turn a caught value into a scrubbed report. Pure: the context is passed in. */
export function normalise(input: unknown, kind: ErrorKind, boundary: string, ctx: Context, extra?: { componentStack?: string; visit?: string; count?: number; time?: number }): ClientErrorReport {
  const d = describe(input);
  const stack = scrubStack(d.stack);
  const message = scrubMessage(d.message);
  const name = d.name.replace(/[^A-Za-z0-9_.$: ]/g, '').slice(0, 80) || 'Error';
  const boundaryName = boundary.replace(/[^A-Za-z0-9_/.:>-]/g, '').slice(0, 120);
  const ua = uaClass(ctx.userAgent);
  return {
    id: uuid(),
    time: extra?.time ?? Date.now(),
    kind,
    name,
    message,
    stack,
    componentStack: scrubStack(extra?.componentStack ?? '', LIMITS.component),
    boundary: boundaryName,
    route: routeTemplate(ctx.route),
    build: ctx.build.replace(/[^A-Za-z0-9+._-]/g, '').slice(0, 64),
    fingerprint: fingerprintOf(kind, name, message, stack, boundaryName),
    count: extra?.count ?? 1,
    visit: extra?.visit ?? '',
    uaFamily: ua.family,
    uaVersion: ua.version,
    osFamily: ua.os,
    viewport: viewportClass(ctx.width),
    locale: ctx.locale.replace(/[^A-Za-z0-9_-]/g, '').slice(0, 16),
    online: ctx.online,
    reducedMotion: ctx.reducedMotion,
    theme: ctx.theme.replace(/[^A-Za-z0-9_-]/g, '').slice(0, 32),
  };
}

/** The report as text for a ticket or an email to support. */
export function reportText(r: ClientErrorReport): string {
  return [
    `Error reference: ${r.id.slice(0, 8)}`,
    `${r.name}: ${r.message}`,
    `Where: ${r.boundary || '(none)'} ${r.route}`,
    `Build: ${r.build || '(unknown)'}`,
    `Browser: ${r.uaFamily} ${r.uaVersion} on ${r.osFamily}, ${r.viewport} viewport, ${r.locale}`,
    '',
    r.stack,
  ].join('\n');
}

/** Dropped when serialized: keeps a pathological message from being rejected whole. */
export function fit(report: ClientErrorReport): ClientErrorReport {
  let out = report;
  while (JSON.stringify(out).length > LIMITS.report && out.stack.length > 200) {
    out = { ...out, stack: out.stack.slice(0, Math.floor(out.stack.length / 2)) };
  }
  return JSON.stringify(out).length > LIMITS.report ? { ...out, stack: '', componentStack: '', message: out.message.slice(0, 100) } : out;
}

// ---- the reporter ----------------------------------------------------------------

export interface Transport {
  /** POST a batch; resolves with the outcome, never rejects. */
  send(reports: ClientErrorReport[], opts: { unloading: boolean }): Promise<{ ok: boolean; status: number; off?: boolean; retryAfter?: number }>;
  loadPolicy(): Promise<ClientErrorPolicy | null>;
}

export interface ReporterOptions {
  transport: Transport;
  context: () => Context;
  now?: () => number;
  random?: () => number;
  setTimer?: (fn: () => void, ms: number) => unknown;
  clearTimer?: (handle: unknown) => void;
}

interface Seen {
  first: number;
  pending: number;
  tailTimer: unknown;
  template: ClientErrorReport | null;
}

export class Reporter {
  private readonly o: Required<Omit<ReporterOptions, 'transport' | 'context'>> & Pick<ReporterOptions, 'transport' | 'context'>;
  private policy: ClientErrorPolicy = DEFAULT_POLICY;
  private policyAt = 0;
  private queue: ClientErrorReport[] = [];
  private seen = new Map<string, Seen>();
  private statuses = new Map<string, ReportStatus>();
  private listeners = new Map<string, Set<(s: ReportStatus) => void>>();
  private sentCount = 0;
  private failures = 0;
  private openUntil = 0;
  private pausedUntil = 0;
  private flushTimer: unknown = null;
  private userOff = false;
  private reporting = false;
  private consoleWindow: number[] = [];
  readonly visit: string;

  constructor(options: ReporterOptions) {
    this.o = {
      now: () => Date.now(),
      random: () => Math.random(),
      setTimer: (fn, ms) => setTimeout(fn, ms),
      clearTimer: (h) => clearTimeout(h as ReturnType<typeof setTimeout>),
      ...options,
    };
    this.visit = uuid().replace(/-/g, '').slice(0, 12);
  }

  /** True while the circuit breaker is open or the server asked us to wait. */
  get suspended(): boolean {
    return this.o.now() < Math.max(this.openUntil, this.pausedUntil);
  }

  get captureConsole(): boolean {
    return this.policy.captureConsole;
  }
  get captureCsp(): boolean {
    return this.policy.captureCsp;
  }

  /**
   * Report a caught value. Returns a handle (with a reference id) even when the
   * report is dropped, so a boundary can always show a reference and a state.
   */
  report(input: unknown, kind: ErrorKind, boundary: string, extra?: { componentStack?: string }): ReportHandle | null {
    if (this.reporting) return null;
    this.reporting = true;
    try {
      const ctx = this.o.context();
      const report = normalise(input, kind, boundary, ctx, { ...extra, visit: this.visit, time: this.o.now() });
      const handle = this.handleFor(report);
      this.admit(report);
      return handle;
    } catch {
      return null;
    } finally {
      this.reporting = false;
    }
  }

  /** Read the server policy once (console capture needs it before any report). */
  async warmPolicy(): Promise<void> {
    try {
      const policy = await this.o.transport.loadPolicy();
      this.policyAt = this.o.now();
      if (policy !== null) this.policy = policy;
    } catch { /* defaults stay */ }
  }

  /** Console errors are throttled further than the other kinds. */
  allowConsole(): boolean {
    const t = this.o.now();
    this.consoleWindow = this.consoleWindow.filter((x) => t - x < 60_000);
    if (this.consoleWindow.length >= LIMITS.consolePerMinute) return false;
    this.consoleWindow.push(t);
    return true;
  }

  private admit(report: ClientErrorReport): void {
    const t = this.o.now();
    const entry = this.seen.get(report.fingerprint);
    if (entry !== undefined && t - entry.first < LIMITS.dedupeMs) {
      // The same defect again within the window: fold into one follow-up.
      entry.pending += 1;
      entry.template = report;
      this.setStatus(report.id, this.statuses.get(report.id) ?? 'pending');
      if (entry.tailTimer === null) {
        entry.tailTimer = this.o.setTimer(() => this.sendTail(report.fingerprint), Math.max(1000, LIMITS.dedupeMs - (t - entry.first)));
      }
      return;
    }
    this.seen.set(report.fingerprint, { first: t, pending: 0, tailTimer: null, template: null });
    if (this.seen.size > 200) this.seen.delete(this.seen.keys().next().value as string);
    this.enqueue(report);
  }

  private sendTail(fingerprint: string): void {
    const entry = this.seen.get(fingerprint);
    if (entry === undefined) return;
    entry.tailTimer = null;
    if (entry.pending > 0 && entry.template !== null) {
      this.enqueue({ ...entry.template, id: uuid(), count: entry.pending });
    }
    this.seen.delete(fingerprint);
  }

  private enqueue(report: ClientErrorReport): void {
    if (this.userOff || !this.policy.enabled) {
      this.setStatus(report.id, 'off');
      return;
    }
    if (this.queue.length >= LIMITS.queue) {
      this.setStatus(report.id, 'failed');
      return;
    }
    this.queue.push(fit(report));
    this.schedule();
  }

  private schedule(): void {
    if (this.flushTimer !== null) return;
    this.flushTimer = this.o.setTimer(() => {
      this.flushTimer = null;
      void this.flush(false);
    }, LIMITS.flushMs);
  }

  /** Send what is queued. `unloading` prefers a transport that survives the page. */
  async flush(unloading: boolean): Promise<void> {
    if (this.flushTimer !== null) {
      this.o.clearTimer(this.flushTimer);
      this.flushTimer = null;
    }
    // Tails are sent on unload too: they carry the counts.
    if (unloading) for (const fp of [...this.seen.keys()]) this.sendTail(fp);
    if (this.queue.length === 0) return;
    try {
      if (this.o.now() - this.policyAt > LIMITS.policyMs) {
        const policy = await this.o.transport.loadPolicy();
        this.policyAt = this.o.now();
        if (policy !== null) this.policy = policy;
      }
      if (!this.policy.enabled) {
        this.markAll('off');
        return;
      }
      // The policy is only known now: apply the sample rate and the per-page cap.
      this.queue = this.queue.filter((r) => {
        if (this.policy.sampleRate < 1 && this.o.random() >= this.policy.sampleRate) {
          this.setStatus(r.id, 'off');
          return false;
        }
        return true;
      });
      const room = Math.max(0, this.policy.maxPerSession - this.sentCount);
      for (const r of this.queue.slice(room)) this.setStatus(r.id, 'failed');
      this.queue = this.queue.slice(0, room);
      if (this.queue.length === 0) return;
      if (this.suspended) {
        this.markAll('failed');
        return;
      }
      while (this.queue.length > 0) {
        const batch = this.queue.splice(0, LIMITS.batch);
        const result = await this.o.transport.send(batch, { unloading });
        if (result.off === true) {
          this.userOff = true;
          for (const r of batch) this.setStatus(r.id, 'off');
          this.queue = [];
          return;
        }
        if (result.ok) {
          this.failures = 0;
          this.sentCount += batch.length;
          for (const r of batch) this.setStatus(r.id, 'reported');
          continue;
        }
        for (const r of batch) this.setStatus(r.id, 'failed');
        if (result.status === 429) {
          this.pausedUntil = this.o.now() + Math.max(5, result.retryAfter ?? 30) * 1000;
          this.queue = [];
          return;
        }
        if (++this.failures >= LIMITS.breakerFailures) {
          this.openUntil = this.o.now() + LIMITS.breakerMs;
          this.queue = [];
          this.failures = 0;
          return;
        }
      }
    } catch {
      this.markAll('failed');
    }
  }

  private markAll(status: ReportStatus): void {
    for (const r of this.queue) this.setStatus(r.id, status);
    this.queue = [];
  }

  // ---- statuses and handles ----

  private latest = new Map<string, ClientErrorReport>();

  private handleFor(report: ClientErrorReport): ReportHandle {
    this.latest.set(report.id, report);
    if (this.latest.size > 50) this.latest.delete(this.latest.keys().next().value as string);
    this.statuses.set(report.id, 'pending');
    const id = report.id;
    return {
      ref: id.replace(/-/g, '').slice(0, 8),
      status: () => this.statuses.get(id) ?? 'pending',
      subscribe: (listener) => {
        let set = this.listeners.get(id);
        if (set === undefined) this.listeners.set(id, (set = new Set()));
        set.add(listener);
        return () => set.delete(listener);
      },
      details: () => reportText(report),
    };
  }

  private setStatus(id: string, status: ReportStatus): void {
    this.statuses.set(id, status);
    for (const l of this.listeners.get(id) ?? []) {
      try {
        l(status);
      } catch { /* a listener's failure is not ours to report */ }
    }
  }
}

// ---- the page singleton ------------------------------------------------------------

let buildOverride = '';
/** The build id the "new version" notice compares (`version+webBuild`). */
export function setReportBuild(id: string): void {
  buildOverride = id;
}

function entryBuild(): string {
  try {
    const src = document.querySelector<HTMLScriptElement>('script[type="module"][src*="/assets/"]')?.getAttribute('src') ?? '';
    const m = /([A-Za-z0-9_.-]+)\.m?js/.exec(src);
    return m?.[1] ?? '';
  } catch {
    return '';
  }
}

function pageContext(): Context {
  const w = typeof window === 'undefined' ? undefined : window;
  let theme = '';
  let reduced = false;
  try {
    theme = document.documentElement.dataset['theme'] ?? '';
    reduced = w?.matchMedia?.('(prefers-reduced-motion: reduce)').matches === true;
  } catch { /* defaults */ }
  return {
    route: typeof location === 'undefined' ? '' : location.hash,
    build: buildOverride !== '' ? buildOverride : entryBuild(),
    userAgent: typeof navigator === 'undefined' ? '' : navigator.userAgent,
    width: w?.innerWidth ?? 1024,
    locale: typeof document === 'undefined' ? '' : document.documentElement.lang,
    online: typeof navigator === 'undefined' ? true : navigator.onLine,
    reducedMotion: reduced,
    theme,
  };
}

const ANON = '/api/observability/client-errors';
const OPT_KEY = 'mw.errorReports';

/**
 * The person's last known `privacy.errorReports` choice, cached in this browser.
 * The anonymous route cannot know a signed-out visitor's opt-out, so it is only
 * used while the cache does not say 'off'. The signed-in route answers
 * `reporting` on every post and Settings writes the choice here when it changes.
 */
export function cachedErrorReports(): 'on' | 'off' | null {
  try {
    const v = localStorage.getItem(OPT_KEY);
    return v === 'on' || v === 'off' ? v : null;
  } catch {
    return null;
  }
}
export function cacheErrorReports(value: 'on' | 'off'): void {
  try { localStorage.setItem(OPT_KEY, value); } catch { /* storage unavailable */ }
}
const SESSION = `${ANON}/session`;

export function browserTransport(): Transport {
  // 'in' once the session route answered, 'out' after a 401; the anonymous route
  // (and `sendBeacon`) carry reports from the sign-in page and from an unload.
  let mode: 'unknown' | 'in' | 'out' = 'unknown';
  let outAt = 0;
  const body = (reports: ClientErrorReport[]): string => JSON.stringify({ reports });
  return {
    async loadPolicy() {
      try {
        const res = await apiFetch(withBase(`${ANON}/policy`), { cache: 'no-store', pace: 'background' });
        if (!res.ok) return null;
        const p = (await res.json()) as Partial<ClientErrorPolicy>;
        return {
          enabled: p.enabled !== false,
          sampleRate: typeof p.sampleRate === 'number' ? p.sampleRate : 1,
          maxPerSession: typeof p.maxPerSession === 'number' ? p.maxPerSession : 20,
          captureConsole: p.captureConsole === true,
          captureCsp: p.captureCsp !== false,
        };
      } catch {
        return null;
      }
    },
    async send(reports, { unloading }) {
      const payload = body(reports);
      try {
        if (mode !== 'out' || Date.now() - outAt > 60_000) {
          const res = await apiFetch(withBase(SESSION), {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: payload,
            keepalive: payload.length < 60_000,
            credentials: 'same-origin',
            pace: 'background',
          });
          if (res.status !== 401) {
            mode = 'in';
            if (res.status === 429) return { ok: false, status: 429, retryAfter: Number(res.headers.get('Retry-After') ?? 30) };
            if (!res.ok) return { ok: false, status: res.status };
            const info = (await res.json().catch(() => ({}))) as { reporting?: string };
            if (info.reporting === 'off' || info.reporting === 'on') cacheErrorReports(info.reporting);
            return info.reporting === 'off' ? { ok: false, status: 200, off: true } : { ok: true, status: res.status };
          }
          mode = 'out';
          outAt = Date.now();
        }
        // A person who opted out while signed in must not be reported on the
        // anonymous route (or by the unload beacon) after the session ends.
        if (cachedErrorReports() === 'off') return { ok: false, status: 200, off: true };
        const anon = reports.slice(0, 5);
        const anonPayload = body(anon);
        if (unloading && typeof navigator.sendBeacon === 'function') {
          const ok = navigator.sendBeacon(withBase(ANON), new Blob([anonPayload], { type: 'application/json' }));
          return { ok, status: ok ? 204 : 0 };
        }
        const res = await apiFetch(withBase(ANON), {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: anonPayload,
          keepalive: true,
          credentials: 'omit',
          pace: 'background',
        });
        if (res.status === 429) return { ok: false, status: 429, retryAfter: Number(res.headers.get('Retry-After') ?? 30) };
        return { ok: res.ok, status: res.status };
      } catch {
        return { ok: false, status: 0 };
      }
    },
  };
}

let singleton: Reporter | null = null;

export function reporter(): Reporter {
  singleton ??= new Reporter({ transport: browserTransport(), context: pageContext });
  return singleton;
}

/** Replace the page reporter (tests). */
export function setReporterForTests(r: Reporter | null): void {
  singleton = r;
}

/**
 * Report an error caught by a boundary. Safe to call from a fallback: it never
 * throws and returns `null` when nothing could be recorded.
 */
const handles = new WeakMap<object, ReportHandle | null>();
export function reportBoundaryError(error: unknown, boundary: string, componentStack?: string): ReportHandle | null {
  try {
    const key = typeof error === 'object' && error !== null ? error : null;
    if (key !== null && handles.has(key)) return handles.get(key) ?? null;
    const kind: ErrorKind = isChunkFailure(error) ? 'chunk' : 'boundary';
    const handle = reporter().report(error, kind, boundary, componentStack === undefined ? undefined : { componentStack });
    if (key !== null) handles.set(key, handle);
    return handle;
  } catch {
    return null;
  }
}

export function isChunkFailure(error: unknown): boolean {
  const text = error instanceof Error ? `${error.name} ${error.message}` : typeof error === 'string' ? error : '';
  return /ChunkLoadError|Failed to fetch dynamically imported module|Importing a module script failed|error loading dynamically imported module|Loading chunk [\w-]+ failed/i.test(text);
}

/** A worker error (sanitizer pool, crypto worker, PDF bridge, ...). */
export function reportWorkerError(worker: string, event: ErrorEvent | Error | unknown): void {
  try {
    const input = event instanceof ErrorEvent ? Object.assign(new Error(event.message || 'Worker error'), { name: 'WorkerError', stack: event.error instanceof Error ? event.error.stack : `at ${event.filename}:${event.lineno}:${event.colno}` }) : event;
    reporter().report(input, 'worker', `Worker/${worker}`);
  } catch { /* never throws */ }
}

let installed = false;

/**
 * Attach the page-wide catchers once. Called from `main.tsx` before render; it
 * does no network work until the first report.
 */
export function installClientErrors(): void {
  if (installed || typeof window === 'undefined') return;
  installed = true;
  const guard = <T extends unknown[]>(fn: (...args: T) => void) => (...args: T): void => {
    try {
      fn(...args);
    } catch { /* never throws */ }
  };
  window.addEventListener('error', guard((event: Event) => {
    const e = event as ErrorEvent;
    const target = event.target as (HTMLElement & { src?: string; href?: string }) | Window | null;
    if (target !== null && target !== window && 'tagName' in target) {
      // A failed same-origin script or stylesheet. Images and media are left out:
      // their sources can be message content.
      const tag = target.tagName;
      if (tag !== 'SCRIPT' && tag !== 'LINK') return;
      const url = (target.src ?? target.href ?? '').toString();
      if (!url.startsWith(location.origin)) return;
      const err = Object.assign(new Error(`Failed to load ${tag.toLowerCase()} ${new URL(url).pathname}`), { name: 'ResourceError', stack: '' });
      reporter().report(err, 'window.onerror', 'resource');
      return;
    }
    if (e.error === undefined && e.message === 'ResizeObserver loop completed with undelivered notifications.') return;
    const input = e.error ?? Object.assign(new Error(e.message || 'Script error'), { stack: `at ${e.filename}:${e.lineno}:${e.colno}` });
    reporter().report(input, isChunkFailure(input) ? 'chunk' : 'window.onerror', 'window');
  }), true);
  window.addEventListener('unhandledrejection', guard((event: PromiseRejectionEvent) => {
    const reason: unknown = event.reason;
    reporter().report(reason, isChunkFailure(reason) ? 'chunk' : 'unhandledrejection', 'window');
  }));
  window.addEventListener('securitypolicyviolation', guard((event: SecurityPolicyViolationEvent) => {
    if (!reporter().captureCsp) return;
    // The directive and the blocked origin only: never the full blocked URL.
    let blocked = event.blockedURI;
    try { blocked = blocked.includes('://') ? new URL(blocked).origin : blocked.slice(0, 40); } catch { blocked = blocked.slice(0, 40); }
    const err = Object.assign(new Error(`CSP ${event.effectiveDirective} blocked ${blocked}`), { name: 'CspViolation', stack: event.sourceFile !== '' ? `at ${event.sourceFile}:${event.lineNumber}:${event.columnNumber}` : '' });
    reporter().report(err, 'csp', 'csp');
  }));
  // Console capture and CSP reporting depend on the server policy: read it once,
  // a little after boot, never on the critical path.
  window.setTimeout(guard(() => void reporter().warmPolicy()), 8000);
  window.addEventListener('pagehide', guard(() => void reporter().flush(true)));
  document.addEventListener('visibilitychange', guard(() => {
    if (document.visibilityState === 'hidden') void reporter().flush(true);
  }));
  try {
    navigator.serviceWorker?.addEventListener('messageerror', guard(() => {
      reporter().report(Object.assign(new Error('Service worker message could not be deserialised'), { name: 'ServiceWorkerError' }), 'worker', 'ServiceWorker');
    }));
  } catch { /* no service worker */ }
  // Console errors: wrapped only when the server policy asks for it, which is
  // known after the first flush; the wrapper checks the flag on every call.
  const original = console.error.bind(console);
  console.error = (...args: unknown[]): void => {
    original(...args);
    try {
      const r = reporter();
      if (!r.captureConsole || !r.allowConsole()) return;
      const first = args.find((a) => a instanceof Error);
      const input = first ?? Object.assign(new Error(args.map((a) => (typeof a === 'string' ? a : '')).join(' ').trim() || 'console.error'), { name: 'ConsoleError', stack: '' });
      r.report(input, 'console', 'console');
    } catch { /* never throws */ }
  };
}
