import { createHmac } from 'node:crypto';
import { test, expect, type Page } from '../csrf.ts';
import { signInManually } from '../helpers.ts';

// t77: `twofa.lastFactorRemoval = block` against a real server. The last
// passkey or authenticator app cannot be removed: its remove button is shown
// unavailable with "Add another sign-in method first" (beside it and in its
// tooltip), and the server refuses the removal with `lastFactorBlocked`. A
// factor goes once another one remains.
//
// Chromium and Firefox both run the authenticator-app leg. The passkey leg
// needs Chromium's virtual authenticators (CDP WebAuthn), so it runs there only.
//
// The server is `serve_last_factor_browser_fixture`
// (crates/mw-server/tests/t39_passkey_policy.rs), restarted for each browser
// (a fresh database each time):
//
//   MW_T77_LAST_FACTOR_WEB=<built apps/web/dist> MW_T77_LAST_FACTOR_PORT=42277 \
//     cargo test -p mw-server --test t39_passkey_policy serve_last_factor_browser_fixture -- --ignored
//   PLAYWRIGHT_PASSKEY_BASE_URL=http://localhost:42277 \
//     npx playwright test -c e2e/passkeys/playwright.last-factor.config.ts --project=chromium

const USER = 'testuser@example.org';
const PASS = 'testpass';
const SHOTS = process.env['MW_T77_SHOTS'];
const HINT = 'Add another sign-in method first.';
const TOOLTIP = 'Add another sign-in method first. Your organization requires at least one passkey or authenticator app on your account.';

async function shot(page: Page, name: string): Promise<void> {
  const project = test.info().project.name;
  await page.screenshot({ path: SHOTS ? `${SHOTS}/${project}-${name}.png` : test.info().outputPath(`${name}.png`), fullPage: false });
}

async function openSecurity(page: Page): Promise<void> {
  const sections = page.getByRole('navigation', { name: 'Settings sections' });
  if (!await sections.isVisible()) {
    const show = page.getByRole('button', { name: 'Show folders sidebar' });
    if (await show.isVisible()) await show.click();
    const drawer = page.getByRole('button', { name: 'Open folders and apps' });
    if (await drawer.isVisible()) await drawer.click();
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
  }
  await sections.getByRole('button', { name: 'Security centre', exact: true }).click();
}

/** Answer the "Confirm it is you" step-up with the password, when it is asked. */
async function confirmIfAsked(page: Page, ready: ReturnType<Page['getByTestId']>): Promise<void> {
  const dialog = page.getByRole('dialog', { name: 'Confirm it is you' });
  await expect(dialog.or(ready).first()).toBeVisible({ timeout: 30_000 });
  if (await dialog.isVisible()) {
    await dialog.getByLabel('Password').fill(PASS);
    await dialog.getByRole('button', { name: 'Confirm and continue' }).click();
  }
}

/** RFC 6238 code (SHA-1, 6 digits, 30 s) for a base32 secret. */
function totpCode(base32: string, at = Date.now()): string {
  const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
  let bits = '';
  for (const ch of base32.replace(/=+$/, '').toUpperCase()) bits += alphabet.indexOf(ch).toString(2).padStart(5, '0');
  const key = Buffer.from(bits.match(/.{8}/g)!.map((b) => parseInt(b, 2)));
  const counter = Buffer.alloc(8);
  counter.writeBigUInt64BE(BigInt(Math.floor(at / 1000 / 30)));
  const mac = createHmac('sha1', key).update(counter).digest();
  const offset = mac[mac.length - 1]! & 0x0f;
  const n = (mac.readUInt32BE(offset) & 0x7fffffff) % 1_000_000;
  return n.toString().padStart(6, '0');
}

/** The next TOTP code the server has not seen yet (codes are single use per step). */
async function freshCode(secret: string, used: Set<string>): Promise<string> {
  for (;;) {
    const code = totpCode(secret);
    if (!used.has(code)) { used.add(code); return code; }
    await new Promise((r) => setTimeout(r, 1_000));
  }
}

/** The remove button of the only factor: unavailable, explained, tooltip on hover. */
async function expectKept(page: Page, button: ReturnType<Page['getByRole']>, label: string): Promise<void> {
  await expect(button).toHaveAttribute('aria-disabled', 'true');
  // aria-disabled, not `disabled`: it stays focusable and hoverable.
  expect(await button.evaluate((b) => (b as HTMLButtonElement).disabled)).toBe(false);
  await expect(page.getByTestId('last-factor-hint')).toHaveText(HINT);
  await button.hover();
  const tip = page.getByRole('tooltip');
  await expect(tip).toHaveText(TOOLTIP, { timeout: 5_000 });
  await shot(page, `${label}-kept-tooltip`);
  // Clicking it sends nothing.
  let sent = false;
  const watch = (r: { url(): string }) => { if (/\/api\/account\/2fa\/(totp\/disable|passkey\/remove)$/.test(r.url())) sent = true; };
  page.on('request', watch);
  await button.click({ force: true }); // past Playwright's aria-disabled actionability wait
  await page.waitForTimeout(500);
  page.off('request', watch);
  expect(sent, 'a kept factor sends no removal').toBe(false);
  await page.mouse.move(0, 0);
}

test('block keeps the last factor; a factor goes once another remains', async ({ page, context, browserName }) => {
  test.setTimeout(240_000);
  await signInManually(page, { username: USER, password: PASS });
  await openSecurity(page);
  const used = new Set<string>();

  // ── authenticator app (Chromium and Firefox) ──────────────────────────────
  await page.getByRole('button', { name: 'Set up an authenticator app', exact: true }).click();
  await confirmIfAsked(page, page.getByTestId('totp-secret'));
  const secret = (await page.getByTestId('totp-secret').textContent())!.trim();
  await page.getByLabel('Authenticator code').fill(await freshCode(secret, used));
  await page.getByRole('button', { name: 'Confirm', exact: true }).click();
  await expect(page.getByTestId('recovery-codes')).toBeVisible({ timeout: 30_000 });
  await page.getByTestId('recovery-ack').click();
  await expect(page.getByTestId('totp-on')).toBeVisible();

  const removeTotp = page.getByRole('button', { name: 'Remove authenticator', exact: true });
  await expectKept(page, removeTotp, '01-totp');

  // The server refuses it too (after a valid step-up), recovery codes or not.
  const stepped = await page.request.post('/api/security/step-up', { data: { password: PASS, code: await freshCode(secret, used) } });
  expect(stepped.status(), await stepped.text()).toBe(200);
  const refused = await page.request.post('/api/account/2fa/totp/disable', { data: {} });
  expect(refused.status()).toBe(409);
  expect((await refused.json()).code).toBe('lastFactorBlocked');
  const regen = await page.request.post('/api/account/2fa/recovery/regenerate', { data: {} });
  expect(regen.status()).toBe(200);
  const again = await page.request.post('/api/account/2fa/totp/disable', { data: {} });
  expect(again.status()).toBe(409);
  expect((await (await page.request.get('/api/account/2fa')).json()).totp).toBe(true);

  if (browserName !== 'chromium') {
    test.info().annotations.push({ type: 'scope', description: 'passkey leg needs CDP WebAuthn (Chromium only)' });
    return;
  }

  // ── passkeys (Chromium virtual authenticator) ─────────────────────────────
  const cdp = await context.newCDPSession(page);
  await cdp.send('WebAuthn.enable');
  await cdp.send('WebAuthn.addVirtualAuthenticator', { options: {
    protocol: 'ctap2', ctap2Version: 'ctap2_1', transport: 'internal', hasResidentKey: true,
    hasUserVerification: true, isUserVerified: true, automaticPresenceSimulation: true,
  } });
  // The proof from above is still valid (nothing was removed): add a passkey.
  await page.reload();
  await openSecurity(page);
  await page.getByRole('button', { name: 'Add a passkey', exact: true }).click();
  await confirmIfAsked(page, page.getByTestId('recovery-codes').or(page.getByTestId('passkey-list')));
  const codes = page.getByTestId('recovery-ack');
  if (await codes.isVisible()) await codes.click();
  await expect(page.getByTestId('passkey-list')).toBeVisible({ timeout: 30_000 });
  await expect(page.getByTestId('last-factor-hint')).toHaveCount(0);

  // Two factors: both remove buttons are live. Remove the authenticator app
  // through the UI (step up first: adding the passkey ended the proof).
  await expect(removeTotp).not.toHaveAttribute('aria-disabled', 'true');
  const removePasskey = page.getByRole('button', { name: 'Remove', exact: true });
  await expect(removePasskey).not.toHaveAttribute('aria-disabled', 'true');
  await shot(page, '02-two-factors-live');
  const stepped2 = await page.request.post('/api/security/step-up', { data: { password: PASS, code: await freshCode(secret, used) } });
  expect(stepped2.status(), await stepped2.text()).toBe(200);
  const disabled = page.waitForResponse((r) => r.url().endsWith('/api/account/2fa/totp/disable'));
  await removeTotp.click();
  expect((await disabled).status()).toBe(200);
  await expect(page.getByTestId('totp-on')).toHaveCount(0);

  // Now the passkey is the last factor.
  await expectKept(page, removePasskey, '03-passkey');
  const status = await (await page.request.get('/api/account/2fa')).json();
  expect(status.passkeys).toHaveLength(1);
  const handle = status.passkeys[0].handle as string;
  // Step up with the passkey (the removal above ended the proof), then the
  // server refuses the removal by name.
  const begin = await (await page.request.post('/api/security/step-up/passkey', { data: {} })).json();
  const proof = await page.evaluate(async (c) => {
    const bytes = (v: string) => Uint8Array.from(atob(v.replace(/-/g, '+').replace(/_/g, '/')), (x) => x.charCodeAt(0));
    const b64 = (v: ArrayBuffer) => btoa(String.fromCharCode(...new Uint8Array(v))).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
    const cred = await navigator.credentials.get({ publicKey: { challenge: bytes(c.challenge), rpId: c.rpId, userVerification: 'required', allowCredentials: (c.credentialIds as string[]).map((id) => ({ id: bytes(id), type: 'public-key' as const })) } }) as PublicKeyCredential;
    const r = cred.response as AuthenticatorAssertionResponse;
    return { credentialId: b64(cred.rawId), clientDataJson: b64(r.clientDataJSON), authenticatorData: b64(r.authenticatorData), signature: b64(r.signature), userHandle: r.userHandle === null ? null : b64(r.userHandle) };
  }, begin);
  const stepped3 = await page.request.post('/api/security/step-up', { data: { passkey: proof } });
  expect(stepped3.status(), await stepped3.text()).toBe(200);
  const refusedPk = await page.request.post('/api/account/2fa/passkey/remove', { data: { handle } });
  expect(refusedPk.status()).toBe(409);
  expect((await refusedPk.json()).code).toBe('lastFactorBlocked');
  expect((await (await page.request.get('/api/account/2fa')).json()).passkeys).toHaveLength(1);
});
