// t52w: a boot-time read that was sent more than once now shares one request.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { setSwrCachesEnabled } from './swrCache.ts';
import { clearPublicConfigCache } from './publicConfig.ts';
import { createLinkedAccountsApi } from './linkedAccounts.ts';
import { activateAccountPreferences, suspendAccountPreferences } from './accountPreferenceLifecycle.ts';
import { invalidateLinkedAccounts, setPrimaryMailboxAuthority } from './linkedMailboxSession.ts';

const AUTHORITY = 'msa1_primary_authority';
const list = { primary: { accountId: 'acct-a', address: 'a@example.test', storageAuthority: AUTHORITY }, accounts: [], policy: { enabled: true, maxAccounts: 4, supportedAuthentication: [] } };
const policy = { enabled: true, rules: [], quietHours: { enabled: false } };

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

describe('linked mailbox list', () => {
  it('API objects of one account share one request, each with its own copy', async () => {
    const first = createLinkedAccountsApi(), second = createLinkedAccountsApi(), third = createLinkedAccountsApi();
    const [a, b] = await Promise.all([first.list(), second.list()]);
    const c = await third.list();
    expect(a).toEqual(list);
    expect(a).not.toBe(b);
    expect(c).toEqual(list);
    expect(network).toHaveBeenCalledTimes(1);
    for (const api of [first, second, third]) api.dispose();
  });

  it('one caller disposing does not cancel the read another is waiting for', async () => {
    const first = createLinkedAccountsApi(), second = createLinkedAccountsApi();
    const a = first.list(), b = second.list();
    first.dispose();
    await expect(a).rejects.toMatchObject({ name: 'AbortError' });
    expect(await b).toEqual(list);
    second.dispose();
  });

  it('reads again after invalidateLinkedAccounts, a write, or an account change', async () => {
    const api = createLinkedAccountsApi();
    await api.list();
    invalidateLinkedAccounts();
    await api.list();
    expect(network).toHaveBeenCalledTimes(2);
    network.mockResolvedValueOnce(Response.json({ primaryMailbox: null }));
    await api.setPrimary(null);
    await api.list();
    expect(network).toHaveBeenCalledTimes(4);
    api.dispose();
    activateAccountPreferences('acct-a', 'msa1_another_authority');
    setPrimaryMailboxAuthority('msa1_another_authority');
    const other = createLinkedAccountsApi();
    network.mockResolvedValueOnce(Response.json({ ...list, primary: { ...list.primary, storageAuthority: 'msa1_another_authority' } }));
    expect((await other.list()).primary.storageAuthority).toBe('msa1_another_authority');
    expect(network).toHaveBeenCalledTimes(5);
    other.dispose();
  });
});

