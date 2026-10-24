// When a message the user is looking at becomes read (t28-e16, defect D-16).
//
// Every automatic `$seen` write goes through `decideMarkRead`. Before this file
// nothing wrote `$seen` at all: a message opened in the reader stayed unread on
// the server forever, and the folder's unread count never moved.
//
// The decision is a pure function of a policy and an event, so the next layer
// (t32-e47: admin and user conditions — dwell, on open, on scroll-past, never,
// per folder) extends `MarkReadPolicy` and `MarkReadEvent` here and leaves the
// caller in `mail.ts` alone. Today the policy comes from the per-device
// preferences (`markRead`, `markReadDelayMs` in screens/Settings/prefs.ts).

import { loadPrefs, MAX_MARK_READ_DELAY_MS, type MarkReadMode } from '../../screens/Settings/prefs.ts';
import type { Email, Id } from '../../api/jmap-types.ts';

/** How automatic mark-read behaves. */
export interface MarkReadPolicy {
  mode: MarkReadMode;
  /** How long the message must stay open before it is marked, in ms. */
  delayMs: number;
}

/** What happened to a message that might make it read. Only `open` exists so far. */
export interface MarkReadEvent {
  kind: 'open';
  email: Email;
  /** The folder the list is showing, or `null` during a search. */
  mailboxId: Id | null;
}

/** Mark the message after `delayMs`, unless it stopped being open; or leave it. */
export type MarkReadDecision = { mark: true; delayMs: number } | { mark: false };

/** The policy in force: the device preferences, read fresh on every call so a
 *  change in Settings applies to the next message opened. */
export function currentMarkReadPolicy(): MarkReadPolicy {
  const prefs = loadPrefs();
  return { mode: prefs.markRead, delayMs: prefs.markReadDelayMs };
}

/** Whether, and after how long, `event` marks its message read under `policy`. */
export function decideMarkRead(policy: MarkReadPolicy, event: MarkReadEvent): MarkReadDecision {
  if (event.email.keywords?.['$seen'] === true) return { mark: false };
  if (policy.mode === 'never') return { mark: false };
  const delayMs = Number.isFinite(policy.delayMs)
    ? Math.min(Math.max(0, Math.floor(policy.delayMs)), MAX_MARK_READ_DELAY_MS)
    : 0;
  return { mark: true, delayMs };
}
