import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { AdminOAuthClients, dcrValues, dcrWrite, type DcrPolicyApi } from './index.tsx';
import { noScope, readScope, type DcrPolicy } from '../../../api/admin/policy.ts';
import { renderInShell } from '../shell/testkit.tsx';
import { expectNoAxeViolations, expectNoInlineStyle } from '../shell/uxTestkit.ts';
import { stripIsolates } from '../../../i18n';

beforeEach(() => history.replaceState(null, '', '/admin#/oauth-clients'));

const OFF: DcrPolicy = { enabled: false, requireInitialAccessToken: false, allowedRedirectHostSuffixes: [], defaultScope: null, updatedAt: null };

function setup(p: DcrPolicy = OFF) {
  const api: DcrPolicyApi = {
    load: vi.fn(async () => ({ kind: 'ok' as const, data: p })),
    save: vi.fn(async (w) => ({ ...w, updatedAt: '2026-10-25T10:00:00Z' })),
  };
  const utils = renderInShell(() => <AdminOAuthClients api={api} />);
  return { ...utils, api };
}

describe('OAuth clients screen (dynamic client registration)', () => {
  it('shows the policy off by default, with help, docs links and the never-changed history', async () => {
    setup();
    const enabled = await screen.findByRole('checkbox', { name: 'Let apps register themselves' });
    expect(enabled).not.toBeChecked();
    expect(enabled).toHaveAccessibleDescription(/RFC 7591/);
    expect(screen.getByRole('link', { name: 'Learn more about Allowed redirect hosts' }).getAttribute('href')).toMatch(/docs\/admin\/oauth-clients\.md#allowedRedirectHostSuffixes$/);
    expect(screen.getByText('No host: every registration is refused.')).toBeInTheDocument();
    expect(stripIsolates(screen.getByTestId('field-history').textContent ?? '')).toMatch(/Not changed from the console/);
  });

  it('refuses to turn registration on without a redirect host, before any request', async () => {
    const { api } = setup();
    fireEvent.click(await screen.findByRole('checkbox', { name: 'Let apps register themselves' }));
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByText(/Add at least one redirect host first/)).toBeInTheDocument();
    expect(api.save).not.toHaveBeenCalled();
  });

  it('adds hosts with the list editor, grants a scope and saves the whole policy, keeping unknown scope members', async () => {
    const stored = { ...noScope(), accounts: 'all', mcp_tools: ['mail.search'] };
    const { api } = setup({ ...OFF, defaultScope: stored });
    fireEvent.click(await screen.findByRole('checkbox', { name: 'Let apps register themselves' }));
    const hosts = screen.getByRole('textbox', { name: 'Allowed redirect hosts' });
    fireEvent.input(hosts, { target: { value: 'Apps.Example.org' } });
    fireEvent.keyDown(hosts, { key: 'Enter' });
    fireEvent.input(hosts, { target: { value: 'not a host!' } });
    fireEvent.keyDown(hosts, { key: 'Enter' });
    expect(await screen.findByText(/is not a host name/)).toBeInTheDocument();
    const scope = screen.getByRole('group', { name: 'Granted scope' });
    fireEvent.click(within(scope).getByRole('checkbox', { name: 'Read' }));
    fireEvent.click(within(scope).getByRole('checkbox', { name: 'Mail' }));
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(api.save).toHaveBeenCalledTimes(1));
    expect(api.save).toHaveBeenCalledWith({
      enabled: true,
      requireInitialAccessToken: false,
      allowedRedirectHostSuffixes: ['apps.example.org'],
      defaultScope: { ...stored, read: true, mail: true },
    });
    await waitFor(() => expect(stripIsolates(screen.getByTestId('field-history').textContent ?? '')).toMatch(/Last changed on/));
  });

  it('a permission without a part of the account is refused before any request', async () => {
    const { api } = setup({ ...OFF, allowedRedirectHostSuffixes: ['example.org'] });
    const scope = await screen.findByRole('group', { name: 'Granted scope' });
    fireEvent.click(within(scope).getByRole('checkbox', { name: 'Send' }));
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByText(/a permission without a part of the account grants nothing/)).toBeInTheDocument();
    expect(api.save).not.toHaveBeenCalled();
  });

  it('removing a host moves focus to the next Remove button', async () => {
    setup({ ...OFF, allowedRedirectHostSuffixes: ['a.example', 'b.example'] });
    const remove = await screen.findByRole('button', { name: /Remove .*a\.example/ });
    remove.focus();
    fireEvent.click(remove);
    await waitFor(() => expect(document.activeElement).toBe(screen.getByRole('button', { name: /Remove .*b\.example/ })));
  });

  it('has no axe violations and no inline style', async () => {
    const { container } = setup({ ...OFF, allowedRedirectHostSuffixes: ['example.org'] });
    await screen.findByRole('group', { name: 'Granted scope' });
    await expectNoAxeViolations(container);
    expectNoInlineStyle(container);
  });
});

describe('DCR policy values', () => {
  it('reads a stored scope leniently and writes it back whole', () => {
    expect(readScope('junk')).toEqual(noScope());
    expect(readScope({ read: true, accounts: 'all', folders: { subset: ['INBOX', 3] } })).toMatchObject({ read: true, accounts: 'all', folders: { subset: ['INBOX'] } });
    const p: DcrPolicy = { enabled: true, requireInitialAccessToken: true, allowedRedirectHostSuffixes: ['x.org'], defaultScope: { ...noScope(), pim: true }, updatedAt: null };
    expect(dcrWrite(dcrValues(p))).toEqual({ enabled: true, requireInitialAccessToken: true, allowedRedirectHostSuffixes: ['x.org'], defaultScope: { ...noScope(), pim: true } });
  });
});
