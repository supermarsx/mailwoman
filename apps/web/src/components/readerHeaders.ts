// Reader facts from the optional message headers: Expires / Expiry-Date,
// Reply-By, Sensitivity and Supersedes (RFC 2156, RFC 4021). Pure parsing of a
// raw header block, with the date rules a reader needs (an unreadable date is
// ignored, never guessed).

export type SensitivityLevel = 'personal' | 'private' | 'company-confidential';

export interface HeaderFacts {
  /** Expires or Expiry-Date. */
  expires?: Date;
  replyBy?: Date;
  sensitivity?: SensitivityLevel;
  /** The Message-ID this message replaces, with angle brackets. */
  supersedes?: string;
}

function unfold(block: string): string {
  return block.replace(/\r?\n[ \t]+/g, ' ');
}

function field(block: string, name: string): string | undefined {
  const m = new RegExp(`^${name}:[ \\t]*(.*)$`, 'im').exec(block);
  return m === null ? undefined : m[1]!.trim();
}

/** A date header value to a Date; RFC 5322 dates and ISO 8601 are read, comments `(...)` dropped. */
export function parseHeaderDate(value: string | undefined): Date | undefined {
  if (value === undefined) return undefined;
  const cleaned = value.replace(/\([^)]*\)/g, '').trim();
  if (cleaned === '' || cleaned.length > 100) return undefined;
  const t = Date.parse(cleaned);
  return Number.isNaN(t) ? undefined : new Date(t);
}

export function parseHeaderFacts(raw: string): HeaderFacts {
  const end = /\r?\n\r?\n/.exec(raw);
  const block = unfold(end === null ? raw : raw.slice(0, end.index));
  const out: HeaderFacts = {};
  const expires = parseHeaderDate(field(block, 'Expires')) ?? parseHeaderDate(field(block, 'Expiry-Date'));
  if (expires) out.expires = expires;
  // RFC 4021 allows a list; the first date is the one shown.
  const replyBy = parseHeaderDate(field(block, 'Reply-By')?.split(';')[0]);
  if (replyBy) out.replyBy = replyBy;
  const sens = field(block, 'Sensitivity')?.toLowerCase();
  if (sens === 'personal' || sens === 'private' || sens === 'company-confidential') out.sensitivity = sens;
  const sup = /<[^<>\s@]+@[^<>\s@]+>/.exec(field(block, 'Supersedes') ?? '');
  if (sup) out.supersedes = sup[0];
  return out;
}

export type ExpiryMode = 'off' | 'dim' | 'archive-suggestion';
export const isExpiryMode = (v: unknown): v is ExpiryMode => v === 'off' || v === 'dim' || v === 'archive-suggestion';

export function isExpired(facts: HeaderFacts, now: Date): boolean {
  return facts.expires !== undefined && facts.expires.getTime() <= now.getTime();
}

export function hasFacts(facts: HeaderFacts): boolean {
  return facts.expires !== undefined || facts.replyBy !== undefined || facts.sensitivity !== undefined || facts.supersedes !== undefined;
}

/** The facts from the `header:<Name>:asText` values an `Email/get` returned. */
export function factsFromEmail(email: {
  'header:Expires:asText'?: string | null;
  'header:Expiry-Date:asText'?: string | null;
  'header:Reply-By:asText'?: string | null;
  'header:Sensitivity:asText'?: string | null;
  'header:Supersedes:asText'?: string | null;
}): HeaderFacts {
  const lines = [
    ['Expires', email['header:Expires:asText']],
    ['Expiry-Date', email['header:Expiry-Date:asText']],
    ['Reply-By', email['header:Reply-By:asText']],
    ['Sensitivity', email['header:Sensitivity:asText']],
    ['Supersedes', email['header:Supersedes:asText']],
  ].flatMap(([name, value]) => (typeof value === 'string' && value !== '' ? [`${name}: ${value.replace(/[\r\n]+/g, ' ')}`] : []));
  return parseHeaderFacts(lines.join('\n'));
}
