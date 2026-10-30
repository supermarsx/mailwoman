import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import type { Client } from './client.ts';
import type { JmapSession } from './jmap-types.ts';
import { setSwrCachesEnabled } from './swrCache.ts';
import { clearSessionCache, peekSession, withSessionCache } from './sessionCache.ts';
import { installLinkedMailbox } from './linkedMailboxSession.ts';

function session(state: string, account = 'acc-primary'): JmapSession {
  return {
    capabilities: {}, accounts: { [account]: { name: account } as never }, primaryAccounts: { 'urn:ietf:params:jmap:mail': account },
    username: account, apiUrl: '/jmap/api', downloadUrl: '', uploadUrl: '', eventSourceUrl: '', state,
  };
}

/** A client whose session answer depends on the context it is asked for. */
function fakeClient(answers: Record<string, () => JmapSession>) {
  const unauth = new Set<() => void>();
  let sessionState = 's1';
  const client = {
    login: vi.fn(async () => ({ ok: true })),
    logout: vi.fn(async () => undefined),
    me: vi.fn(),
    session: vi.fn(async (context?: { linkId: string | null }) => answers[context?.linkId ?? 'primary']!()),
    jmap: vi.fn(async () => ({ methodResponses: [], sessionState })),
    sanitize: vi.fn(),
    onNetwork: vi.fn(() => () => undefined),
    onUnauthenticated: vi.fn((l: () => void) => { unauth.add(l); return () => unauth.delete(l); }),
  } as unknown as Client & { session: ReturnType<typeof vi.fn> };
  return { client, setSessionState: (s: string) => { sessionState = s; }, fireUnauthenticated: () => { for (const l of unauth) l(); } };
}

const LINK = { linkId: 'link-1', accountId: 'acc-child', storageAuthority: 'child-authority-0123456789' };

describe('sessionCache (t99)', () => {
  beforeEach(() => { setSwrCachesEnabled(true); installLinkedMailbox(null); clearSessionCache(); });
  afterEach(() => { installLinkedMailbox(null); setSwrCachesEnabled(false); });

  it('one request serves every session() of the same mailbox', async () => {
    const { client } = fakeClient({ primary: () => session('s1') });
    const cached = withSessionCache(client, '');
    const all = await Promise.all([cached.session(), cached.session(), cached.session()]);
    await cached.session();
    expect(all.every((s) => s.state === 's1')).toBe(true);
    expect(client.session).toHaveBeenCalledTimes(1);
  });

  it('keeps each mailbox apart: a linked mailbox never gets the sign-in session', async () => {
    const { client } = fakeClient({ primary: () => session('p1', 'acc-primary'), 'link-1': () => session('c1', 'acc-child') });
    const cached = withSessionCache(client, '');
    expect((await cached.session()).username).toBe('acc-primary');
    installLinkedMailbox(LINK);
    expect((await cached.session()).username).toBe('acc-child');
    // The request named the mailbox it was for.
    expect(client.session).toHaveBeenLastCalledWith({ linkId: 'link-1' });
    installLinkedMailbox(null);
    // Back on the sign-in account? `installLinkedMailbox(null)` after a link is a
    // switch back (previous = the link), so the primary entry is still kept.
    expect((await cached.session()).username).toBe('acc-primary');
    expect(client.session).toHaveBeenCalledTimes(2);
  });

  it('a JMAP answer whose sessionState moved drops the kept session', async () => {
    // This server's sessionState is not the session's own state ('session-0'),
    // so only a change between answers counts.
    let state = 'session-0';
    const { client, setSessionState } = fakeClient({ primary: () => session(state) });
    const cached = withSessionCache(client, '');
    await cached.session();
    await cached.jmap({ using: [], methodCalls: [] });
    await cached.jmap({ using: [], methodCalls: [] });
    expect(peekSession('', null)?.state).toBe('session-0');
    state = 's2';
    setSessionState('s2');
    await cached.jmap({ using: [], methodCalls: [] });
    expect(peekSession('', null)).toBeUndefined();
    expect((await cached.session()).state).toBe('s2');
    expect(client.session).toHaveBeenCalledTimes(2);
  });

  it('an explicit mailbox (the switch verifying its target) always asks the server', async () => {
    const { client } = fakeClient({ 'link-1': () => session('c1', 'acc-child') });
    const cached = withSessionCache(client, '');
    await cached.session({ linkId: 'link-1' });
    await cached.session({ linkId: 'link-1' });
    expect(client.session).toHaveBeenCalledTimes(2);
    expect(peekSession('', 'link-1')?.username).toBe('acc-child');
  });

  it('sign-in, sign-out and a 401 forget every kept session', async () => {
    const { client, fireUnauthenticated } = fakeClient({ primary: () => session('s1') });
    const cached = withSessionCache(client, '');
    await cached.session();
    await cached.logout();
    expect(peekSession('', null)).toBeUndefined();
    await cached.session();
    await cached.login({ username: 'u', password: 'p' } as never);
    expect(peekSession('', null)).toBeUndefined();
    await cached.session();
    fireUnauthenticated();
    expect(peekSession('', null)).toBeUndefined();
  });

  it('is a pass-through when caches are off (the unit-test default)', async () => {
    setSwrCachesEnabled(false);
    const { client } = fakeClient({ primary: () => session('s1') });
    expect(withSessionCache(client, '')).toBe(client);
  });

  it('keys by server base, so two servers never share a session', async () => {
    const { client } = fakeClient({ primary: () => session('s1') });
    await withSessionCache(client, '').session();
    await withSessionCache(client, '/mail').session();
    expect(client.session).toHaveBeenCalledTimes(2);
  });
});
