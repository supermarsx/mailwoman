import { describe, it, expect, vi } from 'vitest';
import {
  analyzeBlockedContent,
  coveringGrant,
  createRemoteImageApi,
  hasBlockedContent,
  scopeFor,
  senderDomain,
  type RemoteImageGrant,
} from './remote-images.ts';
import type { Invocation, JmapRequest, JmapResponse } from './jmap-types.ts';

// A `Pick<Client,'jmap'>` fake that records the request and returns a canned
// response — the grant methods are the whole e6 seam, so we assert their shape.
function fakeClient(response: JmapResponse) {
  const calls: JmapRequest[] = [];
  return {
    calls,
    jmap: vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
      calls.push(body);
      return response;
    }),
  };
}

describe('senderDomain', () => {
  it('extracts the domain, lower-cased', () => {
    expect(senderDomain('Alice@Example.COM')).toBe('example.com');
  });
  it('is empty for an address with no @', () => {
    expect(senderDomain('local-only')).toBe('');
  });
});

describe('scopeFor', () => {
  const ctx = { emailId: 'M1', sender: 'Bob@Spam.Example' };
  it('single uses the message id', () => {
    expect(scopeFor('single', ctx)).toEqual({ kind: 'single', value: 'M1' });
  });
  it('per-sender lower-cases the address', () => {
    expect(scopeFor('per-sender', ctx)).toEqual({ kind: 'per-sender', value: 'bob@spam.example' });
  });
  it('per-domain uses the domain', () => {
    expect(scopeFor('per-domain', ctx)).toEqual({ kind: 'per-domain', value: 'spam.example' });
  });
  it('all uses the empty account-wide value', () => {
    expect(scopeFor('all', ctx)).toEqual({ kind: 'all', value: '' });
  });
});

describe('analyzeBlockedContent', () => {
  it('is empty for null / empty / unmarked HTML', () => {
    expect(analyzeBlockedContent(null)).toEqual({ blockedHosts: [], blockedCount: 0, trackerCount: 0 });
    expect(analyzeBlockedContent('')).toEqual({ blockedHosts: [], blockedCount: 0, trackerCount: 0 });
    expect(analyzeBlockedContent('<p>hello <img src="cid:x"></p>')).toEqual({
      blockedHosts: [],
      blockedCount: 0,
      trackerCount: 0,
    });
    expect(hasBlockedContent(analyzeBlockedContent('<p>x</p>'))).toBe(false);
  });

  it('counts marked blocked resources and dedups + sorts hosts', () => {
    const html = `
      <img data-mw-blocked-host="tracker.evil">
      <img data-mw-blocked-host="cdn.example">
      <img data-mw-blocked-host="tracker.evil" data-mw-tracker>
    `;
    const r = analyzeBlockedContent(html);
    expect(r.blockedCount).toBe(3);
    expect(r.blockedHosts).toEqual(['cdn.example', 'tracker.evil']);
    expect(r.trackerCount).toBe(1);
    expect(hasBlockedContent(r)).toBe(true);
  });

  it('normalizes host case and ignores empty host markers in the host list', () => {
    const html = `<img data-mw-blocked-host="Beacon.Example"><span data-mw-blocked-host="" data-mw-tracker></span>`;
    const r = analyzeBlockedContent(html);
    // Both are blocked resources; only the non-empty host lists.
    expect(r.blockedCount).toBe(2);
    expect(r.blockedHosts).toEqual(['beacon.example']);
    expect(r.trackerCount).toBe(1);
  });
});

describe('coveringGrant', () => {
  const ctx = { emailId: 'M1', sender: 'bob@spam.example' };
  const grant = (scopeKind: RemoteImageGrant['scopeKind'], scopeValue: string): RemoteImageGrant => ({
    scopeKind,
    scopeValue,
    grantedAt: '2026-07-19T00:00:00Z',
  });

  it('returns null when nothing covers the message', () => {
    expect(coveringGrant([grant('per-sender', 'other@x.example')], ctx)).toBeNull();
    expect(coveringGrant([], ctx)).toBeNull();
  });
  it('matches an account-wide all grant', () => {
    expect(coveringGrant([grant('all', '')], ctx)?.scopeKind).toBe('all');
  });
  it('matches the message id for a single grant', () => {
    expect(coveringGrant([grant('single', 'M1')], ctx)?.scopeKind).toBe('single');
    expect(coveringGrant([grant('single', 'M2')], ctx)).toBeNull();
  });
  it('matches sender / domain case-insensitively', () => {
    expect(coveringGrant([grant('per-sender', 'BOB@spam.example')], ctx)?.scopeKind).toBe('per-sender');
    expect(coveringGrant([grant('per-domain', 'Spam.Example')], ctx)?.scopeKind).toBe('per-domain');
  });
});

describe('createRemoteImageApi', () => {
  function okResponse(callId: string, args: Record<string, unknown> = {}): JmapResponse {
    return { methodResponses: [[callId === 'g' ? 'RemoteImage/get' : 'RemoteImage/set', args, callId]], sessionState: 's' };
  }

  it('grant sends RemoteImage/set with a grant arg', async () => {
    const c = fakeClient(okResponse('s'));
    const api = createRemoteImageApi(c);
    await api.grant('acct1', { kind: 'per-domain', value: 'spam.example' });
    const [call] = c.calls[0]!.methodCalls as [Invocation];
    expect(call[0]).toBe('RemoteImage/set');
    expect(call[1]).toEqual({
      accountId: 'acct1',
      grant: { scopeKind: 'per-domain', scopeValue: 'spam.example' },
    });
  });

  it('revoke sends RemoteImage/set with a revoke arg', async () => {
    const c = fakeClient(okResponse('s'));
    const api = createRemoteImageApi(c);
    await api.revoke('acct1', { kind: 'all', value: '' });
    const [call] = c.calls[0]!.methodCalls as [Invocation];
    expect(call[1]).toEqual({ accountId: 'acct1', revoke: { scopeKind: 'all', scopeValue: '' } });
  });

  it('listGrants returns the RemoteImage/get list', async () => {
    const grants: RemoteImageGrant[] = [{ scopeKind: 'all', scopeValue: '', grantedAt: '2026-07-19T00:00:00Z' }];
    const c = fakeClient(okResponse('g', { accountId: 'acct1', list: grants }));
    const api = createRemoteImageApi(c);
    expect(await api.listGrants('acct1')).toEqual(grants);
  });

  it('propagates a JMAP method error', async () => {
    const errRes: JmapResponse = {
      methodResponses: [['error', { type: 'forbidden', description: 'no' }, 's']],
      sessionState: 's',
    };
    const api = createRemoteImageApi(fakeClient(errRes));
    await expect(api.grant('acct1', { kind: 'single', value: 'M1' })).rejects.toThrow(/forbidden/);
  });
});
