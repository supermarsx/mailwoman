import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { apiFetch, startBootstrap } from './http.ts';
import { adoptBootstrap, clearBootstrap, readBootstrap, SEED_LIFETIME_MS, setBootstrapClock } from './bootstrap.ts';
import { activateAccountPreferences, suspendAccountPreferences } from './accountPreferenceLifecycle.ts';
import { installLinkedMailbox, setPrimaryMailboxAuthority } from './linkedMailboxSession.ts';
import { createLinkedAccountsApi } from './linkedAccounts.ts';
import { SettingsService } from '../screens/Settings/service.ts';

const AUTHORITY = 'msa1_primary_authority';
const me = { username: 'alice@example.test', accountId: 'acct-a', storageAuthority: AUTHORITY };
const section = (body: unknown, status = 200, etag = 'W/"x"') => ({ status, etag, body });

function answer(sections: Record<string, unknown>, init: { status?: number; contentType?: string; v?: number } = {}): Response {
  return new Response(JSON.stringify({ v: init.v ?? 1, sections }), {
    status: init.status ?? 200,
    headers: { 'content-type': init.contentType ?? 'application/json', etag: '"combined"' },
  });
}
const signedIn = (): Record<string, unknown> => ({
  me: section(me, 200, '"me-1"'),
  publicConfig: section({ brandName: 'Seeded' }),
  accounts: section({ primary: { accountId: 'acct-a' }, accounts: [] }),
  notifications: section({ enabled: true }, 200, '"n-1"'),
  features: section({ calendar: true }),
  policy: section({ error: 'forbidden' }, 403),
  drafts: section(null, 200),
  version: section('26.20.0'),
  appearanceCustomization: section({ css: '' }),
});

let network: ReturnType<typeof vi.fn>;
beforeEach(() => {
  clearBootstrap();
  network = vi.fn(async (url: string) => Response.json({ from: 'network', url }));
  vi.stubGlobal('fetch', network);
});
afterEach(() => {
  clearBootstrap();
  setBootstrapClock(null);
  installLinkedMailbox(null);
  suspendAccountPreferences();
  vi.unstubAllGlobals();
});

describe('bootstrap seed', () => {
  it('answers the first GET of a section from the seed, the next from the network', async () => {
    adoptBootstrap(Promise.resolve(answer(signedIn())));
    const first = await apiFetch('/api/me', { credentials: 'same-origin' });
    expect(first.status).toBe(200);
    expect(first.headers.get('etag')).toBe('"me-1"');
    expect(await first.json()).toEqual(me);
    expect(network).not.toHaveBeenCalled();
    const second = await apiFetch('/api/me', { credentials: 'same-origin' });
    expect(await second.json()).toEqual({ from: 'network', url: '/api/me' });
    expect(network).toHaveBeenCalledTimes(1);
  });

  it('waits for an answer still on the wire', async () => {
    let resolve!: (value: Response) => void;
    adoptBootstrap(new Promise<Response>((r) => { resolve = r; }));
    const read = apiFetch('/api/public-config');
    resolve(answer(signedIn()));
    expect(await (await read).json()).toEqual({ brandName: 'Seeded' });
    expect(network).not.toHaveBeenCalled();
  });

  it('never seeds a section whose route refused, so the caller handles the refusal itself', async () => {
    adoptBootstrap(Promise.resolve(answer(signedIn())));
    network.mockResolvedValueOnce(Response.json({ error: 'forbidden' }, { status: 403 }));
    const policy = await apiFetch('/api/policy');
    expect(policy.status).toBe(403);
    expect(network).toHaveBeenCalledWith('/api/policy');
  });

  it('keeps the route ETag only where the route sets one, and answers a matching If-None-Match with 304', async () => {
    adoptBootstrap(Promise.resolve(answer(signedIn())));
    const features = await apiFetch('/api/features');
    expect(features.headers.get('etag')).toBeNull();
    const notifications = await apiFetch('/api/account/notifications', { headers: { 'If-None-Match': '"n-1"' } });
    expect(notifications.status).toBe(304);
    expect(network).not.toHaveBeenCalled();
  });

  it('carries empty and text bodies as the route sent them', async () => {
    adoptBootstrap(Promise.resolve(answer(signedIn())));
    const drafts = await apiFetch('/api/account/drafts');
    expect(drafts.status).toBe(200);
    expect(await drafts.text()).toBe('');
    const version = await apiFetch('/api/version');
    expect(await version.text()).toBe('26.20.0');
  });

  it('seeds the customization read only for the account me names', async () => {
    adoptBootstrap(Promise.resolve(answer(signedIn())));
    expect(await (await apiFetch('/api/account/appearance/customization?accountId=acct-b')).json()).toMatchObject({ from: 'network' });
    expect(await (await apiFetch('/api/account/appearance/customization?accountId=acct-a')).json()).toEqual({ css: '' });
  });

  it('serves the account it was fetched for, and checks fence headers against it', async () => {
    adoptBootstrap(Promise.resolve(answer(signedIn())));
    activateAccountPreferences('acct-a', AUTHORITY);
    const other = await apiFetch('/api/account/notifications', { headers: { 'X-Mailwoman-Account': 'acct-a', 'x-mailwoman-storage-authority': 'msa1_someone_else_entirely' } });
    expect(await other.json()).toMatchObject({ from: 'network' });
    const own = await apiFetch('/api/features', { headers: { 'X-Mailwoman-Account': 'acct-a', 'x-mailwoman-storage-authority': AUTHORITY } });
    expect(await own.json()).toEqual({ calendar: true });
  });

  it('answers the first mailbox list and the shell notification read with their own fence headers', async () => {
    const accounts = { primary: { accountId: 'acct-a', address: 'a@example.test', storageAuthority: AUTHORITY }, accounts: [], policy: { enabled: true, maxAccounts: 4, supportedAuthentication: [] } };
    adoptBootstrap(Promise.resolve(answer({ ...signedIn(), accounts: section(accounts) })));
    activateAccountPreferences('acct-a', AUTHORITY);
    setPrimaryMailboxAuthority(AUTHORITY);
    try {
      const api = createLinkedAccountsApi();
      expect(await api.list()).toEqual(accounts);
      api.dispose();
      expect(await new SettingsService().notifications()).toEqual({ enabled: true });
      expect(network).not.toHaveBeenCalled();
    } finally { setPrimaryMailboxAuthority(undefined); }
  });

  it('drops every seed on an account switch', async () => {
    adoptBootstrap(Promise.resolve(answer(signedIn())));
    activateAccountPreferences('acct-a', AUTHORITY);
    expect(await (await apiFetch('/api/me')).json()).toEqual(me);
    activateAccountPreferences('acct-linked', 'msa1_linked_authority_1');
    expect(await (await apiFetch('/api/features')).json()).toMatchObject({ from: 'network' });
    // Back on the first account: the seed is gone for good.
    activateAccountPreferences('acct-a', AUTHORITY);
    expect(await (await apiFetch('/api/public-config')).json()).toMatchObject({ from: 'network' });
  });

  it('drops every seed on sign-out', async () => {
    adoptBootstrap(Promise.resolve(answer(signedIn())));
    activateAccountPreferences('acct-a', AUTHORITY);
    suspendAccountPreferences();
    expect(await (await apiFetch('/api/public-config')).json()).toMatchObject({ from: 'network' });
    expect(await (await apiFetch('/api/me')).json()).toMatchObject({ from: 'network' });
  });

  it('keeps the seed through the suspension that precedes the first session read', async () => {
    adoptBootstrap(Promise.resolve(answer(signedIn())));
    suspendAccountPreferences();
    expect(await (await apiFetch('/api/me')).json()).toEqual(me);
  });

  it('drops the seed when the answer is for another account than the one now active', async () => {
    let resolve!: (value: Response) => void;
    adoptBootstrap(new Promise<Response>((r) => { resolve = r; }));
    activateAccountPreferences('acct-b', 'msa1_other_authority_b');
    resolve(answer(signedIn()));
    expect(await (await apiFetch('/api/features')).json()).toMatchObject({ from: 'network' });
  });

  it('is never consulted while a linked mailbox is selected', async () => {
    adoptBootstrap(Promise.resolve(answer(signedIn())));
    activateAccountPreferences('acct-linked', 'msa1_linked_authority_1');
    installLinkedMailbox({ linkId: 'link-1', accountId: 'acct-linked', storageAuthority: 'msa1_linked_authority_1' } as never);
    await apiFetch('/api/public-config');
    expect(network).toHaveBeenCalledTimes(1);
  });

  it('keeps only public sections when signed out', async () => {
    adoptBootstrap(Promise.resolve(answer({
      me: section({ error: 'unauthorized' }, 401),
      publicConfig: section({ brandName: 'Seeded' }),
      features: section({ calendar: true }),
    })));
    expect(await (await apiFetch('/api/public-config')).json()).toEqual({ brandName: 'Seeded' });
    expect(await (await apiFetch('/api/me')).json()).toMatchObject({ from: 'network' });
    expect(await (await apiFetch('/api/features')).json()).toMatchObject({ from: 'network' });
  });

  it('expires an unused seed', async () => {
    let clock = 1_000;
    setBootstrapClock(() => clock);
    adoptBootstrap(Promise.resolve(answer(signedIn())));
    expect(await (await apiFetch('/api/me')).json()).toEqual(me);
    clock += SEED_LIFETIME_MS;
    expect(await (await apiFetch('/api/features')).json()).toMatchObject({ from: 'network' });
  });

  it('leaves requests with other headers, a body, a signal-free POST or another credential mode to the network', async () => {
    adoptBootstrap(Promise.resolve(answer(signedIn())));
    await apiFetch('/api/me', { headers: { Authorization: 'Bearer t' } });
    await apiFetch('/api/features', { credentials: 'omit' });
    await apiFetch('/api/account/notifications', { method: 'PUT', body: '{}' });
    expect(network).toHaveBeenCalledTimes(3);
  });

  it.each([
    ['a network failure', () => Promise.reject(new TypeError('offline'))],
    ['a 404 from an older server', () => Promise.resolve(new Response('not found', { status: 404 }))],
    ['a 405', () => Promise.resolve(new Response(null, { status: 405 }))],
    ['the SPA shell', () => Promise.resolve(new Response('<!doctype html>', { status: 200, headers: { 'content-type': 'text/html' } }))],
    ['a 304 reaching the page', () => Promise.resolve(new Response(null, { status: 304 }))],
    ['a maintenance refusal', () => Promise.resolve(Response.json({ error: 'maintenance' }, { status: 503 }))],
    ['an unknown version', () => Promise.resolve(answer(signedIn(), { v: 2 }))],
    ['a malformed body', () => Promise.resolve(new Response('{"v":1', { headers: { 'content-type': 'application/json' } }))],
  ])('falls back to the individual reads on %s', async (_label, outer) => {
    adoptBootstrap(outer());
    expect(await (await apiFetch('/api/me')).json()).toMatchObject({ from: 'network' });
    expect(await (await apiFetch('/api/public-config')).json()).toMatchObject({ from: 'network' });
    expect(network).toHaveBeenCalledTimes(2);
  });

  it('ignores a malformed section without losing the others', async () => {
    const seed = await readBootstrap(answer({ ...signedIn(), features: { status: 'ok' } }));
    expect(seed?.sections.has('/api/features')).toBe(false);
    expect(seed?.sections.has('/api/me')).toBe(true);
  });

  it('does not seed a section the server reported as too large', async () => {
    adoptBootstrap(Promise.resolve(answer({ ...signedIn(), drafts: { status: 200, etag: '', body: null, error: 'bodyTooLarge' } })));
    expect(await (await apiFetch('/api/account/drafts')).json()).toMatchObject({ from: 'network' });
  });

  it('starts the request itself, or adopts the one the boot script started', async () => {
    network.mockResolvedValueOnce(answer(signedIn()));
    startBootstrap();
    expect(network).toHaveBeenCalledWith('/api/bootstrap', { credentials: 'same-origin', headers: { accept: 'application/json' } });
    expect(await (await apiFetch('/api/me')).json()).toEqual(me);

    clearBootstrap();
    network.mockClear();
    (globalThis as { __mwBootstrap?: unknown }).__mwBootstrap = Promise.resolve(answer(signedIn()));
    startBootstrap();
    expect((globalThis as { __mwBootstrap?: unknown }).__mwBootstrap).toBeUndefined();
    expect(await (await apiFetch('/api/public-config')).json()).toEqual({ brandName: 'Seeded' });
    expect(network).not.toHaveBeenCalled();
  });
});
