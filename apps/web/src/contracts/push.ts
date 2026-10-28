// FROZEN realtime push contract (plan §2.2). Implemented by e6
// (state/slices/realtime.ts + realtime/**); the server side is mw-server (e10),
// fed by the engine `StateChange` broadcast. Both sides agree on this wire shape.

import type { Id } from '../api/jmap-types.ts';

/** Per-datatype state strings a `StateChange` reports for one account. The V3
 *  PIM datatypes (plan §1.8/§2.2) are additive + optional, so V2 push handling
 *  is unchanged; e10 registers the PIM refetch handlers. */
export interface TypeStates {
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
