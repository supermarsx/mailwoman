// The forced-password-change gate as the API client sees it (t27-e4, OH-1).
//
// mw-server answers every request outside the password-change endpoints with
//   403 {"error":"password change required","passwordChangeRequired":true}
// for an account whose `force_password_change` flag is set. That is not a
// generic failure: the client raises `PasswordChangeRequired` and tells its
// listeners, so the shell can hold the account at the change screen.
//
// A 403 WITHOUT that field stays an ordinary `ApiError` — `POST /api/password`
// itself answers 403 for a wrong current password, and the origin/CSRF guard
// answers 403 too, and neither may be mistaken for the gate.

import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError, PasswordChangeRequired, createClient } from './client.ts';

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

function stub(res: () => Response): void {
  vi.stubGlobal('fetch', vi.fn(async () => res()));
}

const GATE = { error: 'password change required', passwordChangeRequired: true };
const REQ = { using: [], methodCalls: [] };

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('client — passwordChangeRequired', () => {
  it('control: a 200 JMAP response is returned and no listener fires', async () => {
    stub(() => json(200, { methodResponses: [], sessionState: 's' }));
    const client = createClient('');
    const listener = vi.fn();
    client.onPasswordChangeRequired?.(listener);

    await expect(client.jmap(REQ)).resolves.toEqual({ methodResponses: [], sessionState: 's' });
    expect(listener).not.toHaveBeenCalled();
  });

  it('a 403 carrying passwordChangeRequired raises PasswordChangeRequired and notifies', async () => {
    stub(() => json(403, GATE));
    const client = createClient('');
    const listener = vi.fn();
    client.onPasswordChangeRequired?.(listener);

    await expect(client.jmap(REQ)).rejects.toBeInstanceOf(PasswordChangeRequired);
    expect(listener).toHaveBeenCalledTimes(1);

    await expect(client.session()).rejects.toBeInstanceOf(PasswordChangeRequired);
    await expect(client.sanitize('<p>x</p>')).rejects.toBeInstanceOf(PasswordChangeRequired);
    expect(listener).toHaveBeenCalledTimes(3);
  });

  it('is still an ApiError with status 403, so existing status checks keep working', async () => {
    stub(() => json(403, GATE));
    const err = await createClient('')
      .jmap(REQ)
      .catch((e: unknown) => e);
    expect(err).toBeInstanceOf(ApiError);
    expect((err as ApiError).status).toBe(403);
  });

  it('a 403 without the field is a plain ApiError and notifies nobody', async () => {
    stub(() => json(403, { error: 'cross-origin request rejected' }));
    const client = createClient('');
    const listener = vi.fn();
    client.onPasswordChangeRequired?.(listener);

    const err = await client.jmap(REQ).catch((e: unknown) => e);
    expect(err).toBeInstanceOf(ApiError);
    expect(err).not.toBeInstanceOf(PasswordChangeRequired);
    expect(listener).not.toHaveBeenCalled();
  });

  it('a 403 with a non-JSON body is a plain ApiError', async () => {
    stub(() => new Response('forbidden', { status: 403 }));
    const err = await createClient('')
      .jmap(REQ)
      .catch((e: unknown) => e);
    expect(err).toBeInstanceOf(ApiError);
    expect(err).not.toBeInstanceOf(PasswordChangeRequired);
  });

  it('an unsubscribed listener is not called', async () => {
    stub(() => json(403, GATE));
    const client = createClient('');
    const listener = vi.fn();
    const off = client.onPasswordChangeRequired?.(listener);
    off?.();

    await expect(client.jmap(REQ)).rejects.toBeInstanceOf(PasswordChangeRequired);
    expect(listener).not.toHaveBeenCalled();
  });

  it('carries passwordChangeRequired through login() and me()', async () => {
    stub(() => json(200, { username: 'u@example.org', accountId: 'a1', passwordChangeRequired: true }));
    const client = createClient('');

    const fromLogin = await client.login({ jmapUrl: 'https://j.example.org', username: 'u', password: 'p' });
    expect(fromLogin.passwordChangeRequired).toBe(true);
    const fromMe = await client.me();
    expect(fromMe.passwordChangeRequired).toBe(true);
  });

  it('the field is absent for an unflagged account', async () => {
    stub(() => json(200, { username: 'u@example.org', accountId: 'a1' }));
    const me = await createClient('').me();
    expect(me.passwordChangeRequired).toBeUndefined();
  });
});
