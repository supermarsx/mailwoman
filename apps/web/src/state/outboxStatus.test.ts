import { describe, expect, it } from 'vitest';
import { outboxStatus, queuedRows } from './outboxStatus.ts';
import type { OutboxSubmission } from './slices/outbox.ts';

const NOW = Date.parse('2026-11-06T10:00:00Z');
const base: OutboxSubmission = { id: 's', emailId: 'e', identityId: null, sendAt: null, undoStatus: 'pending', mailwomanHoldSeconds: 0 };
const at = (seconds: number): string => new Date(NOW + seconds * 1000).toISOString();

describe('outboxStatus', () => {
  it('a future sendAt is scheduled and editable, with a countdown', () => {
    const s = outboxStatus({ ...base, sendAt: at(3600), mailwomanDueAt: at(3600) }, NOW);
    expect(s).toMatchObject({ kind: 'scheduled', secondsLeft: 3600, canEditTime: true, waiting: true });
  });
  it('an undo window is queued, not editable', () => {
    const s = outboxStatus({ ...base, mailwomanHoldSeconds: 30, mailwomanDueAt: at(20) }, NOW);
    expect(s).toMatchObject({ kind: 'queued', secondsLeft: 20, canEditTime: false });
  });
  it('a due pending row is sending; after a failed attempt it is retrying with its backoff and error', () => {
    expect(outboxStatus({ ...base, mailwomanDueAt: at(-5) }, NOW).kind).toBe('sending');
    const r = outboxStatus({ ...base, mailwomanDueAt: at(-5), mailwomanAttempts: 2, mailwomanLastError: '421 try later', mailwomanNextAttemptAt: at(120) }, NOW);
    expect(r).toMatchObject({ kind: 'retrying', attempts: 2, lastError: '421 try later', nextAttemptAt: at(120) });
  });
  it('a held row is waiting but cannot have its time edited', () => {
    expect(outboxStatus({ ...base, mailwomanHold: 'manual' }, NOW)).toMatchObject({ kind: 'held', waiting: true, canEditTime: false });
  });
  it('a given-up row is failed and can be sent again; uncertain cannot', () => {
    const failed = outboxStatus({ ...base, undoStatus: 'canceled', mailwomanFailed: true, mailwomanLastError: '550 no' }, NOW);
    expect(failed).toMatchObject({ kind: 'failed', canSendAgain: true, waiting: false, lastError: '550 no' });
    expect(outboxStatus({ ...base, mailwomanDeliveryUncertain: true }, NOW)).toMatchObject({ kind: 'uncertain', canSendAgain: false });
  });
  it('queuedRows lists waiting rows soonest first, held last', () => {
    const rows: OutboxSubmission[] = [
      { ...base, id: 'held', mailwomanHold: 'manual' },
      { ...base, id: 'late', sendAt: at(7200), mailwomanDueAt: at(7200) },
      { ...base, id: 'sent', undoStatus: 'final' },
      { ...base, id: 'soon', sendAt: at(60), mailwomanDueAt: at(60) },
    ];
    expect(queuedRows(rows, NOW).map((r) => r.id)).toEqual(['soon', 'late', 'held']);
  });
});
