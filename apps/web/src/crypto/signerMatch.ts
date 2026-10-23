// Picks the candidate PGP public key used to verify a decrypted message's
// embedded signature (t32 D-4). The worker's verdict is only as honest as this
// choice: a key bound to a different address that verifies the signature would
// show "verified" for a sender it does not belong to. So a key qualifies only
// when one of its addresses is EXACTLY the sender address (whole address case-
// folded, domain IDN-normalised to its ASCII form), and it is neither revoked
// nor expired. The signing time is not known before decryption, so expiry is
// judged at verify time (`now`).

import type { CryptoKey } from '../api/crypto-types.ts';

/** Characters a mail domain never contains but a URL host parser would act on. */
const NON_DOMAIN = /[\s/\\?#:@%[\]]/u;

/** Canonical form of an address, or undefined when it has no usable domain. */
export function canonicalAddress(address: string): string | undefined {
  const at = address.lastIndexOf('@');
  if (at <= 0 || at === address.length - 1) return undefined;
  const local = address.slice(0, at).toLowerCase();
  const domain = address.slice(at + 1);
  if (NON_DOMAIN.test(domain)) return undefined;
  let ascii: string;
  try {
    // The WHATWG URL host parser applies IDNA (UTS #46) mapping and punycode.
    ascii = new URL(`http://${domain}`).hostname;
  } catch {
    return undefined;
  }
  if (ascii === '') return undefined;
  return `${local}@${ascii}`;
}

/** Whether a key may be used to verify a signature at `now`. */
function isLive(k: CryptoKey, now: Date): boolean {
  if (k.trust === 'revoked') return false;
  if (k.expiresAt === null) return true;
  const expires = Date.parse(k.expiresAt);
  return !Number.isNaN(expires) && expires > now.getTime();
}

/** The armored public key of the first live PGP key bound to exactly `address`. */
export function pickSignerArmor(
  address: string,
  keys: readonly CryptoKey[],
  now: Date,
): string | undefined {
  const sender = canonicalAddress(address);
  if (sender === undefined) return undefined;
  for (const k of keys) {
    if (k.kind !== 'pgp' || k.publicKeyArmored === null || !isLive(k, now)) continue;
    if (k.addresses.some((a) => canonicalAddress(a) === sender)) return k.publicKeyArmored;
  }
  return undefined;
}
