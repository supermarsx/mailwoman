// Forced password change (t27-e4, OH-1).
//
// An account whose admin `force_password_change` flag is set is allowed to sign
// in, but the server refuses everything except the password-change endpoints
// (403 `{"error":"password change required","passwordChangeRequired":true}`).
// The shell therefore must not mount the mailbox or issue a JMAP request for
// such an account: it renders the change form and a sign-out control, and
// nothing else, until `/api/me` stops reporting the flag.
//
// Every negative here ("no JMAP request") is preceded by a control through the
// same shell showing that an unflagged account DOES issue one — otherwise a
// build that never loads mail for anyone would pass.
//
// The HTTP bodies below are the SERVER's shapes (crates/mw-server/src/passwd.rs
// and the t27 plan §4 e3 step 7), not shapes chosen for the client.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@solidjs/testing-library';

const hooks = vi.hoisted(() => ({
  me: null as null | (() => Promise<unknown>),
  jmap: null as null | ReturnType<typeof import('vitest').vi.fn>,
  session: null as null | ReturnType<typeof import('vitest').vi.fn>,
  logout: null as null | ReturnType<typeof import('vitest').vi.fn>,
  gate: null as null | (() => void),
}));

vi.mock('./api/transport.ts', () => ({
  createConfiguredClient: () => ({
    login: vi.fn(),
    logout: (...args: unknown[]) => hooks.logout!(...args),
    me: () => hooks.me!(),
    session: (...args: unknown[]) => hooks.session!(...args),
    jmap: (...args: unknown[]) => hooks.jmap!(...args),
    sanitize: vi.fn(async (html: string) => html),
    onNetwork: vi.fn(() => () => undefined),
    onPasswordChangeRequired: (listener: () => void) => {
      hooks.gate = listener;
      return () => undefined;
    },
  }),
}));

const { App } = await import('./App.tsx');

const NORMAL = { username: 'me@example.org', accountId: 'acct1', csrfToken: 't' };
const FLAGGED = { ...NORMAL, passwordChangeRequired: true };

/** `GET /api/password/policy` as mw-server answers it for a flagged account. */
const SERVER_POLICY = {
  description: 'at least 8 characters',
  minLength: 8,
  requireUpper: false,
  requireLower: false,
  requireDigit: false,
  requireSymbol: false,
  forceChange: true,
};

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

/** Route the module-level `fetch` the password form uses; everything else 404s. */
function stubFetch(change: () => Response): ReturnType<typeof vi.fn> {
  const f = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    const url = String(input);
    if (url.endsWith('/api/password/policy')) return json(200, SERVER_POLICY);
    if (url.endsWith('/api/password') && init?.method === 'POST') return change();
    return json(404, {});
  });
  globalThis.fetch = f as unknown as typeof fetch;
  return f;
}

/**
 * Stands in for the DOM `WebSocket` the realtime client opens once a mail
 * session exists, and records each connection attempt. (jsdom's own WebSocket
 * cannot connect here and rejects asynchronously.)
 */
const sockets: string[] = [];
class RecordingWebSocket {
  onopen: unknown = null;
  onmessage: unknown = null;
  onclose: unknown = null;
  onerror: unknown = null;
  readyState = 0;
  constructor(url: string) {
    sockets.push(url);
  }
  send(): void {}
  close(): void {}
}

function fillAndSubmit(): void {
  fireEvent.input(screen.getByLabelText('Current password'), { target: { value: 'OldPass12' } });
  fireEvent.input(screen.getByLabelText('New password'), { target: { value: 'NewPass12' } });
  fireEvent.input(screen.getByLabelText('Confirm new password'), { target: { value: 'NewPass12' } });
  fireEvent.click(screen.getByRole('button', { name: 'Change password' }));
}

beforeEach(() => {
  hooks.jmap = vi.fn(async () => ({
    methodResponses: [['Mailbox/get', { accountId: 'acct1', state: 's', list: [], notFound: [] }, 'c0']],
    sessionState: 's',
  }));
  hooks.session = vi.fn(async () => ({
    capabilities: {},
    accounts: { acct1: { name: 'Test', isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
    primaryAccounts: { 'urn:ietf:params:jmap:mail': 'acct1' },
    username: 'me@example.org',
    apiUrl: '/a',
    downloadUrl: '/d',
    uploadUrl: '/u',
    eventSourceUrl: '/e',
    state: 's0',
  }));
  hooks.logout = vi.fn(async () => undefined);
  hooks.gate = null;
  sockets.length = 0;
  vi.stubGlobal('WebSocket', RecordingWebSocket);
  stubFetch(() => json(200, { changed: true, credentialsResealed: 1, zeroaccessRewrapRequired: false }));
  localStorage.clear();
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('forced password change holds the account at the change screen', () => {
  it('control: an unflagged account boots into mail (a JMAP request is issued)', async () => {
    hooks.me = () => Promise.resolve(NORMAL);

    render(() => <App />);

    await waitFor(() => expect(hooks.jmap).toHaveBeenCalled());
    expect(screen.queryByTestId('forced-password-change')).toBeNull();
    // …and the realtime push connection is opened for it.
    await waitFor(() => expect(sockets.length).toBeGreaterThan(0));
  });

  it('a flagged account gets the change form and a sign-out control, and no JMAP request', async () => {
    hooks.me = () => Promise.resolve(FLAGGED);

    render(() => <App />);

    await waitFor(() => expect(screen.getByTestId('forced-password-change')).toBeTruthy());
    await waitFor(() => expect(screen.getByLabelText('Current password')).toBeTruthy());
    expect(screen.getByRole('button', { name: 'Sign out' })).toBeTruthy();
    // The server's `forceChange: true` drives the banner.
    await waitFor(() => expect(screen.getByTestId('force-change-banner')).toBeTruthy());

    // Neither the JMAP API nor the JMAP session document was requested: both
    // would be a 403 for this account.
    expect(hooks.jmap).not.toHaveBeenCalled();
    expect(hooks.session).not.toHaveBeenCalled();
    // No realtime push connection either (the control above shows one is
    // opened for an ordinary session).
    expect(sockets).toEqual([]);
  });

  it('continues into mail once the change succeeds and /api/me drops the flag', async () => {
    let changed = false;
    hooks.me = () => Promise.resolve(changed ? NORMAL : FLAGGED);
    const fetchMock = stubFetch(() => {
      changed = true;
      return json(200, { changed: true, credentialsResealed: 1, zeroaccessRewrapRequired: false });
    });

    render(() => <App />);
    await waitFor(() => expect(screen.getByLabelText('Current password')).toBeTruthy());
    expect(hooks.jmap).not.toHaveBeenCalled();

    fillAndSubmit();

    // /api/me is re-read, the hold is released, and mail loads.
    await waitFor(() => expect(screen.queryByTestId('forced-password-change')).toBeNull());
    await waitFor(() => expect(hooks.jmap).toHaveBeenCalled());

    const post = fetchMock.mock.calls.find(([, init]) => (init as RequestInit | undefined)?.method === 'POST');
    expect(JSON.parse(String((post?.[1] as RequestInit).body))).toEqual({
      oldPassword: 'OldPass12',
      newPassword: 'NewPass12',
    });
  });

  it('stays held, says so, and issues no JMAP request when the server has no password backend', async () => {
    hooks.me = () => Promise.resolve(FLAGGED);
    stubFetch(() => json(501, { error: 'password change not configured' }));

    render(() => <App />);
    await waitFor(() => expect(screen.getByLabelText('Current password')).toBeTruthy());

    fillAndSubmit();

    const alert = await screen.findByTestId('passwd-error');
    expect(alert.textContent).toContain('password change not configured');
    expect(alert.textContent).toContain('This server is not set up to change passwords');
    expect(alert.textContent).toContain('administrator');
    expect(screen.getByTestId('forced-password-change')).toBeTruthy();
    expect(hooks.jmap).not.toHaveBeenCalled();
  });

  it('stays held when the server reports success but /api/me still carries the flag', async () => {
    hooks.me = () => Promise.resolve(FLAGGED);

    render(() => <App />);
    await waitFor(() => expect(screen.getByLabelText('Current password')).toBeTruthy());

    fillAndSubmit();

    await waitFor(() => expect(screen.getByTestId('forced-still-required')).toBeTruthy());
    expect(screen.getByTestId('forced-password-change')).toBeTruthy();
    expect(hooks.jmap).not.toHaveBeenCalled();
  });

  it('enters the hold mid-session when a request is refused with passwordChangeRequired', async () => {
    hooks.me = () => Promise.resolve(NORMAL);

    render(() => <App />);
    // Control: the session started as an ordinary one.
    await waitFor(() => expect(hooks.jmap).toHaveBeenCalled());
    expect(screen.queryByTestId('forced-password-change')).toBeNull();
    expect(hooks.gate).not.toBeNull();

    // The API client saw a 403 `passwordChangeRequired` (an admin set the flag
    // while this tab was open) and reports it.
    hooks.gate!();

    await waitFor(() => expect(screen.getByTestId('forced-password-change')).toBeTruthy());
  });

  it('sign out leaves the hold for the login screen', async () => {
    hooks.me = () => Promise.resolve(FLAGGED);

    render(() => <App />);
    await waitFor(() => expect(screen.getByTestId('forced-password-change')).toBeTruthy());

    fireEvent.click(screen.getByRole('button', { name: 'Sign out' }));

    await waitFor(() => expect(hooks.logout).toHaveBeenCalled());
    await waitFor(() => expect(screen.queryByTestId('forced-password-change')).toBeNull());
    expect(screen.getByRole('button', { name: 'Sign in' })).toBeTruthy();
  });
});
