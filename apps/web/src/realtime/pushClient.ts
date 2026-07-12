// WebSocket/EventSource JMAP push client (plan §2.2, §3 e6).
//
// Opens the best available transport and streams RFC 8887 `StateChange` objects
// to subscribers. Transport ladder ws → sse → poll (contracts/push.ts): a WS to
// `/jmap/ws` is primary; on failure it falls to an EventSource on
// `/jmap/eventsource`; on failure it falls to passive polling (transport
// 'poll'), from which it periodically re-attempts the whole ladder to recover.
// A 30 s heartbeat detects a dead-but-open socket; reconnects use backoff.
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

/** Lifecycle signal the connection model maps to a `ConnectionState`. */
export type PushStatus = 'connecting' | 'reconnecting' | 'open' | 'degraded' | 'closed';

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
  const wsUrl = opts.wsUrl ?? defaultWsUrl();
  const sseUrl = opts.sseUrl ?? '/jmap/eventsource';
  const WS = opts.WebSocketImpl ?? (globalThis.WebSocket as unknown as WebSocketCtor | undefined);
  const ES =
    opts.EventSourceImpl ?? (globalThis.EventSource as unknown as EventSourceCtor | undefined);
  const heartbeatMs = opts.heartbeatMs ?? HEARTBEAT_MS;
  const backoff = opts.backoff ?? [500, 1000, 2000, 4000, 8000];
  const maxAttemptsPerRung = opts.maxAttemptsPerRung ?? 2;
  const rungDelayMs = opts.rungDelayMs ?? 100;
  const upgradeIntervalMs = opts.upgradeIntervalMs ?? 30_000;
  const nowFn = opts.now ?? Date.now;

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

  /** A transport dropped (failed to open or closed after opening). */
  function handleDrop(): void {
    if (closed) return;
    stopHeartbeat();
    clearCurrent();
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
      attemptsOnRung = 0;
      transport = 'ws';
      status('open');
      startHeartbeat();
    };
    ws.onmessage = (ev) => handleData(ev.data);
    ws.onerror = () => {
      /* onclose follows */
    };
    ws.onclose = () => handleDrop();
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
      attemptsOnRung = 0;
      ladderIndex = 0;
      openRung(0);
    },
    reconnect(): void {
      closed = false;
      stopHeartbeat();
      clearReconnect();
      clearUpgrade();
      clearCurrent();
      attemptsOnRung = 0;
      openRung(0);
    },
    close(): void {
      closed = true;
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

function defaultWsUrl(): string {
  if (typeof location === 'undefined') return '/jmap/ws';
  const scheme = location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${scheme}//${location.host}/jmap/ws`;
}
