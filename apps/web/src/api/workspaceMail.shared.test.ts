// t52w: a boot-time read that was sent more than once now shares one request.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { setSwrCachesEnabled } from './swrCache.ts';
import { clearPublicConfigCache } from './publicConfig.ts';
import { activateAccountPreferences, preferenceAccount, suspendAccountPreferences } from './accountPreferenceLifecycle.ts';
import { setPrimaryMailboxAuthority } from './linkedMailboxSession.ts';
import { readWorkspaceNotifications, type WorkspaceMailbox } from './workspaceMail.ts';
import { SettingsService } from '../screens/Settings/service.ts';

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

describe('notification policy', () => {
  const target = (): WorkspaceMailbox => ({ linkId: null, accountId: 'acct-a', storageAuthority: AUTHORITY, owner: preferenceAccount()!, label: 'a@example.test', current: true });

  it('the login summary uses the shell read of the active mailbox instead of asking again', async () => {
    await new SettingsService().notifications();
    expect(await readWorkspaceNotifications(target())).toEqual(policy);
    expect(network).toHaveBeenCalledTimes(1);
  });

  it('shares a shell read still in flight', async () => {
    const [shell, summary] = await Promise.all([new SettingsService().notifications(), Promise.resolve().then(() => readWorkspaceNotifications(target()))]);
    expect(shell).toEqual(policy);
    expect(summary).toEqual(policy);
    expect(network).toHaveBeenCalledTimes(1);
  });

  it('the shell itself always asks the server, and another mailbox never reuses the read', async () => {
    const service = new SettingsService();
    await service.notifications();
    await service.notifications();
    expect(network).toHaveBeenCalledTimes(2);
    await readWorkspaceNotifications({ ...target(), linkId: 'link-b', accountId: 'acct-b', storageAuthority: 'msa1_linked_authority_b', current: false });
    expect(network).toHaveBeenCalledTimes(3);
  });

  it('asks on its own after an account change', async () => {
    await new SettingsService().notifications();
    activateAccountPreferences('acct-a', AUTHORITY + '_x');
    activateAccountPreferences('acct-a', AUTHORITY);
    await readWorkspaceNotifications(target());
    expect(network).toHaveBeenCalledTimes(2);
  });
});
