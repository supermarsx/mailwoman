// t32-h9: the sign-in form asks for the address and the password, nothing else.
// The server decides which mail server an address signs in to (MW_UPSTREAM_DEFAULT,
// later the admin's domain table); the browser never names, sees or confirms one.
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
    username: 'ada@example.org',
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
    login: vi.fn(async (_input: LoginInput): Promise<Me> => ({ username: 'ada@example.org', accountId: 'acct1' })),
    logout: vi.fn(async () => undefined),
    me: vi.fn(async (): Promise<Me> => ({ username: 'ada@example.org', accountId: 'acct1' })),
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

function signIn(address: string): void {
  fireEvent.input(screen.getByLabelText('Email address'), { target: { value: address } });
  fireEvent.input(screen.getByLabelText('Password'), { target: { value: 'pw' } });
  fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));
}

describe('Login › no server entry (t32-h9)', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('has the address and the password and no server, URL, host or port field', () => {
    renderLogin(fakeClient());
    // Press every button on the form that is not the sign-in itself or the
    // password's show/hide toggle (t32-h17): none of them may bring up a field for
    // a server.
    for (const button of screen.queryAllByRole('button')) {
      if (button.textContent !== 'Sign in' && button.getAttribute('aria-label') !== 'Show password') fireEvent.click(button);
    }
    expect(screen.queryByLabelText('JMAP server URL')).toBeNull();
    expect(document.querySelector('input[type=url]')).toBeNull();
    expect(screen.queryByText(/server URL|JMAP|IMAP|host|port/i)).toBeNull();
    expect(screen.queryByRole('button', { name: /server|manual/i })).toBeNull();
    const inputs = [...document.querySelectorAll('form input')] as HTMLInputElement[];
    expect(inputs.map((el) => el.type).sort()).toEqual(['password', 'text']);
    expect(screen.getByLabelText('Email address')).toBeInTheDocument();
  });

  it('sends the address and the password only, without asking the server for a lookup', async () => {
    const fetchMock = vi.fn(async (_input: RequestInfo | URL) => new Response('[]', { status: 200 }));
    vi.stubGlobal('fetch', fetchMock);
    const client = fakeClient();
    renderLogin(client);
    signIn(' ada@example.org ');
    await vi.waitFor(() => expect(client.login).toHaveBeenCalledTimes(1));
    expect(client.login).toHaveBeenCalledWith({ username: 'ada@example.org', password: 'pw' });
    expect(fetchMock.mock.calls.some(([url]) => String(url).includes('/api/discover'))).toBe(false);
  });

  it('says the address is not set up on this server when the server cannot place it', async () => {
    const client = fakeClient({
      login: vi.fn(async () => {
        throw new ApiError(404, 'address not configured');
      }),
    });
    renderLogin(client);
    signIn('ada@elsewhere.example');
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'This address is not set up on this server. Contact your administrator.',
    );
    // Still the same two fields: nothing to type a server into.
    expect(document.querySelectorAll('form input')).toHaveLength(2);
    expect(screen.queryByTestId('login-refused-note')).toBeNull();
  });
});
