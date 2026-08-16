// Paging half of the `Email/query` builders (t22-e4).
//
// A separate file from `jmap.test.ts` so the pre-paging builder assertions there
// stay untouched — they are the control for "an unpaged call still emits exactly
// what it emitted before", which is asserted here against the same shape.

import { describe, it, expect } from 'vitest';
import { listMailbox, searchEmails } from './jmap.ts';
import type { EmailQueryArgs, JmapRequest } from './jmap-types.ts';

function queryArgs(req: JmapRequest): EmailQueryArgs & Record<string, unknown> {
  const call = req.methodCalls.find((c) => c[0] === 'Email/query');
  if (call === undefined) throw new Error('no Email/query in request');
  return call[1] as EmailQueryArgs & Record<string, unknown>;
}

describe('listMailbox paging window', () => {
  it('emits no position at all when unpaged, so page 1 is byte-identical to pre-paging', () => {
    const args = queryArgs(listMailbox('acct1', 'mbox9'));
    expect(args).toEqual({
      accountId: 'acct1',
      filter: { inMailbox: 'mbox9' },
      sort: [{ property: 'receivedAt', isAscending: false }],
      limit: 50,
      calculateTotal: true,
    });
    expect('position' in args).toBe(false);
    expect('anchor' in args).toBe(false);
  });

  it('carries position and limit for a continuation — the request master never sent', () => {
    const args = queryArgs(listMailbox('acct1', 'mbox9', 50, { position: 50, calculateTotal: false }));
    expect(args.position).toBe(50);
    expect(args.limit).toBe(50);
    // Continuations must NOT re-ask for the total: it cannot change while paging
    // a fixed query and it costs a COUNT(*) over the whole folder each time.
    expect(args.calculateTotal).toBe(false);
  });

  it('omits an explicit position of 0 rather than sending the default', () => {
    const args = queryArgs(listMailbox('acct1', 'mbox9', 50, { position: 0 }));
    expect('position' in args).toBe(false);
  });

  it('carries anchor and anchorOffset, and drops position when both are given', () => {
    const args = queryArgs(
      listMailbox('acct1', 'mbox9', 25, { anchor: 'msg-500', anchorOffset: -10, position: 99 }),
    );
    expect(args.anchor).toBe('msg-500');
    expect(args.anchorOffset).toBe(-10);
    // RFC 8620 §5.5 makes these mutually exclusive; sending both is a client bug,
    // so the builder resolves it rather than leaving the server to arbitrate.
    expect('position' in args).toBe(false);
    expect(args.limit).toBe(25);
  });

  it('omits anchorOffset when it was not asked for', () => {
    const args = queryArgs(listMailbox('acct1', 'mbox9', 50, { anchor: 'msg-1' }));
    expect(args.anchor).toBe('msg-1');
    expect('anchorOffset' in args).toBe(false);
  });

  it('still fetches headers for exactly the queried ids by result reference', () => {
    const req = listMailbox('acct1', 'mbox9', 50, { position: 100 });
    const get = req.methodCalls.find((c) => c[0] === 'Email/get');
    expect(get?.[1]['#ids']).toEqual({ resultOf: 'q', name: 'Email/query', path: '/ids' });
    // One round-trip per page, not one per page plus a hydrate.
    expect(req.methodCalls).toHaveLength(2);
  });
});

describe('searchEmails paging window', () => {
  it('pages identically to listMailbox, preserving the filter', () => {
    const args = queryArgs(searchEmails('acct1', { text: 'from:kim' }, 50, { position: 150, calculateTotal: false }));
    expect(args.filter).toEqual({ text: 'from:kim' });
    expect(args.position).toBe(150);
    expect(args.calculateTotal).toBe(false);
  });

  it('is unchanged when unpaged', () => {
    const args = queryArgs(searchEmails('acct1', { text: 'q' }));
    expect(args).toEqual({
      accountId: 'acct1',
      filter: { text: 'q' },
      sort: [{ property: 'receivedAt', isAscending: false }],
      limit: 50,
      calculateTotal: true,
    });
  });
});
