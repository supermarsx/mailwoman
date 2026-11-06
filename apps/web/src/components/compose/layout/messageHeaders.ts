// Optional message headers (Options tab): serialisation and validation. Pure
// functions; the send path passes the result through as extra headers, after the
// server re-validates with the same rules.

export type Importance = 'low' | 'normal' | 'high';
export type Sensitivity = 'normal' | 'personal' | 'private' | 'company-confidential';
export type AutoSubmitted = 'no' | 'auto-generated' | 'auto-replied';

export interface MessageHeaderOptions {
  importance?: Importance;
  sensitivity?: Sensitivity;
  /** Disposition-Notification-To address. */
  readReceiptTo?: string;
  /** Legacy Return-Receipt-To address (admin gated). */
  returnReceiptTo?: string | undefined;
  replyBy?: Date | null;
  /** Flag the sent copy for follow-up at the Reply-By time (the follow-up feature); not a header. */
  remindIfNoReply?: boolean;
  expires?: Date | null;
  supersedes?: string;
  autoSubmitted?: AutoSubmitted | undefined;
  inReplyTo?: string;
  references?: string[];
  custom?: { name: string; value: string }[];
}

export interface HeaderGates { allowAutoSubmitted: boolean; allowThreadingEdit: boolean; allowCustom: boolean; allowLegacyReturnReceipt: boolean }
export const DEFAULT_GATES: HeaderGates = { allowAutoSubmitted: false, allowThreadingEdit: false, allowCustom: false, allowLegacyReturnReceipt: false };

export const MAX_CUSTOM_HEADERS = 20;
const NAME_MAX = 64;
const VALUE_MAX = 998;

const DENY = new Set(['from', 'to', 'cc', 'bcc', 'subject', 'date', 'message-id', 'mime-version', 'received', 'return-path', 'sender']);
export function isDeniedHeader(name: string): boolean {
  const n = name.trim().toLowerCase();
  return DENY.has(n) || n.startsWith('content-') || n.startsWith('x-mailwoman-');
}

/** RFC 5322 field-name: printable US-ASCII except colon. */
export const isHeaderName = (n: string): boolean => /^[\x21-\x39\x3b-\x7e]+$/.test(n) && n.length <= NAME_MAX;
export const hasLineBreak = (v: string): boolean => [...v].some((c) => [13, 10, 0x85, 0x2028, 0x2029, 0].includes(c.charCodeAt(0)));

export type HeaderError = 'name' | 'value' | 'denied' | 'duplicate' | 'tooMany' | 'crlf';

export function validateCustomHeader(h: { name: string; value: string }): HeaderError | null {
  if (hasLineBreak(h.name) || hasLineBreak(h.value)) return 'crlf';
  if (!isHeaderName(h.name.trim())) return 'name';
  if (isDeniedHeader(h.name)) return 'denied';
  if (h.value.length > VALUE_MAX) return 'value';
  return null;
}

const DAYS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
const p2 = (n: number): string => String(n).padStart(2, '0');

/** RFC 5322 date-time in UTC (`Tue, 03 Nov 2026 17:00:00 +0000`). */
export function rfc5322Date(d: Date): string {
  return `${DAYS[d.getUTCDay()]}, ${p2(d.getUTCDate())} ${MONTHS[d.getUTCMonth()]} ${d.getUTCFullYear()} ${p2(d.getUTCHours())}:${p2(d.getUTCMinutes())}:${p2(d.getUTCSeconds())} +0000`;
}

const MSGID = /^<[^<>\s@]+@[^<>\s@]+>$/;
export const isMessageId = (v: string): boolean => MSGID.test(v.trim()) && v.length <= 255;
const EMAIL = /^[^\s<>@,;"]+@[^\s<>@,;"]+\.[^\s<>@,;"]+$/;
export const isMailbox = (v: string): boolean => EMAIL.test(v.trim());

/** The X-Priority number for an importance (RFC 2156 has no number; X-Priority is the de-facto one). */
export const X_PRIORITY: Record<Importance, string> = { high: '1', normal: '3', low: '5' };
export const X_MSMAIL: Record<Importance, string> = { high: 'High', normal: 'Normal', low: 'Low' };

export interface BuiltHeaders { headers: [string, string][]; errors: { field: string; error: HeaderError | 'invalid' }[] }

/**
 * The headers to add. `sendLegacyPriority` is `mail.priority.sendHeaders`: when
 * off only `Importance` is written. Anything invalid is dropped and reported.
 */
export function buildHeaders(o: MessageHeaderOptions, gates: HeaderGates = DEFAULT_GATES, sendLegacyPriority = true): BuiltHeaders {
  const headers: [string, string][] = [];
  const errors: BuiltHeaders['errors'] = [];
  const bad = (field: string, error: HeaderError | 'invalid'): void => { errors.push({ field, error }); };

  if (o.importance && o.importance !== 'normal') {
    headers.push(['Importance', o.importance]);
    if (sendLegacyPriority) { headers.push(['X-Priority', X_PRIORITY[o.importance]]); headers.push(['X-MSMail-Priority', X_MSMAIL[o.importance]]); }
  }
  if (o.sensitivity && o.sensitivity !== 'normal') headers.push(['Sensitivity', o.sensitivity]);
  if (o.readReceiptTo) { if (isMailbox(o.readReceiptTo) && !hasLineBreak(o.readReceiptTo)) headers.push(['Disposition-Notification-To', o.readReceiptTo.trim()]); else bad('readReceiptTo', 'invalid'); }
  if (o.returnReceiptTo) {
    if (!gates.allowLegacyReturnReceipt) bad('returnReceiptTo', 'denied');
    else if (isMailbox(o.returnReceiptTo) && !hasLineBreak(o.returnReceiptTo)) headers.push(['Return-Receipt-To', o.returnReceiptTo.trim()]);
    else bad('returnReceiptTo', 'invalid');
  }
  if (o.replyBy) headers.push(['Reply-By', rfc5322Date(o.replyBy)]);
  if (o.expires) { const v = rfc5322Date(o.expires); headers.push(['Expires', v]); headers.push(['Expiry-Date', v]); }
  if (o.supersedes) { if (isMessageId(o.supersedes)) headers.push(['Supersedes', o.supersedes.trim()]); else bad('supersedes', 'invalid'); }
  if (o.autoSubmitted) {
    if (!gates.allowAutoSubmitted) bad('autoSubmitted', 'denied');
    else headers.push(['Auto-Submitted', o.autoSubmitted]);
  }
  if (o.inReplyTo !== undefined || o.references) {
    if (!gates.allowThreadingEdit) bad('threading', 'denied');
    else {
      if (o.inReplyTo && isMessageId(o.inReplyTo)) headers.push(['In-Reply-To', o.inReplyTo.trim()]);
      else if (o.inReplyTo) bad('inReplyTo', 'invalid');
      const refs = (o.references ?? []).filter(isMessageId);
      if (refs.length) headers.push(['References', refs.join(' ')]);
    }
  }
  if (o.custom?.length) {
    if (!gates.allowCustom) bad('custom', 'denied');
    else {
      const seen = new Set<string>();
      o.custom.forEach((h, i) => {
        if (i >= MAX_CUSTOM_HEADERS) { bad(`custom.${i}`, 'tooMany'); return; }
        const err = validateCustomHeader(h);
        if (err) { bad(`custom.${i}`, err); return; }
        const key = h.name.trim().toLowerCase();
        if (seen.has(key)) { bad(`custom.${i}`, 'duplicate'); return; }
        seen.add(key);
        headers.push([h.name.trim(), h.value.trim()]);
      });
    }
  }
  return { headers, errors };
}

/** Admin `compose.headers.forced` ("Name=value" pairs) override the user's headers of the same name. */
export function applyForced(headers: [string, string][], forced: readonly string[]): [string, string][] {
  const out = headers.slice();
  for (const pair of forced) {
    const eq = pair.indexOf('=');
    if (eq <= 0) continue;
    const name = pair.slice(0, eq).trim();
    const value = pair.slice(eq + 1).trim();
    if (!isHeaderName(name) || isDeniedHeader(name) || hasLineBreak(value) || value.length > VALUE_MAX) continue;
    const at = out.findIndex(([n]) => n.toLowerCase() === name.toLowerCase());
    if (at >= 0) out[at] = [name, value]; else out.push([name, value]);
  }
  return out;
}

/** JSON text for a draft: dates as ISO strings. */
export function serializeHeaderOptions(o: MessageHeaderOptions): string {
  return JSON.stringify({ ...o, replyBy: o.replyBy?.toISOString(), expires: o.expires?.toISOString() });
}

const SENS: readonly Sensitivity[] = ['normal', 'personal', 'private', 'company-confidential'];
const AUTO_VALUES: readonly AutoSubmitted[] = ['no', 'auto-generated', 'auto-replied'];

/** Read a draft's options back; anything unknown or malformed is dropped. */
export function parseHeaderOptions(text: string): MessageHeaderOptions {
  let raw: unknown;
  try { raw = JSON.parse(text); } catch { return {}; }
  if (raw === null || typeof raw !== 'object') return {};
  const r = raw as Record<string, unknown>;
  const out: MessageHeaderOptions = {};
  if (SENS.includes(r['sensitivity'] as Sensitivity)) out.sensitivity = r['sensitivity'] as Sensitivity;
  if (AUTO_VALUES.includes(r['autoSubmitted'] as AutoSubmitted)) out.autoSubmitted = r['autoSubmitted'] as AutoSubmitted;
  for (const key of ['replyBy', 'expires'] as const) {
    const v = r[key];
    if (typeof v === 'string') { const d = new Date(v); if (!Number.isNaN(d.getTime())) out[key] = d; }
  }
  if (r['remindIfNoReply'] === true) out.remindIfNoReply = true;
  if (typeof r['supersedes'] === 'string' && isMessageId(r['supersedes'])) out.supersedes = r['supersedes'];
  if (typeof r['returnReceiptTo'] === 'string' && isMailbox(r['returnReceiptTo'])) out.returnReceiptTo = r['returnReceiptTo'];
  if (Array.isArray(r['custom'])) {
    out.custom = (r['custom'] as unknown[]).flatMap((h) => {
      if (h === null || typeof h !== 'object') return [];
      const { name, value } = h as Record<string, unknown>;
      return typeof name === 'string' && typeof value === 'string' && validateCustomHeader({ name, value }) === null ? [{ name, value }] : [];
    }).slice(0, MAX_CUSTOM_HEADERS);
  }
  return out;
}
