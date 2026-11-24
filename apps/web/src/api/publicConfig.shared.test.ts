// t52w: a boot-time read that was sent more than once now shares one request.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { setSwrCachesEnabled } from './swrCache.ts';
import { clearPublicConfigCache, publicConfig, refreshPublicConfig } from './publicConfig.ts';
import { activateAccountPreferences, suspendAccountPreferences } from './accountPreferenceLifecycle.ts';
import { setPrimaryMailboxAuthority } from './linkedMailboxSession.ts';

const AUTHORITY = 'msa1_primary_authority';
const list = { primary: { accountId: 'acct-a', address: 'a@example.test', storageAuthority: AUTHORITY }, accounts: [], policy: { enabled: true, maxAccounts: 4, supportedAuthentication: [] } };
const policy = { enabled: true, rules: [], quietHours: { enabled: false } };
const path = (call: unknown[]): string => new URL(String(call[0]), 'http://localhost').pathname;

let network: ReturnType<typeof vi.fn>;
beforeEach(() => {
  setSwrCachesEnabled(true);
  clearPublicConfigCache();
  network = vi.fn(async (url: string) => {
    const route = new URL(url, 'http://localhost').pathname;
    if (route === '/api/public-config') return Response.json({ brandName: 'Mail', theme: 'auto', accent: 'blue', loginNotice: null, usernameDomainHint: null });
    if (route === '/api/accounts') return Response.json(list);
    if (route === '/api/account/notifications') return Response.json(policy);
    return Response.json({});
  });
  vi.stubGlobal('fetch', network);
  activateAccountPreferences('acct-a', AUTHORITY);
  setPrimaryMailboxAuthority(AUTHORITY);
});
afterEach(() => {
  setSwrCachesEnabled(false);
  clearPublicConfigCache();
  setPrimaryMailboxAuthority(undefined);
  suspendAccountPreferences();
  vi.unstubAllGlobals();
});

describe('public configuration', () => {
  it('a bounded refresh shares the request and fills the cache the sign-in page reads', async () => {
    const controller = new AbortController();
    const [bounded, plain] = await Promise.all([refreshPublicConfig(controller.signal), publicConfig()]);
    expect(bounded.brandName).toBe('Mail');
    expect(plain.brandName).toBe('Mail');
    await publicConfig();
    expect(network.mock.calls.filter((call) => path(call) === '/api/public-config')).toHaveLength(1);
  });

  it('an aborted bounded refresh ends only its own wait', async () => {
    let release!: () => void;
    network.mockImplementationOnce(() => new Promise<Response>((resolve) => { release = () => resolve(Response.json({ brandName: 'Late' })); }));
    const controller = new AbortController();
    const bounded = refreshPublicConfig(controller.signal);
    const plain = publicConfig();
    controller.abort();
    await expect(bounded).rejects.toBeDefined();
    release();
    expect((await plain).brandName).toBe('Late');
    expect((await publicConfig()).brandName).toBe('Late');
    expect(network).toHaveBeenCalledTimes(1);
  });
});

