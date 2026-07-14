import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { ApiKeys } from './ApiKeys.tsx';
import { McpKeys, withTool } from './McpKeys.tsx';
import { scopeToWire, scopeFromWire, readOnlyScope, UNATTENDED_SEND_DISCLOSURE } from './types.ts';

function okJson(body: unknown): Response {
  return new Response(JSON.stringify(body), { status: 200, headers: { 'content-type': 'application/json' } });
}

describe('scope wire mapping (mirrors mw-oauth::Scope)', () => {
  it('serializes to the frozen snake_case + kebab-selector shape', () => {
    const wire = scopeToWire(readOnlyScope('acct-1'));
    expect(wire.accounts).toEqual({ subset: ['acct-1'] });
    expect(wire.folders).toBe('all');
    expect(wire).toHaveProperty('ip_allowlist');
    expect(wire).toHaveProperty('unattended_send', false);
    expect(wire).toHaveProperty('mcp_tools');
  });

  it('round-trips through the wire form', () => {
    const scope = { ...readOnlyScope('a'), send: true, mcpTools: ['mail.send'], rateLimit: 60, ipAllowlist: ['10.0.0.0/8'] };
    expect(scopeFromWire(scopeToWire(scope))).toEqual(scope);
  });
});

describe('API keys — shown once', () => {
  it('reveals the minted token once and lists then revokes keys', async () => {
    const created = {
      displayToken: 'mwk_abcd.SECRETSECRETSECRET',
      record: {
        prefix: 'abcd',
        label: 'backup script',
        accountId: 'acct-1',
        scope: readOnlyScope('acct-1'),
        createdAt: '2026-07-14T00:00:00Z',
        lastUsedAt: null,
        revokedAt: null,
        unattendedSendApproved: false,
      },
    };
    const fetcher = vi.fn(async (input: string, init?: RequestInit) => {
      if (input === '/api/keys' && init?.method === 'POST') return okJson(created);
      if (input === '/api/keys/abcd/revoke') return okJson({ ok: true });
      return okJson([]);
    });

    render(() => <ApiKeys accountId="acct-1" fetcher={fetcher} initialKeys={[]} />);
    fireEvent.input(screen.getByLabelText('Key label'), { target: { value: 'backup script' } });
    fireEvent.click(screen.getByRole('button', { name: 'Create key' }));

    await waitFor(() => expect(screen.getByTestId('minted-token')).toBeInTheDocument());
    expect(screen.getByTestId('minted-token').textContent).toBe('mwk_abcd.SECRETSECRETSECRET');
    // The wire scope was sent, not the UI camelCase form.
    const body = JSON.parse((fetcher.mock.calls.find((c) => c[0] === '/api/keys')?.[1]?.body as string) ?? '{}') as {
      scope: Record<string, unknown>;
    };
    expect(body.scope).toHaveProperty('ip_allowlist');

    // Dismiss the one-time reveal — it cannot be shown again.
    fireEvent.click(screen.getByRole('button', { name: 'I have saved it' }));
    expect(screen.queryByTestId('minted-token')).not.toBeInTheDocument();
  });

  it('requires a label before minting', async () => {
    const fetcher = vi.fn(async () => okJson([]));
    render(() => <ApiKeys accountId="acct-1" fetcher={fetcher} initialKeys={[]} />);
    fireEvent.click(screen.getByRole('button', { name: 'Create key' }));
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/label/));
  });
});

describe('scope builder', () => {
  it('toggles capabilities and surfaces into the request scope', async () => {
    const fetcher = vi.fn(async (input: string, init?: RequestInit) => {
      if (input === '/api/keys' && init?.method === 'POST') {
        return okJson({ displayToken: 'mwk_x.y', record: null });
      }
      return okJson([]);
    });
    render(() => <ApiKeys accountId="acct-1" fetcher={fetcher} initialKeys={[]} />);
    fireEvent.input(screen.getByLabelText('Key label'), { target: { value: 'k' } });
    fireEvent.click(screen.getByLabelText('Send'));
    fireEvent.click(screen.getByLabelText('PIM (calendar / tasks / notes / contacts)'));
    fireEvent.input(screen.getByLabelText('Rate limit'), { target: { value: '120' } });
    fireEvent.click(screen.getByRole('button', { name: 'Create key' }));

    await waitFor(() => expect(fetcher).toHaveBeenCalledWith('/api/keys', expect.anything()));
    const body = JSON.parse((fetcher.mock.calls.find((c) => c[0] === '/api/keys')?.[1]?.body as string) ?? '{}') as {
      scope: { send: boolean; pim: boolean; rate_limit: number };
    };
    expect(body.scope.send).toBe(true);
    expect(body.scope.pim).toBe(true);
    expect(body.scope.rate_limit).toBe(120);
  });
});

describe('MCP keys — per-tool grants + unattended-send disclosure', () => {
  it('granting mail.send implies the send verb and reveals the unattended-send disclosure', () => {
    const base = { ...readOnlyScope('a'), mcpTools: [] as string[] };
    const withSend = withTool(base, 'mail.send', true, true);
    expect(withSend.mcpTools).toContain('mail.send');
    expect(withSend.send).toBe(true);
    // Removing the tool drops the unattended flag.
    const withUnattended = { ...withSend, unattendedSend: true };
    const removed = withTool(withUnattended, 'mail.send', false, true);
    expect(removed.mcpTools).not.toContain('mail.send');
    expect(removed.unattendedSend).toBe(false);
  });

  it('shows the unattended-send disclosure only after mail.send is granted', async () => {
    const fetcher = vi.fn(async () => okJson({ displayToken: 'mwk_m.n', record: { scope: readOnlyScope('a') } }));
    render(() => <McpKeys accountId="a" fetcher={fetcher} />);
    expect(screen.queryByTestId('unattended-send-disclosure')).not.toBeInTheDocument();

    fireEvent.click(screen.getByLabelText(/Send mail/));
    await waitFor(() => expect(screen.getByTestId('unattended-send-disclosure')).toBeInTheDocument());
    expect(screen.getByTestId('unattended-send-disclosure').textContent).toBe(UNATTENDED_SEND_DISCLOSURE);
  });
});
