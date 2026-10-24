// Stock Playwright on purpose, NOT `./csrf.ts`'s `test`: nothing may add the token
// here. Importing a constant from `./csrf.ts` patches nothing; only its `test`
// fixtures do.
import { test, expect, type APIRequestContext } from '@playwright/test';
import { ANONYMOUS_PAIR } from './csrf.ts';
import { CREDS, ENGINE_CREDS } from './helpers.ts';

/**
 * Is the stack under test running strict CSRF (`MW_CSRF_STRICT=true`)?
 *
 * Every e2e stack is meant to: it is what `deploy.sh` writes for a new
 * installation, and the specs beside this one — the web client through
 * `apiFetch`, the request-fixture specs through `./csrf.ts` — all send the token,
 * so they pass against a server that checks nothing just as well. This spec is
 * the one that fails there. It is in the `testMatch` of every project that has a
 * server of its own, so each CI job checks the stack it started; if a compose file
 * or a workflow step loses `MW_CSRF_STRICT`, that job goes red here.
 *
 * The signed-out cases need no account: the guard runs before any handler, so a
 * signed-out `POST /api/logout` is enough (the handler answers 204 to anyone). The
 * signed-in case signs in to the project's mailbox account, because a jar that
 * holds a real session is what a forged request rides on.
 *
 * `POST /api/session/rotate` is the session-gated write: it is in the router every
 * mode mounts (`crates/mw-server/src/lib.rs`, beside `/api/login`), answers 401
 * without a session and 200 with one.
 *
 * It never skips: a skip is how a stack that is not strict would go unnoticed.
 */

/**
 * The `/api/login` body for the mailbox account of each project that runs this
 * spec. A project that adds this spec to its `testMatch` must be listed here; an
 * unlisted one fails rather than skipping.
 */
// The server picks the mail server from its MW_UPSTREAM_DEFAULT; a sign-in sends
// only the user name and the password (t32-h9).
const MAILBOX_ACCOUNT: Record<string, { username: string; password: string }> = {
  // Proxy mode: the in-repo JMAP mock (`mock`), or the one the v6/v7 jobs publish
  // on loopback.
  mock: CREDS,
  v6: CREDS,
  v7: CREDS,
  // Engine mode over Greenmail.
  ...Object.fromEntries(
    ['engine', 'pim', 'crypto', 'push', 't12'].map((name) => [
      name,
      { username: ENGINE_CREDS.username, password: ENGINE_CREDS.password },
    ]),
  ),
};

const CSRF_REFUSAL = { error: 'missing or invalid CSRF token' };

/** The `mw_csrf` value in `ctx`'s jar, or undefined. */
async function csrfCookie(ctx: APIRequestContext): Promise<string | undefined> {
  const { cookies } = await ctx.storageState();
  return cookies.find((c) => c.name === 'mw_csrf' && c.value !== '')?.value;
}

test.describe('the stack under test enforces the CSRF token', () => {
  test('a write without X-CSRF-Token is refused, and the matching pair is let through', async ({ request }) => {
    // No cookie, no header: what a client that knows nothing of the token sends.
    const bare = await request.post('/api/logout');
    expect(bare.status(), 'MW_CSRF_STRICT is on for this stack').toBe(403);
    expect(await bare.json()).toEqual(CSRF_REFUSAL);

    // A header that does not match the cookie is refused the same way.
    const mismatch = await request.post('/api/logout', {
      headers: { cookie: 'mw_csrf=e2e-one', 'x-csrf-token': 'e2e-two' },
    });
    expect(mismatch.status()).toBe(403);
    expect(await mismatch.json()).toEqual(CSRF_REFUSAL);

    // Control: the same write with the pair reaches the handler. This is what
    // shows the 403s above come from the token check and not from the route.
    const paired = await request.post('/api/logout', {
      headers: { cookie: 'mw_csrf=e2e-pair', 'x-csrf-token': 'e2e-pair' },
    });
    expect(paired.status(), 'the pair passes the guard').toBe(204);
  });

  test('a matching pair without a session gets the auth gate’s 401 on a session-gated write', async ({ request }) => {
    // Without the pair the guard answers first.
    const bare = await request.post('/api/session/rotate');
    expect(bare.status(), 'MW_CSRF_STRICT is on for this stack').toBe(403);
    expect(await bare.json()).toEqual(CSRF_REFUSAL);

    // With it the request reaches the handler, which finds no session. This is
    // the pair the request-fixture specs send for their signed-out writes.
    const anonymous = await request.post('/api/session/rotate', { headers: ANONYMOUS_PAIR });
    expect(anonymous.status(), 'the pair passes the guard and the route refuses the caller').toBe(401);
  });

  test('a signed-in write without X-CSRF-Token is refused, and with it is accepted', async ({ request }, testInfo) => {
    const account = MAILBOX_ACCOUNT[testInfo.project.name];
    expect(account, `project "${testInfo.project.name}" is listed in MAILBOX_ACCOUNT`).toBeDefined();

    const login = await request.post('/api/login', { data: account });
    expect(login.status(), 'mailbox sign-in').toBe(200);
    // Precondition: the jar now holds the session and the token a forged request
    // would ride on.
    const { cookies } = await request.storageState();
    expect(cookies.some((c) => c.name === 'mw_session' && c.value !== ''), 'the jar holds mw_session').toBe(true);
    const token = await csrfCookie(request);
    expect(token, 'the jar holds mw_csrf').toBeDefined();

    // The jar sends both cookies; nothing sends the header.
    const forged = await request.post('/api/session/rotate');
    expect(forged.status(), 'a signed-in write without the header is refused').toBe(403);
    expect(await forged.json()).toEqual(CSRF_REFUSAL);

    // The same write with the cookie repeated in the header is the session's own.
    const own = await request.post('/api/session/rotate', { headers: { 'x-csrf-token': token as string } });
    expect(own.status(), 'a signed-in write with the header is accepted').toBe(200);
    expect(await own.json()).toMatchObject({ ok: true });
  });
});
