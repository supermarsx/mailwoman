import { afterEach, describe, expect, it, vi } from 'vitest';
import { createRoot } from 'solid-js';
import { createLinkedLive, type ActiveLinkedMailbox } from './linkedLive.ts';
import { isMailboxChange, type WebSocketLike } from './pushClient.ts';
import type { MailboxChange } from '../contracts/push.ts';
import { createRealtimeSlice } from '../state/slices/realtime.ts';
import { createClient } from '../api/client.ts';
import { installLinkedMailbox, setPrimaryMailboxAuthority } from '../api/linkedMailboxSession.ts';

const PRIMARY = 'msa1_primary';
const CHILD = 'msa1_child';

function frame(over: Partial<MailboxChange> = {}): MailboxChange {
  return {
    '@type': 'MailboxChange',
    link: 'link-1',
    storageAuthority: CHILD,
    changed: { Email: '2', Mailbox: '2', EmailSubmission: '1', Thread: '2' },
    inboxUnread: 3,
    ...over,
  };
}

function harness(active: ActiveLinkedMailbox | null = null) {
  let open = active;
  const onActiveChanged = vi.fn();
  const onAuthorityMismatch = vi.fn();
  const live = createLinkedLive({
    active: () => open,
    primaryAuthority: () => PRIMARY,
    onActiveChanged,
    onAuthorityMismatch,
  });
  return { live, onActiveChanged, onAuthorityMismatch, setOpen: (a: ActiveLinkedMailbox | null) => (open = a) };
}

describe('linked live routing', () => {
  it('refetches the open linked mailbox only for the datatypes that moved', () => {
    const h = harness({ linkId: 'link-1', storageAuthority: CHILD });
    h.live.apply(frame());
    expect(h.onActiveChanged).toHaveBeenLastCalledWith(['Email', 'Mailbox', 'EmailSubmission', 'Thread']);
    h.live.apply(frame({ changed: { Email: '3', Mailbox: '2', EmailSubmission: '1', Thread: '3' } }));
    expect(h.onActiveChanged).toHaveBeenLastCalledWith(['Email', 'Thread']);
    h.live.apply(frame({ changed: { Email: '3', Mailbox: '2', EmailSubmission: '1', Thread: '3' } }));
    expect(h.onActiveChanged).toHaveBeenCalledTimes(2);
  });

  it('a counts-only frame updates the badge and refetches nothing', () => {
    const h = harness({ linkId: 'link-1', storageAuthority: CHILD });
    h.live.apply(frame({ changed: {}, inboxUnread: 7 }));
    expect(h.onActiveChanged).not.toHaveBeenCalled();
    expect(h.live.unread('link-1', CHILD)).toBe(7);
  });

  it('an inactive link only moves its unread count', () => {
    const h = harness(null);
    h.live.apply(frame({ inboxUnread: 4 }));
    expect(h.onActiveChanged).not.toHaveBeenCalled();
    expect(h.live.unread('link-1', CHILD)).toBe(4);
    h.live.apply(frame({ changed: { Email: '9' }, inboxUnread: 5 }));
    expect(h.live.unread('link-1', CHILD)).toBe(5);
    expect(h.live.unread('link-2', CHILD)).toBeNull();
  });

  it('a count is read back only under the authority the caller holds', () => {
    const h = harness(null);
    h.live.apply(frame({ inboxUnread: 4 }));
    expect(h.live.unread('link-1', 'msa1_other')).toBeNull();
    expect(h.live.unread('link-1')).toBeNull();
  });

  it('a frame for the open link under another authority refetches nothing and asks for a re-list', () => {
    const h = harness({ linkId: 'link-1', storageAuthority: CHILD });
    h.live.apply(frame({ storageAuthority: 'msa1_moved' }));
    expect(h.onActiveChanged).not.toHaveBeenCalled();
    expect(h.onAuthorityMismatch).toHaveBeenCalledTimes(1);
  });

  it('the sign-in mailbox is accepted only under the sign-in authority, and never refetches here', () => {
    const h = harness({ linkId: 'link-1', storageAuthority: CHILD });
    h.live.apply(frame({ link: null, storageAuthority: 'msa1_someone', inboxUnread: 9 }));
    expect(h.live.unread(null)).toBeNull();
    h.live.apply(frame({ link: null, storageAuthority: PRIMARY, inboxUnread: 2 }));
    expect(h.live.unread(null)).toBe(2);
    expect(h.onActiveChanged).not.toHaveBeenCalled();
  });

  it('switching to a link that already moved refetches on its next change', () => {
    const h = harness(null);
    h.live.apply(frame());
    h.setOpen({ linkId: 'link-1', storageAuthority: CHILD });
    h.live.apply(frame({ changed: { Email: '5', Mailbox: '5', EmailSubmission: '1', Thread: '5' } }));
    expect(h.onActiveChanged).toHaveBeenLastCalledWith(['Email', 'Mailbox', 'Thread']);
  });

  it('reset forgets every count', () => {
    const h = harness(null);
    h.live.apply(frame());
    h.live.reset();
    expect(h.live.unread('link-1', CHILD)).toBeNull();
  });
});

describe('MailboxChange decoding', () => {
  it('accepts the server frame and rejects malformed ones', () => {
    expect(isMailboxChange(frame())).toBe(true);
    expect(isMailboxChange(frame({ link: null, changed: {} }))).toBe(true);
    for (const bad of [
      { ...frame(), '@type': 'StateChange' },
      { ...frame(), storageAuthority: 'acct-1' },
      { ...frame(), link: '' },
      { ...frame(), link: 7 },
      { ...frame(), changed: null },
      { ...frame(), changed: { Email: 2 } },
      { ...frame(), inboxUnread: -1 },
      { ...frame(), inboxUnread: 1.5 },
    ]) {
      expect(isMailboxChange(bad), JSON.stringify(bad)).toBe(false);
    }
  });
});

class FakeWs implements WebSocketLike {
  static last: FakeWs | undefined;
  onopen: ((ev: unknown) => void) | null = null;
  onmessage: ((ev: { data: unknown }) => void) | null = null;
  onerror: ((ev: unknown) => void) | null = null;
  onclose: ((ev: unknown) => void) | null = null;
  constructor(public url: string) {
    FakeWs.last = this;
  }
  send(): void {}
  close(): void {}
}

describe('realtime slice routes MailboxChange frames from the socket', () => {
  afterEach(() => {
    installLinkedMailbox(null);
    setPrimaryMailboxAuthority(undefined);
    FakeWs.last = undefined;
    vi.unstubAllGlobals();
  });

  it('updates picker counts for inactive mailboxes and refetches the open link', () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('{}', { status: 200 })));
    setPrimaryMailboxAuthority(PRIMARY);
    createRoot((dispose) => {
      const onActiveLinkedChanged = vi.fn();
      const onChanged = vi.fn();
      const slice = createRealtimeSlice(
        { client: createClient(''), showToast: () => undefined },
        { push: { WebSocketImpl: FakeWs, backoff: [60_000] }, onActiveLinkedChanged, onChanged },
      );
      slice.startRealtime();
      const ws = FakeWs.last!;
      ws.onopen?.({});
      const send = (v: unknown) => ws.onmessage?.({ data: JSON.stringify(v) });

      // The sign-in mailbox is open: a child's change is a badge, not a refetch.
      send(frame({ inboxUnread: 6 }));
      send(frame({ link: null, storageAuthority: PRIMARY, inboxUnread: 1, changed: {} }));
      expect(slice.mailboxUnread('link-1', CHILD)).toBe(6);
      expect(slice.mailboxUnread(null)).toBe(1);
      expect(onActiveLinkedChanged).not.toHaveBeenCalled();
      expect(onChanged).not.toHaveBeenCalled();

      // The child is open: its change refetches it.
      installLinkedMailbox({ linkId: 'link-1', accountId: 'child-acct', storageAuthority: CHILD });
      send(frame({ changed: { Email: '8', Mailbox: '8', EmailSubmission: '1', Thread: '8' }, inboxUnread: 7 }));
      expect(onActiveLinkedChanged).toHaveBeenCalledWith(['Email', 'Mailbox', 'Thread']);
      expect(slice.mailboxUnread('link-1', CHILD)).toBe(7);

      // A malformed frame is dropped unseen.
      send({ '@type': 'MailboxChange', link: 'link-1', storageAuthority: 'raw-id', changed: {}, inboxUnread: 99 });
      expect(slice.mailboxUnread('link-1', CHILD)).toBe(7);

      slice.stopRealtime();
      expect(slice.mailboxUnread('link-1', CHILD)).toBeNull();
      dispose();
    });
  });
});
