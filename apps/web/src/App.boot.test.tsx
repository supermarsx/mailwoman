// Boot stability (t22-e10).
//
// `init()` treats a 401 as the logged-out ANSWER, not a failure — so anything
// that rejects out of it is a real boot failure: the server is unreachable, or
// answered something unusable. It used to be fired as `void app.init()`, which
// made such a failure an unhandled rejection, and the shell then fell through to
// the LOGIN FORM because `me()` is null either way. The user was invited to
// authenticate against a server that had just failed to answer them.
//
// The distinction is invisible to a test that only checks "some screen rendered",
// so each assertion here names WHICH screen.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@solidjs/testing-library';

const hooks = vi.hoisted(() => ({
  me: null as null | (() => Promise<unknown>),
}));

vi.mock('./api/transport.ts', () => ({
  createConfiguredClient: () => ({
    login: vi.fn(),
    logout: vi.fn(async () => undefined),
    me: () => hooks.me!(),
    session: vi.fn(async () => ({
      capabilities: {},
      accounts: {},
      primaryAccounts: {},
      username: 'me@example.org',
      apiUrl: '/a',
      downloadUrl: '/d',
      uploadUrl: '/u',
      eventSourceUrl: '/e',
      state: 's0',
    })),
    jmap: vi.fn(async () => ({ methodResponses: [], sessionState: 's' })),
    sanitize: vi.fn(async (html: string) => html),
    onNetwork: vi.fn(() => () => undefined),
  }),
}));

const { App } = await import('./App.tsx');
const { ApiError, NetworkError } = await import('./api/client.ts');

beforeEach(() => {
  // The Assist gateway probe and any other boot fetch: answer, don't hang.
  globalThis.fetch = vi.fn(async () => new Response('{}', { status: 404 })) as unknown as typeof fetch;
  localStorage.clear();
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe('boot failure is not a login prompt', () => {
  it('shows a transport failure with a retry, and NOT the login form', async () => {
    hooks.me = () => Promise.reject(new NetworkError('connect ECONNREFUSED'));

    render(() => <App />);

    await waitFor(() => expect(screen.getByRole('alert')).toBeTruthy());
    expect(screen.getByRole('alert').textContent).toContain('Can’t reach the server');
    // The regression this closes: a login form for a server that just failed.
    expect(screen.queryByLabelText(/password/i)).toBeNull();
    expect(screen.getByTestId('async-retry')).toBeTruthy();
  });

  it('retry re-runs the boot, and the app recovers when the server answers', async () => {
    let calls = 0;
    hooks.me = () => {
      calls += 1;
      // Down once, then reachable and simply not authenticated.
      return calls === 1
        ? Promise.reject(new NetworkError('down'))
        : Promise.reject(new ApiError(401, 'not authenticated'));
    };

    render(() => <App />);
    await waitFor(() => expect(screen.getByTestId('async-retry')).toBeTruthy());

    fireEvent.click(screen.getByTestId('async-retry'));

    // Recovered: the failure is gone and the ordinary logged-out screen is up.
    await waitFor(() => expect(screen.queryByRole('alert')).toBeNull());
    expect(calls).toBe(2);
  });

  it('a 401 is the logged-out ANSWER and still renders login, not an error', async () => {
    // The control. Without it, "shows an error when boot fails" would also hold
    // for a build that shows an error for every unauthenticated visitor.
    hooks.me = () => Promise.reject(new ApiError(401, 'not authenticated'));

    render(() => <App />);

    await waitFor(() => expect(screen.queryByTestId('async-pending')).toBeNull());
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('the boot spinner is bounded like every other pending state', async () => {
    // `me()` never settles. Before, `authChecked` never flipped and the boot
    // spinner was the whole UI, permanently.
    vi.useFakeTimers();
    hooks.me = () => new Promise(() => undefined);

    render(() => <App />);
    expect(screen.getByTestId('async-pending')).toBeTruthy();

    await vi.advanceTimersByTimeAsync(15_000);

    expect(screen.queryByTestId('async-pending')).toBeNull();
    expect(screen.getByRole('alert')).toBeTruthy();
    vi.useRealTimers();
  });
});

// t32: a desktop or mobile shell talks to the server it was built or provisioned
// for, and the user cannot change it. A boot failure there shows Retry and says
// to contact the administrator; there is no way to a server field.
describe('boot in a native shell', () => {
  interface G {
    __TAURI_INTERNALS__?: unknown;
    __MW_CONFIG__?: unknown;
  }
  const g = globalThis as unknown as G;

  afterEach(() => {
    delete g.__TAURI_INTERNALS__;
    delete g.__MW_CONFIG__;
  });

  it('a boot failure offers Retry and the administrator line, and no Change server', async () => {
    g.__TAURI_INTERNALS__ = {};
    g.__MW_CONFIG__ = { serverUrl: 'https://mail.example.org', native: true };
    hooks.me = () => Promise.reject(new NetworkError('connect ECONNREFUSED'));

    render(() => <App />);

    await waitFor(() => expect(screen.getByRole('alert')).toBeTruthy());
    expect(screen.getByTestId('async-retry')).toBeTruthy();
    expect(screen.getByTestId('boot-contact-administrator')).toHaveTextContent(
      'If this keeps happening, contact your administrator.',
    );
    expect(screen.queryByRole('button', { name: 'Change server' })).toBeNull();
    expect(screen.queryByTestId('boot-change-server')).toBeNull();
    expect(screen.queryByTestId('login-server-address')).toBeNull();
    // Only Retry: no other control on the card.
    expect(screen.getAllByRole('button').map((b) => b.getAttribute('data-testid'))).toEqual(['async-retry']);
  });

  it('a browser boot failure has neither Change server nor the administrator line', async () => {
    hooks.me = () => Promise.reject(new NetworkError('connect ECONNREFUSED'));

    render(() => <App />);

    await waitFor(() => expect(screen.getByRole('alert')).toBeTruthy());
    expect(screen.getByTestId('async-retry')).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Change server' })).toBeNull();
    expect(screen.queryByTestId('boot-contact-administrator')).toBeNull();
  });

  it('a shell with no configured server says so, shows no form and requests nothing', async () => {
    g.__TAURI_INTERNALS__ = {};
    g.__MW_CONFIG__ = { serverUrl: null, native: true };
    const me = vi.fn(() => Promise.reject(new ApiError(401, 'not authenticated')));
    hooks.me = me;

    render(() => <App />);

    expect(screen.getByTestId('shell-not-configured')).toHaveTextContent(
      'This app has not been set up for a server. Contact your administrator.',
    );
    expect(document.querySelector('input')).toBeNull();
    expect(screen.queryByRole('button')).toBeNull();
    await Promise.resolve();
    expect(me).not.toHaveBeenCalled();
    expect(globalThis.fetch).not.toHaveBeenCalled();
  });
});
