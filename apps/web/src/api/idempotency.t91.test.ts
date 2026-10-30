// t91: the recovery layer covers creates and linked-mailbox actions.
//
// Per route type:
//   - the layer itself: an `idempotencyKey` is sent as `Idempotency-Key`, the
//     same on every attempt, and makes a POST retryable; a refusal (401) is
//     never retried; a re-applied (rebased) request gets a key of its own;
//   - `keyedAttempt`: a manual retry of the same request after a failure in
//     transit reuses the key; after an answer it does not;
//   - JMAP (`client.jmap`): a create of a covered type is keyed and retried on
//     a 503; mail sends and plain updates are not keyed;
//   - linked mailboxes: link and renew are keyed and retried on a 503, never on
//     a 401; switch is retried; an unlink whose answer was lost (404) is done;
//   - settings records: an identity / saved-search create is keyed, an update
//     is idempotent; a signature upsert is retried;
//   - the UI end: one toast with Retry, which sends the same request again.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { setNativeTransport } from './http.ts';
import { activateAccountPreferences, suspendAccountPreferences } from './accountPreferenceLifecycle.ts';
import { clearKeptKeys, keyedAttempt, keyedWrite, recoverRequest, recoveryRandom, RecoveryError } from './recovery.ts';
import { createClient, isKeyedCreate } from './client.ts';
import { createLinkedAccountsApi, type LinkedMailbox } from './linkedAccounts.ts';
import { SettingsService } from '../screens/Settings/service.ts';
import { showFailure } from '../components/recoverableAction.ts';
import { toast, toasts } from '../components/toast/index.ts';
import type { JmapRequest } from './jmap-types.ts';

const key = (init: RequestInit | undefined): string | null => new Headers(init?.headers).get('idempotency-key');
const unavailable = (): Response => new Response('unavailable', { status: 503 });

beforeEach(() => {
  activateAccountPreferences('acct', 'msa1_parent_authority');
  document.cookie = 'mw_csrf=token; Path=/';
  recoveryRandom.next = () => 1;
  vi.spyOn(console, 'debug').mockImplementation(() => undefined);
  vi.useFakeTimers();
  clearKeptKeys();
});
afterEach(() => {
  suspendAccountPreferences();
  setNativeTransport(null);
  vi.unstubAllGlobals();
  vi.useRealTimers();
  vi.restoreAllMocks();
  recoveryRandom.next = () => Math.random();
  document.cookie = 'mw_csrf=; Max-Age=0; Path=/';
  toast.clear();
});

/** Run `promise` to completion through the backoff timers. */
async function settle<T>(promise: Promise<T>): Promise<T> {
  let out: { value: T } | { error: unknown } | undefined;
  void promise.then((value) => { out = { value }; }, (error: unknown) => { out = { error }; });
  // Web Crypto's digest is not a timer: keep running timers until it settles.
  for (let i = 0; out === undefined && i < 200; i += 1) {
    await vi.runAllTimersAsync();
    if (out === undefined) await new Promise<void>((resolve) => { queueMicrotask(resolve); });
    if (out === undefined) await vi.advanceTimersByTimeAsync(1);
  }
  if (out === undefined) throw new Error('did not settle');
  if ('error' in out) throw out.error;
  return out.value;
}

describe('the recovery layer with an idempotency key', () => {
  it('sends one key on every attempt and retries a POST through two 503s', async () => {
    const send = vi.fn(async (_init: RequestInit | undefined) => Response.json({ ok: true }));
    send.mockResolvedValueOnce(unavailable()).mockResolvedValueOnce(unavailable());
    const response = await settle(recoverRequest('/api/x', { method: 'POST', body: '{}' }, send, keyedWrite('k-1').recovery));
    expect(response.status).toBe(200);
    expect(send).toHaveBeenCalledTimes(3);
    expect(send.mock.calls.map((call) => key(call[0]))).toEqual(['k-1', 'k-1', 'k-1']);
  });

  it('sends a POST without a key once', async () => {
    const send = vi.fn(async () => unavailable());
    const response = await settle(recoverRequest('/api/x', { method: 'POST' }, send, { idempotent: false }));
    expect(response.status).toBe(503);
    expect(send).toHaveBeenCalledTimes(1);
  });

  it('never retries a 401, keyed or not', async () => {
    const send = vi.fn(async () => Response.json({ code: 'linkedPasswordRejected' }, { status: 401 }));
    const response = await settle(recoverRequest('/api/accounts/link', { method: 'POST' }, send, keyedWrite('k-401').recovery));
    expect(response.status).toBe(401);
    expect(send).toHaveBeenCalledTimes(1);
  });

  it('gives a re-applied request a key of its own', async () => {
    const send = vi.fn(async (_init: RequestInit | undefined) => Response.json({ ok: true }));
    send.mockResolvedValueOnce(new Response(null, { status: 412 }));
    await settle(recoverRequest('/api/x', { method: 'PUT', body: 'a' }, send, { ...keyedWrite('k-r').recovery, rebase: async () => ({ method: 'PUT', body: 'b' }) }));
    expect(send.mock.calls.map((call) => key(call[0]))).toEqual(['k-r', 'k-r.r']);
  });
});

describe('keyedAttempt (a manual retry of the same request)', () => {
  it('reuses the key after a failure in transit, and not after an answer', async () => {
    const seen: string[] = [];
    await expect(keyedAttempt('r', async (k) => { seen.push(k); throw new RecoveryError('network'); })).rejects.toThrow();
    await keyedAttempt('r', async (k) => { seen.push(k); });
    await keyedAttempt('r', async (k) => { seen.push(k); });
    expect(seen[0]).toBe(seen[1]);
    expect(seen[2]).not.toBe(seen[1]);
  });

  it('forgets the key after a refusal', async () => {
    const seen: string[] = [];
    await expect(keyedAttempt('q', async (k) => { seen.push(k); throw Object.assign(new Error('no'), { status: 400 }); })).rejects.toThrow();
    await keyedAttempt('q', async (k) => { seen.push(k); });
    expect(seen[0]).not.toBe(seen[1]);
  });

  it('keeps a key for a while, not for ever', async () => {
    let now = 0;
    const seen: string[] = [];
    await expect(keyedAttempt('t', async (k) => { seen.push(k); throw new RecoveryError('server'); }, () => now)).rejects.toThrow();
    now = 10 * 60 * 1000;
    await keyedAttempt('t', async (k) => { seen.push(k); }, () => now);
    expect(seen[0]).not.toBe(seen[1]);
  });
});

describe('JMAP creates', () => {
  const note: JmapRequest = { using: ['urn:ietf:params:jmap:core', 'urn:mailwoman:notes'], methodCalls: [['Note/set', { accountId: 'a', create: { new: { title: 'x' } } }, 's']] };
  const ok = (): Response => Response.json({ methodResponses: [['Note/set', { accountId: 'a', created: { new: { id: 'n1' } } }, 's']], sessionState: 's' });

  it('tells a keyed create from everything else', () => {
    expect(isKeyedCreate(note)).toBe(true);
    for (const name of ['ContactCard/set', 'CalendarEvent/set', 'Task/set', 'MailRule/set', 'Mailbox/set', 'ContactGroup/set']) {
      expect(isKeyedCreate({ using: [], methodCalls: [[name, { accountId: 'a', create: { new: {} } }, 's']] } as JmapRequest)).toBe(true);
    }
    expect(isKeyedCreate({ using: [], methodCalls: [['Note/set', { accountId: 'a', update: { n: {} } }, 's']] } as JmapRequest)).toBe(false);
    expect(isKeyedCreate({ using: [], methodCalls: [['Email/set', { accountId: 'a', create: { d: {} } }, 's']] } as JmapRequest)).toBe(false);
    expect(isKeyedCreate({ using: [], methodCalls: [['Note/set', { accountId: 'a', create: { new: {} } }, 's'], ['EmailSubmission/set', { accountId: 'a', create: { s: {} } }, 'e']] } as JmapRequest)).toBe(false);
    expect(isKeyedCreate({ using: [], methodCalls: [['Note/set', { accountId: 'a', create: { new: {} } }, 's'], ['Note/get', { accountId: 'a', ids: null }, 'g']] } as JmapRequest)).toBe(true);
  });

  it('keys a create and retries it through a 503 with the same key', async () => {
    const send = vi.fn(async (_url: string, _init?: RequestInit) => ok());
    send.mockResolvedValueOnce(unavailable());
    vi.stubGlobal('fetch', send);
    const res = await settle(createClient('').jmap(note));
    expect(res.methodResponses[0]?.[1]).toMatchObject({ created: { new: { id: 'n1' } } });
    expect(send).toHaveBeenCalledTimes(2);
    const keys = send.mock.calls.map((call) => key(call[1]));
    expect(keys[0]).toBeTruthy();
    expect(keys[1]).toBe(keys[0]);
  });

  it('uses the key the caller gives, and sends an unkeyed update once', async () => {
    const send = vi.fn(async (_url: string, _init?: RequestInit) => ok());
    vi.stubGlobal('fetch', send);
    await settle(createClient('').jmap(note, { idempotencyKey: 'given' }));
    expect(key(send.mock.calls[0]?.[1])).toBe('given');
    send.mockReset();
    send.mockResolvedValue(unavailable());
    const update: JmapRequest = { using: note.using, methodCalls: [['Note/set', { accountId: 'a', update: { n1: { title: 'y' } } }, 's']] };
    await expect(settle(createClient('').jmap(update))).rejects.toMatchObject({ status: 503 });
    expect(send).toHaveBeenCalledTimes(1);
    expect(key(send.mock.calls[0]?.[1])).toBeNull();
  });
});

describe('linked-mailbox actions', () => {
  const row: LinkedMailbox = { id: 'link-1', accountId: '1', address: 'child@example.org', displayName: 'Child', storageAuthority: 'msa1_child', status: 'ready', revision: 'r1' };
  const created = (): Response => Response.json({ account: row }, { status: 201 });

  it('links with a key and retries through a 503 with the same key', async () => {
    const send = vi.fn(async (_url: string, _init?: RequestInit) => created());
    send.mockResolvedValueOnce(unavailable());
    vi.stubGlobal('fetch', send);
    const api = createLinkedAccountsApi();
    try {
      await settle(api.link({ username: 'child@example.org', password: 'secret' }));
      expect(send).toHaveBeenCalledTimes(2);
      expect(key(send.mock.calls[0]?.[1])).toBeTruthy();
      expect(key(send.mock.calls[1]?.[1])).toBe(key(send.mock.calls[0]?.[1]));
    } finally { api.dispose(); }
  });

  it('never resends a password the server refused (401)', async () => {
    const send = vi.fn(async () => Response.json({ code: 'linkedPasswordRejected' }, { status: 401 }));
    vi.stubGlobal('fetch', send);
    const api = createLinkedAccountsApi();
    try {
      await expect(settle(api.renew('link-1', { password: 'wrong' }))).rejects.toMatchObject({ status: 401, code: 'linkedPasswordRejected' });
      expect(send).toHaveBeenCalledTimes(1);
    } finally { api.dispose(); }
  });

  it('pressing Link again after a failure in transit reuses the key', async () => {
    const send = vi.fn(async (_url: string, _init?: RequestInit) => unavailable());
    vi.stubGlobal('fetch', send);
    const api = createLinkedAccountsApi();
    try {
      const input = { username: 'child@example.org', password: 'secret' };
      await expect(settle(api.link(input))).rejects.toMatchObject({ status: 503 });
      send.mockResolvedValue(created());
      await settle(api.link(input));
      const keys = new Set(send.mock.calls.map((call) => key(call[1])));
      expect(keys.size).toBe(1);
    } finally { api.dispose(); }
  });

  it('retries a switch, and treats a lost unlink answer (404) as done', async () => {
    const send = vi.fn(async (url: string, _init?: RequestInit) => (url.endsWith('/switch') ? Response.json({ context: { linkId: 'link-1' } }) : new Response(null, { status: 404 })));
    send.mockResolvedValueOnce(unavailable());
    vi.stubGlobal('fetch', send);
    const api = createLinkedAccountsApi();
    try {
      await settle(api.authorize('link-1'));
      expect(send).toHaveBeenCalledTimes(2);
      send.mockClear();
      send.mockResolvedValueOnce(unavailable());
      await expect(settle(api.unlink(row))).resolves.toBeUndefined();
      expect(send).toHaveBeenCalledTimes(2);
      expect(send.mock.calls[1]?.[1]?.method).toBe('DELETE');
    } finally { api.dispose(); }
  });

  it('a lost unlink answer whose retry is refused 412 counts as done once the link is no longer listed', async () => {
    let listed: LinkedMailbox[] = [];
    const send = vi.fn(async (_url: string, init?: RequestInit) => {
      if (init?.method === 'DELETE') return Response.json({ code: 'revisionMismatch' }, { status: 412 });
      return Response.json({ primary: { accountId: 'acct', address: 'p@example.org', storageAuthority: 'msa1_parent_authority' }, accounts: listed, policy: { enabled: true, maxAccounts: 4, supportedAuthentication: [] } });
    });
    send.mockResolvedValueOnce(unavailable());
    vi.stubGlobal('fetch', send);
    const api = createLinkedAccountsApi();
    try {
      await expect(settle(api.unlink(row))).resolves.toBeUndefined();
      // Still listed: the 412 is a real conflict and is reported.
      listed = [row];
      await expect(settle(api.unlink(row))).rejects.toMatchObject({ status: 412 });
    } finally { api.dispose(); }
  });
});

describe('settings records', () => {
  it('keys an identity create and retries it; an update is idempotent and unkeyed', async () => {
    const send = vi.fn(async (_url: string, _init?: RequestInit) => Response.json({ ok: true, id: 'i1' }));
    send.mockResolvedValueOnce(unavailable());
    vi.stubGlobal('fetch', send);
    const service = new SettingsService();
    await settle(service.upsertIdentity({ id: '', name: 'Desk', email: 'a@example.org' }));
    expect(send).toHaveBeenCalledTimes(2);
    expect(key(send.mock.calls[0]?.[1])).toBeTruthy();
    expect(key(send.mock.calls[1]?.[1])).toBe(key(send.mock.calls[0]?.[1]));
    send.mockClear();
    send.mockResolvedValueOnce(unavailable());
    await settle(service.upsertIdentity({ id: 'i1', name: 'Desk', email: 'a@example.org' }));
    expect(send).toHaveBeenCalledTimes(2);
    expect(key(send.mock.calls[0]?.[1])).toBeNull();
  });

  it('keys a saved-search create; retries a signature upsert', async () => {
    const send = vi.fn(async (_url: string, _init?: RequestInit) => Response.json({ ok: true, id: 's1' }));
    send.mockResolvedValueOnce(unavailable());
    vi.stubGlobal('fetch', send);
    const service = new SettingsService();
    await settle(service.upsertSavedSearch({ id: '', name: 'Bills', queryJson: '{}', asFolder: true }));
    expect(key(send.mock.calls[1]?.[1])).toBe(key(send.mock.calls[0]?.[1]));
    send.mockClear();
    send.mockResolvedValueOnce(unavailable());
    await settle(service.upsertSignature({ name: 'Work', body: 'Hi' } as never));
    expect(send).toHaveBeenCalledTimes(2);
  });
});

describe('the failure toast', () => {
  it('shows one toast with Retry; Retry repeats the action and a success removes it', async () => {
    vi.useRealTimers();
    let fail = true;
    const retry = vi.fn(async () => { if (fail) throw new RecoveryError('network'); });
    showFailure({ key: 'probe', error: new RecoveryError('network'), message: (reason) => `Could not. ${reason}`, retry });
    showFailure({ key: 'probe', error: new RecoveryError('network'), message: (reason) => `Could not. ${reason}`, retry });
    expect(toasts().filter((item) => item.id === 'recover:probe')).toHaveLength(1);
    const shown = toasts().find((item) => item.id === 'recover:probe')!;
    expect(shown.message).toBe('Could not. The server could not be reached.');
    shown.action!.run();
    await vi.waitFor(() => expect(retry).toHaveBeenCalledTimes(1));
    await vi.waitFor(() => expect(toasts().filter((item) => item.id === 'recover:probe')).toHaveLength(1));
    fail = false;
    toasts().find((item) => item.id === 'recover:probe')!.action!.run();
    await vi.waitFor(() => expect(toasts().some((item) => item.id === 'recover:probe')).toBe(false));
  });

  it('shows nothing for an abort', () => {
    showFailure({ key: 'gone', error: new DOMException('x', 'AbortError'), message: () => 'x', retry: async () => undefined });
    expect(toasts().some((item) => item.id === 'recover:gone')).toBe(false);
  });
});
