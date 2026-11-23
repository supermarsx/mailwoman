// V6 zero-access storage module (SPEC §9, plan §2.6 / §3 e8). Public surface for e11
// to mount into Settings/an account screen (this module does NOT touch the router or
// Settings.tsx — ownership boundary). Everything composes the existing `mw-crypto`
// `za*` wasm exports through a dedicated worker; no crypto is hand-rolled in JS.
//
// e11 WIRE-UP:
//   import { ZeroAccessSettings } from '@/modules/zeroaccess';
//   import { spawnZeroAccessWorker } from '@/modules/zeroaccess';
//   <ZeroAccessSettings za={spawnZeroAccessWorker()} />
// Endpoints this module calls (e11 to satisfy):
//   GET  /api/zeroaccess                         → ZeroAccessAccount
//   POST /api/zeroaccess/enable  (ZeroAccessEnablePayload)
//   POST /api/zeroaccess/disable
//   POST /api/zeroaccess/pair/offer      ({publicB64})            → {pairingId}
//   POST /api/zeroaccess/pair/envelope   ({pairingId, envelopeB64})
//   GET  /api/zeroaccess/pair/envelope/:id                        → {envelopeB64|null}

import type { ZeroAccessAccount } from './service.ts';

/** Whether zero-access is enabled for an account, and what the server still sees. */
export interface ZeroAccessStatus {
  readonly enabled: boolean;
  /**
   * What the server still holds and can read (SPEC §9.2), in short form; the full
   * wording is `ZA_SERVER_STILL_SEES` in `./disclosure.ts`. Stored mail is sealed
   * with the SERVER's key whether or not this mode is on, so the mode does not
   * defend against the server, its operator, or a database copy taken with the key.
   * It keeps the account out of the search index. No searchable-encryption claim
   * is made.
   */
  readonly serverVisibleMetadata: readonly string[];
}

/** Default status (disabled) + the server-visible list. */
export const ZERO_ACCESS_DEFAULT: ZeroAccessStatus = {
  enabled: false,
  serverVisibleMetadata: [
    'message bodies and envelopes (sealed with the server key)',
    'mail in transit',
    'message sizes',
    'timestamps',
    'envelope routing for IMAP/SMTP proxying',
  ],
};

/** Map the server account record into the compact status summary. */
export function toStatus(account: ZeroAccessAccount): ZeroAccessStatus {
  return { enabled: account.enabled, serverVisibleMetadata: ZERO_ACCESS_DEFAULT.serverVisibleMetadata };
}

export { ZeroAccessSettings } from './ZeroAccessSettings.tsx';
export { DevicePairing } from './DevicePairing.tsx';
export { Qr } from './Qr.tsx';
export { spawnZeroAccessWorker } from './worker.ts';
export {
  ZeroAccessService,
  describeKdf,
  type ZeroAccessAccount,
  type ZeroAccessEnablePayload,
  type ZeroAccessSession,
  type PairedDevice,
  type Fetcher,
} from './service.ts';
export { PairingService, sasMatches, type PairingOffer, type PairingCompletion, type PairingSeal } from './pairing.ts';
export {
  ZA_KDF_INTERACTIVE,
  ZA_SUBKEY_LABELS,
  utf8ToB64,
  b64ToUtf8,
  type ZeroAccessCrypto,
  type ZaKdfParams,
  type ZaKeyRef,
  type ZaSubkeyLabel,
} from './crypto.ts';
export {
  ZA_PROTECTS,
  ZA_SERVER_STILL_SEES,
  ZA_ACTIVE_SERVER_CAVEAT,
  ZA_NO_SEARCH_CLAIM,
  ZA_RECOVERY_TRADEOFF,
} from './disclosure.ts';
export { passkeySupported, passkeySecretB64 } from './passkey.ts';
export { encodeQr, type EcLevel } from './qr.ts';
