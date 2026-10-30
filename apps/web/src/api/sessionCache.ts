// The JMAP session resource, read once per mailbox and kept (t99).
//
// Before this every surface asked `/jmap/session` itself: the folder list, the
// quota meter, the outbox, calendar, contacts, notes, tasks, rules, keys and the
// composer, so one mailbox load sent it four to seven times, and calendar sent
// it twice in a row before its first call. The session changes rarely and JMAP
// says when it does: every `/jmap/api` answer carries `sessionState`, the
// session's own `state` (RFC 8620 §3.4). So:
//
//   * `session()` with no explicit context answers from memory (stale-while-
//     revalidate: see `swrCache.ts`), keyed by the server base and the open
//     mailbox (the sign-in account, or the linked mailbox's link id), so one
//     mailbox's session is never handed to another (t87);
//   * a JMAP answer whose `sessionState` differs from the previous answer's
//     drops the kept session, so the next read fetches it again;
//   * `session({linkId})` (the mailbox switch verifies the target with it) always
//     asks the server, and keeps the answer;
//   * sign-in, sign-out, a 401 and the forced-password-change refusal drop
//     everything.

import type { Client } from './client.ts';
import type { JmapSession } from './jmap-types.ts';
import { activeLinkedMailbox, onLinkedMailboxChange } from './linkedMailboxSession.ts';
import { createSwrCache, scopedKey, swrCachesEnabled } from './swrCache.ts';

/** Served without a request for this long… */
export const SESSION_FRESH_MS = 60_000;
/** …and with a background refresh up to this age. */
export const SESSION_MAX_AGE_MS = 10 * 60_000;

const cache = createSwrCache<JmapSession>({ freshMs: SESSION_FRESH_MS, maxAgeMs: SESSION_MAX_AGE_MS });

/** The cache key of a mailbox's session on `base`. `linkId` null = the sign-in account. */
export function sessionKey(base: string, linkId: string | null): string {
  return scopedKey('jmap-session', base, linkId);
}

function currentLinkId(): string | null {
  return activeLinkedMailbox()?.linkId ?? null;
}

/** Forget every kept session (sign-in, sign-out, 401). */
export function clearSessionCache(): void {
  cache.clear();
  seenStates.clear();
}

/** The kept session of a mailbox, without a request (tests, diagnostics). */
export function peekSession(base: string, linkId: string | null): JmapSession | undefined {
  return cache.peek(sessionKey(base, linkId));
}

/** The `sessionState` each kept session was last seen with. */
const seenStates = new Map<string, string>();

/**
 * A JMAP answer's `sessionState` for the mailbox that sent it. RFC 8620 has it
 * equal the session's `state`; this server sends a state that also moves with
 * the account's data (measured: `session-0` vs `e62m61s0p3c0`), so the test is
 * "did it change since the last answer", not "does it equal the kept state": a
 * change drops the kept session, so the next read fetches it again.
 */
export function noteSessionState(base: string, linkId: string | null, sessionState: unknown): void {
  if (typeof sessionState !== 'string' || sessionState === '') return;
  const key = sessionKey(base, linkId);
  const previous = seenStates.get(key);
  seenStates.set(key, sessionState);
  if (previous !== undefined && previous !== sessionState) cache.invalidate(key);
}

let watchingLinks = false;
function watchLinkedMailboxes(): void {
  if (watchingLinks) return;
  watchingLinks = true;
  // A sign-in or sign-out reinstalls the sign-in context (`context` and
  // `previous` both null): whoever signs in next starts with nothing kept.
  // A switch between mailboxes keeps each mailbox's own entry.
  onLinkedMailboxChange((change) => {
    if (change.context === null && change.previous === null) clearSessionCache();
  });
}

/**
 * `client` with its `session()` served from the cache and its JMAP answers
 * checked for a changed `sessionState`. Everything else is `client`'s own.
 */
export function withSessionCache(client: Client, base: string): Client {
  if (!swrCachesEnabled()) return client;
  watchLinkedMailboxes();
  client.onUnauthenticated?.(() => clearSessionCache());
  client.onPasswordChangeRequired?.(() => clearSessionCache());
  return {
    ...client,
    async login(input) {
      clearSessionCache();
      try {
        return await client.login(input);
      } finally {
        clearSessionCache();
      }
    },
    async logout() {
      clearSessionCache();
      try {
        await client.logout();
      } finally {
        clearSessionCache();
      }
    },
    session(context) {
      if (context !== undefined) {
        // An explicit mailbox: the caller is verifying it, so ask the server.
        return cache.refresh(sessionKey(base, context.linkId), () => client.session(context));
      }
      // The request names the mailbox the key names, so a switch while it is in
      // flight cannot store one mailbox's session under another's key.
      const linkId = currentLinkId();
      return cache.get(sessionKey(base, linkId), () => client.session({ linkId }));
    },
    async jmap(body, opts) {
      const linkId = currentLinkId();
      const response = await client.jmap(body, opts);
      if (linkId === currentLinkId()) noteSessionState(base, linkId, response.sessionState);
      return response;
    },
  };
}
