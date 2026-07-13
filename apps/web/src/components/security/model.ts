// Plain-language + tone model for the Reader Security panel (plan §3 e3 / §7.3).
// Pure, presentation-only helpers that turn the FROZEN `SecurityVerdict`
// (`api/security-types.ts`) enum tokens into human copy + a severity tone the
// panel colours by. Kept out of the component so the mapping is unit-testable and
// the component stays declarative. No I/O, no engine coupling.

import type {
  AuthResult,
  AttachmentRisk,
  AttachmentRiskKind,
  SecurityAnomaly,
  SecurityVerdict,
  SignatureStatus,
  SignatureVerdict,
} from '../../api/security-types.ts';

/** Severity tone the panel colours by (worst tone wins for the chip). */
export type SecurityTone = 'good' | 'warning' | 'bad' | 'neutral';

const TONE_RANK: Record<SecurityTone, number> = { neutral: 0, good: 1, warning: 2, bad: 3 };

/** The highest-severity tone in `tones` (neutral if empty). */
export function worstTone(tones: SecurityTone[]): SecurityTone {
  return tones.reduce<SecurityTone>(
    (acc, t) => (TONE_RANK[t] > TONE_RANK[acc] ? t : acc),
    'neutral',
  );
}

// ── Email authentication (DKIM/SPF/DMARC/ARC) ────────────────────────────────

/** Plain-language for a shared DKIM/SPF/DMARC/ARC result (frozen §2.1). */
export const AUTH_RESULT_LABEL: Record<AuthResult, string> = {
  pass: 'passed',
  fail: 'failed',
  none: 'not present',
  neutral: 'neutral',
  temperror: 'temporary error',
  permerror: 'permanent error',
};

export const AUTH_RESULT_TONE: Record<AuthResult, SecurityTone> = {
  pass: 'good',
  fail: 'bad',
  none: 'neutral',
  neutral: 'warning',
  temperror: 'warning',
  permerror: 'bad',
};

// ── Signature / certificate ──────────────────────────────────────────────────

/** The 3-state signature status copy (FROZEN UI contract, §2.1). */
export const SIGNATURE_STATUS_LABEL: Record<SignatureStatus, string> = {
  verified: 'Signature verified',
  'unverified-key': 'Signed — signer key not verified',
  invalid: 'Signature is invalid',
  none: 'Not signed',
};

export const SIGNATURE_STATUS_TONE: Record<SignatureStatus, SecurityTone> = {
  verified: 'good',
  'unverified-key': 'warning',
  invalid: 'bad',
  none: 'neutral',
};

export const CHAIN_STATUS_LABEL: Record<
  NonNullable<SignatureVerdict['chainStatus']>,
  string
> = {
  trusted: 'Trusted chain',
  untrusted: 'Untrusted chain',
  expired: 'Chain expired',
  unknown: 'Chain unknown',
};

export const CHAIN_STATUS_TONE: Record<
  NonNullable<SignatureVerdict['chainStatus']>,
  SecurityTone
> = {
  trusted: 'good',
  untrusted: 'warning',
  expired: 'warning',
  unknown: 'neutral',
};

export const REVOCATION_STATUS_LABEL: Record<
  NonNullable<SignatureVerdict['revocationStatus']>,
  string
> = {
  good: 'Not revoked',
  revoked: 'Key revoked',
  unknown: 'Revocation unknown',
};

export const REVOCATION_STATUS_TONE: Record<
  NonNullable<SignatureVerdict['revocationStatus']>,
  SecurityTone
> = {
  good: 'good',
  revoked: 'bad',
  unknown: 'neutral',
};

/** Overall tone of the signature block (worst of status / chain / revocation / keyChange). */
export function signatureTone(sig: SignatureVerdict): SecurityTone {
  const tones: SecurityTone[] = [SIGNATURE_STATUS_TONE[sig.status]];
  if (sig.chainStatus) tones.push(CHAIN_STATUS_TONE[sig.chainStatus]);
  if (sig.revocationStatus) tones.push(REVOCATION_STATUS_TONE[sig.revocationStatus]);
  if (sig.keyChanged) tones.push('warning');
  return worstTone(tones);
}

// ── Attachment risk ──────────────────────────────────────────────────────────

export const ATTACHMENT_RISK_LABEL: Record<AttachmentRiskKind, string> = {
  none: 'No known risk',
  macro: 'Contains macros',
  executable: 'Executable file',
  'encrypted-archive': 'Encrypted archive',
  'double-extension': 'Double file extension',
};

export const ATTACHMENT_RISK_TONE: Record<AttachmentRiskKind, SecurityTone> = {
  none: 'neutral',
  macro: 'warning',
  executable: 'bad',
  'encrypted-archive': 'warning',
  'double-extension': 'bad',
};

/** Overall tone for one attachment (its risk, escalated by a type mismatch). */
export function attachmentTone(a: AttachmentRisk): SecurityTone {
  const tones: SecurityTone[] = [ATTACHMENT_RISK_TONE[a.risk]];
  if (a.mismatch) tones.push('warning');
  return worstTone(tones);
}

// ── Anomalies ────────────────────────────────────────────────────────────────

/** Plain-language for each anomaly enum token (frozen §2.1). */
export const ANOMALY_LABEL: Record<SecurityAnomaly, string> = {
  replyToMismatch: 'Reply-To address differs from the sender',
  envelopeFromDivergence: 'Envelope sender differs from the From address',
  messageIdDomainAnomaly: "Message-ID domain doesn't match the sender",
  dateSkew: 'Send date looks skewed',
  punycodeSender: 'Sender uses punycode (possible look-alike domain)',
};

// ── Overall verdict tone (drives the collapsed chip colour) ──────────────────

/** The worst tone across every facet of a verdict — the chip's colour. */
export function overallTone(v: SecurityVerdict): SecurityTone {
  const tones: SecurityTone[] = [
    AUTH_RESULT_TONE[v.auth.dkim.result],
    AUTH_RESULT_TONE[v.auth.spf.result],
    AUTH_RESULT_TONE[v.auth.dmarc.result],
    AUTH_RESULT_TONE[v.auth.arc.result],
  ];
  if (v.auth.dmarc.result === 'pass' && !v.auth.dmarc.aligned) tones.push('warning');
  if (v.signature) tones.push(signatureTone(v.signature));
  for (const a of v.attachments) tones.push(attachmentTone(a));
  if (v.anomalies.length > 0) tones.push('warning');
  return worstTone(tones);
}

// ── Formatting ───────────────────────────────────────────────────────────────

/** A compact human delay (`1.2 s`, `340 ms`, `2 m`) — `null` renders as a dash. */
export function formatDelay(ms: number | null): string {
  if (ms === null) return '—';
  if (ms < 1000) return `${ms} ms`;
  if (ms < 60_000) return `${(ms / 1000).toFixed(1)} s`;
  return `${Math.round(ms / 60_000)} m`;
}

// ── Sender controls (the props/callback boundary e8 wires to SenderControl/set) ─

/** A sender-control action (frozen §2.2 `SenderControl/set`). */
export type SenderControlAction =
  | 'block'
  | 'silence'
  | 'ignore-conversation'
  | 'report-phishing'
  | 'report-junk';

/** The `SenderControl/set` argument shape (frozen §2.2). */
export interface SenderControlRequest {
  emailId?: string;
  address?: string;
  threadId?: string;
  action: SenderControlAction;
  abuseReport?: boolean;
}

/** The `SenderControl/set` result shape (frozen §2.2). */
export interface SenderControlResult {
  updated: boolean;
  mailRuleId?: string;
}

/** Button copy for each sender control. */
export const SENDER_CONTROL_LABEL: Record<SenderControlAction, string> = {
  block: 'Block sender',
  silence: 'Silence sender',
  'ignore-conversation': 'Ignore conversation',
  'report-phishing': 'Report phishing',
  'report-junk': 'Report junk',
};

/** Short past-tense confirmation used in the panel's live region. */
export const SENDER_CONTROL_DONE: Record<SenderControlAction, string> = {
  block: 'Sender blocked',
  silence: 'Sender silenced',
  'ignore-conversation': 'Conversation ignored',
  'report-phishing': 'Reported as phishing',
  'report-junk': 'Reported as junk',
};

/** The controls that phrase the danger (destructive/report) styling. */
export const SENDER_CONTROL_DANGER: ReadonlySet<SenderControlAction> = new Set([
  'block',
  'report-phishing',
]);

/**
 * The standalone default `onSenderControl` (plan §3 e3). It performs NO real
 * dispatch — it returns a synthetic mock `SenderControl/set` result so the panel
 * is usable + testable on its own. e8 replaces this with the real engine dispatch
 * when it mounts the component into the Reader toolbar.
 */
export async function defaultSenderControl(
  req: SenderControlRequest,
): Promise<SenderControlResult> {
  const materialisesRule = req.action === 'block' || req.action === 'ignore-conversation';
  const result: SenderControlResult = { updated: true };
  if (materialisesRule) result.mailRuleId = `mock-rule-${req.action}`;
  return result;
}
