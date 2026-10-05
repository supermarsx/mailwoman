// 401s as the API client reports them to `onUnauthenticated` listeners.
//
// The endpoints that need a session report a 401; `/api/login` does not, because
// its 401 is a refused credential. The forced-password-change 403 has its own
// listener (client.passwordgate.test.ts) and is not reported here.

import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError, PasswordChangeRequired, createClient } from './client.ts';

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

function stub(res: () => Response): void {
  vi.stubGlobal('fetch', vi.fn(async () => res()));
}

const REQ = { using: [], methodCalls: [] };

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('client — onUnauthenticated', () => {
  it('control: a 200 notifies nobody', async () => {
    stub(() => json(200, { methodResponses: [], sessionState: 's' }));
    const client = createClient('');
    const listener = vi.fn();
    client.onUnauthenticated?.(listener);
    await client.jmap(REQ);
    expect(listener).not.toHaveBeenCalled();
  });

  it('a 401 from each session endpoint notifies and rejects with ApiError 401', async () => {
    stub(() => json(401, { error: 'unauthorized' }));
    const client = createClient('');
    const listener = vi.fn();
    client.onUnauthenticated?.(listener);

    for (const call of [
      () => client.jmap(REQ),
      () => client.session(),
      () => client.me(),
      () => client.sanitize('<p>x</p>'),
    ]) {
      const err = await call().catch((e: unknown) => e);
      expect(err).toBeInstanceOf(ApiError);
      expect((err as ApiError).status).toBe(401);
    }
    expect(listener).toHaveBeenCalledTimes(4);
  });

  it('a refused sign-in is not reported', async () => {
    stub(() => json(401, { error: 'invalid credentials' }));
    const client = createClient('');
    const listener = vi.fn();
    client.onUnauthenticated?.(listener);
    await expect(
      client.login({ jmapUrl: 'https://mail.example.org', username: 'u', password: 'wrong' }),
    ).rejects.toMatchObject({ status: 401, message: 'invalid credentials' });
    expect(listener).not.toHaveBeenCalled();
  });

  it('the forced-password-change 403 and a 500 are not reported', async () => {
    const client = createClient('');
    const listener = vi.fn();
    client.onUnauthenticated?.(listener);

    stub(() => json(403, { error: 'password change required', passwordChangeRequired: true }));
    await expect(client.jmap(REQ)).rejects.toBeInstanceOf(PasswordChangeRequired);
    stub(() => json(500, { error: 'boom' }));
    await expect(client.jmap(REQ)).rejects.toMatchObject({ status: 500 });
    expect(listener).not.toHaveBeenCalled();
  });

  it('an unsubscribed listener is no longer called', async () => {
    stub(() => json(401, { error: 'unauthorized' }));
    const client = createClient('');
    const listener = vi.fn();
    const off = client.onUnauthenticated?.(listener);
    off?.();
    await expect(client.jmap(REQ)).rejects.toMatchObject({ status: 401 });
    expect(listener).not.toHaveBeenCalled();
  });
});
