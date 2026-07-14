import { test, expect, type APIRequestContext } from '@playwright/test';
import { V7, expectMounted } from './v7-helpers.ts';

/**
 * t10 §11 — Masked-email (SPEC §28.4) live E2E against the REAL 26.10 server.
 *
 * Drives the mounted `/api/masked/*` surface through Playwright's `request` fixture (a
 * real HTTP client with a cookie jar): generate an alias → it appears in the list; the
 * disable/enable/delete lifecycle reflects in the list; and per-user scoping isolates
 * accounts. Every route is mailbox-session-authed (fail-closed). Self-skips loudly when
 * no 26.10 stack is reachable, matching the v6/v7 precedent.
 *
 * NOTE (e13 handoff): the on-send From-rewrite is DEFERRED (no per-send alias-selection
 * seam yet), so this spec asserts the alias LIFECYCLE + scoping, NOT automatic envelope
 * rewrite on send.
 */

/** Cookie-authenticate a mailbox session in a fresh request context; returns accountId. */
async function loginAs(ctx: APIRequestContext, username: string, password: string): Promise<string> {
  const resp = await ctx.post('/api/login', {
    data: { jmapUrl: V7.jmapUrl, username, password },
  });
  expect(resp.status(), `mailbox login (${username}) succeeds`).toBe(200);
  return (await resp.json()).accountId as string;
}

interface Alias {
  id: string; email: string; state: string; target: string; description: string | null;
}
async function listAliases(ctx: APIRequestContext): Promise<Alias[]> {
  const resp = await ctx.get('/api/masked');
  expect(resp.status()).toBe(200);
  return (await resp.json()).aliases as Alias[];
}

test.describe('Masked-email lifecycle on the real server', () => {
  test('the /api/masked surface is mounted and fail-closed without a session', async ({ request }) => {
    const list = await request.get('/api/masked');
    expectMounted(list.status(), 'GET /api/masked');
    expect(list.status(), 'unauthenticated ⇒ 401 (mounted + fail-closed)').toBe(401);
  });

  test('generate → list → disable → enable → delete round-trips in the UI surface', async ({ playwright, baseURL }) => {
    const ctx = await playwright.request.newContext({ baseURL });
    await loginAs(ctx, V7.mailUser, V7.mailPass);

    // Generate a fresh alias with a description; it comes back enabled with a unique address.
    const gen = await ctx.post('/api/masked', { data: { description: 'e15 shopping' } });
    expect(gen.status(), 'generate ⇒ 201').toBe(201);
    const alias = (await gen.json()) as Alias;
    expect(alias.email).toMatch(/@/);
    expect(alias.state).toBe('enabled');

    // It appears in the list.
    let aliases = await listAliases(ctx);
    expect(aliases.some((a) => a.id === alias.id && a.email === alias.email)).toBe(true);

    // A second alias must be a DISTINCT address (unguessable, per-create seed).
    const gen2 = await ctx.post('/api/masked', { data: { description: 'e15 second' } });
    const alias2 = (await gen2.json()) as Alias;
    expect(alias2.email).not.toBe(alias.email);

    // Disable → the list reflects `disabled`.
    const disabled = await ctx.post(`/api/masked/${alias.id}/state`, { data: { state: 'disabled' } });
    expect(disabled.status()).toBe(200);
    expect((await disabled.json()).state).toBe('disabled');
    aliases = await listAliases(ctx);
    expect(aliases.find((a) => a.id === alias.id)!.state).toBe('disabled');

    // Enable again → back to `enabled`.
    const enabled = await ctx.post(`/api/masked/${alias.id}/state`, { data: { state: 'enabled' } });
    expect((await enabled.json()).state).toBe('enabled');

    // An invalid state is rejected (400) — the lifecycle is a closed set.
    const bad = await ctx.post(`/api/masked/${alias.id}/state`, { data: { state: 'bogus' } });
    expect(bad.status()).toBe(400);

    // Delete (soft tombstone) → 204 and it disappears from the active list.
    expect((await ctx.delete(`/api/masked/${alias.id}`)).status()).toBe(204);
    aliases = await listAliases(ctx);
    expect(aliases.some((a) => a.id === alias.id)).toBe(false);

    // Cleanup the second alias too.
    await ctx.delete(`/api/masked/${alias2.id}`);
    await ctx.dispose();
  });

  test('per-user scoping: one account cannot see or mutate another account’s alias', async ({ playwright, baseURL }) => {
    const other = process.env['MW_E2E_USERNAME_B'];
    test.skip(
      other === undefined,
      'per-user scoping needs a SECOND mailbox account (MW_E2E_USERNAME_B/PASSWORD_B); the single-account mock cannot prove it.',
    );

    const a = await playwright.request.newContext({ baseURL });
    const b = await playwright.request.newContext({ baseURL });
    await loginAs(a, V7.mailUser, V7.mailPass);
    await loginAs(b, other!, process.env['MW_E2E_PASSWORD_B']!);

    const gen = await a.post('/api/masked', { data: { description: 'private' } });
    const alias = (await gen.json()) as Alias;

    // Account B cannot see A's alias, and a cross-account mutation is a uniform 404.
    expect((await listAliases(b)).some((x) => x.id === alias.id)).toBe(false);
    expect((await b.post(`/api/masked/${alias.id}/state`, { data: { state: 'disabled' } })).status()).toBe(404);
    expect((await b.delete(`/api/masked/${alias.id}`)).status()).toBe(404);
    // A still owns it.
    expect((await listAliases(a)).some((x) => x.id === alias.id)).toBe(true);

    await a.delete(`/api/masked/${alias.id}`);
    await a.dispose();
    await b.dispose();
  });
});

test.beforeEach(async ({ request }, testInfo) => {
  const probe = await request.get('/api/masked').catch(() => null);
  test.skip(probe === null, `[e15 SKIP] ${testInfo.title}: no 26.10 mw-server reachable.`);
});
