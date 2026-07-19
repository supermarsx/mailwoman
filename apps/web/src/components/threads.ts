// Conversation threading for the message list (W2).
//
// The engine already assigns every message a JWZ `threadId` (see mw-engine
// `thread.rs`; it arrives on `Email.threadId`). This module is the UI-only half:
// it folds the FLAT, already-sorted `listMessages()` array into a list of
// *visual rows* the virtualizer renders — a singleton message stays one row, a
// multi-message conversation collapses to a single head row that expands to its
// members in place. The output is a flat, uniform-height array so the existing
// §23 100k-row virtualization (one `ROW_HEIGHT`, windowed slice) is unchanged.
//
// Grouping is deliberately conservative so a list with no repeated threadId is
// byte-identical to the pre-threading flat list: a message with no `threadId`
// keys on its own id, so it can never be merged with an unrelated message that
// also lacks one.

import type { Email } from '../api/jmap-types.ts';

export type ThreadRowKind = 'single' | 'head' | 'child';

/** One rendered row of the (virtualized) list. `single` and `child` carry a
 *  single message; `head` is a collapsed conversation whose `email` is the most
 *  recent member (the representative shown collapsed). */
export interface ThreadVisualRow {
  kind: ThreadRowKind;
  /** Group key: `threadId ?? id`. Stable across expand/collapse. */
  key: string;
  /** single/child: the message. head: the representative (latest) message. */
  email: Email;
  /** head: number of messages in the conversation. single/child: 1. */
  count: number;
  /** head: any member is unread. */
  unread: boolean;
  /** head: is the conversation currently expanded. */
  expanded: boolean;
  /** head: distinct sender addresses across the conversation. */
  senderCount: number;
  /** head: any member carries an attachment. */
  hasAttachment: boolean;
}

/** The group key for a message: its threadId, or its own id when the engine
 *  gave it none (so unrelated thread-less messages never merge). */
export function threadKey(email: Email): string {
  return email.threadId ?? email.id;
}

/**
 * Fold `emails` (already sorted: pinned first, then newest-first) into visual
 * rows, honoring the `expanded` set of conversation keys.
 *
 * - A key with one message → one `single` row (identical to the flat list).
 * - A key with several → one `head` row at the position of the group's FIRST
 *   occurrence in `emails` (so a pinned/newest member keeps the conversation
 *   where the flat sort placed it), followed — only when expanded — by its
 *   members in chronological order (oldest first, the natural reading order).
 *
 * The representative (`head.email`) is the member with the latest `receivedAt`,
 * independent of array order, so a head always shows the freshest message.
 */
export function groupThreads(emails: Email[], expanded: ReadonlySet<string>): ThreadVisualRow[] {
  const order: string[] = [];
  const groups = new Map<string, Email[]>();
  for (const e of emails) {
    const key = threadKey(e);
    let arr = groups.get(key);
    if (arr === undefined) {
      arr = [];
      groups.set(key, arr);
      order.push(key);
    }
    arr.push(e);
  }

  const out: ThreadVisualRow[] = [];
  for (const key of order) {
    const members = groups.get(key);
    if (members === undefined || members.length === 0) continue;
    if (members.length === 1) {
      const only = members[0] as Email;
      out.push({
        kind: 'single',
        key,
        email: only,
        count: 1,
        unread: only.keywords?.['$seen'] !== true,
        expanded: false,
        senderCount: 1,
        hasAttachment: only.hasAttachment === true,
      });
      continue;
    }
    const rep = members.reduce((a, b) => (b.receivedAt > a.receivedAt ? b : a), members[0] as Email);
    const isExpanded = expanded.has(key);
    const senders = new Set(members.map((m) => (m.from?.[0]?.email ?? '').toLowerCase()));
    out.push({
      kind: 'head',
      key,
      email: rep,
      count: members.length,
      unread: members.some((m) => m.keywords?.['$seen'] !== true),
      expanded: isExpanded,
      senderCount: senders.size,
      hasAttachment: members.some((m) => m.hasAttachment === true),
    });
    if (isExpanded) {
      const chrono = [...members].sort((a, b) => a.receivedAt.localeCompare(b.receivedAt));
      for (const m of chrono) {
        out.push({
          kind: 'child',
          key,
          email: m,
          count: 1,
          unread: m.keywords?.['$seen'] !== true,
          expanded: false,
          senderCount: 1,
          hasAttachment: m.hasAttachment === true,
        });
      }
    }
  }
  return out;
}
