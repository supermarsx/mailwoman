import { test, expect, type Page } from '../csrf.ts';
import { signInManually } from '../helpers.ts';

// t39-e6: the administrator passkey policy against a real server, with
// Chromium's virtual authenticators (CDP WebAuthn); no navigator.credentials
// mock. The server is `serve_passkey_policy_browser_fixture`
// (crates/mw-server/tests/t39_passkey_policy.rs): proxy mode, policy
// "platform + user verification required".
//
//   cargo test -p mw-server --test t39_passkey_policy serve_passkey_policy_browser_fixture -- --ignored
//   (MW_T39_PASSKEY_WEB=<built apps/web/dist>, MW_T39_PASSKEY_PORT=42239)
//   PLAYWRIGHT_PASSKEY_BASE_URL=http://localhost:42239 (an IP is not a valid WebAuthn RP ID) \
//     npx playwright test -c e2e/passkeys/playwright.passkeys.config.ts policy.spec.ts

const USER = 'testuser@example.org';
const PASS = 'testpass';
const SHOTS = process.env['MW_T39_SHOTS'];

async function shot(page: Page, name: string): Promise<void> {
  await page.screenshot({ path: SHOTS ? `${SHOTS}/${name}.png` : test.info().outputPath(`${name}.png`), fullPage: true });
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

/** A raw create() as a non-conforming client would make it: it does not ask for UV. */
async function createWithoutUv(page: Page, o: Record<string, unknown>) {
  return page.evaluate(async (c) => {
    const bytes = (v: string) => Uint8Array.from(atob(v.replace(/-/g, '+').replace(/_/g, '/')), (s) => s.charCodeAt(0));
    const b64 = (v: ArrayBuffer) => btoa(String.fromCharCode(...new Uint8Array(v))).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
    const cred = await navigator.credentials.create({ publicKey: {
      challenge: bytes(c['challenge'] as string), rp: { id: c['rpId'] as string, name: 'Mailwoman' },
      user: { id: bytes(c['userHandle'] as string), name: c['userName'] as string, displayName: c['userName'] as string },
      pubKeyCredParams: [{ type: 'public-key', alg: -7 }],
      authenticatorSelection: { authenticatorAttachment: 'platform', userVerification: 'discouraged', residentKey: 'discouraged' },
      attestation: 'none', timeout: 10_000,
    } }) as PublicKeyCredential;
    const r = cred.response as AuthenticatorAttestationResponse;
    const flags = new Uint8Array(r.getAuthenticatorData())[32] ?? 0;
    return { body: { clientDataJson: b64(r.clientDataJSON), attestationObject: b64(r.attestationObject), transports: r.getTransports().join(','), authenticatorAttachment: cred.authenticatorAttachment ?? undefined, label: 'No UV' }, uv: (flags & 0x04) !== 0 };
  }, o);
}

/** A raw get() that does not ask for UV. */
async function getWithoutUv(page: Page, c: Record<string, unknown>) {
  return page.evaluate(async (c) => {
    const bytes = (v: string) => Uint8Array.from(atob(v.replace(/-/g, '+').replace(/_/g, '/')), (s) => s.charCodeAt(0));
    const b64 = (v: ArrayBuffer) => btoa(String.fromCharCode(...new Uint8Array(v))).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
    const cred = await navigator.credentials.get({ publicKey: {
      challenge: bytes(c['challenge'] as string), rpId: c['rpId'] as string, userVerification: 'discouraged', timeout: 10_000,
      allowCredentials: (c['credentialIds'] as string[]).map((id) => ({ id: bytes(id), type: 'public-key' as const })),
    } }) as PublicKeyCredential;
    const r = cred.response as AuthenticatorAssertionResponse;
    return { credentialId: b64(cred.rawId), clientDataJson: b64(r.clientDataJSON), authenticatorData: b64(r.authenticatorData), signature: b64(r.signature), uv: ((new Uint8Array(r.authenticatorData)[32] ?? 0) & 0x04) !== 0 };
  }, c);
}

test('platform + UV required: a UV passkey registers through Settings, a non-UV authenticator is refused', async ({ page, context }) => {
  test.setTimeout(180_000);
  const cdp = await context.newCDPSession(page);
  await cdp.send('WebAuthn.enable');
  const { authenticatorId: good } = await cdp.send('WebAuthn.addVirtualAuthenticator', { options: {
    protocol: 'ctap2', ctap2Version: 'ctap2_1', transport: 'internal', hasResidentKey: true,
    hasUserVerification: true, isUserVerified: true, automaticPresenceSimulation: true,
  } });

  await signInManually(page, { username: USER, password: PASS });
  await openSecurity(page);
  const rules = page.getByTestId('passkey-policy');
  await expect(rules).toContainText('are stored on this device');
  await expect(rules).toContainText('check your PIN, fingerprint or face');
  await expect(page.getByTestId('passkey-list')).toHaveCount(0);

  // The registration options carry the policy.
  const add = page.getByRole('button', { name: 'Add a passkey', exact: true });
  const begin = page.waitForResponse((r) => r.url().endsWith('/api/account/2fa/passkey/begin') && r.ok());
  await add.click();
  // Adding a factor asks for recent verification first; confirm with the password.
  const dialog = page.getByRole('dialog', { name: 'Confirm it is you' });
  await dialog.waitFor({ timeout: 15_000 });
  await shot(page, 'e6-00-step-up');
  await dialog.getByLabel('Password').fill(PASS);
  await dialog.getByRole('button', { name: 'Confirm and continue' }).click();
  const beginBody = await (await begin).json();
  expect(beginBody.authenticatorAttachment).toBe('platform');
  expect(beginBody.userVerification).toBe('required');
  await expect(page.getByTestId('recovery-codes')).toBeVisible({ timeout: 30_000 });
  await shot(page, 'e6-01-registered-platform-uv');
  await page.getByTestId('recovery-ack').click();
  await expect(page.getByTestId('passkey-list')).toContainText('passkey');
  const status = await (await page.request.get('/api/account/2fa')).json();
  expect(status.passkeys).toHaveLength(1);
  expect(status.passkeys[0].meetsPolicy).toBe(true);

  // Sign-in: the enrolled passkey without UV is refused by name; with UV it signs in.
  const signIn = await page.context().browser()!.newContext({ baseURL: test.info().project.use.baseURL! });
  const other = await signIn.newPage();
  try {
    const cdp2 = await signIn.newCDPSession(other);
    await cdp2.send('WebAuthn.enable');
    const { authenticatorId: copy } = await cdp2.send('WebAuthn.addVirtualAuthenticator', { options: {
      protocol: 'ctap2', ctap2Version: 'ctap2_1', transport: 'internal', hasResidentKey: true,
      hasUserVerification: true, isUserVerified: false, automaticPresenceSimulation: true,
    } });
    const { credentials } = await cdp.send('WebAuthn.getCredentials', { authenticatorId: good });
    for (const credential of credentials) await cdp2.send('WebAuthn.addCredential', { authenticatorId: copy, credential });
    await other.goto('/');
    const pending = await (await other.request.post('/api/login', { data: { username: USER, password: PASS } })).json();
    expect(pending.twofaRequired).toBe(true);
    expect(pending.webauthn.userVerification).toBe('required');
    const noUv = await getWithoutUv(other, pending.webauthn);
    expect(noUv.uv).toBe(false);
    const { uv: _uv, ...proof } = noUv;
    void _uv;
    const answer = await other.request.post('/api/login/2fa', { data: { ...proof, pendingToken: pending.pendingToken, method: 'webauthn' } });
    expect(answer.status()).toBe(403);
    expect((await answer.json()).code).toBe('passkeyPolicyUserVerification');
    // The same refusal in the sign-in screen.
    await other.goto('/');
    await other.getByLabel('Email or username', { exact: true }).fill(USER);
    await other.getByLabel('Password', { exact: true }).fill(PASS);
    await other.getByRole('button', { name: 'Sign in', exact: true }).click();
    await cdp2.send('WebAuthn.setUserVerified', { authenticatorId: copy, isUserVerified: false });
    await other.getByRole('button', { name: 'Use a passkey' }).click();
    const loginAlert = other.getByRole('alert');
    await expect(loginAlert).toBeVisible({ timeout: 60_000 });
    await shot(other, 'e6-03-signin-non-uv');
    test.info().annotations.push({ type: 'signin-refusal', description: (await loginAlert.textContent()) ?? '' });
  } finally {
    await signIn.close();
  }

  // Adding a factor ended the earlier proof: confirm again with the enrolled,
  // user-verifying passkey (the step-up route), before swapping authenticators.
  const stepBegin = await page.request.post('/api/security/step-up/passkey', { data: {} });
  expect(stepBegin.status()).toBe(200);
  const stepChallenge = await stepBegin.json();
  expect(stepChallenge.userVerification).toBe('required');
  const stepProof = await page.evaluate(async (c) => {
    const bytes = (v: string) => Uint8Array.from(atob(v.replace(/-/g, '+').replace(/_/g, '/')), (x) => x.charCodeAt(0));
    const b64 = (v: ArrayBuffer) => btoa(String.fromCharCode(...new Uint8Array(v))).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
    const cred = await navigator.credentials.get({ publicKey: { challenge: bytes(c.challenge), rpId: c.rpId, userVerification: 'required', allowCredentials: (c.credentialIds as string[]).map((id) => ({ id: bytes(id), type: 'public-key' as const })) } }) as PublicKeyCredential;
    const r = cred.response as AuthenticatorAssertionResponse;
    return { credentialId: b64(cred.rawId), clientDataJson: b64(r.clientDataJSON), authenticatorData: b64(r.authenticatorData), signature: b64(r.signature), userHandle: r.userHandle === null ? null : b64(r.userHandle) };
  }, stepChallenge);
  const stepped = await page.request.post('/api/security/step-up', { data: { passkey: stepProof } });
  expect(stepped.status(), await stepped.text()).toBe(200);

  // A second authenticator that cannot verify the user at all.
  await cdp.send('WebAuthn.removeVirtualAuthenticator', { authenticatorId: good });
  await cdp.send('WebAuthn.addVirtualAuthenticator', { options: {
    protocol: 'ctap2', ctap2Version: 'ctap2_1', transport: 'internal', hasResidentKey: true,
    hasUserVerification: false, automaticPresenceSimulation: true,
  } });
  // Through the UI: the browser itself finds no authenticator for the options,
  // and the user is told what the policy asks for.
  await page.getByRole('button', { name: 'Add a passkey', exact: true }).click();
  const alert = page.getByRole('alert');
  await expect(alert).toBeVisible({ timeout: 90_000 });
  await shot(page, 'e6-02-non-uv-refused-ui');
  const uiText = (await alert.textContent()) ?? '';

  // A client that does not ask for UV: the server refuses the attestation.
  const raw = await page.request.post('/api/account/2fa/passkey/begin', { data: {} });
  expect(raw.status(), await raw.text()).toBe(200);
  const made = await createWithoutUv(page, await raw.json());
  expect(made.uv).toBe(false);
  const refused = await page.request.post('/api/account/2fa/passkey/finish', { data: made.body });
  expect(refused.status()).toBe(403);
  expect((await refused.json()).code).toBe('passkeyPolicyUserVerification');
  expect((await (await page.request.get('/api/account/2fa')).json()).passkeys).toHaveLength(1);

  test.info().annotations.push({ type: 'ui-refusal', description: uiText });
});
