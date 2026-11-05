import { isMailbox, parseRecipients, splitAddressList } from '../../api/jmap.ts';
import { normalizeTrustedDomain } from './recipientTrust.ts';

/** How a pre-send check behaves. `deny` is the only value that blocks sending. */
export type CheckPolicy = 'allow' | 'warn' | 'warn-remember' | 'deny';
export const CHECK_POLICIES: readonly CheckPolicy[] = ['allow', 'warn', 'warn-remember', 'deny'];
export function isCheckPolicy(value: unknown): value is CheckPolicy {
  return typeof value === 'string' && (CHECK_POLICIES as readonly string[]).includes(value);
}

export type CheckId = 'invalidRecipients' | 'missingAttachment' | 'emptySubject' | 'externalRecipients';
export const CHECK_ORDER: readonly CheckId[] = ['invalidRecipients', 'emptySubject', 'missingAttachment', 'externalRecipients'];
/** The settings key holding each check's policy. */
export const CHECK_KEYS: Readonly<Record<CheckId, string>> = {
  invalidRecipients: 'mail.compose.checkInvalidRecipients',
  missingAttachment: 'mail.compose.checkMissingAttachment',
  emptySubject: 'mail.compose.checkEmptySubject',
  externalRecipients: 'mail.externalRecipientWarning',
};

/** The effective policy from a raw setting value. `off` (the older external-recipient
 *  value) means allow; `confirm` keeps its own review checkbox, so it adds no dialog item. */
export function resolvePolicy(raw: unknown): CheckPolicy {
  if (raw === 'off' || raw === 'confirm') return 'allow';
  return isCheckPolicy(raw) ? raw : 'warn';
}

export interface SendIssue { id: CheckId; details: string[] }
export interface SendPlan {
  /** Issues whose policy is `deny`: sending is refused. */
  denied: SendIssue[];
  /** Issues to put in the one confirmation dialog, in check order. */
  asked: SendIssue[];
  /** The asked issues that offer "don't ask again" (policy warn-remember and not locked). */
  rememberable: CheckId[];
}

export function planSend(
  issues: readonly SendIssue[],
  policies: Readonly<Record<CheckId, CheckPolicy>>,
  locked: Readonly<Partial<Record<CheckId, boolean>>> = {},
): SendPlan {
  const ordered = CHECK_ORDER.map(id => issues.find(issue => issue.id === id)).filter((issue): issue is SendIssue => issue !== undefined);
  const denied = ordered.filter(issue => policies[issue.id] === 'deny');
  const asked = ordered.filter(issue => policies[issue.id] === 'warn' || policies[issue.id] === 'warn-remember');
  const rememberable = asked.filter(issue => policies[issue.id] === 'warn-remember' && locked[issue.id] !== true).map(issue => issue.id);
  return { denied, asked, rememberable };
}

/** Recipient entries (To, Cc, Bcc) that are not valid mailboxes. */
export function invalidRecipients(fields: readonly string[]): string[] {
  const bad: string[] = [];
  for (const field of fields) {
    for (const entry of splitAddressList(field)) {
      if (entry.trim() === '') continue;
      const address = parseRecipients(entry)[0];
      if (!address || !isMailbox(address.email)) bad.push(entry.trim());
    }
  }
  return [...new Set(bad)];
}

/** The domains of external recipient addresses, in order, without repeats. */
export function externalDomains(addresses: readonly string[]): string[] {
  const out: string[] = [];
  for (const address of addresses) {
    const at = address.lastIndexOf('@');
    const raw = at >= 0 ? address.slice(at + 1) : address;
    const domain = (at >= 0 ? normalizeTrustedDomain(raw) : null) ?? raw.toLowerCase();
    if (!out.includes(domain)) out.push(domain);
  }
  return out;
}

const QUOTE_HEADERS = [
  /^\s*on\b.{3,200}\bwrote:\s*$/iu,
  /^\s*em\b.{3,200}\bescreveu:\s*$/iu,
  /^\s*le\b.{3,200}\ba écrit\s*:\s*$/iu,
  /^\s*am\b.{3,200}\bschrieb\b.{0,100}:\s*$/iu,
  /^\s*el\b.{3,200}\bescribió:\s*$/iu,
  /^\s*il\b.{3,200}\bha scritto:\s*$/iu,
  /^\s*-{2,}\s*(original message|forwarded message|mensagem original|mensagem reencaminhada|message d.origine|ursprüngliche nachricht)\b.*$/iu,
  /^\s*_{5,}\s*$/u,
];
const OUTLOOK_FROM = /^\s*(from|de|von|da|van)\s*:/iu;
const OUTLOOK_FOLLOW = /^\s*(sent|date|enviado|enviada|data|gesendet|envoyé|inviato|verzonden|to|para|subject|assunto)\s*:/iu;

/** The text the sender wrote: without quoted lines, the quoted original and the signature. */
export function ownText(text: string): string {
  const lines = text.split(/\r?\n/u);
  const kept: string[] = [];
  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i]!;
    if (/^--\s?$/u.test(line)) break;
    if (QUOTE_HEADERS.some(pattern => pattern.test(line))) break;
    if (OUTLOOK_FROM.test(line) && lines.slice(i + 1, i + 5).some(next => OUTLOOK_FOLLOW.test(next))) break;
    if (/^\s*>/u.test(line)) continue;
    kept.push(line);
  }
  return kept.join('\n');
}

const ATTACHMENT_WORDS = [
  'attach(?:ed|ment|ments|ing)?', 'enclosed', 'enclosing', 'enclosure',
  'anexo|anexos|anexa|anexas|anexado|anexada|anexados|anexadas|anexei|anexamos|em anexo',
  'adjunto|adjuntos|adjunta|adjuntas|archivo adjunto|adjuntamos',
  'pièce jointe|pièces jointes|ci-joint|ci-jointe|joint|jointe|jointes',
  'angehängt|angehängte|anhang|anbei|beigefügt|im anhang',
  'allegato|allegata|allegati|allegate|in allegato',
  'bijlage|bijlagen|bijgevoegd|bijgevoegde',
];
const ATTACHMENT_RE = new RegExp(`(?<![\\p{L}\\p{N}])(?:${ATTACHMENT_WORDS.join('|')})(?![\\p{L}\\p{N}])`, 'iu');

/** Whether the sender's own text (not the quote or signature) mentions an attachment. */
export function mentionsAttachment(text: string): boolean {
  return ATTACHMENT_RE.test(ownText(text));
}
