import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, screen } from '@solidjs/testing-library';
import { Login } from './Login.tsx';
import { AppContext } from '../state/context.ts';
import { createAppState } from '../state/store.ts';
import { ApiError, type Client, type LoginInput, type Me } from '../api/client.ts';
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

describe('Login', () => {
  it('renders the fields and hint', () => {
    renderLogin(fakeClient());
    expect(screen.getByText('JMAP server URL')).toBeInTheDocument();
    expect(screen.getByText('Username')).toBeInTheDocument();
    expect(screen.getByText('Password')).toBeInTheDocument();
    expect(screen.getByText(/testuser@example.org/)).toBeInTheDocument();
  });

  it('submits credentials to the client', async () => {
    const client = fakeClient();
    renderLogin(client);

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

    fireEvent.input(screen.getByPlaceholderText('https://jmap.example.org'), {
      target: { value: 'https://jmap.example.org' },
    });
    fireEvent.input(screen.getByLabelText('Username'), { target: { value: 'x' } });
    fireEvent.input(screen.getByLabelText('Password'), { target: { value: 'y' } });
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));

    expect(await screen.findByRole('alert')).toHaveTextContent('Invalid credentials');
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
