// Zero-access key-hierarchy re-wrap on password change (SPEC §18.3 / §9, plan §3 e7).
//
// When a zero-access account changes its password, the per-account DATA KEY (which
// seals every row and must NOT change) has to be re-wrapped under a KEK derived from
// the NEW password. This is done entirely client-side through the existing `mw-crypto`
// `za*` worker — NO cryptography is hand-rolled in JS here; every primitive is a call
// into the worker facade (`ZeroAccessCrypto`). The server only ever receives the new
// wrapped material + KDF params, never a plaintext key (plan §1.4).
//
// HARD ORDERING (plan hard constraint): the recovery-phrase pre-prompt is derived +
// surfaced BEFORE the change is applied, so a sender who loses the new password (or a
// re-wrap that is interrupted) can still recover the data key. `recoveryPhraseBefore`
// MUST be awaited before `rewrapUnderNewPassword` / the change POST.

import type { ZeroAccessCrypto, ZaKdfParams } from '../zeroaccess/crypto.ts';
import { ZA_KDF_INTERACTIVE } from '../zeroaccess/crypto.ts';
import type { ZeroAccessAccount } from '../zeroaccess/service.ts';

/** The new wrapped material to hand to `POST /api/password` (never a raw key). */
export interface RewrapResult {
  readonly saltB64: string;
  readonly kdfParams: ZaKdfParams;
  readonly wrappedDataKeyB64: string;
}

export interface RewrapInputs {
  readonly za: ZeroAccessCrypto;
  /** The account's current server-side zero-access state (salt/KDF/wrapped key). */
  readonly account: ZeroAccessAccount;
  /** The current (old) unlock secret, base64. */
  readonly oldSecretB64: string;
  /** The new unlock secret, base64. */
  readonly newSecretB64: string;
}

function requireEnabled(account: ZeroAccessAccount): asserts account is Required<ZeroAccessAccount> {
  if (
    !account.enabled ||
    account.saltB64 === undefined ||
    account.kdfParams === undefined ||
    account.wrappedDataKeyB64 === undefined
  ) {
    throw new Error('account is not zero-access enabled — nothing to re-wrap');
  }
}

/** A fresh 16-byte KDF salt, base64. Uses the platform RNG (not hand-rolled crypto). */
function randomSaltB64(): string {
  const salt = new Uint8Array(16);
  crypto.getRandomValues(salt);
  let bin = '';
  for (const b of salt) bin += String.fromCharCode(b);
  return btoa(bin);
}

/**
 * Derive + return the recovery phrase for the CURRENT key hierarchy. This is the
 * pre-prompt the UI shows BEFORE applying the change (the safety net). It unlocks the
 * old root from the old secret and asks the worker to serialise it as a phrase — the
 * data key itself never leaves the worker.
 */
export async function recoveryPhraseBefore(
  za: ZeroAccessCrypto,
  account: ZeroAccessAccount,
  oldSecretB64: string,
): Promise<string> {
  requireEnabled(account);
  const { keyRef: rootRef } = await za.deriveRootKey({
    secretB64: oldSecretB64,
    saltB64: account.saltB64,
    mCost: account.kdfParams.mCost,
    tCost: account.kdfParams.tCost,
    pCost: account.kdfParams.pCost,
  });
  const { phrase } = await za.recoveryPhrase({ keyRef: rootRef });
  return phrase;
}

/**
 * Re-wrap the key hierarchy under the new password: unwrap the existing data key with
 * the old KEK, then wrap the SAME data key under a KEK derived from the new secret and
 * a fresh salt. Returns only the wrapped material for the server. The data key is
 * unchanged, so every already-sealed row stays readable.
 */
export async function rewrapUnderNewPassword(inputs: RewrapInputs): Promise<RewrapResult> {
  const { za, account, oldSecretB64, newSecretB64 } = inputs;
  requireEnabled(account);

  // Unlock the current hierarchy → recover the data key ref (in-worker).
  const { keyRef: oldRoot } = await za.deriveRootKey({
    secretB64: oldSecretB64,
    saltB64: account.saltB64,
    mCost: account.kdfParams.mCost,
    tCost: account.kdfParams.tCost,
    pCost: account.kdfParams.pCost,
  });
  const { keyRef: oldKek } = await za.deriveKek({ keyRef: oldRoot });
  const { keyRef: dataKeyRef } = await za.unwrapKey({ kekRef: oldKek, blobB64: account.wrappedDataKeyB64 });

  // Derive the NEW root/KEK from the new secret + a fresh salt, re-wrap the data key.
  const kdf = account.kdfParams ?? ZA_KDF_INTERACTIVE;
  const saltB64 = randomSaltB64();
  const { keyRef: newRoot } = await za.deriveRootKey({
    secretB64: newSecretB64,
    saltB64,
    mCost: kdf.mCost,
    tCost: kdf.tCost,
    pCost: kdf.pCost,
  });
  const { keyRef: newKek } = await za.deriveKek({ keyRef: newRoot });
  const { blobB64: wrappedDataKeyB64 } = await za.wrapKey({ kekRef: newKek, dataKeyRef });

  return { saltB64, kdfParams: kdf, wrappedDataKeyB64 };
}
