import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@solidjs/testing-library';
import { AdminScreen } from './index.tsx';
import { mockAdminApi } from './testkit.tsx';
import { offlineShell } from './shell/testkit.tsx';
import { AdminApiError, createHttpAdminApi } from '../../state/slices/admin.ts';

describe('Admin sign-in gate — session lifetime', () => {
  it('states how long an admin session lasts', async () => {
    render(() => <AdminScreen api={mockAdminApi({ session: vi.fn(async () => null) })} {...offlineShell()} />);
    const form = await screen.findByRole('form', { name: 'Admin sign in' });
    expect(form).toHaveTextContent('30 minutes without activity');
    expect(form).toHaveTextContent('12 hours');
  });

  it('does not say a session ended when there simply is none', async () => {
    render(() => <AdminScreen api={mockAdminApi({ session: vi.fn(async () => null) })} {...offlineShell()} />);
    await screen.findByRole('form', { name: 'Admin sign in' });
    expect(screen.queryByText('Your admin session has ended. Sign in again.')).toBeNull();
  });

  it('returns to the gate and says so when the server ends a session in use', async () => {
    const api = mockAdminApi();
    render(() => <AdminScreen api={api} {...offlineShell()} />);
    // Control: signed in, the panel is showing.
    expect(await screen.findByRole('link', { name: 'Domains' })).toBeInTheDocument();
    expect(screen.queryByRole('form', { name: 'Admin sign in' })).toBeNull();

    // The slice installs the hook; the HTTP client calls it on a 401.
    expect(api.onSessionEnded).toBeTypeOf('function');
    api.onSessionEnded!();

    expect(await screen.findByRole('form', { name: 'Admin sign in' })).toBeInTheDocument();
    expect(screen.getByTestId('toast')).toHaveTextContent('Your admin session has ended');
    expect(screen.getByRole('form', {name:'Admin sign in'})).not.toHaveTextContent('Your admin session has ended');
    api.onSessionEnded!();
    expect(screen.getAllByText('Your admin session has ended. Sign in again.')).toHaveLength(1);
    expect(screen.queryByRole('link', { name: 'Domains' })).toBeNull();
  });
});

describe('Admin sign-in gate — second step (TOTP)', () => {
  /** A server whose admin account has TOTP: the password answer is a challenge. */
  function totpServer(verify: (token: string, code: string) => Promise<{ username: string }>) {
    let signedIn = false;
    const api = mockAdminApi({
      session: vi.fn(async () => (signedIn ? { username: 'ops' } : null)),
      login: vi.fn(async () => ({ twofaRequired: true, pendingToken: 'pt-1', factors: ['totp'] }) as never),
    });
    const verifyTotp = vi.fn(async (token: string, code: string) => {
      const s = await verify(token, code);
      signedIn = true;
      return s;
    });
    return { api, verifyTotp };
  }

  async function signInWithPassword(): Promise<void> {
    const form = await screen.findByRole('form', { name: 'Admin sign in' });
    fireEvent.input(screen.getByLabelText('Admin username'), { target: { value: 'ops' } });
    fireEvent.input(screen.getByLabelText('Password'), { target: { value: 'pw' } });
    fireEvent.submit(form);
  }

  it('a challenge does not open the console; it asks for the code', async () => {
    const { api, verifyTotp } = totpServer(async () => ({ username: 'ops' }));
    render(() => <AdminScreen api={api} {...offlineShell()} verifyTotp={verifyTotp} />);
    await signInWithPassword();
    const step = await screen.findByRole('form', { name: 'Admin sign in, second step' });
    expect(step).toHaveTextContent('ops');
    expect(screen.queryByRole('navigation', { name: 'Admin sections' })).toBeNull();
    expect(screen.getByLabelText('Authentication code')).toHaveAttribute('autocomplete', 'one-time-code');
  });

  it('a valid code starts the session and opens the console', async () => {
    const { api, verifyTotp } = totpServer(async () => ({ username: 'ops' }));
    render(() => <AdminScreen api={api} {...offlineShell()} verifyTotp={verifyTotp} />);
    await signInWithPassword();
    const step = await screen.findByRole('form', { name: 'Admin sign in, second step' });
    fireEvent.input(screen.getByLabelText('Authentication code'), { target: { value: '123 456' } });
    fireEvent.submit(step);
    await waitFor(() => expect(verifyTotp).toHaveBeenCalledWith('pt-1', '123456'));
    expect(await screen.findByRole('navigation', { name: 'Admin sections' })).toBeInTheDocument();
  });

  it('a wrong code stays on the step and says what to do', async () => {
    const { api, verifyTotp } = totpServer(async () => {
      throw new AdminApiError(401, 'bad code');
    });
    render(() => <AdminScreen api={api} {...offlineShell()} verifyTotp={verifyTotp} />);
    await signInWithPassword();
    const step = await screen.findByRole('form', { name: 'Admin sign in, second step' });
    fireEvent.input(screen.getByLabelText('Authentication code'), { target: { value: '000000' } });
    fireEvent.submit(step);
    expect(await screen.findByRole('alert')).toHaveTextContent('That code is not valid');
    expect(screen.queryByRole('navigation', { name: 'Admin sections' })).toBeNull();
  });

  it('Start again returns to the password form', async () => {
    const { api, verifyTotp } = totpServer(async () => ({ username: 'ops' }));
    render(() => <AdminScreen api={api} {...offlineShell()} verifyTotp={verifyTotp} />);
    await signInWithPassword();
    await screen.findByRole('form', { name: 'Admin sign in, second step' });
    fireEvent.click(screen.getByRole('button', { name: 'Start again' }));
    expect(await screen.findByRole('form', { name: 'Admin sign in' })).toBeInTheDocument();
  });

  it('too many attempts (429) says to wait, not that the password is wrong', async () => {
    const api = mockAdminApi({
      session: vi.fn(async () => null),
      login: vi.fn(async () => {
        throw new AdminApiError(429, 'slow down');
      }),
    });
    render(() => <AdminScreen api={api} {...offlineShell()} />);
    await signInWithPassword();
    expect(await screen.findByRole('alert')).toHaveTextContent('Too many sign-in attempts');
  });
});

describe('createHttpAdminApi — a 401 on a panel route ends the session', () => {
  // The body is what `unauthorized()` in crates/mw-server/src/admin.rs sends for an
  // unknown or expired session.
  const expired = (): Response =>
    new Response(JSON.stringify({ error: 'admin authentication required' }), { status: 401 });

  it('calls onSessionEnded for a 401 on a data route', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => expired()));
    const api = createHttpAdminApi('');
    const ended = vi.fn();
    api.onSessionEnded = ended;
    await expect(api.listUsers()).rejects.toThrow(/401/);
    expect(ended).toHaveBeenCalledTimes(1);
    vi.unstubAllGlobals();
  });

  it('does not call it for the session probe or a wrong password', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => expired()));
    const api = createHttpAdminApi('');
    const ended = vi.fn();
    api.onSessionEnded = ended;
    expect(await api.session()).toBeNull();
    await expect(api.login('root', 'wrong')).rejects.toThrow('invalid admin credentials');
    expect(ended).not.toHaveBeenCalled();
    vi.unstubAllGlobals();
  });

  it('does not call it for a successful request', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('[]', { status: 200 })));
    const api = createHttpAdminApi('');
    const ended = vi.fn();
    api.onSessionEnded = ended;
    await waitFor(async () => expect(await api.listUsers()).toEqual([]));
    expect(ended).not.toHaveBeenCalled();
    vi.unstubAllGlobals();
  });
});


describe('Admin retry cooldown', () => {
  it('shows server retry delay and does not automatically resend credentials', async () => {
    const login = vi.fn(async () => { throw new AdminApiError(429, 'cooldown', 3); });
    render(() => <AdminScreen api={mockAdminApi({ session: vi.fn(async () => null), login })} {...offlineShell()} />);
    const form = await screen.findByRole('form', { name: 'Admin sign in' });
    vi.useFakeTimers();
    try {
      fireEvent.submit(form);
      await vi.advanceTimersByTimeAsync(0);
      expect(screen.getByText('Try again in 3 seconds.')).toBeInTheDocument();
      expect(screen.getByRole('button', {name: /^Sign in$/})).toBeDisabled();
      fireEvent.submit(form);
      expect(login).toHaveBeenCalledTimes(1);
      await vi.advanceTimersByTimeAsync(3000);
      expect(screen.getByRole('button', {name: /^Sign in$/})).toBeEnabled();
      expect(login).toHaveBeenCalledTimes(1);
    } finally {
      vi.useRealTimers();
    }
  });
});
