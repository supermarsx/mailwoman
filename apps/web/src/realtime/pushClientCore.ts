// WebSocket/EventSource JMAP push client (plan §2.2, §3 e6).
//
// Opens the best available transport and streams RFC 8887 `StateChange` objects
// to subscribers. Transport ladder ws → sse → poll (contracts/push.ts): a WS to
// `/jmap/ws` is primary; on failure it falls to an EventSource on
// `/jmap/eventsource`; on failure it falls to passive polling (transport
// 'poll'), with a bounded number of automatic retries before explicit recovery.
// A 30 s heartbeat detects a dead-but-open socket; reconnects use backoff.
//
// A session the server has ended is not a dropped connection: the server closes
// the WS with code 1008 when its periodic re-check finds the session gone, and a
// reconnect attempt is then refused with 401. A WS close 1008 stops the client at
// once; any other drop also asks the session endpoint whether the session still
// exists, and a 401/403 answer stops it. Either way it reports 'unauthorized'
// and stays stopped until `connect()`/`reconnect()` (after the next sign-in).
//
// The DOM `WebSocket`/`EventSource` constructors and the wall clock are
// injectable so the ladder, heartbeat and backoff are deterministically unit
// testable under jsdom (which ships neither transport).

import {
  HEARTBEAT_MS,
  POLL_JITTER,
  RECONNECT_MAX_FAILURES,
  RECONNECT_STABLE_MS,
  SESSION_CHECK_MIN_MS,
  TRANSPORT_LADDER,
  type MailboxChange,
  type PushClient,
  type PushTransport,
  type StateChange,
} from '../contracts/push.ts';
import { withBase } from '../api/basePath.ts';
import { closeKind, jittered, reconnectDelay, type BackoffConfig, type CloseKind } from './reconnectPolicy.ts';

/** Lifecycle signal the connection model maps to a `ConnectionState`. */
export type PushStatus =
  | 'connecting'
  | 'reconnecting'
  | 'open'
  | 'degraded'
  | 'closed'
  | 'unauthorized';

/** What the session check found: the session is gone, or it could not tell. */
export type SessionCheck = 'ok' | 'unauthorized' | 'unknown';

/** WS close code the server sends when its re-check finds the session ended. */
export const WS_CLOSE_SESSION_ENDED = 1008;

/** Extra facts about a status; present on 'open'. */
export interface PushStatusDetail {
  /** The drop was a restart/busy close or followed two or more failures: stagger the catch-up read. */
  spreadCatchUp: boolean;
}

/** Minimal structural view of the DOM `WebSocket` this client relies on. */
export interface WebSocketLike {
  onopen: ((ev: unknown) => void) | null;
  onmessage: ((ev: { data: unknown }) => void) | null;
  onerror: ((ev: unknown) => void) | null;
  onclose: ((ev: unknown) => void) | null;
  send(data: string): void;
  close(): void;
}
export type WebSocketCtor = new (url: string) => WebSocketLike;

/** Minimal structural view of the DOM `EventSource` this client relies on. */
export interface EventSourceLike {
  onopen: ((ev: unknown) => void) | null;
  onmessage: ((ev: { data: unknown }) => void) | null;
  onerror: ((ev: unknown) => void) | null;
  close(): void;
}
export type EventSourceCtor = new (
  url: string,
  init?: { withCredentials?: boolean },
) => EventSourceLike;

export interface PushClientOptions {
  /** WS endpoint; defaults to same-origin `/jmap/ws`. */
  wsUrl?: string;
  /** SSE endpoint; defaults to same-origin `/jmap/eventsource`. */
  sseUrl?: string;
  /**
   * Optional native bearer token (plan §2.2). WebSocket/EventSource cannot set an
   * `Authorization` header, so when present it is appended as an `access_token`
   * query param on the WS/SSE URLs. The browser omits it → the cookie path,
   * unchanged.
   */
  bearer?: string;
  WebSocketImpl?: WebSocketCtor;
  EventSourceImpl?: EventSourceCtor;
  /** Heartbeat/keepalive interval (default 30 s, §2.2). */
  heartbeatMs?: number;
  /** Maximum time for a WS/SSE opening handshake (default 10 s). */
  connectTimeoutMs?: number;
  /**
   * Fixed reconnect schedule (ms), the last value repeating, with no jitter. For
   * tests; when omitted the client uses full-jitter exponential backoff
   * (`backoffBaseMs`, `backoffCapMs`, `restartSpreadMs`).
   */
  backoff?: number[];
  /** Full-jitter base (default 1 s) and cap (default 60 s). */
  backoffBaseMs?: number;
  backoffCapMs?: number;
  /** First retry after close 1012 is uniform in [0, this] (default 15 s). */
  restartSpreadMs?: number;
  /** A connection open this long counts as stable and resets the failure count (default 30 s). */
  stableAfterMs?: number;
  /** Minimum gap between session checks (default 30 s). */
  sessionCheckMinMs?: number;
  /** Random source in [0, 1), injectable for tests. */
  random?: () => number;
  /** Attempts on one rung before falling to the next (default 2). */
  maxAttemptsPerRung?: number;
  /** Delay before trying the next rung down; fixed when given, else the jittered backoff (100 ms with a fixed `backoff`). */
  rungDelayMs?: number;
  /** Initial wait on 'poll' before retrying the ladder (default 30 s, then exponential backoff). */
  upgradeIntervalMs?: number;
  /** Automatic full-ladder retries without valid activity (default 3, maximum 10). */
  maxAutomaticUpgrades?: number;
  /** Wall clock (default `Date.now`), injectable for fake-timer tests. */
  now?: () => number;
  /** Lifecycle callback for the connection model / toast. */
  onStatus?: (status: PushStatus, transport: PushTransport, detail?: PushStatusDetail) => void;
  /**
   * Asks the server whether the session still exists, after a drop. Defaults to
   * a GET of the JMAP session resource (`/jmap/session`) with the cookie or the
   * bearer: 401/403 → 'unauthorized', any other answer → 'ok', no answer →
   * 'unknown'.
   */
  checkSession?: () => Promise<SessionCheck>;
}

/** The concrete client; adds `reconnect()` on top of the frozen `PushClient`. */
export interface PushClientImpl extends PushClient {
  /** Drop everything and re-attempt from the top of the ladder now. */
  reconnect(): void;
  /** Subscribe to decoded `MailboxChange` extension frames (t46). */
  onMailboxChange?(handler: (change: MailboxChange) => void): () => void;
}

const PING = JSON.stringify({ '@type': 'ping' });

function isStateChange(v: unknown): v is StateChange {
  if (typeof v !== 'object' || v === null) return false;
  const o = v as Record<string, unknown>;
  return o['@type'] === 'StateChange' && typeof o['changed'] === 'object' && o['changed'] !== null;
}

/** A well-formed `MailboxChange` (t46). Anything else is dropped unseen. */
export function isMailboxChange(v: unknown): v is MailboxChange {
  if (typeof v !== 'object' || v === null) return false;
  const o = v as Record<string, unknown>;
  const link = o['link'];
  const changed = o['changed'];
  const unread = o['inboxUnread'];
  return (
    o['@type'] === 'MailboxChange' &&
    (link === null || (typeof link === 'string' && link.length > 0 && link.length <= 128)) &&
    typeof o['storageAuthority'] === 'string' &&
    (o['storageAuthority'] as string).startsWith('msa1_') &&
    typeof changed === 'object' &&
    changed !== null &&
    !Array.isArray(changed) &&
    Object.values(changed).every((state) => typeof state === 'string') &&
    (unread === undefined || (Number.isSafeInteger(unread) && (unread as number) >= 0))
  );
}

type Current =
  | { kind: 'ws'; ws: WebSocketLike }
  | { kind: 'sse'; es: EventSourceLike }
  | null;

export function createPushClient(opts: PushClientOptions = {}): PushClientImpl {
  const wsUrl = withBearer(opts.wsUrl ?? defaultWsUrl(), opts.bearer);
  const sseUrl = withBearer(opts.sseUrl ?? defaultSseUrl(), opts.bearer);
  const WS = opts.WebSocketImpl ?? (globalThis.WebSocket as unknown as WebSocketCtor | undefined);
  const ES =
    opts.EventSourceImpl ?? (globalThis.EventSource as unknown as EventSourceCtor | undefined);
  const heartbeatMs = opts.heartbeatMs ?? HEARTBEAT_MS;
  const connectTimeoutMs = opts.connectTimeoutMs ?? 10_000;
  const maxAttemptsPerRung = opts.maxAttemptsPerRung ?? 2;
  const rungDelayMs = opts.rungDelayMs ?? 100;
  const random = opts.random ?? Math.random;
  const backoffConfig: BackoffConfig = {
    ...(opts.backoffBaseMs !== undefined ? { baseMs: opts.backoffBaseMs } : {}),
    ...(opts.backoffCapMs !== undefined ? { capMs: opts.backoffCapMs } : {}),
    ...(opts.restartSpreadMs !== undefined ? { restartSpreadMs: opts.restartSpreadMs } : {}),
  };
  const stableAfterMs = opts.stableAfterMs ?? RECONNECT_STABLE_MS;
  const sessionCheckMinMs = opts.sessionCheckMinMs ?? SESSION_CHECK_MIN_MS;
  const upgradeIntervalMs = opts.upgradeIntervalMs ?? 30_000;
  const maxAutomaticUpgrades = Math.max(0, Math.min(10, Math.floor(opts.maxAutomaticUpgrades ?? 3)));
  const nowFn = opts.now ?? Date.now;
  const checkSession = opts.checkSession ?? (async () => 'unknown' as const);

  const handlers = new Set<(c: StateChange) => void>();
  const mailboxHandlers = new Set<(c: MailboxChange) => void>();
  let current: Current = null;
  let transport: PushTransport = 'offline';
  let ladderIndex = 0;
  let attemptsOnRung = 0;
  let automaticUpgrades = 0;
  // Consecutive failed attempts across every rung; the backoff exponent. Bounded
  // by RECONNECT_MAX_FAILURES, reset by a stable connection or an explicit connect.
  let failures = 0;
  // Kind of the latest drop; 'busy' stays until a stream opens. `restarted`
  // records a restart/busy close since the last open (it staggers the catch-up).
  let lastCloseKind: CloseKind = 'drop';
  let restarted = false;
  let openedAt: number | undefined;
  let lastCheckAt = Number.NEGATIVE_INFINITY;
  let checking = false;
  let closed = true;
  let lastActivity = 0;
  let hb: ReturnType<typeof setInterval> | undefined;
  let openingTimer: ReturnType<typeof setTimeout> | undefined;
  let reconnectTimer: ReturnType<typeof setTimeout> | undefined;
  let upgradeTimer: ReturnType<typeof setTimeout> | undefined;
  // Bumped on every open, connect, reconnect and close, so a session check that
  // answers after any of those is ignored.
  let generation = 0;

  function status(s: PushStatus, detail?: PushStatusDetail): void {
    if (detail === undefined) opts.onStatus?.(s, transport);
    else opts.onStatus?.(s, transport, detail);
  }

  /** The stream opened: report it, naming whether the catch-up should be staggered. */
  function reportOpen(): void {
    const spreadCatchUp = failures >= 2 || restarted;
    restarted = false;
    lastCloseKind = 'drop';
    openedAt = nowFn();
    status('open', { spreadCatchUp });
  }

  function emit(c: StateChange): void {
    for (const h of handlers) h(c);
  }

  function handleData(data: unknown): void {
    if (typeof data !== 'string') return;
    let obj: unknown;
    try {
      obj = JSON.parse(data);
    } catch {
      return;
    }
    const changed = isStateChange(obj);
    const mailbox = isMailboxChange(obj);
    const ping = typeof obj === 'object' && obj !== null && (obj as Record<string, unknown>)['@type'] === 'ping';
    if (!changed && !mailbox && !ping) return;
    lastActivity = nowFn();
    attemptsOnRung = 0;
    automaticUpgrades = 0;
    if (isStateChange(obj)) emit(obj);
    else if (isMailboxChange(obj)) for (const h of mailboxHandlers) h(obj);
  }

  function stopHeartbeat(): void {
    if (hb !== undefined) {
      clearInterval(hb);
      hb = undefined;
    }
  }

  function startHeartbeat(): void {
    stopHeartbeat();
    lastActivity = nowFn();
    hb = setInterval(() => {
      if (nowFn() - lastActivity > heartbeatMs * 2) {
        // Open but silent past two intervals — treat as dead and reconnect.
        handleDrop();
        return;
      }
      if (current?.kind === 'ws') {
        try {
          current.ws.send(PING);
        } catch {
          handleDrop();
        }
      }
    }, heartbeatMs);
  }

  function clearOpening(): void {
    if (openingTimer !== undefined) { clearTimeout(openingTimer); openingTimer = undefined; }
  }

  function clearCurrent(): void {
    clearOpening();
    if (current === null) return;
    if (current.kind === 'ws') {
      const { ws } = current;
      ws.onopen = ws.onmessage = ws.onerror = ws.onclose = null;
      try {
        ws.close();
      } catch {
        // already closed
      }
    } else {
      const { es } = current;
      es.onopen = es.onmessage = es.onerror = null;
      try {
        es.close();
      } catch {
        // already closed
      }
    }
    current = null;
  }

  function clearReconnect(): void {
    if (reconnectTimer !== undefined) {
      clearTimeout(reconnectTimer);
      reconnectTimer = undefined;
    }
  }
  function clearUpgrade(): void {
    if (upgradeTimer !== undefined) {
      clearTimeout(upgradeTimer);
      upgradeTimer = undefined;
    }
  }

  /** Delay before the next attempt. A fixed `backoff` schedule (tests) is not jittered. */
  function nextDelay(attempt: number, rungSwitch: boolean): number {
    if (rungSwitch && opts.rungDelayMs !== undefined) return opts.rungDelayMs;
    if (opts.backoff !== undefined) {
      if (rungSwitch) return rungDelayMs;
      const i = Math.min(Math.max(attempt - 1, 0), opts.backoff.length - 1);
      return opts.backoff[i] ?? 0;
    }
    // Full jitter on the count of failures across all rungs, so thousands of
    // clients do not reconnect to a restarting server in the same second.
    return reconnectDelay(lastCloseKind, failures, random, backoffConfig);
  }

  /** An explicit connect/reconnect starts a fresh retry budget. */
  function resetFailures(): void {
    failures = 0;
    restarted = false;
    lastCloseKind = 'drop';
    openedAt = undefined;
    lastCheckAt = Number.NEGATIVE_INFINITY;
  }

  /** The server ended the session: stop, and do not reconnect. */
  function endUnauthorized(): void {
    if (closed) return;
    closed = true;
    generation += 1;
    stopHeartbeat();
    clearReconnect();
    clearUpgrade();
    clearCurrent();
    transport = 'offline';
    status('unauthorized');
  }

  /**
   * Ask whether the session survived; the reconnect proceeds meanwhile. Runs only
   * from the second consecutive failure, one at a time, and at most once per
   * `sessionCheckMinMs`, so a restart does not send every client an extra request.
   */
  function verifySession(): void {
    if (failures < 2 || checking) return;
    const at = nowFn();
    if (at - lastCheckAt < sessionCheckMinMs) return;
    lastCheckAt = at;
    checking = true;
    const asked = generation;
    checkSession().then(
      (result) => {
        checking = false;
        if (result === 'unauthorized' && asked === generation) endUnauthorized();
      },
      () => {
        checking = false;
      },
    );
  }

  /** A transport dropped (failed to open or closed after opening). */
  function handleDrop(kind: CloseKind = 'drop'): void {
    if (closed) return;
    stopHeartbeat();
    clearCurrent();
    if (openedAt !== undefined && nowFn() - openedAt >= stableAfterMs) failures = 0;
    openedAt = undefined;
    failures = Math.min(failures + 1, RECONNECT_MAX_FAILURES);
    if (kind === 'restart' || kind === 'busy') restarted = true;
    // A refused handshake after 1013 is still the server asking for time.
    lastCloseKind = kind === 'drop' && lastCloseKind === 'busy' ? 'busy' : kind;
    verifySession();
    status('reconnecting');
    attemptsOnRung += 1;
    if (attemptsOnRung < maxAttemptsPerRung) {
      const delay = nextDelay(attemptsOnRung, false);
      clearReconnect();
      reconnectTimer = setTimeout(() => openRung(ladderIndex), delay);
      return;
    }
    attemptsOnRung = 0;
    if (ladderIndex < TRANSPORT_LADDER.length - 1) {
      const next = ladderIndex + 1;
      const delay = nextDelay(attemptsOnRung, true);
      clearReconnect();
      reconnectTimer = setTimeout(() => openRung(next), delay);
    } else {
      openRung(ladderIndex); // terminal 'poll'
    }
  }

  function openRung(i: number): void {
    if (closed) return;
    ladderIndex = i;
    const rung = TRANSPORT_LADDER[i];
    if (rung === 'ws') openWs();
    else if (rung === 'sse') openSse();
    else enterPoll();
  }

  function openWs(): void {
    if (WS === undefined) {
      handleDrop();
      return;
    }
    status(attemptsOnRung > 0 ? 'reconnecting' : 'connecting');
    let ws: WebSocketLike;
    try {
      ws = new WS(wsUrl);
    } catch {
      handleDrop();
      return;
    }
    current = { kind: 'ws', ws };
    openingTimer = setTimeout(handleDrop, connectTimeoutMs);
    ws.onopen = () => {
      clearOpening();
      generation += 1;
      transport = 'ws';
      reportOpen();
      startHeartbeat();
    };
    ws.onmessage = (ev) => handleData(ev.data);
    ws.onerror = () => {
      /* onclose follows */
    };
    ws.onclose = (ev) => {
      const kind = closeKind((ev as { code?: unknown } | null)?.code);
      if (kind === 'ended') endUnauthorized();
      else handleDrop(kind);
    };
  }

  function openSse(): void {
    if (ES === undefined) {
      handleDrop();
      return;
    }
    status(attemptsOnRung > 0 ? 'reconnecting' : 'connecting');
    let es: EventSourceLike;
    try {
      es = new ES(sseUrl, { withCredentials: true });
    } catch {
      handleDrop();
      return;
    }
    current = { kind: 'sse', es };
    openingTimer = setTimeout(handleDrop, connectTimeoutMs);
    es.onopen = () => {
      clearOpening();
      generation += 1;
      transport = 'sse';
      reportOpen();
      startHeartbeat();
    };
    es.onmessage = (ev) => handleData(ev.data);
    es.onerror = () => handleDrop();
  }

  function enterPoll(): void {
    transport = 'poll';
    status('degraded');
    // Keep on-demand refresh available without endlessly retrying blocked endpoints.
    // An explicit reconnect (or new sign-in) starts a fresh recovery budget.
    clearUpgrade();
    if (automaticUpgrades >= maxAutomaticUpgrades) return;
    const delay = jittered(Math.min(upgradeIntervalMs * 2 ** automaticUpgrades, 300_000), POLL_JITTER, random);
    upgradeTimer = setTimeout(() => {
      automaticUpgrades += 1;
      attemptsOnRung = 0;
      openRung(0);
    }, delay);
  }

  return {
    connect(): void {
      if (!closed) return;
      closed = false;
      automaticUpgrades = 0;
      resetFailures();
      generation += 1;
      attemptsOnRung = 0;
      ladderIndex = 0;
      openRung(0);
    },
    reconnect(): void {
      closed = false;
      automaticUpgrades = 0;
      resetFailures();
      generation += 1;
      stopHeartbeat();
      clearReconnect();
      clearUpgrade();
      clearCurrent();
      attemptsOnRung = 0;
      openRung(0);
    },
    close(): void {
      closed = true;
      generation += 1;
      stopHeartbeat();
      clearReconnect();
      clearUpgrade();
      clearCurrent();
      transport = 'offline';
      status('closed');
    },
    onStateChange(handler): () => void {
      handlers.add(handler);
      return () => handlers.delete(handler);
    },
    onMailboxChange(handler): () => void {
      mailboxHandlers.add(handler);
      return () => mailboxHandlers.delete(handler);
    },
    transport(): PushTransport {
      return transport;
    },
  };
}

// Sub-path hosting (t20 B4): both realtime endpoints carry the deploy prefix.
// `withBase` is the identity function at the origin root, so the URLs these two
// produce there are byte-identical to the pre-sub-path ones.

/** Default WS endpoint — absolute, because `WebSocket` has no relative form. */
function defaultWsUrl(): string {
  const path = withBase('/jmap/ws');
  if (typeof location === 'undefined') return path;
  const scheme = location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${scheme}//${location.host}${path}`;
}

/** Default SSE endpoint — same-origin, so a path suffices. */
function defaultSseUrl(): string {
  return withBase('/jmap/eventsource');
}

/** Append the native `access_token` query param when a bearer is supplied. */
function withBearer(url: string, bearer?: string): string {
  if (bearer === undefined || bearer.length === 0) return url;
  const sep = url.includes('?') ? '&' : '?';
  return `${url}${sep}access_token=${encodeURIComponent(bearer)}`;
}

/**
 * GET the JMAP session resource to learn whether the session still exists. In a
 * native shell `apiFetch` sends it to the configured server with the bearer.
 */
