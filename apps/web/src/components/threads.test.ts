import { describe, it, expect } from 'vitest';
import { groupThreads, threadKey } from './threads.ts';
import { mkEmail } from './appHarness.tsx';

const NONE: ReadonlySet<string> = new Set();

describe('groupThreads (W2 conversation folding)', () => {
  it('keys a thread-less message on its own id so unrelated singletons never merge', () => {
    const a = mkEmail('a');
    const b = mkEmail('b');
    expect(threadKey(a)).toBe('a');
    expect(threadKey(b)).toBe('b');
    const rows = groupThreads([a, b], NONE);
    expect(rows.map((r) => r.kind)).toEqual(['single', 'single']);
    expect(rows.map((r) => r.email.id)).toEqual(['a', 'b']);
  });

  it('preserves the flat order for a list with no repeated threadId', () => {
    const emails = Array.from({ length: 5 }, (_, i) => mkEmail(`m${i}`));
    const rows = groupThreads(emails, NONE);
    expect(rows).toHaveLength(5);
    expect(rows.every((r) => r.kind === 'single')).toBe(true);
    expect(rows.map((r) => r.email.id)).toEqual(['m0', 'm1', 'm2', 'm3', 'm4']);
  });

  it('collapses a shared-threadId conversation to one head at its first position', () => {
    const emails = [
      mkEmail('newest', { threadId: 't1', receivedAt: '2026-01-03T00:00:00Z' }),
      mkEmail('mid', { threadId: 't1', receivedAt: '2026-01-02T00:00:00Z' }),
      mkEmail('solo', { receivedAt: '2026-01-02T12:00:00Z' }),
      mkEmail('oldest', { threadId: 't1', receivedAt: '2026-01-01T00:00:00Z' }),
    ];
    const rows = groupThreads(emails, NONE);
    // head (for t1, at its first occurrence) then the solo single.
    expect(rows.map((r) => r.kind)).toEqual(['head', 'single']);
    const head = rows[0]!;
    expect(head.key).toBe('t1');
    expect(head.count).toBe(3);
    // Representative is the newest member regardless of array position.
    expect(head.email.id).toBe('newest');
  });

  it('expands a conversation to its members in chronological order', () => {
    const emails = [
      mkEmail('newest', { threadId: 't1', receivedAt: '2026-01-03T00:00:00Z' }),
      mkEmail('mid', { threadId: 't1', receivedAt: '2026-01-02T00:00:00Z' }),
      mkEmail('oldest', { threadId: 't1', receivedAt: '2026-01-01T00:00:00Z' }),
    ];
    const rows = groupThreads(emails, new Set(['t1']));
    expect(rows.map((r) => r.kind)).toEqual(['head', 'child', 'child', 'child']);
    expect(rows[0]!.expanded).toBe(true);
    // Members oldest → newest under the head.
    expect(rows.slice(1).map((r) => r.email.id)).toEqual(['oldest', 'mid', 'newest']);
  });

  it('aggregates unread, distinct senders, and attachment flags onto the head', () => {
    const emails = [
      mkEmail('a', { threadId: 't1', from: [{ name: null, email: 'alice@x.org' }], keywords: { $seen: true } }),
      mkEmail('b', {
        threadId: 't1',
        from: [{ name: null, email: 'bob@x.org' }],
        keywords: { $seen: false },
        hasAttachment: true,
      }),
    ];
    const head = groupThreads(emails, NONE)[0]!;
    expect(head.kind).toBe('head');
    expect(head.unread).toBe(true); // b is unread
    expect(head.senderCount).toBe(2); // alice + bob
    expect(head.hasAttachment).toBe(true); // b has one
  });

  it('does not merge two thread-less messages that both lack a threadId', () => {
    const rows = groupThreads([mkEmail('x'), mkEmail('y')], NONE);
    expect(rows).toHaveLength(2);
    expect(rows.every((r) => r.kind === 'single')).toBe(true);
  });
});
