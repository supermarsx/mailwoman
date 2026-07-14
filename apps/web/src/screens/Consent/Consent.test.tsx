import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { ConsentScreen } from './index.tsx';
import { parseAuthorizeParams, type ConsentContext } from './service.ts';
import { scopeToWire, readOnlyScope } from '../../modules/apikeys/index.ts';

const params = {
  responseType: 'code',
  clientId: 'client-1',
  redirectUri: 'https://app.example/cb',
  codeChallenge: 'CHALLENGE',
  codeChallengeMethod: 'S256',
  resource: 'https://mail.example/',
};

function context(overrides: Partial<ConsentContext> = {}): ConsentContext {
  return {
    clientId: 'client-1',
    clientName: 'Acme Assistant',
    approved: true,
    redirectUri: 'https://app.example/cb',
    resource: 'https://mail.example/',
    requestedScope: scopeToWire({ ...readOnlyScope('acct-1'), send: true }),
    ...overrides,
  };
}

function okJson(body: unknown): Response {
  return new Response(JSON.stringify(body), { status: 200, headers: { 'content-type': 'application/json' } });
}

describe('parseAuthorizeParams', () => {
  it('parses the OAuth authorize query, including a JSON scope', () => {
    const scope = scopeToWire(readOnlyScope('acct-1'));
    const q = `?response_type=code&client_id=c&redirect_uri=${encodeURIComponent('https://x/cb')}&code_challenge=CH&code_challenge_method=S256&resource=${encodeURIComponent('https://r/')}&scope=${encodeURIComponent(JSON.stringify(scope))}`;
    const p = parseAuthorizeParams(q);
    expect(p.clientId).toBe('c');
    expect(p.codeChallengeMethod).toBe('S256');
    expect(p.scope).toEqual(scope);
  });
});

describe('consent screen', () => {
  it('shows the client, approval badge, and requested scope', async () => {
    render(() => <ConsentScreen params={params} initialContext={context()} onRedirect={vi.fn()} />);
    await waitFor(() => expect(screen.getByText('Acme Assistant')).toBeInTheDocument());
    expect(screen.getByTestId('client-approved')).toBeInTheDocument();
    expect(screen.getByTestId('requested-scope').textContent).toMatch(/send/);
  });

  it('flags an unapproved client', async () => {
    render(() => <ConsentScreen params={params} initialContext={context({ approved: false })} onRedirect={vi.fn()} />);
    await waitFor(() => expect(screen.getByTestId('client-unapproved')).toBeInTheDocument());
  });

  it('grant posts approve=true and redirects', async () => {
    const onRedirect = vi.fn();
    const fetcher = vi.fn(async (input: string, init?: RequestInit) => {
      if (input === '/oauth/decision') {
        const body = JSON.parse((init?.body as string) ?? '{}') as { approve: boolean };
        expect(body.approve).toBe(true);
        return okJson({ redirectUri: 'https://app.example/cb?code=AUTHCODE' });
      }
      return okJson(context());
    });
    render(() => <ConsentScreen params={params} initialContext={context()} fetcher={fetcher} onRedirect={onRedirect} />);
    await waitFor(() => expect(screen.getByText('Acme Assistant')).toBeInTheDocument());
    fireEvent.click(screen.getByRole('button', { name: 'Allow' }));
    await waitFor(() => expect(onRedirect).toHaveBeenCalledWith('https://app.example/cb?code=AUTHCODE'));
  });

  it('deny posts approve=false and redirects', async () => {
    const onRedirect = vi.fn();
    const fetcher = vi.fn(async (input: string, init?: RequestInit) => {
      if (input === '/oauth/decision') {
        const body = JSON.parse((init?.body as string) ?? '{}') as { approve: boolean };
        expect(body.approve).toBe(false);
        return okJson({ redirectUri: 'https://app.example/cb?error=access_denied' });
      }
      return okJson(context());
    });
    render(() => <ConsentScreen params={params} initialContext={context()} fetcher={fetcher} onRedirect={onRedirect} />);
    await waitFor(() => expect(screen.getByText('Acme Assistant')).toBeInTheDocument());
    fireEvent.click(screen.getByRole('button', { name: 'Deny' }));
    await waitFor(() => expect(onRedirect).toHaveBeenCalledWith('https://app.example/cb?error=access_denied'));
  });

  it('shows the unattended-send disclosure when the scope requests it', async () => {
    const scope = scopeToWire({ ...readOnlyScope('acct-1'), send: true, mcpTools: ['mail.send'], unattendedSend: true });
    render(() => <ConsentScreen params={params} initialContext={context({ requestedScope: scope })} onRedirect={vi.fn()} />);
    await waitFor(() => expect(screen.getByTestId('consent-unattended-send')).toBeInTheDocument());
  });
});
