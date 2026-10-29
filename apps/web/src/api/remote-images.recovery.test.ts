// t82: "Could not update remote image permissions. Please try again."
//
// 1. Root cause: with a linked mailbox open, `apiFetch` refused the grant routes
//    before sending (`LinkedContextUnsupportedError`: they were not in the
//    audited linked scope). They now go out with the child's selector.
// 2. The grant/revoke writes are sent with recovery through `apiFetch`: a
//    transient 503 is retried (they are idempotent), a 428 resumes with the
//    verified account authority, and a final failure carries its reason.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { setNativeTransport } from './http.ts';
import { installLinkedMailbox } from './linkedMailboxSession.ts';
import { linkedRoute } from './linkedAccountRouting.ts';
import { activateAccountPreferences, suspendAccountPreferences } from './accountPreferenceLifecycle.ts';
import { createRemoteImageApi } from './remote-images.ts';
import { recoveryRandom, RecoveryError } from './recovery.ts';

const child = { linkId: 'child-link', accountId: 'child', storageAuthority: 'msa1_child_verified' };
const header = (init: RequestInit | undefined, name: string): string | null => new Headers(init?.headers).get(name);
const scope = { kind: 'single' as const, value: 'm1' };

beforeEach(() => {
  activateAccountPreferences('child', child.storageAuthority);
  document.cookie = 'mw_csrf=token; Path=/';
  recoveryRandom.next = () => 1;
  vi.spyOn(console, 'debug').mockImplementation(() => undefined);
});
afterEach(() => {
  installLinkedMailbox(null);
  suspendAccountPreferences();
  setNativeTransport(null);
  vi.unstubAllGlobals();
  vi.useRealTimers();
  vi.restoreAllMocks();
  recoveryRandom.next = () => Math.random();
  document.cookie = 'mw_csrf=; Max-Age=0; Path=/';
});

describe('remote-image grants in a linked mailbox (the reported failure)', () => {
  it('routes the grant endpoints as mailbox routes', () => {
    expect(linkedRoute('/api/remote-images/grants')).toBe('mailbox');
    expect(linkedRoute('/api/remote-images/grant')).toBe('mailbox');
    expect(linkedRoute('/api/remote-images/revoke')).toBe('mailbox');
    expect(linkedRoute('/api/image-proxy')).toBe('unsupported');
  });

  it('sends grant, revoke and the list with the child selector instead of refusing them', async () => {
    installLinkedMailbox(child);
    const send = vi.fn(async (url: string, _init?: RequestInit) => (url.endsWith('/grants') ? Response.json({ accountId: 'child', list: [] }) : Response.json({ ok: true })));
    vi.stubGlobal('fetch', send);
    const api = createRemoteImageApi();
    await api.grant('child', scope);
    await api.revoke('child', scope);
    await api.listGrants('child');
    expect(send).toHaveBeenCalledTimes(3);
    for (const call of send.mock.calls) expect(header(call[1], 'x-mailwoman-link')).toBe(child.linkId);
    expect(send.mock.calls[0]?.[1]).not.toHaveProperty('recovery');
  });
});

describe('remote-image writes recover', () => {
  it('retries a 503 and succeeds without throwing', async () => {
    vi.useFakeTimers();
    const send = vi.fn(async () => Response.json({ ok: true }));
    send.mockResolvedValueOnce(new Response('unavailable', { status: 503 }));
    vi.stubGlobal('fetch', send);
    const done = createRemoteImageApi().grant('child', scope);
    await vi.runAllTimersAsync();
    await expect(done).resolves.toBeUndefined();
    expect(send).toHaveBeenCalledTimes(2);
  });

  it('resumes a 428 storageAuthorityRequired with the verified authority', async () => {
    const send = vi.fn(async (_url: string, _init?: RequestInit) => Response.json({ ok: true }));
    send.mockResolvedValueOnce(Response.json({ code: 'storageAuthorityRequired' }, { status: 428 }));
    vi.stubGlobal('fetch', send);
    await createRemoteImageApi().grant('child', scope);
    expect(send).toHaveBeenCalledTimes(2);
    expect(header(send.mock.calls[1]?.[1], 'x-mailwoman-storage-authority')).toBe(child.storageAuthority);
  });

  it('throws one error with its reason after the last attempt', async () => {
    vi.useFakeTimers();
    const send = vi.fn(async () => new Response('bad gateway', { status: 502 }));
    vi.stubGlobal('fetch', send);
    const done = createRemoteImageApi().grant('child', scope).catch((e: unknown) => e);
    await vi.runAllTimersAsync();
    const error = await done;
    expect(error).toBeInstanceOf(RecoveryError);
    expect((error as RecoveryError).reason).toBe('server');
    expect(send).toHaveBeenCalledTimes(4);
  });
});
