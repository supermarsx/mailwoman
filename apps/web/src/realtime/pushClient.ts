// WebSocket/EventSource JMAP push client (plan §2.2, §3 e6).
//
// Opens the best available transport and streams RFC 8887 `StateChange` objects
// to subscribers. Transport ladder ws → sse → poll (contracts/push.ts): a WS to
// `/jmap/ws` is primary; on failure it falls to an EventSource on
// `/jmap/eventsource`; on failure it falls to passive polling (transport
// 'poll'), from which it periodically re-attempts the whole ladder to recover.
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
  TRANSPORT_LADDER,
  type PushClient,
  type PushTransport,
  type StateChange,
} from '../contracts/push.ts';
import { withBase } from '../api/basePath.ts';
import { apiFetch } from '../api/http.ts';

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
  /** Reconnect backoff schedule (ms); the last value repeats. */
  backoff?: number[];
  /** Attempts on one rung before falling to the next (default 2). */
  maxAttemptsPerRung?: number;
  /** Delay before trying the next rung down (default 100 ms). */
  rungDelayMs?: number;
  /** While on 'poll', how often to re-attempt the whole ladder (default 30 s). */
  upgradeIntervalMs?: number;
  /** Wall clock (default `Date.now`), injectable for fake-timer tests. */
  now?: () => number;
  /** Lifecycle callback for the connection model / toast. */
  onStatus?: (status: PushStatus, transport: PushTransport) => void;
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
}

const PING = JSON.stringify({ '@type': 'ping' });

function isStateChange(v: unknown): v is StateChange {
  if (typeof v !== 'object' || v === null) return false;
  const o = v as Record<string, unknown>;
  return o['@type'] === 'StateChange' && typeof o['changed'] === 'object' && o['changed'] !== null;
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
  const backoff = opts.backoff ?? [500, 1000, 2000, 4000, 8000];
  const maxAttemptsPerRung = opts.maxAttemptsPerRung ?? 2;
  const rungDelayMs = opts.rungDelayMs ?? 100;
  const upgradeIntervalMs = opts.upgradeIntervalMs ?? 30_000;
  const nowFn = opts.now ?? Date.now;
  const checkSession = opts.checkSession ?? (() => defaultCheckSession(opts.bearer));

  const handlers = new Set<(c: StateChange) => void>();
  let current: Current = null;
  let transport: PushTransport = 'offline';
  let ladderIndex = 0;
  let attemptsOnRung = 0;
  let closed = true;
  let lastActivity = 0;
  let hb: ReturnType<typeof setInterval> | undefined;
  let reconnectTimer: ReturnType<typeof setTimeout> | undefined;
  let upgradeTimer: ReturnType<typeof setTimeout> | undefined;
  // Bumped on every open, connect, reconnect and close, so a session check that
  // answers after any of those is ignored.
  let generation = 0;

  function status(s: PushStatus): void {
    opts.onStatus?.(s, transport);
  }

  function emit(c: StateChange): void {
    for (const h of handlers) h(c);
  }

  function handleData(data: unknown): void {
    lastActivity = nowFn();
    if (typeof data !== 'string') return;
    let obj: unknown;
    try {
      obj = JSON.parse(data);
    } catch {
      return;
    }
    if (isStateChange(obj)) emit(obj);
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

  function clearCurrent(): void {
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

  function backoffFor(attempt: number): number {
    const i = Math.min(Math.max(attempt - 1, 0), backoff.length - 1);
    return backoff[i] ?? 0;
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

  /** Ask whether the session survived the drop; the reconnect proceeds meanwhile. */
  function verifySession(): void {
    const asked = generation;
    checkSession().then(
      (result) => {
        if (result === 'unauthorized' && asked === generation) endUnauthorized();
      },
      () => undefined,
    );
  }

  /** A transport dropped (failed to open or closed after opening). */
  function handleDrop(): void {
    if (closed) return;
    stopHeartbeat();
    clearCurrent();
    verifySession();
    status('reconnecting');
    attemptsOnRung += 1;
    if (attemptsOnRung < maxAttemptsPerRung) {
      const delay = backoffFor(attemptsOnRung);
      clearReconnect();
      reconnectTimer = setTimeout(() => openRung(ladderIndex), delay);
      return;
    }
    attemptsOnRung = 0;
    if (ladderIndex < TRANSPORT_LADDER.length - 1) {
      const next = ladderIndex + 1;
      clearReconnect();
      reconnectTimer = setTimeout(() => openRung(next), rungDelayMs);
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
    ws.onopen = () => {
      generation += 1;
      attemptsOnRung = 0;
      transport = 'ws';
      status('open');
      startHeartbeat();
    };
    ws.onmessage = (ev) => handleData(ev.data);
    ws.onerror = () => {
      /* onclose follows */
    };
    ws.onclose = (ev) => {
      if ((ev as { code?: unknown } | null)?.code === WS_CLOSE_SESSION_ENDED) endUnauthorized();
      else handleDrop();
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
    es.onopen = () => {
      generation += 1;
      attemptsOnRung = 0;
      transport = 'sse';
      status('open');
      startHeartbeat();
    };
    es.onmessage = (ev) => handleData(ev.data);
    es.onerror = () => handleDrop();
  }

  function enterPoll(): void {
    transport = 'poll';
    status('degraded');
    // Passive: the app falls back to on-demand refresh. Periodically re-attempt
    // the full ladder so a recovered network upgrades back to ws/sse.
    clearUpgrade();
    upgradeTimer = setTimeout(() => {
      attemptsOnRung = 0;
      openRung(0);
    }, upgradeIntervalMs);
  }

  return {
    connect(): void {
      if (!closed) return;
      closed = false;
      generation += 1;
      attemptsOnRung = 0;
      ladderIndex = 0;
      openRung(0);
    },
    reconnect(): void {
      closed = false;
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
async function defaultCheckSession(bearer?: string): Promise<SessionCheck> {
  if (typeof fetch !== 'function') return 'unknown';
  try {
    const headers: Record<string, string> = {};
    if (bearer !== undefined && bearer.length > 0) headers['Authorization'] = `Bearer ${bearer}`;
    const res = await apiFetch(withBase('/jmap/session'), { credentials: 'include', headers });
    return res.status === 401 || res.status === 403 ? 'unauthorized' : 'ok';
  } catch {
    return 'unknown';
  }
}
