// FROZEN Mailwoman security-verdict types (plan §2.1 / §7.3) — the SecurityVerdict
// shape the Reader Security panel renders, byte-for-byte with
// `crates/mw-crypto/src/types.rs` (`SecurityVerdict` + sub-types, re-exported by
// the engine security module). camelCase; parity-critical (plan §1.5).
//
// Authored by e0; e3 (Security panel) consumes these against the mock verdict
// fixture until e8 wires `SecurityVerdict/get` + the client decrypt/verify merge.
// The 3-state `signature.status` is the FROZEN UI contract.

import type { UtcDate } from './jmap-types.ts';

/** Shared DKIM/SPF/DMARC/ARC result enum (frozen §2.1). */
export type AuthResult =
  | 'pass'
  | 'fail'
  | 'none'
  | 'neutral'
  | 'temperror'
  | 'permerror';

export interface DkimVerdict {
  result: AuthResult;
  domain: string | null;
  selector: string | null;
}

export interface SpfVerdict {
  result: AuthResult;
  domain: string | null;
}

export interface DmarcVerdict {
  result: AuthResult;
  policy: 'none' | 'quarantine' | 'reject' | null;
  aligned: boolean;
}

export interface ArcVerdict {
  result: AuthResult;
  chainLength: number;
}

/** The email-authentication block, computed server-side by `mail-auth`. */
export interface AuthVerdict {
  dkim: DkimVerdict;
  spf: SpfVerdict;
  dmarc: DmarcVerdict;
  arc: ArcVerdict;
}

/** One `Received`-chain hop (ASN/country are optional GeoIP enrichment). */
export interface ReceivedHop {
  index: number;
  byHost: string | null;
  fromHost: string | null;
  protocol: string | null;
  timestamp: UtcDate | null;
  delayMs: number | null;
  asn: number | null;
  asnOrg: string | null;
  country: string | null;
}

/** The 3-state signature/cert status (FROZEN UI contract, §2.1). */
export type SignatureStatus = 'verified' | 'unverified-key' | 'invalid' | 'none';

/**
 * The signature/cert verdict (§2.1). Reused as the WASM verify/decrypt/sign
 * return (`contracts/crypto.ts` `SignatureVerdict`) — same shape both sides.
 */
export interface SignatureVerdict {
  kind: 'pgp' | 'smime';
  status: SignatureStatus;
  signerKeyId: string | null;
  algorithm: string | null;
  keyCreatedAt: UtcDate | null;
  keyExpiresAt: UtcDate | null;
  chainStatus: 'trusted' | 'untrusted' | 'expired' | 'unknown' | null;
  revocationStatus: 'good' | 'revoked' | 'unknown' | null;
  keyChanged: boolean;
}

/** Whether/how the message is encrypted (`decryptsClientSide` flags the WASM path). */
export interface EncryptionInfo {
  kind: 'pgp' | 'smime' | 'none';
  isEncrypted: boolean;
  decryptsClientSide: boolean;
}

/** Per-attachment risk analysis (ext-vs-magic mismatch + a risk token). */
export type AttachmentRiskKind =
  | 'none'
  | 'macro'
  | 'executable'
  | 'encrypted-archive'
  | 'double-extension';

export interface AttachmentRisk {
  name: string;
  declaredType: string | null;
  detectedType: string | null;
  mismatch: boolean;
  risk: AttachmentRiskKind;
}

/** Anomaly enum tokens (frozen §2.1). */
export type SecurityAnomaly =
  | 'replyToMismatch'
  | 'envelopeFromDivergence'
  | 'messageIdDomainAnomaly'
  | 'dateSkew'
  | 'punycodeSender';

/**
 * The §7.3 security verdict (frozen field-for-field — parity-critical). Computed
 * server-side (all public) except `encryption.decryptsClientSide`, which flags
 * the client WASM-decrypt path. A separate `SecurityVerdict/get` — NOT grafted on
 * `Email/get` (plan §1.10 / risk #15).
 */
export interface SecurityVerdict {
  emailId: string;
  auth: AuthVerdict;
  plainLanguage: string;
  received: ReceivedHop[];
  signature: SignatureVerdict | null;
  encryption: EncryptionInfo;
  attachments: AttachmentRisk[];
  anomalies: SecurityAnomaly[];
}
