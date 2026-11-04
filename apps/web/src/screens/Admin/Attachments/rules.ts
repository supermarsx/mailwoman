// Attachment rule helpers: normalising what an admin types, checking it with the
// same rules the server enforces (crates/mw-server/src/attachment_policy.rs
// `valid_rule`), spotting a rule that sits in two lists, and byte sizes with units.

export const MAX_RULES = 100;
export const MAX_RULE_LENGTH = 127;

export type RuleError = 'empty' | 'extension' | 'mime' | 'long';
export type Normalised = { ok: true; rule: string } | { ok: false; error: RuleError; input: string };

const EXT_PART = /^[a-z0-9_-]+$/;
const MIME_TOKEN = /^[a-z0-9!#$&^_.+-]+$/;

/** `.PDF`, `*.pdf`, ` pdf ` -> `pdf`; `tar.gz` stays; `IMAGE/*` -> `image/*`. */
export function normaliseRule(raw: string, mime: boolean): Normalised {
  let rule = raw.trim().toLowerCase();
  if (rule === '') return { ok: false, error: 'empty', input: raw };
  if (!mime) rule = rule.replace(/^\*?\./, '');
  if (rule.length > MAX_RULE_LENGTH) return { ok: false, error: 'long', input: raw };
  if (mime) {
    const [top, sub, ...rest] = rule.split('/');
    const token = (s: string | undefined): boolean => s !== undefined && MIME_TOKEN.test(s);
    return rest.length === 0 && token(top) && (sub === '*' || token(sub)) ? { ok: true, rule } : { ok: false, error: 'mime', input: raw };
  }
  if (rule.length > 32 || !rule.split('.').every((part) => EXT_PART.test(part))) return { ok: false, error: 'extension', input: raw };
  return { ok: true, rule };
}

/** Split pasted text on whitespace, commas, semicolons and newlines. */
export function splitBulk(text: string): string[] {
  return text.split(/[\s,;]+/).filter((s) => s !== '');
}

export interface BulkResult { rules: string[]; duplicates: string[]; invalid: Extract<Normalised, { ok: false }>[]; overflow: string[] }

/** Merge pasted or imported text into `existing`: valid new rules are appended, repeats and bad entries are reported. */
export function addBulk(existing: readonly string[], text: string, mime: boolean): BulkResult {
  const rules = [...existing];
  const duplicates: string[] = [];
  const invalid: Extract<Normalised, { ok: false }>[] = [];
  const overflow: string[] = [];
  for (const raw of splitBulk(text)) {
    const n = normaliseRule(raw, mime);
    if (!n.ok) { invalid.push(n); continue; }
    if (rules.includes(n.rule)) { duplicates.push(n.rule); continue; }
    if (rules.length >= MAX_RULES) { overflow.push(n.rule); continue; }
    rules.push(n.rule);
  }
  return { rules, duplicates, invalid, overflow };
}

/** One rule per line, for Export. */
export const exportText = (rules: readonly string[]): string => `${rules.join('\n')}\n`;

export type ListName = 'allowed' | 'blocked' | 'warn';
export type ConflictKind = 'blockedOverAllowed' | 'blockedOverWarn' | 'allowedAndWarn';
export interface Conflict { rule: string; other: string; list: ListName; kind: ConflictKind }

/** Whether MIME pattern `pattern` matches `rule` (equal, or `type/*` over `type/sub`). */
export function covers(pattern: string, rule: string, mime: boolean): boolean {
  if (pattern === rule) return true;
  if (!mime) return false;
  return pattern.endsWith('/*') && rule.startsWith(pattern.slice(0, -1));
}

/** Rules of `list` that also match a rule of another list, and which side wins. */
export function conflictsFor(list: ListName, rules: readonly string[], others: Readonly<Record<ListName, readonly string[]>>, mime: boolean): Conflict[] {
  const out: Conflict[] = [];
  const check = (otherList: ListName, kind: ConflictKind): void => {
    for (const rule of rules) {
      for (const other of others[otherList]) {
        if (covers(rule, other, mime) || covers(other, rule, mime)) out.push({ rule, other, list, kind });
      }
    }
  };
  if (list === 'allowed') { check('blocked', 'blockedOverAllowed'); check('warn', 'allowedAndWarn'); }
  if (list === 'blocked') { check('allowed', 'blockedOverAllowed'); check('warn', 'blockedOverWarn'); }
  if (list === 'warn') { check('blocked', 'blockedOverWarn'); check('allowed', 'allowedAndWarn'); }
  return out;
}

/** Which list a registry key edits, and whether it holds MIME patterns. */
export function listOf(key: string): { list: ListName; mime: boolean } | undefined {
  const m = /^attachments\.(allowed|blocked|warn)(Extensions|MimeTypes)$/.exec(key);
  return m ? { list: m[1] as ListName, mime: m[2] === 'MimeTypes' } : undefined;
}

export const BYTE_UNITS = ['B', 'KiB', 'MiB', 'GiB'] as const;
export type ByteUnit = (typeof BYTE_UNITS)[number];
const FACTOR: Record<ByteUnit, number> = { B: 1, KiB: 1024, MiB: 1024 ** 2, GiB: 1024 ** 3 };

/** The largest unit that divides `bytes` evenly (or that keeps the amount at least 1). */
export function splitBytes(bytes: number): { amount: number; unit: ByteUnit } {
  if (!Number.isFinite(bytes) || bytes <= 0) return { amount: 0, unit: 'MiB' };
  for (const unit of ['GiB', 'MiB', 'KiB'] as const) {
    if (bytes % FACTOR[unit] === 0) return { amount: bytes / FACTOR[unit], unit };
  }
  return { amount: bytes, unit: 'B' };
}

export const toBytes = (amount: number, unit: ByteUnit): number => Math.round(amount * FACTOR[unit]);
