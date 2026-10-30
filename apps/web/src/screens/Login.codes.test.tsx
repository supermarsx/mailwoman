// Sign-in refusals the server explains (t32-q A7 `signin.refusalDetail`, worker
// f01): 403 accountDisabled / signinNotAllowed get their own localised text, and
// a refused answer's request id is shown as a reference. The server's English is
// never shown.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, screen } from '@solidjs/testing-library';
import { Login } from './Login.tsx';
import { AppContext } from '../state/context.ts';
import { createAppState } from '../state/store.ts';
import { ApiError, createClient, errorDetails, type Client } from '../api/client.ts';
import { t } from '../test/i18n.ts';

function client(login: Client['login']): Client {
  return {
    login,
    logout: vi.fn(async () => undefined),
    me: vi.fn(async () => {
      throw new ApiError(401, 'not authenticated');
    }),
    session: vi.fn(),
    jmap: vi.fn(),
    sanitize: vi.fn(async (h: string) => h),
    onNetwork: vi.fn(() => () => undefined),
  } as unknown as Client;
}

function renderLogin(c: Client) {
  const app = createAppState(c);
  return render(() => (
    <AppContext.Provider value={app}>
      <Login />
    </AppContext.Provider>
  ));
}

async function signIn(): Promise<void> {
  fireEvent.input(screen.getByLabelText('Email or username'), { target: { value: 'ada@example.org' } });
  fireEvent.input(screen.getByLabelText('Password', { exact: true }), { target: { value: 'pw' } });
  fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));
}

describe('Login › explained refusals', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('explains maintenance without claiming the credentials were rejected', async () => {
    renderLogin(client(async () => { throw new ApiError(503, 'paused', {code:'maintenance'}); }));
    await signIn();
    expect(await screen.findByRole('alert')).toHaveTextContent(t('auth-maintenance'));
    expect(screen.queryByTestId('login-refused-note')).toBeNull();
  });

  it('403 accountDisabled says the account is disabled, with the reference', async () => {
    renderLogin(
      client(async () => {
        throw new ApiError(403, 'request failed with 403', { code: 'accountDisabled', requestId: 'req-7f3a' });
      }),
    );
    await signIn();
    expect(await screen.findByRole('alert')).toHaveTextContent(t('auth-account-disabled'));
    expect(screen.getByTestId('login-reference')).toHaveTextContent(t('auth-reference', { id: 'req-7f3a' }));
    expect(screen.queryByTestId('login-refused-note')).toBeNull();
  });

  it('403 signinNotAllowed says the address may not sign in here', async () => {
    renderLogin(
      client(async () => {
        throw new ApiError(403, 'request failed with 403', { code: 'signinNotAllowed' });
      }),
    );
    await signIn();
    expect(await screen.findByRole('alert')).toHaveTextContent(t('auth-signin-not-allowed'));
    expect(screen.queryByTestId('login-reference')).toBeNull();
  });

  it('a uniform 401 keeps its generic text and hides account help by default', async () => {
    renderLogin(
      client(async () => {
        throw new ApiError(401, 'invalid credentials', { requestId: 'req-1' });
      }),
    );
    await signIn();
    expect(await screen.findByRole('alert')).toHaveTextContent(t('auth-invalid-credentials'));
    expect(screen.queryByTestId('login-refused-note')).toBeNull();
    expect(screen.getByTestId('login-reference')).toHaveTextContent('req-1');
  });
});

describe('api/client › refusal details', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('reads the code and the request id, and leaves the body readable', async () => {
    const res = new Response(JSON.stringify({ code: 'accountDisabled', error: 'disabled' }), {
      status: 403,
      headers: { 'content-type': 'application/json', 'x-request-id': 'abc-123' },
    });
    expect(await errorDetails(res)).toEqual({ code: 'accountDisabled', requestId: 'abc-123' });
    expect(await res.json()).toEqual({ code: 'accountDisabled', error: 'disabled' });
    expect(await errorDetails(new Response('nope', { status: 500 }))).toEqual({});
  });

  it('login carries them on the ApiError', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () =>
        new Response(JSON.stringify({ code: 'signinNotAllowed', error: 'x' }), {
          status: 403,
          headers: { 'content-type': 'application/json', 'x-request-id': 'r-9' },
        }),
      ),
    );
    const err = (await createClient('')
      .login({ username: 'ada@example.org', password: 'pw' })
      .catch((e: unknown) => e)) as ApiError;
    expect(err).toBeInstanceOf(ApiError);
    expect([err.status, err.code, err.requestId]).toEqual([403, 'signinNotAllowed', 'r-9']);
  });
});
