// FROZEN Mailwoman crypto object types (plan §2.1) — the keyring / DLP / mail-rule
// shapes the web client and the engine agree on, byte-for-byte with
// `crates/mw-crypto/src/types.rs` (re-exported by `crates/mw-engine/src/security/
// types.rs`). Mailwoman-native, camelCase; NOT an IETF draft.
//
// Authored by e0; the Batch-B web modules (e2 key-mgmt, e4 compose-crypto) consume
// these against the mock (`mw-mock-jmap`) until e8 swaps in the real engine + the
// WASM crypto worker. Field names / enum tokens match §2.1 EXACTLY — drift is a
// build failure (the V2/V3 parity gate, plan §1.5).

import type { Id, UtcDate } from './jmap-types.ts';

/** Mailwoman crypto/security capability URNs advertised in the session (§2.2). */
export const CAP_CRYPTO = 'urn:mailwoman:crypto';
export const CAP_SECURITY = 'urn:mailwoman:security';

/** A key kind (frozen §2.1). */
export type KeyKind = 'pgp' | 'smime';

/** TOFU trust state of a key (frozen §2.1). */
export type KeyTrust = 'unverified' | 'tofu' | 'verified' | 'revoked';

/** Where a key came from (frozen §2.1). */
export type KeySource =
  | 'generated'
  | 'imported'
  | 'pkcs12'
  | 'wkd'
  | 'vks'
  | 'harvested'
  | 'autocrypt-header';

/** One TOFU key-history entry (a fingerprint first seen at a time). */
export interface KeyHistoryEntry {
  fingerprint: string;
  seenAt: UtcDate;
}

/**
 * A PGP or S/MIME key/cert (§2.1). Own keys carry an opaque
 * `encryptedPrivateBackup` (client-encrypted — the server NEVER decrypts it,
 * plan §1.2 / risk #4); harvested/contact keys are public-only. Per-contact
 * association reuses the V3 `ContactCard.pgpKey`/`smimeCert` fields.
 */
export interface CryptoKey {
  id: Id;
  kind: KeyKind;
  isOwn: boolean;
  addresses: string[];
  fingerprint: string;
  keyId: string;
  algorithm: string;
  createdAt: UtcDate;
  expiresAt: UtcDate | null;
  /** Armored PGP public key — `null` for S/MIME. */
  publicKeyArmored: string | null;
  /** PEM certificate (S/MIME) — `null` for PGP. */
  certPem: string | null;
  trust: KeyTrust;
  autocrypt: boolean;
  source: KeySource;
  hasPrivate: boolean;
  /** Opaque client-encrypted private backup — the server never decrypts it. */
  encryptedPrivateBackup: string | null;
  verifiedAt: UtcDate | null;
  keyHistory: KeyHistoryEntry[];
}

// ── DLP (§2.1) ───────────────────────────────────────────────────────────────

/** A built-in DLP detector (frozen §2.1). */
export type DlpDetector = 'pan' | 'iban' | 'national-id' | 'ssn' | 'custom-regex';

/** A DLP rule action (frozen §2.1). */
export type DlpAction = 'warn' | 'block' | 'require-encryption' | 'notify-admin';

/** The match conditions of a DLP rule (config-sourced). */
export interface DlpConditions {
  detectors: DlpDetector[];
  customRegex: string | null;
  dictionaries: string[];
  attachmentTypes: string[];
  maxAttachmentSize: number | null;
  recipientDomains: string[];
  recipientDomainMode: 'in' | 'notIn' | null;
  classification: string | null;
}

/** A DLP rule (config/env-sourced in V4 — the admin panel is V6, plan §1.8). */
export interface DlpRule {
  id: Id;
  name: string;
  enabled: boolean;
  priority: number;
  conditions: DlpConditions;
  action: DlpAction;
  message: string;
}

/** One rule's evaluation against an outbound draft (redacted — never content). */
export interface DlpVerdict {
  ruleId: Id;
  ruleName: string;
  action: DlpAction;
  matchedDetectors: string[];
  excerptRedacted: string;
  blocked: boolean;
}

// ── Mail rules (§2.1) — the block/silence/ignore surface over rules.rs ────────

/** A mail-rule condition type / operator (frozen §2.1). */
export type MailRuleConditionType = 'from' | 'to' | 'subject' | 'thread';
export type MailRuleOp = 'is' | 'contains';

/** A mail-rule action type (frozen §2.1). */
export type MailRuleActionType = 'move' | 'tag' | 'stop' | 'suppressNotify' | 'archive';

export interface MailRuleCondition {
  type: MailRuleConditionType;
  op: MailRuleOp;
  value: string;
}

export interface MailRuleAction {
  type: MailRuleActionType;
  value: string | null;
}

/** A mail rule (block/silence/ignore materialize here; also the Sieve round-trip). */
export interface MailRule {
  id: Id;
  name: string;
  matchAll: boolean;
  conditions: MailRuleCondition[];
  actions: MailRuleAction[];
  enabled: boolean;
  runsAt: 'server-sieve' | 'engine';
}
