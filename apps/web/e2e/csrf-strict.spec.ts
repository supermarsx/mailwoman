// Stock Playwright on purpose, NOT `./csrf.ts`: nothing may add the token here.
import { test, expect } from '@playwright/test';

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
 * It needs no account. The guard runs before any handler, so a signed-out
 * `POST /api/logout` is enough: the handler itself answers 204 to anyone.
 *
 * It never skips: a skip is how a stack that is not strict would go unnoticed.
 */
test.describe('the stack under test enforces the CSRF token', () => {
  test('a write without X-CSRF-Token is refused, and the matching pair is let through', async ({ request }) => {
    // No cookie, no header: what a client that knows nothing of the token sends.
    const bare = await request.post('/api/logout');
    expect(bare.status(), 'MW_CSRF_STRICT is on for this stack').toBe(403);
    expect(await bare.json()).toEqual({ error: 'missing or invalid CSRF token' });

    // A header that does not match the cookie is refused the same way.
    const mismatch = await request.post('/api/logout', {
      headers: { cookie: 'mw_csrf=e2e-one', 'x-csrf-token': 'e2e-two' },
    });
    expect(mismatch.status()).toBe(403);
    expect(await mismatch.json()).toEqual({ error: 'missing or invalid CSRF token' });

    // Control: the same write with the pair reaches the handler. This is what
    // shows the 403s above come from the token check and not from the route.
    const paired = await request.post('/api/logout', {
      headers: { cookie: 'mw_csrf=e2e-pair', 'x-csrf-token': 'e2e-pair' },
    });
    expect(paired.status(), 'the pair passes the guard').toBe(204);
  });
});
