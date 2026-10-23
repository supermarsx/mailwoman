import { describe, it, expect, vi, afterEach } from 'vitest';
import { createRoot, createSignal } from 'solid-js';
import { CAPABILITIES, createAdminMeLoader, createCan, type AdminGrants } from './can.ts';
import { SECTIONS } from './sections.ts';

afterEach(() => vi.unstubAllGlobals());

const json = (status: number, body: unknown): Response =>
  new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });

describe('GET /admin/me loader', () => {
  it('404 (no RBAC on this server yet) means the env admin: every capability', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('', { status: 404 })));
    expect(await createAdminMeLoader('')()).toEqual({ kind: 'all', reason: 'no-rbac' });
  });

  it('a listed answer is used as is', async () => {
    const me = { username: 'h', displayName: null, roles: ['Helpdesk'], capabilities: ['users.read'], domainScope: null, totpEnrolled: false };
    vi.stubGlobal('fetch', vi.fn(async () => json(200, me)));
    expect(await createAdminMeLoader('')()).toEqual({ kind: 'listed', me });
  });

  it('an unreadable answer falls back to showing everything (the server still enforces)', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => json(500, {})));
    expect((await createAdminMeLoader('')()).kind).toBe('all');
    vi.stubGlobal('fetch', vi.fn(async () => json(200, { username: 'x' })));
    expect((await createAdminMeLoader('')()).kind).toBe('all');
    vi.stubGlobal('fetch', vi.fn(async () => Promise.reject(new TypeError('offline'))));
    expect((await createAdminMeLoader('')()).kind).toBe('all');
  });
});

describe('can()', () => {
  it('allows only listed capabilities and gives the reason for the rest', () => {
    createRoot((dispose) => {
      const [g] = createSignal<AdminGrants>({
        kind: 'listed',
        me: { username: 'h', displayName: null, roles: [], capabilities: ['users.read'], domainScope: null, totpEnrolled: false },
      });
      const c = createCan(g);
      expect(c.can('users.read')).toBe(true);
      expect(c.can('users.write')).toBe(false);
      expect(c.reason('users.read')).toBeNull();
      expect(c.reason('users.write')).toBe('Your role does not include users.write.');
      dispose();
    });
  });

  it('all-capable grants allow everything', () => {
    createRoot((dispose) => {
      const [g] = createSignal<AdminGrants>({ kind: 'all', reason: 'no-rbac' });
      const c = createCan(g);
      for (const cap of CAPABILITIES) expect(c.can(cap)).toBe(true);
      dispose();
    });
  });

  it('every section names a capability from the catalogue', () => {
    for (const s of SECTIONS) expect(CAPABILITIES).toContain(s.capability);
  });
});
