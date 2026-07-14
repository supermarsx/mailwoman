// WebAuthn-PRF passkey secret source for zero-access (SPEC §9.1, plan §3 e8). The
// passkey path derives the root-key SECRET from a passkey's PRF extension output
// instead of a typed passphrase — passwordless, phishing-resistant. The PRF bytes are
// fed to `zaDeriveRootKey` exactly like passphrase bytes; the passkey never leaves the
// authenticator and the PRF output never leaves the client.
//
// Feature-detected and gracefully optional: where WebAuthn/PRF is unavailable (older
// browsers, jsdom tests) the UI falls back to the passphrase path.

/** Stable PRF salt so the same passkey yields the same secret across logins. */
const PRF_SALT = new TextEncoder().encode('mailwoman/zero-access/prf/v1');

interface PrfExtensionResults {
  prf?: { results?: { first?: BufferSource } };
}

/** True if this browser exposes WebAuthn (a prerequisite for the PRF path). */
export function passkeySupported(): boolean {
  return typeof PublicKeyCredential !== 'undefined' && typeof navigator !== 'undefined' && 'credentials' in navigator;
}

function bytesToB64(buf: ArrayBuffer): string {
  const bytes = new Uint8Array(buf);
  let bin = '';
  for (const b of bytes) bin += String.fromCharCode(b);
  return btoa(bin);
}

/**
 * Obtain the base64 PRF secret from a passkey assertion. `credentialId` (base64) is the
 * user's registered zero-access passkey; `challenge` is a fresh server challenge. Returns
 * the PRF `first` output as base64, ready for `zaDeriveRootKey`. Throws if PRF is not
 * available for this credential.
 */
export async function passkeySecretB64(credentialIdB64: string, challenge: Uint8Array): Promise<string> {
  if (!passkeySupported()) throw new Error('passkeys are not supported in this browser');
  const rawId = Uint8Array.from(atob(credentialIdB64), (c) => c.charCodeAt(0));
  const assertion = (await navigator.credentials.get({
    publicKey: {
      challenge: challenge as BufferSource,
      allowCredentials: [{ id: rawId as BufferSource, type: 'public-key' }],
      userVerification: 'required',
      extensions: { prf: { eval: { first: PRF_SALT as BufferSource } } } as AuthenticationExtensionsClientInputs,
    },
  })) as PublicKeyCredential | null;
  if (assertion === null) throw new Error('passkey assertion was cancelled');
  const results = assertion.getClientExtensionResults() as unknown as PrfExtensionResults;
  const first = results.prf?.results?.first;
  if (first === undefined) throw new Error('this passkey did not return a PRF secret');
  const buf = first instanceof ArrayBuffer ? first : (first as ArrayBufferView).buffer;
  return bytesToB64(buf as ArrayBuffer);
}
