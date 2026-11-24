// FROZEN realtime push contract (plan §2.2). Implemented by e6
// (state/slices/realtime.ts + realtime/**); the server side is mw-server (e10),
// fed by the engine `StateChange` broadcast. Both sides agree on this wire shape.

import type { Id } from '../api/jmap-types.ts';

/** Per-datatype state strings a `StateChange` reports for one account. The V3
 *  PIM datatypes (plan §1.8/§2.2) are additive + optional, so V2 push handling
 *  is unchanged; e10 registers the PIM refetch handlers. */
export interface TypeStates {
  Settings?: string;
  Session?: string;
  Notification?: string;
  WebVersion?: string;
  Email?: string;
  Mailbox?: string;
  EmailSubmission?: string;
  Thread?: string;
  // ── V3 PIM datatypes (§2.2) ──
  Calendar?: string;
  CalendarEvent?: string;
  Task?: string;
  Note?: string;
  AddressBook?: string;
  ContactCard?: string;
  ContactGroup?: string;
}

/**
 * The RFC 8887 `StateChange` object pushed over `/jmap/ws` and (identically) as
 * EventSource `data:` frames on `/jmap/eventsource`. On receipt the client calls
 * the matching per-type changes method and refetches.
 */
export interface StateChange {
  '@type': 'StateChange';
  changed: Record<Id, TypeStates>;
}

/**
 * Mailwoman extension frame (t46), on the same stream: a linked mailbox's
 * change, or the sign-in mailbox's (`link: null`) while the stream carries
 * links. It names the link and the mailbox's opaque storage authority, never a
 * raw account id. `changed` is empty for a counts-only frame (a link joining
 * the stream); `inboxUnread` is the Inbox unread count, when known.
 */
export interface MailboxChange {
  '@type': 'MailboxChange';
  link: string | null;
  storageAuthority: string;
  changed: TypeStates;
  inboxUnread?: number;
}

/** The transport actually in use, for the connection-status toast (e6). */
export type PushTransport = 'ws' | 'sse' | 'poll' | 'offline';

/** Heartbeat/keepalive interval and reconnect policy (plan §2.2). */
export const HEARTBEAT_MS = 30_000;

/**
 * Reconnect policy (t35-e4). Delays use full jitter: uniform in [0, ceiling],
 * ceiling = min(cap, base * 2^(consecutive failures - 1)). Close 1012 (server
 * restarting) spreads the first retry over RESTART_SPREAD_MS; close 1013 (try
 * again later) waits at least that long. A connection that stayed open for
 * RECONNECT_STABLE_MS starts the count again.
 */
export const RECONNECT_BASE_MS = 1_000;
export const RECONNECT_CAP_MS = 60_000;
export const RESTART_SPREAD_MS = 15_000;
export const RECONNECT_STABLE_MS = 30_000;
/** Largest failure count kept; 2^(n-1) * base is far above the cap by then. */
export const RECONNECT_MAX_FAILURES = 16;
/** The session check after a drop runs at most this often. */
export const SESSION_CHECK_MIN_MS = 30_000;
/** A catch-up that follows a restart or repeated failures starts within this window. */
export const CATCHUP_SPREAD_MS = 2_000;
/** Poll-rung interval jitter, as a fraction either side (0.2 = plus or minus 20 %). */
export const POLL_JITTER = 0.2;

/** The fallback ladder when a transport fails: WS → SSE → poll. */
export const TRANSPORT_LADDER: readonly PushTransport[] = ['ws', 'sse', 'poll'];

/** The push client e6 implements over the frozen transports. */
export interface PushClient {
  /** Open the best available transport and begin receiving `StateChange`s. */
  connect(): void;
  /** Tear down the connection (logout / teardown). */
  close(): void;
  /** Subscribe to decoded state changes; returns an unsubscribe fn. */
  onStateChange(handler: (change: StateChange) => void): () => void;
  /** The transport currently in use. */
  transport(): PushTransport;
}
