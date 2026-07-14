import { test, expect, type Page } from '@playwright/test';

/**
 * LIVE OIDC SSO login E2E (t9-e6, plan §10 DoD). Drives a REAL Chromium browser against a
 * REAL Keycloak IdP (docker-compose.ci.yml `keycloak`, realm scripts/keycloak/realm.json):
 *
 *   "Sign in with Keycloak" (a full redirect to GET /api/sso/{id}/begin) → Keycloak login
 *   form → redirect back → authenticated inbox — a real openidconnect discovery / JWKS /
 *   auth-code+PKCE round-trip. The deep headless proof runs through the same server routes in
 *   crates/mw-server/tests/sso_live.rs (`oidc_login_end_to_end`), which the SSO-E2E job runs
 *   alongside.
 *
 * NOTE (escalated, t9-e6): the login screen currently renders NO SSO button because the web
 * client `listSsoProviders` (apps/web/src/modules/sso/client.ts) parses the response as a
 * bare array while the server returns `{ "providers": [...] }` — so `renders_the_button`
 * is an HONEST-RED regression guard until that one-line parse fix lands. The login FLOW
 * itself is proven by navigating the button's own href (`/api/sso/{id}/begin`), which is
 * exactly what the button triggers.
 *
 * Self-skips loudly when no SSO-enabled mw-server + OIDC provider is reachable.
 */

const OIDC_BEGIN = '/api/sso/corp-oidc/begin';
const KC_USER = process.env['MW_SSO_KC_USER'] ?? 'ada';
const KC_PASS = process.env['MW_SSO_KC_PASS'] ?? 'keycloak-test-pw';
const KC_EMAIL = process.env['MW_SSO_KC_EMAIL'] ?? 'ada@mailwoman.test';

async function keycloakLogin(page: Page): Promise<void> {
  await page.waitForURL(/\/realms\/mailwoman\/protocol\/openid-connect|\/login-actions\//, {
    timeout: 15_000,
  });
  await page.fill('#username', KC_USER);
  await page.fill('#password', KC_PASS);
  await page.click('#kc-login, input[type="submit"], button[type="submit"]');
}

test.describe('SSO OIDC (26.9) — live Keycloak browser login', () => {
  test.beforeEach(async ({ request }, testInfo) => {
    const probe = await request.get('/api/sso/providers').catch(() => null);
    test.skip(
      probe === null || !probe.ok(),
      `[t9-e6 SKIP] ${testInfo.title}: no SSO-enabled mw-server reachable (start the sso stack).`,
    );
    const body = await probe!.json();
    const hasOidc = (body.providers ?? []).some((p: { kind?: string }) => p.kind === 'oidc');
    test.skip(!hasOidc, `[t9-e6 SKIP] ${testInfo.title}: no OIDC provider advertised.`);
  });

  test('Keycloak OIDC login → authenticated inbox', async ({ page }) => {
    // Navigate the exact redirect the "Sign in with Keycloak" button issues (its href is
    // ssoBeginPath = /api/sso/{id}/begin). mw-server 303s to Keycloak.
    await page.goto(OIDC_BEGIN);

    // Real Keycloak login page → submit the test user's credentials.
    await keycloakLogin(page);

    // The IdP 302s to the OIDC callback; mw-server exchanges the code (JWKS-validated ID
    // token) and 303s back into the SPA carrying the session cookie.
    await page.waitForURL(
      (url) => !url.pathname.includes('/realms/') && !url.search.includes('sso_error'),
      { timeout: 20_000 },
    );

    // Authenticated: the SSO session serves the mailbox surface as the resolved account.
    const me = await page.request.get('/api/me');
    expect(me.status(), 'the OIDC session authenticates /api/me').toBe(200);
    const who = (await me.json()) as { username?: string };
    expect(
      (who.username ?? '').toLowerCase(),
      'the OIDC identity resolved to the seeded Mailwoman account',
    ).toBe(KC_EMAIL.toLowerCase());
  });

  test('login screen renders the "Sign in with Keycloak" button', async ({ page }) => {
    // The user-facing contract: an enabled IdP surfaces a full-redirect <a data-sso-id> on
    // the login screen (Login.tsx). HONEST-RED until the escalated listSsoProviders parse
    // fix lands (client reads a bare array; server returns { providers: [...] }).
    await page.goto('/');
    const oidcBtn = page.locator('a[data-sso-id="corp-oidc"], a[data-sso-id]').first();
    await expect(
      oidcBtn,
      'the "Sign in with Keycloak" button renders (see t9-e6 escalation if this fails)',
    ).toBeVisible();
    await expect(oidcBtn).toHaveAttribute('href', OIDC_BEGIN);
  });
});
