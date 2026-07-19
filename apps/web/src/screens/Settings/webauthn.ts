// WebAuthn ceremony plumbing for login-2FA enrolment + assertion (t16 e15).
//
// This is the create/get side of the SAME plumbing the zero-access PRF path uses
// (`modules/zeroaccess/passkey.ts`) — feature-detect, call `navigator.credentials`,
// read the ceremony outputs — but for a second-factor credential (no PRF): a
// registration (`create`) at enrolment and an assertion (`get`) at login. Server
// RP verification is the new part (`crates/mw-mfa`); the browser calls are the
// familiar shape. base64url is used both ways to match the server's `b64`
// (URL_SAFE_NO_PAD) exactly, though the server tolerates either alphabet.

import { passkeySupported } from '../../modules/zeroaccess/passkey.ts';

export { passkeySupported };

// ES256 (-7) then EdDSA (-8): the two algorithms `mw-mfa` verifies at launch.
const PUB_KEY_CRED_PARAMS: PublicKeyCredentialParameters[] = [
  { alg: -7, type: 'public-key' },
  { alg: -8, type: 'public-key' },
];

function b64urlToBytes(s: string): Uint8Array {
  const padded = s.replace(/-/g, '+').replace(/_/g, '/').padEnd(Math.ceil(s.length / 4) * 4, '=');
  const bin = atob(padded);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i += 1) out[i] = bin.charCodeAt(i);
  return out;
}

function bytesToB64url(buf: ArrayBuffer): string {
  const bytes = new Uint8Array(buf);
  let bin = '';
  for (const b of bytes) bin += String.fromCharCode(b);
  return btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function toUv(v: string): UserVerificationRequirement {
  return v === 'required' || v === 'discouraged' ? v : 'preferred';
}

/** The material `POST …/passkey/finish` needs after a registration ceremony. */
export interface RegistrationResult {
  clientDataJson: string;
  attestationObject: string;
  transports: string;
}

/** Run `navigator.credentials.create` for a 2FA passkey enrolment challenge. */
export async function registerPasskey(challenge: {
  challenge: string;
  rpId: string;
  userHandle: string;
  userName: string;
  userVerification: string;
}): Promise<RegistrationResult> {
  if (!passkeySupported()) throw new Error('passkeys are not supported in this browser');
  const cred = (await navigator.credentials.create({
    publicKey: {
      challenge: b64urlToBytes(challenge.challenge) as BufferSource,
      rp: { id: challenge.rpId, name: challenge.rpId },
      user: {
        id: b64urlToBytes(challenge.userHandle) as BufferSource,
        name: challenge.userName,
        displayName: challenge.userName,
      },
      pubKeyCredParams: PUB_KEY_CRED_PARAMS,
      authenticatorSelection: { userVerification: toUv(challenge.userVerification) },
      attestation: 'none',
    },
  })) as PublicKeyCredential | null;
  if (cred === null) throw new Error('passkey registration was cancelled');
  const response = cred.response as AuthenticatorAttestationResponse;
  const transports =
    typeof response.getTransports === 'function' ? response.getTransports().join(',') : '';
  return {
    clientDataJson: bytesToB64url(response.clientDataJSON),
    attestationObject: bytesToB64url(response.attestationObject),
    transports,
  };
}

/** The material `POST /api/login/2fa` (method "webauthn") needs after an assertion. */
export interface AssertionResult {
  credentialId: string;
  clientDataJson: string;
  authenticatorData: string;
  signature: string;
}

/** Run `navigator.credentials.get` for a login-time second-factor assertion. */
export async function assertPasskey(challenge: {
  challenge: string;
  credentialIds: readonly string[];
  rpId: string;
  userVerification: string;
}): Promise<AssertionResult> {
  if (!passkeySupported()) throw new Error('passkeys are not supported in this browser');
  const assertion = (await navigator.credentials.get({
    publicKey: {
      challenge: b64urlToBytes(challenge.challenge) as BufferSource,
      rpId: challenge.rpId,
      allowCredentials: challenge.credentialIds.map((id) => ({
        id: b64urlToBytes(id) as BufferSource,
        type: 'public-key' as const,
      })),
      userVerification: toUv(challenge.userVerification),
    },
  })) as PublicKeyCredential | null;
  if (assertion === null) throw new Error('passkey assertion was cancelled');
  const response = assertion.response as AuthenticatorAssertionResponse;
  return {
    credentialId: bytesToB64url(assertion.rawId),
    clientDataJson: bytesToB64url(response.clientDataJSON),
    authenticatorData: bytesToB64url(response.authenticatorData),
    signature: bytesToB64url(response.signature),
  };
}
