import type { Page } from '@playwright/test';

// These are Chromium's real WebAuthn create/get ceremonies with a CTAP2 virtual
// authenticator. No navigator.credentials mock or server verifier bypass.
export async function ceremony(page: Page, challenge: Record<string, unknown>, register: boolean) {
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
