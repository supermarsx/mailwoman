import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, screen } from '@solidjs/testing-library';
import { Login } from './Login.tsx';
import { AppContext } from '../state/context.ts';
import { createAppState } from '../state/store.ts';
import { ApiError, TwoFactorRequired, type Client, type LoginInput, type Me } from '../api/client.ts';
import { CAP_MAIL, type JmapResponse, type JmapSession } from '../api/jmap-types.ts';

function fakeClient(overrides: Partial<Client> = {}): Client {
  const session: JmapSession = {
    capabilities: {},
    accounts: { acct1: { name: 'Test', isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
    primaryAccounts: { [CAP_MAIL]: 'acct1' },
    username: 'testuser@example.org',
    apiUrl: '/jmap/api',
    downloadUrl: '/jmap/download',
    uploadUrl: '/jmap/upload',
    eventSourceUrl: '/jmap/eventsource',
    state: 's0',
  };
  const emptyMailboxGet: JmapResponse = {
    methodResponses: [['Mailbox/get', { accountId: 'acct1', state: 's', list: [], notFound: [] }, 'c0']],
    sessionState: 's0',
  };
  return {
    login: vi.fn(async (_input: LoginInput): Promise<Me> => ({ username: 'testuser@example.org', accountId: 'acct1' })),
    logout: vi.fn(async () => undefined),
    me: vi.fn(async (): Promise<Me> => ({ username: 'testuser@example.org', accountId: 'acct1' })),
    session: vi.fn(async () => session),
    jmap: vi.fn(async () => emptyMailboxGet),
    sanitize: vi.fn(async (html: string) => html),
    onNetwork: vi.fn(() => () => undefined),
    ...overrides,
  };
}

function renderLogin(client: Client) {
  const app = createAppState(client);
  return render(() => <AppContext.Provider value={app}>{<Login />}</AppContext.Provider>);
}

/**
 * The screen opens on the email lookup (t28-e4); the server URL and username
 * fields these cases drive are behind "Enter server details manually".
 */
function showManual(): void {
  fireEvent.click(screen.getByRole('button', { name: 'Enter server details manually' }));
}

describe('Login', () => {
  it('renders the manual fields once they are asked for', () => {
    renderLogin(fakeClient());
    showManual();
    expect(screen.getByText('JMAP server URL')).toBeInTheDocument();
    expect(screen.getByText('Username')).toBeInTheDocument();
    expect(screen.getByText('Password')).toBeInTheDocument();
  });

  // The mock backend's credentials used to be printed on every deployment's
  // sign-in screen. The server has no signal that marks a mock deployment, so
  // the line is gone rather than conditional.
  it('does not print the mock account credentials', () => {
    renderLogin(fakeClient());
    expect(screen.queryByText(/testpass/)).toBeNull();
    showManual();
    expect(screen.queryByText(/testpass/)).toBeNull();
  });

  it('submits credentials to the client', async () => {
    const client = fakeClient();
    renderLogin(client);

    showManual();
    fireEvent.input(screen.getByPlaceholderText('https://jmap.example.org'), {
      target: { value: 'https://jmap.example.org' },
    });
    fireEvent.input(screen.getByLabelText('Username'), { target: { value: 'testuser@example.org' } });
    fireEvent.input(screen.getByLabelText('Password'), { target: { value: 'testpass' } });
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));

    await vi.waitFor(() => {
      expect(client.login).toHaveBeenCalledWith({
        jmapUrl: 'https://jmap.example.org',
        username: 'testuser@example.org',
        password: 'testpass',
      });
    });
  });

  it('shows an error on 401', async () => {
    const client = fakeClient({
      login: vi.fn(async () => {
        throw new ApiError(401, 'invalid credentials');
      }),
    });
    renderLogin(client);

    showManual();
    fireEvent.input(screen.getByPlaceholderText('https://jmap.example.org'), {
      target: { value: 'https://jmap.example.org' },
    });
    fireEvent.input(screen.getByLabelText('Username'), { target: { value: 'x' } });
    fireEvent.input(screen.getByLabelText('Password'), { target: { value: 'y' } });
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));

    expect(await screen.findByRole('alert')).toHaveTextContent('Invalid credentials');
  });

  // A disabled account is refused with the SAME 401 body as a wrong password
  // (t27 plan §4 e3: one shape, so the refusal is not an account-state oracle).
  // The screen therefore cannot say "this account is disabled"; it states the
  // possibility next to the refusal.
  it('a 401 also says the account may have been disabled', async () => {
    const client = fakeClient({
      login: vi.fn(async () => {
        throw new ApiError(401, 'invalid credentials');
      }),
    });
    renderLogin(client);
    // Control: the note is not part of the form before a refusal.
    expect(screen.queryByTestId('login-refused-note')).toBeNull();

    showManual();
    fireEvent.input(screen.getByPlaceholderText('https://jmap.example.org'), {
      target: { value: 'https://jmap.example.org' },
    });
    fireEvent.input(screen.getByLabelText('Username'), { target: { value: 'x' } });
    fireEvent.input(screen.getByLabelText('Password'), { target: { value: 'y' } });
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));

    const note = await screen.findByTestId('login-refused-note');
    expect(note).toHaveTextContent('the account may have been disabled by an administrator');
  });

  it('an unreachable server does not show the disabled-account note', async () => {
    const client = fakeClient({
      login: vi.fn(async () => {
        throw new Error('boom');
      }),
    });
    renderLogin(client);

    showManual();
    fireEvent.input(screen.getByPlaceholderText('https://jmap.example.org'), {
      target: { value: 'https://jmap.example.org' },
    });
    fireEvent.input(screen.getByLabelText('Username'), { target: { value: 'x' } });
    fireEvent.input(screen.getByLabelText('Password'), { target: { value: 'y' } });
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));

    expect(await screen.findByRole('alert')).toHaveTextContent('Could not reach the server');
    expect(screen.queryByTestId('login-refused-note')).toBeNull();
  });

  it('a flagged account is signed in without loading mail', async () => {
    const client = fakeClient({
      login: vi.fn(
        async (_input: LoginInput): Promise<Me> => ({
          username: 'testuser@example.org',
          accountId: 'acct1',
          passwordChangeRequired: true,
        }),
      ),
    });
    const app = createAppState(client);
    render(() => <AppContext.Provider value={app}>{<Login />}</AppContext.Provider>);

    showManual();
    fireEvent.input(screen.getByPlaceholderText('https://jmap.example.org'), {
      target: { value: 'https://jmap.example.org' },
    });
    fireEvent.input(screen.getByLabelText('Username'), { target: { value: 'testuser@example.org' } });
    fireEvent.input(screen.getByLabelText('Password'), { target: { value: 'testpass' } });
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));

    await vi.waitFor(() => expect(app.me()?.passwordChangeRequired).toBe(true));
    // The session exists, but no JMAP request was made for it.
    expect(client.jmap).not.toHaveBeenCalled();
  });

  it('control: an unflagged account loads mail on login', async () => {
    const client = fakeClient();
    const app = createAppState(client);
    render(() => <AppContext.Provider value={app}>{<Login />}</AppContext.Provider>);

    showManual();
    fireEvent.input(screen.getByPlaceholderText('https://jmap.example.org'), {
      target: { value: 'https://jmap.example.org' },
    });
    fireEvent.input(screen.getByLabelText('Username'), { target: { value: 'testuser@example.org' } });
    fireEvent.input(screen.getByLabelText('Password'), { target: { value: 'testpass' } });
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));

    await vi.waitFor(() => expect(client.jmap).toHaveBeenCalled());
  });
});

/**
 * `POST /api/discover` as the server implements it
 * (`crates/mw-server/src/lib.rs:2262-2306`). The 200 body is the serialised
 * `AccountCandidate` (`crates/mw-autoconfig/src/lib.rs:86-93`): `tls` is
 * kebab-case (`:35-44`), `auth` lowercase (`:47-54`), `source` kebab-case
 * (`:57-75`), and an absent POP3 server is `null`, not omitted.
 */
const SRV_CANDIDATE = {
  imap: { host: 'imap.example.org', port: 993, tls: 'implicit' },
  pop3: null,
  smtp: { host: 'smtp.example.org', port: 587, tls: 'start-tls' },
  auth: 'password',
  source: 'srv',
};

function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

/** Stub `fetch`: `/api/discover` answers with `discover()`, anything else with `[]`. */
function stubDiscover(discover: () => Response | Promise<Response>): ReturnType<typeof vi.fn> {
  const fetchMock = vi.fn(async (input: RequestInfo | URL, _init?: RequestInit) =>
    String(input).endsWith('/api/discover') ? discover() : json([]),
  );
  vi.stubGlobal('fetch', fetchMock);
  return fetchMock;
}

function discoverCalls(fetchMock: ReturnType<typeof vi.fn>): unknown[][] {
  return fetchMock.mock.calls.filter(([url]) => String(url).endsWith('/api/discover'));
}

function typeEmailAndPassword(email: string): void {
  fireEvent.input(screen.getByLabelText('Email address'), { target: { value: email } });
  fireEvent.input(screen.getByLabelText('Password'), { target: { value: 'testpass' } });
}

describe('Login › server lookup', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('opens on an email address and a password, without the server fields', () => {
    renderLogin(fakeClient());
    expect(screen.getByLabelText('Email address')).toBeInTheDocument();
    expect(screen.getByLabelText('Password')).toBeInTheDocument();
    expect(screen.queryByLabelText('JMAP server URL')).toBeNull();
    expect(screen.queryByLabelText('Username')).toBeNull();
  });

  it('looks the address up, shows the server, and signs in only after confirmation', async () => {
    const fetchMock = stubDiscover(() => json(SRV_CANDIDATE));
    const client = fakeClient();
    renderLogin(client);
    // Precondition: nothing has been found before the lookup.
    expect(screen.getByTestId('login-discovered')).toBeEmptyDOMElement();

    typeEmailAndPassword('ada@example.org');
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));

    const confirm = await screen.findByRole('button', { name: 'Sign in with this server' });
    const [url, init] = discoverCalls(fetchMock)[0] as [string, RequestInit];
    expect(url).toBe('/api/discover');
    expect(init.method).toBe('POST');
    expect(JSON.parse(String(init.body))).toEqual({ email: 'ada@example.org' });

    const shown = screen.getByTestId('login-discovered');
    expect(shown).toHaveTextContent('imap.example.org');
    expect(shown).toHaveTextContent('port 993, TLS.');
    expect(shown).toHaveTextContent("Source: the domain's DNS SRV records.");
    // The password has gone nowhere yet.
    expect(client.login).not.toHaveBeenCalled();

    fireEvent.click(confirm);
    await vi.waitFor(() => {
      expect(client.login).toHaveBeenCalledWith({
        jmapUrl: 'imaps://imap.example.org:993',
        username: 'ada@example.org',
        password: 'testpass',
      });
    });
  });

  it('signs in to a STARTTLS server with the imap:// form', async () => {
    stubDiscover(() =>
      json({ ...SRV_CANDIDATE, imap: { host: 'mail.example.org', port: 143, tls: 'start-tls' } }),
    );
    const client = fakeClient();
    renderLogin(client);
    typeEmailAndPassword('ada@example.org');
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Sign in with this server' }));

    expect(screen.getByTestId('login-discovered')).toHaveTextContent('port 143, STARTTLS.');
    await vi.waitFor(() => {
      expect(client.login).toHaveBeenCalledWith({
        jmapUrl: 'imap://mail.example.org:143',
        username: 'ada@example.org',
        password: 'testpass',
      });
    });
  });

  // `source: "jmap"`: the candidate's `imap`/`smtp` carry the JMAP API host
  // (`mw-autoconfig/src/lib.rs:60-64, 194-206`), so the sign-in must use the
  // session URL the lookup fetched (`:177`), not an imaps:// URL to port 443.
  it('signs in to a JMAP domain with its session URL', async () => {
    const api = { host: 'api.example.org', port: 443, tls: 'implicit' };
    stubDiscover(() => json({ imap: api, pop3: null, smtp: api, auth: 'password', source: 'jmap' }));
    const client = fakeClient();
    renderLogin(client);
    typeEmailAndPassword('ada@example.org');
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Sign in with this server' }));

    await vi.waitFor(() => {
      expect(client.login).toHaveBeenCalledWith({
        jmapUrl: 'https://example.org/.well-known/jmap',
        username: 'ada@example.org',
        password: 'testpass',
      });
    });
  });

  // The other 200 shape: no autoconfig candidate, only a `_jmap._tcp` SRV
  // record (`crates/mw-server/src/lib.rs:2287-2289`).
  it('signs in through a _jmap._tcp SRV record when that is all there is', async () => {
    stubDiscover(() => json({ source: 'jmap-srv', jmapSrv: { host: 'jmap.example.net', port: 8443 } }));
    const client = fakeClient();
    renderLogin(client);
    typeEmailAndPassword('ada@example.org');
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Sign in with this server' }));

    await vi.waitFor(() => {
      expect(client.login).toHaveBeenCalledWith({
        jmapUrl: 'https://jmap.example.net:8443/.well-known/jmap',
        username: 'ada@example.org',
        password: 'testpass',
      });
    });
  });

  it('says so, and offers no sign-in, when the provider expects OAuth', async () => {
    stubDiscover(() => json({ ...SRV_CANDIDATE, auth: 'oauth2', source: 'provider-db' }));
    const client = fakeClient();
    renderLogin(client);
    // Control: the sentence is not on the screen before the lookup.
    expect(screen.queryByTestId('login-oauth-only')).toBeNull();

    typeEmailAndPassword('ada@example.org');
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));

    expect(await screen.findByTestId('login-oauth-only')).toHaveTextContent(
      'This provider requires OAuth sign-in, which this build does not offer yet.',
    );
    expect(screen.queryByRole('button', { name: 'Sign in with this server' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Sign in' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Enter server details manually' })).toBeInTheDocument();
    expect(client.login).not.toHaveBeenCalled();
  });

  // 404 `{"error":"no configuration discovered"}` (`lib.rs:2295-2299`).
  it('opens the manual fields with the address kept when nothing is found', async () => {
    stubDiscover(() => json({ error: 'no configuration discovered' }, 404));
    const client = fakeClient();
    renderLogin(client);
    typeEmailAndPassword('ada@nowhere.example');
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));

    expect(await screen.findByRole('alert')).toHaveTextContent(
      /No server settings were found for .?nowhere\.example.?\. Enter the server details below\./,
    );
    expect(screen.getByLabelText('JMAP server URL')).toHaveValue('');
    expect(screen.getByLabelText('Username')).toHaveValue('ada@nowhere.example');
    expect(screen.getByLabelText('Password')).toHaveValue('testpass');
    expect(document.activeElement).toBe(screen.getByLabelText('JMAP server URL'));
    expect(client.login).not.toHaveBeenCalled();
  });

  // 400 (`lib.rs:2290-2294`), 429 (`discover_ratelimit.rs:158-164`), 502
  // (`lib.rs:2300-2304`), and a request that never reached the server.
  it.each([
    [400, 'The server did not accept'],
    [429, 'Too many server lookups were made from this network.'],
    [502, 'The server lookup did not complete.'],
  ])('a %i from the lookup opens the manual fields with a reason', async (status, reason) => {
    stubDiscover(() => json({ error: 'x' }, status));
    renderLogin(fakeClient());
    typeEmailAndPassword('ada@example.org');
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));

    expect(await screen.findByRole('alert')).toHaveTextContent(reason);
    expect(screen.getByLabelText('Username')).toHaveValue('ada@example.org');
  });

  it('a lookup that cannot reach the server opens the manual fields', async () => {
    stubDiscover(() => Promise.reject(new TypeError('Failed to fetch')));
    renderLogin(fakeClient());
    typeEmailAndPassword('ada@example.org');
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));

    expect(await screen.findByRole('alert')).toHaveTextContent('The server lookup did not complete.');
    expect(screen.getByLabelText('Username')).toHaveValue('ada@example.org');
  });

  // Host and port come from DNS and from files the mail domain publishes.
  it('does not build a sign-in from a host that is not a host name', async () => {
    stubDiscover(() =>
      json({ ...SRV_CANDIDATE, imap: { host: 'imap.example.org/x?y', port: 993, tls: 'implicit' } }),
    );
    const client = fakeClient();
    renderLogin(client);
    typeEmailAndPassword('ada@example.org');
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));

    expect(await screen.findByRole('alert')).toHaveTextContent('The server lookup did not complete.');
    expect(screen.queryByRole('button', { name: 'Sign in with this server' })).toBeNull();
    expect(client.login).not.toHaveBeenCalled();
  });

  it('forgets the found server when the address is edited', async () => {
    const fetchMock = stubDiscover(() => json(SRV_CANDIDATE));
    const client = fakeClient();
    renderLogin(client);
    typeEmailAndPassword('ada@example.org');
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    await screen.findByRole('button', { name: 'Sign in with this server' });

    fireEvent.input(screen.getByLabelText('Email address'), { target: { value: 'ada@example.net' } });
    expect(screen.getByTestId('login-discovered')).toBeEmptyDOMElement();
    expect(screen.queryByRole('button', { name: 'Sign in with this server' })).toBeNull();

    // The next submit looks the new address up; it does not sign in to the old server.
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    await vi.waitFor(() => expect(discoverCalls(fetchMock)).toHaveLength(2));
    expect(client.login).not.toHaveBeenCalled();
  });

  // The server does not say whether it is in proxy or engine mode, and a 401
  // does not say which of server, username and password was wrong.
  it('a refused sign-in to a found server opens the manual fields with what was sent', async () => {
    stubDiscover(() => json(SRV_CANDIDATE));
    const client = fakeClient({
      login: vi.fn(async () => {
        throw new ApiError(401, 'invalid credentials');
      }),
    });
    renderLogin(client);
    expect(screen.queryByTestId('login-discovered-refused-note')).toBeNull();
    typeEmailAndPassword('ada@example.org');
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Sign in with this server' }));

    expect(await screen.findByRole('alert')).toHaveTextContent('Invalid credentials');
    expect(screen.getByTestId('login-refused-note')).toBeInTheDocument();
    expect(screen.getByTestId('login-discovered-refused-note')).toHaveTextContent('imaps://imap.example.org:993');
    expect(screen.getByLabelText('JMAP server URL')).toHaveValue('imaps://imap.example.org:993');
    expect(screen.getByLabelText('Username')).toHaveValue('ada@example.org');
  });

  it('returns from the manual fields to the lookup', () => {
    renderLogin(fakeClient());
    showManual();
    fireEvent.click(screen.getByRole('button', { name: 'Look up the server from an email address' }));
    expect(screen.getByLabelText('Email address')).toBeInTheDocument();
    expect(screen.queryByLabelText('JMAP server URL')).toBeNull();
  });
});

describe('Login › 2FA', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  function submitCreds(): void {
    showManual();
    fireEvent.input(screen.getByPlaceholderText('https://jmap.example.org'), {
      target: { value: 'https://jmap.example.org' },
    });
    fireEvent.input(screen.getByLabelText('Username'), { target: { value: 'u@example.org' } });
    fireEvent.input(screen.getByLabelText('Password'), { target: { value: 'pw' } });
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));
  }

  it('renders the challenge (no session) when login reports twofaRequired', async () => {
    // A correct password that is 2FA-gated: the client throws before any session.
    const client = fakeClient({
      login: vi.fn(async () => {
        throw new TwoFactorRequired({ pendingToken: 'tok', factors: ['totp', 'recovery'] });
      }),
    });
    renderLogin(client);
    submitCreds();

    // The second-factor challenge replaces the credential form…
    expect(await screen.findByTestId('twofa-challenge')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Sign in' })).toBeNull();
    // …and NO session bootstrap ran (no downgrade: /api/me is never consulted).
    expect(client.me).not.toHaveBeenCalled();
  });

  it('re-inits the session only after a factor verifies', async () => {
    // /api/login/2fa (the challenge verify) succeeds; app.init() then reads /api/me.
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => new Response('{}', { status: 200 })),
    );
    const client = fakeClient({
      login: vi.fn(async () => {
        throw new TwoFactorRequired({ pendingToken: 'tok', factors: ['totp'] });
      }),
    });
    renderLogin(client);
    submitCreds();

    await screen.findByTestId('twofa-challenge');
    expect(client.me).not.toHaveBeenCalled(); // still no session before the factor

    fireEvent.input(screen.getByLabelText('Authenticator code'), { target: { value: '123456' } });
    fireEvent.click(screen.getByTestId('challenge-verify'));

    // The factor cleared → the normal post-login bootstrap runs (client.me()).
    await vi.waitFor(() => expect(client.me).toHaveBeenCalled());
  });

  it('shows an enrollment notice when a required user has nothing enrolled', async () => {
    const client = fakeClient({
      login: vi.fn(async () => {
        throw new TwoFactorRequired({
          pendingToken: 'tok',
          factors: ['totp', 'webauthn'],
          enrollmentRequired: true,
        });
      }),
    });
    renderLogin(client);
    submitCreds();

    expect(await screen.findByTestId('twofa-enroll-required')).toBeInTheDocument();
    // The verify challenge is NOT offered (there is nothing to verify against).
    expect(screen.queryByTestId('twofa-challenge')).toBeNull();
    expect(client.me).not.toHaveBeenCalled();
  });

  it('returns to the credential form from the challenge', async () => {
    const client = fakeClient({
      login: vi.fn(async () => {
        throw new TwoFactorRequired({ pendingToken: 'tok', factors: ['totp'] });
      }),
    });
    renderLogin(client);
    submitCreds();

    await screen.findByTestId('twofa-challenge');
    fireEvent.click(screen.getByRole('button', { name: 'Back to sign in' }));
    expect(screen.getByRole('button', { name: 'Sign in' })).toBeInTheDocument();
  });
});

describe('Login › SSO', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    if (typeof history !== 'undefined') history.pushState({}, '', '/');
  });

  it('renders no SSO controls when the provider list is empty (login unchanged)', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => new Response('[]', { status: 200 })),
    );
    renderLogin(fakeClient());
    // Password sign-in still present; no "Sign in with…" buttons appear.
    expect(screen.getByRole('button', { name: 'Sign in' })).toBeInTheDocument();
    await Promise.resolve();
    await Promise.resolve();
    expect(screen.queryByRole('link', { name: /Sign in with/ })).toBeNull();
  });

  it('renders a "Sign in with <IdP>" link per enabled provider', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () =>
        new Response(
          JSON.stringify([
            { id: 'corp-oidc', kind: 'oidc', displayName: 'Acme SSO' },
            { id: 'corp-saml', kind: 'saml', displayName: 'Contoso SAML' },
          ]),
          { status: 200 },
        ),
      ),
    );
    renderLogin(fakeClient());
    const oidc = await screen.findByRole('link', { name: 'Sign in with Acme SSO' });
    expect(oidc).toHaveAttribute('href', '/api/sso/corp-oidc/begin');
    const saml = screen.getByRole('link', { name: 'Sign in with Contoso SAML' });
    expect(saml).toHaveAttribute('href', '/api/sso/corp-saml/begin');
  });

  it('shows a uniform error when the browser returns with ?sso_error', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => new Response('[]', { status: 200 })),
    );
    history.pushState({}, '', '/?sso_error=denied');
    renderLogin(fakeClient());
    expect(screen.getByRole('alert')).toHaveTextContent(/Single sign-on did not complete/);
  });
});
