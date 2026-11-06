// Pure helpers for Admin > Data requests: the due-date countdown, which actions a
// request's status allows, and the on-time share.
import type { AdminDataRequest, DataRequestStats } from '../../../api/admin/dataRequests.ts';

const DAY = 86_400;

/** Whole days until the due date (negative when past); `null` for a request that is answered. */
export function daysLeft(request: Pick<AdminDataRequest, 'dueAt' | 'dueState'>, nowSeconds: number): number | null {
  if (request.dueState === 'none') return null;
  const diff = request.dueAt - nowSeconds;
  return diff >= 0 ? Math.ceil(diff / DAY) : -Math.ceil(-diff / DAY);
}

export interface Actions {
  verify: boolean; approve: boolean; reject: boolean; extend: boolean; complete: boolean;
  hold: boolean; release: boolean; run: boolean;
}

const OPEN = new Set(['awaiting_confirmation', 'awaiting_verification', 'awaiting_approval', 'queued', 'processing', 'scheduled', 'in_progress', 'blocked', 'failed']);

/** What an administrator can do to a request in its current state. */
export function actionsFor(r: Pick<AdminDataRequest, 'status' | 'kind' | 'legalHold'>): Actions {
  const open = OPEN.has(r.status);
  return {
    verify: r.status === 'awaiting_verification' || r.status === 'awaiting_confirmation',
    approve: r.status === 'awaiting_approval' || (r.status === 'blocked' && !r.legalHold.active),
    reject: open && r.status !== 'processing',
    extend: open,
    complete: r.status === 'in_progress',
    hold: open && r.kind === 'erasure' && r.legalHold.source !== 'policy' && !r.legalHold.active,
    release: r.legalHold.active && r.legalHold.source === 'request',
    run: r.status === 'failed' || r.status === 'queued' || r.status === 'scheduled',
  };
}

/** Share of answered requests that were answered by their due date, in whole percent; `null` when none. */
export function onTimePercent(stats: Pick<DataRequestStats, 'answered' | 'onTime'>): number | null {
  return stats.answered === 0 ? null : Math.round((stats.onTime / stats.answered) * 100);
}

/** What a bulk action needs besides the selection: a note, a reason, days or an assignee. */
export type BulkField = 'none' | 'note' | 'reason' | 'extend' | 'assignee';
export const BULK_ACTIONS = ['approve', 'reject', 'extend', 'assign', 'hold', 'release-hold'] as const;
export type BulkKind = (typeof BULK_ACTIONS)[number];
export function bulkField(action: BulkKind): BulkField {
  return action === 'approve' ? 'note' : action === 'reject' || action === 'hold' ? 'reason' : action === 'extend' ? 'extend' : action === 'assign' ? 'assignee' : 'none';
}
/** Whether the bulk form holds what the action needs. */
export function bulkReady(action: BulkKind, input: { text: string; days: number }, count: number): boolean {
  if (count < 1 || count > 200) return false;
  const field = bulkField(action);
  if (field === 'reason') return input.text.trim() !== '';
  if (field === 'extend') return input.text.trim() !== '' && Number.isInteger(input.days) && input.days >= 1;
  return true;
}

/** The CSV file name for the exported log. */
export function logFileName(now: Date): string {
  return `data-requests-${now.toISOString().slice(0, 10)}.csv`;
}
