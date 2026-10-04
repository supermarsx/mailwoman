import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@solidjs/testing-library';
import { AdminScreen } from './index.tsx';
import { mockAdminApi } from './testkit.tsx';
import { createHttpAdminApi } from '../../state/slices/admin.ts';

describe('Admin sign-in gate — session lifetime', () => {
  it('states how long an admin session lasts', async () => {
    render(() => <AdminScreen api={mockAdminApi({ session: vi.fn(async () => null) })} />);
    const form = await screen.findByRole('form', { name: 'Admin sign in' });
    expect(form).toHaveTextContent('30 minutes without activity');
    expect(form).toHaveTextContent('12 hours');
  });

  it('does not say a session ended when there simply is none', async () => {
    render(() => <AdminScreen api={mockAdminApi({ session: vi.fn(async () => null) })} />);
    await screen.findByRole('form', { name: 'Admin sign in' });
    expect(screen.queryByTestId('admin-session-ended')).toBeNull();
  });

  it('returns to the gate and says so when the server ends a session in use', async () => {
    const api = mockAdminApi();
    render(() => <AdminScreen api={api} />);
    // Control: signed in, the panel is showing.
    expect(await screen.findByRole('button', { name: 'Domains' })).toBeInTheDocument();
    expect(screen.queryByRole('form', { name: 'Admin sign in' })).toBeNull();

    // The slice installs the hook; the HTTP client calls it on a 401.
    expect(api.onSessionEnded).toBeTypeOf('function');
    api.onSessionEnded!();

    expect(await screen.findByRole('form', { name: 'Admin sign in' })).toBeInTheDocument();
    expect(screen.getByTestId('admin-session-ended')).toHaveTextContent('Your admin session has ended');
    expect(screen.queryByRole('button', { name: 'Domains' })).toBeNull();
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
