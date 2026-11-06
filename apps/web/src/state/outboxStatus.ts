// What the Outbox reader's status header and the Outbox bell say about one
// submission: a plain label kind, when it is due, and the retry facts.

import type { OutboxSubmission } from './slices/outbox.ts';
import { outboxStateOf } from './slices/outbox.ts';

export type OutboxKind = 'held' | 'queued' | 'scheduled' | 'sending' | 'retrying' | 'failed' | 'uncertain' | 'sent' | 'canceled';

export interface OutboxStatus {
  kind: OutboxKind;
  /** When the engine will send it, if it is waiting. */
  dueAt: string | null;
  /** Whole seconds until `dueAt`, never below 0; null when there is no due time. */
  secondsLeft: number | null;
  attempts: number;
  lastError: string;
  /** The next retry after a failed attempt (the backoff), if the engine has one. */
  nextAttemptAt: string | null;
  /** The send time can be edited (a plain scheduled row, not a hold). */
  canEditTime: boolean;
  /** Recall, send now and cancel apply (the row is still waiting). */
  waiting: boolean;
  /** "Send again" applies (the engine gave up). */
  canSendAgain: boolean;
}

export function outboxStatus(sub: OutboxSubmission, now = Date.now()): OutboxStatus {
  const state = outboxStateOf(sub, now);
  const dueAt = sub.mailwomanDueAt ?? sub.sendAt ?? null;
  const dueMs = dueAt === null ? Number.NaN : new Date(dueAt).getTime();
  const secondsLeft = Number.isFinite(dueMs) ? Math.max(0, Math.ceil((dueMs - now) / 1000)) : null;
  const attempts = sub.mailwomanAttempts ?? 0;
  const waiting = state === 'held' || state === 'scheduled' || state === 'holding';
  let kind: OutboxKind;
  switch (state) {
    case 'held': kind = 'held'; break;
    case 'scheduled': kind = 'scheduled'; break;
    case 'holding':
      // Due and still pending: the engine is on it, or backing off after a failed attempt.
      kind = attempts > 0 ? 'retrying' : secondsLeft !== null && secondsLeft > 0 ? 'queued' : 'sending';
      break;
    default: kind = state;
  }
  return {
    kind,
    dueAt,
    secondsLeft,
    attempts,
    lastError: sub.mailwomanLastError ?? '',
    nextAttemptAt: sub.mailwomanNextAttemptAt ?? null,
    canEditTime: state === 'scheduled',
    waiting,
    canSendAgain: state === 'failed',
  };
}

/** Waiting rows, soonest first (held rows have no time and sort last). */
export function queuedRows(subs: readonly OutboxSubmission[], now = Date.now()): OutboxSubmission[] {
  const waiting = subs.filter((s) => outboxStatus(s, now).waiting);
  const key = (s: OutboxSubmission): number => {
    const due = outboxStatus(s, now).dueAt;
    const ms = due === null ? Number.NaN : new Date(due).getTime();
    return Number.isFinite(ms) ? ms : Number.POSITIVE_INFINITY;
  };
  return waiting.sort((a, b) => key(a) - key(b));
}
