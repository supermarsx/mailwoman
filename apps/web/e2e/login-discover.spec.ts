import { test, expect, type Page } from '@playwright/test';
import { ENGINE_CREDS, sidebarInbox } from './helpers.ts';

/**
 * The sign-in screen's server lookup (t28-e4), against the REAL engine stack
 * (mw-server in MW_MODE=engine over Greenmail — the `engine` project).
 *
 * The screen opens on an email address and a password, asks the server which
 * mail server belongs to the address (`POST /api/discover`), shows the answer,
 * and signs in to it once the user confirms. The server URL and username fields
 * are behind "Enter server details manually", and are where every failed lookup
 * lands, with the typed address kept.
 *
 * What is real here and what is not:
 *  - The negative case calls the real `/api/discover`. The address is in
 *    `.invalid` (RFC 2606), which no rung of the lookup can resolve.
 *  - The positive case answers `/api/discover` itself, with the body the server
 *    serialises (`crates/mw-server/src/lib.rs:2276-2283`, the `AccountCandidate`
 *    of `crates/mw-autoconfig/src/lib.rs:86-93`). The stack has no fixture that
 *    makes a lookup of `example.org` return the in-network `greenmail` host: the
 *    handler does live DNS and HTTPS only. Everything after the lookup — the
 *    `/api/login` call with the URL built from that answer, the IMAP session —
 *    is the real server.
 */

/** Host and port of the engine stack's IMAP server, from the URL the other specs sign in with. */
function engineImap(): { host: string; port: number; tls: 'implicit' | 'start-tls' } {
  const m = /^(imaps?):\/\/([^:/]+):(\d+)$/.exec(ENGINE_CREDS.imapUrl);
  if (m === null) throw new Error(`MW_E2E_ENGINE_IMAP_URL is not imap(s)://host:port: ${ENGINE_CREDS.imapUrl}`);
  return { host: m[2]!, port: Number(m[3]), tls: m[1] === 'imaps' ? 'implicit' : 'start-tls' };
}

async function openLogin(page: Page): Promise<void> {
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toBeVisible();
}

test.describe('Sign-in server lookup (engine mode)', () => {
  test('first paint asks for an email address and a password, not a server URL', async ({ page }) => {
    await openLogin(page);
    await expect(page.getByLabel('Email address')).toBeVisible();
    await expect(page.getByLabel('Password', { exact: true })).toBeVisible();
    await expect(page.getByLabel('JMAP server URL')).toHaveCount(0);
    await expect(page.getByLabel('Username', { exact: true })).toHaveCount(0);
    // The mock backend's credentials are no longer printed on the screen.
    await expect(page.getByText('testpass')).toHaveCount(0);
  });

  test('a found server is shown, confirmed, and signed in to with the URL built from it', async ({ page }) => {
    const imap = engineImap();
    let lookedUp: unknown = null;
    await page.route('**/api/discover', async (route) => {
      lookedUp = route.request().postDataJSON();
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          imap,
          pop3: null,
          smtp: { host: imap.host, port: 3025, tls: 'none' },
          auth: 'password',
          source: 'srv',
        }),
      });
    });
    const loginBodies: Array<{ jmapUrl: string; username: string }> = [];
    page.on('request', (req) => {
      if (req.method() === 'POST' && new URL(req.url()).pathname.endsWith('/api/login')) {
        loginBodies.push(req.postDataJSON() as { jmapUrl: string; username: string });
      }
    });

    await openLogin(page);
    // Precondition: no server is shown, and none is asked for, before the lookup.
    await expect(page.getByLabel('JMAP server URL')).toHaveCount(0);
    await expect(page.getByTestId('login-discovered')).toBeEmpty();

    await page.getByLabel('Email address').fill(ENGINE_CREDS.selfAddress);
    await page.getByLabel('Password', { exact: true }).fill(ENGINE_CREDS.password);
    await page.getByRole('button', { name: 'Sign in', exact: true }).click();

    const confirm = page.getByRole('button', { name: 'Sign in with this server' });
    await expect(confirm).toBeVisible();
    expect(lookedUp).toEqual({ email: ENGINE_CREDS.selfAddress });
    await expect(page.getByTestId('login-discovered')).toContainText(imap.host);
    await expect(page.getByTestId('login-discovered')).toContainText(`port ${imap.port}`);
    // Nothing has been sent to the login endpoint before the confirmation.
    expect(loginBodies).toEqual([]);

    await confirm.click();

    // The sign-in carries the URL built from the lookup and the address as the
    // username.
    const compose = page.getByRole('button', { name: 'Compose' });
    const refusedNote = page.getByTestId('login-discovered-refused-note');
    await expect(compose.or(refusedNote)).toBeVisible();
    expect(loginBodies[0]).toMatchObject({ jmapUrl: ENGINE_CREDS.imapUrl, username: ENGINE_CREDS.selfAddress });

    // Greenmail's login name is the local part (`testuser`), not the address, so
    // on the default stack the server refuses that sign-in. The screen then puts
    // what it sent into the manual fields; correcting the username there and
    // leaving the looked-up URL as it is must sign in. A stack whose login name
    // IS the address is already in the shell and skips this.
    if (await refusedNote.isVisible()) {
      await expect(page.getByLabel('JMAP server URL')).toHaveValue(ENGINE_CREDS.imapUrl);
      await expect(page.getByLabel('Username', { exact: true })).toHaveValue(ENGINE_CREDS.selfAddress);
      await page.getByLabel('Username', { exact: true }).fill(ENGINE_CREDS.username);
      await page.getByRole('button', { name: 'Sign in', exact: true }).click();
    }

    await expect(compose).toBeVisible();
    await expect(sidebarInbox(page)).toBeVisible();
    expect(loginBodies.at(-1)?.jmapUrl).toBe(ENGINE_CREDS.imapUrl);
  });

  test('an address nothing is found for opens the manual fields with the address kept', async ({ page }) => {
    // Each rung of the lookup is an outbound DNS or HTTPS request from the
    // server, and the ones that cannot connect run into their own timeouts.
    test.setTimeout(120_000);
    const address = 'someone@mailwoman-e2e-no-such-domain.invalid';
    await openLogin(page);
    await expect(page.getByLabel('JMAP server URL')).toHaveCount(0);

    await page.getByLabel('Email address').fill(address);
    await page.getByLabel('Password', { exact: true }).fill('irrelevant');
    const answered = page.waitForResponse((res) => new URL(res.url()).pathname.endsWith('/api/discover'), {
      timeout: 90_000,
    });
    await page.getByRole('button', { name: 'Sign in', exact: true }).click();

    // The real endpoint: 404 when every rung misses.
    const res = await answered;
    expect(res.status()).toBe(404);

    await expect(page.getByRole('alert')).toContainText('No server settings were found for');
    await expect(page.getByRole('alert')).toContainText('mailwoman-e2e-no-such-domain.invalid');
    await expect(page.getByLabel('JMAP server URL')).toBeVisible();
    await expect(page.getByLabel('JMAP server URL')).toBeFocused();
    await expect(page.getByLabel('Username', { exact: true })).toHaveValue(address);
    await expect(page.getByLabel('Password', { exact: true })).toHaveValue('irrelevant');
    // No session came out of it.
    await expect(page.getByRole('button', { name: 'Compose' })).toHaveCount(0);
  });
});
