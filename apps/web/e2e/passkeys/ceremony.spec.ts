import { test, expect } from '../csrf.ts';
import type { Page } from '@playwright/test';

// These are Chromium's real WebAuthn create/get ceremonies with a CTAP2 virtual
// authenticator. No navigator.credentials mock or server verifier bypass.
async function ceremony(page: Page, challenge: Record<string, unknown>, register: boolean) {
  return page.evaluate(async ({ challenge: c, register }) => {
    const bytes = (v: string) => Uint8Array.from(atob(v.replace(/-/g, '+').replace(/_/g, '/')), s => s.charCodeAt(0));
    const b64 = (v: ArrayBuffer) => btoa(String.fromCharCode(...new Uint8Array(v))).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
    if (register) {
      const cred = await navigator.credentials.create({ publicKey: {
        challenge: bytes(c.challenge as string), rp: { id: c.rpId as string, name: 'Mailwoman' },
        user: { id: bytes(c.userHandle as string), name: c.userName as string, displayName: c.userName as string },
        pubKeyCredParams: [{ type: 'public-key', alg: -7 }, { type: 'public-key', alg: -8 }, { type: 'public-key', alg: -257 }],
        authenticatorSelection: { residentKey: 'required', userVerification: 'required' }, attestation: 'none',
      } }) as PublicKeyCredential;
      const result = cred.response as AuthenticatorAttestationResponse;
      return { credentialId: b64(cred.rawId), clientDataJson: b64(result.clientDataJSON), attestationObject: b64(result.attestationObject), transports: 'internal', label: 'Browser proof' };
    }
    const cred = await navigator.credentials.get({ publicKey: {
      challenge: bytes(c.challenge as string), rpId: c.rpId as string, userVerification: 'required',
      allowCredentials: (c.credentialIds as string[]).map(id => ({ id: bytes(id), type: 'public-key' as const })),
    } }) as PublicKeyCredential;
    const result = cred.response as AuthenticatorAssertionResponse;
    return { credentialId: b64(cred.rawId), clientDataJson: b64(result.clientDataJSON), authenticatorData: b64(result.authenticatorData), signature: b64(result.signature), userHandle: result.userHandle === null ? null : b64(result.userHandle) };
  }, { challenge, register });
}

test('real discoverable passkey enrollment, account binding, replay refusal and revocation', async ({ page, context }) => {
  const cdp = await context.newCDPSession(page);
  await cdp.send('WebAuthn.enable');
  await cdp.send('WebAuthn.addVirtualAuthenticator', { options: {
    protocol: 'ctap2', ctap2Version: 'ctap2_1', transport: 'internal', hasResidentKey: true,
    hasUserVerification: true, isUserVerified: true, automaticPresenceSimulation: true,
    defaultBackupEligibility: true, defaultBackupState: true,
  } });
  await page.goto('/');
  const login = await page.request.post('/api/login', { data: { username: 'testuser', password: 'testpass' } });
  expect(login.status()).toBe(200);
  const step = await page.request.post('/api/security/step-up', { data: { password: 'testpass' } });
  expect(step.status()).toBe(200);
  const begin = await page.request.post('/api/account/2fa/passkey/begin', { data: {} });
  expect(begin.status()).toBe(200);
  const registrationChallenge = await begin.json();
  expect(registrationChallenge.userVerification).toBe('required');
  expect(registrationChallenge.residentKey).toBe('required');
  const registration = await ceremony(page, registrationChallenge, true);
  const saved = await page.request.post('/api/account/2fa/passkey/finish', { data: registration });
  expect(saved.status()).toBe(200);
  const recoveryCodes = (await saved.json()).recoveryCodes as string[];
  expect(recoveryCodes.length).toBeGreaterThan(0);
  // Adding a factor invalidates the earlier password-only step-up proof.
  expect((await page.request.post('/api/account/2fa/passkey/finish', { data: registration })).status()).toBe(403);
  async function stepUpPasskey() {
    const stepBegin = await page.request.post('/api/security/step-up/passkey', { data: {} });
    expect(stepBegin.status()).toBe(200);
    const stepProof = await ceremony(page, await stepBegin.json(), false);
    expect((await page.request.post('/api/security/step-up', { data: { passkey: stepProof } })).status()).toBe(200);
  }
  await stepUpPasskey();
  expect((await page.request.post('/api/account/2fa/passkey/finish', { data: registration })).status()).toBe(400);
  await page.request.post('/api/logout', { data: {} });

  const recoveryLogin = await (await page.request.post('/api/login', { data: { username: 'testuser', password: 'testpass' } })).json();
  expect(recoveryLogin.twofaRequired).toBe(true);
  expect((await page.request.post('/api/login/2fa', { data: { pendingToken: recoveryLogin.pendingToken, method: 'recovery', code: recoveryCodes[0] } })).status()).toBe(200);
  await page.request.post('/api/logout', { data: {} });
  const recoveryReplay = await (await page.request.post('/api/login', { data: { username: 'testuser', password: 'testpass' } })).json();
  expect((await page.request.post('/api/login/2fa', { data: { pendingToken: recoveryReplay.pendingToken, method: 'recovery', code: recoveryCodes[0] } })).status()).toBe(401);

  async function assertion() {
    const begin = await page.request.post('/api/login/passkey/begin', { data: {} });
    expect(begin.status()).toBe(200);
    const challenge = await begin.json();
    return { ...await ceremony(page, challenge, false), pendingToken: challenge.pendingToken };
  }
  for (const wrong of [null, btoa('different-account')]) {
    const proof = await assertion();
    expect((await page.request.post('/api/login/passkey/finish', { data: { ...proof, userHandle: wrong } })).status()).toBe(401);
    // A failed ceremony also consumes its challenge.
    expect((await page.request.post('/api/login/passkey/finish', { data: proof })).status()).toBe(401);
  }
  const proof = await assertion();
  expect(proof.userHandle).toBeTruthy();
  const flags = Buffer.from(proof.authenticatorData ?? '', 'base64url')[32] ?? 0;
  expect(flags & 0x1c).toBe(0x1c); // UV + backup eligible + currently backed up.
  expect((await page.request.post('/api/login/passkey/finish', { data: proof })).status()).toBe(200);
  expect((await page.request.post('/api/login/passkey/finish', { data: proof })).status()).toBe(401);
  expect((await page.request.get('/api/me')).status()).toBe(200);
  const state = await (await page.request.get('/api/account/2fa')).json();
  expect(state.passkeys).toHaveLength(1);
  // Sensitive revocation requires a fresh, user-verified proof.
  await stepUpPasskey();
  expect((await page.request.post('/api/account/2fa/passkey/remove', { data: { handle: state.passkeys[0].handle } })).status()).toBe(200);
  await page.request.post('/api/logout', { data: {} });
  const revoked = await assertion();
  expect((await page.request.post('/api/login/passkey/finish', { data: revoked })).status()).toBe(401);
});
