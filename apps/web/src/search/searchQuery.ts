// The search query model behind the top-bar search input and its filter chips
// (t88). One parser and one serializer are the source of truth: the chips are a
// view of `parseQuery(input)`, and every chip change writes
// `serializeQuery(next)` back into the input.
//
// The lexer mirrors the engine's (crates/mw-search/src/query.rs) so a chip
// means what the server will run:
//   - `field:value`, `field:"quoted value"`, bare words, `"phrases"`;
//   - `-atom` and `NOT atom` negate; `AND` between atoms is implicit;
//   - inside quotes `\"` is a quote and `\\` a backslash, any other backslash is
//     literal (the engine reads the same escapes since t88);
//   - `(` groups and any top-level `OR` are kept verbatim as one expression
//     term: they are not a flat list of filters, so they are not split into chips.
// Operator names are the engine's and stay English in every locale; only the
// chip labels are translated.

import type { OfflineQuery } from '../offline/search.ts';

/** Operators the engine knows (`is_operator`). Anything else is a literal word. */
export const OPERATORS = [
  'from', 'to', 'cc', 'subject', 'body', 'text', 'filename', 'tag', 'in', 'has', 'is',
  'before', 'after', 'larger', 'smaller', 'pinned',
] as const;
export type Operator = (typeof OPERATORS)[number];

export type Term =
  | { readonly kind: 'field'; readonly op: Operator; readonly value: string; readonly negated: boolean; readonly quoted: boolean }
  | { readonly kind: 'text'; readonly value: string; readonly negated: boolean; readonly quoted: boolean }
  | { readonly kind: 'expr'; readonly text: string };

export interface SearchQuery {
  readonly terms: readonly Term[];
}

export const EMPTY_QUERY: SearchQuery = { terms: [] };

const ATTACHMENT_WORDS = ['attachment', 'attachments', 'file', 'files'];
const CALENDAR_WORDS = ['calendar', 'event', 'events', 'invite', 'invitation', 'invitations'];
const KEYWORDS = new Set(['AND', 'and', '&&', 'OR', 'or', '||', 'NOT', 'not']);

export function isOperator(name: string): name is Operator {
  return (OPERATORS as readonly string[]).includes(name);
}

/** The canonical spelling of an operator's value (`has:files` → `attachment`). */
export function canonicalValue(op: Operator, value: string): string {
  const lower = value.toLowerCase();
  if (op === 'has') {
    if (ATTACHMENT_WORDS.includes(lower)) return 'attachment';
    if (CALENDAR_WORDS.includes(lower)) return 'calendar';
    return value;
  }
  if (op === 'is') return lower === 'starred' ? 'flagged' : lower;
  if (op === 'larger' || op === 'smaller') return lower;
  return value;
}

// ---------------------------------------------------------------------------
// Lexer
// ---------------------------------------------------------------------------

type Tok =
  | { t: 'lparen'; start: number; end: number }
  | { t: 'rparen'; start: number; end: number }
  | { t: 'and' | 'or' | 'not'; start: number; end: number }
  | { t: 'atom'; start: number; end: number; field: Operator | null; value: string; quoted: boolean };

const isSpace = (c: string | undefined): boolean => c !== undefined && /\s/u.test(c);

function readQuoted(chars: readonly string[], start: number): [string, number] {
  let i = start;
  let s = '';
  while (i < chars.length) {
    const c = chars[i]!;
    if (c === '"') return [s, i + 1];
    const next = chars[i + 1];
    if (c === '\\' && (next === '"' || next === '\\')) {
      s += next;
      i += 2;
      continue;
    }
    s += c;
    i += 1;
  }
  return [s, i];
}

function lex(input: string): Tok[] {
  // Code points, like the engine's Vec<char>; positions index this array.
  const chars = Array.from(input);
  const out: Tok[] = [];
  let i = 0;
  while (i < chars.length) {
    const c = chars[i]!;
    if (isSpace(c)) { i += 1; continue; }
    if (c === '(') { out.push({ t: 'lparen', start: i, end: i + 1 }); i += 1; continue; }
    if (c === ')') { out.push({ t: 'rparen', start: i, end: i + 1 }); i += 1; continue; }
    if (c === '-') {
      const next = chars[i + 1];
      if (next !== undefined && !isSpace(next) && next !== ')') out.push({ t: 'not', start: i, end: i + 1 });
      i += 1;
      continue;
    }
    if (c === '"') {
      const [phrase, next] = readQuoted(chars, i + 1);
      out.push({ t: 'atom', start: i, end: next, field: null, value: phrase, quoted: true });
      i = next;
      continue;
    }
    let j = i;
    let word = '';
    while (j < chars.length && !isSpace(chars[j]) && chars[j] !== '(' && chars[j] !== ')' && chars[j] !== '"') {
      word += chars[j];
      j += 1;
    }
    const start = i;
    i = j;
    if (word === 'AND' || word === 'and' || word === '&&') { out.push({ t: 'and', start, end: i }); continue; }
    if (word === 'OR' || word === 'or' || word === '||') { out.push({ t: 'or', start, end: i }); continue; }
    if (word === 'NOT' || word === 'not') { out.push({ t: 'not', start, end: i }); continue; }
    const colon = word.indexOf(':');
    if (colon >= 0) {
      const field = word.slice(0, colon).toLowerCase();
      const value = word.slice(colon + 1);
      if (isOperator(field)) {
        if (value === '' && chars[i] === '"') {
          const [phrase, next] = readQuoted(chars, i + 1);
          i = next;
          out.push({ t: 'atom', start, end: i, field, value: phrase, quoted: true });
          continue;
        }
        out.push({ t: 'atom', start, end: i, field, value, quoted: false });
        continue;
      }
    }
    out.push({ t: 'atom', start, end: i, field: null, value: word, quoted: false });
  }
  return out;
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/** Parse the search input into terms. Never throws. */
export function parseQuery(input: string): SearchQuery {
  const chars = Array.from(input);
  const slice = (start: number, end: number): string => chars.slice(start, end).join('');
  const toks = lex(input);
  if (toks.length === 0) return EMPTY_QUERY;

  // A top-level OR makes the whole query one expression: splitting it into
  // AND-ed chips would say something the server does not run.
  let depth = 0;
  for (const tok of toks) {
    if (tok.t === 'lparen') depth += 1;
    else if (tok.t === 'rparen') depth = Math.max(0, depth - 1);
    else if (tok.t === 'or' && depth === 0) return { terms: [{ kind: 'expr', text: input.trim() }] };
  }

  const terms: Term[] = [];
  let i = 0;
  while (i < toks.length) {
    const tok = toks[i]!;
    if (tok.t === 'and') { i += 1; continue; }
    // Collect a run of NOTs; an odd count negates.
    let negated = false;
    const runStart = tok.start;
    let k = i;
    while (k < toks.length && toks[k]!.t === 'not') { negated = !negated; k += 1; }
    const head = toks[k];
    if (head === undefined) break; // trailing NOT: the engine errors and falls back; nothing to show
    if (head.t === 'lparen' || head.t === 'rparen') {
      // A group (with its NOTs) is kept verbatim as one expression.
      let d = 0;
      let m = k;
      for (; m < toks.length; m += 1) {
        const t = toks[m]!;
        if (t.t === 'lparen') d += 1;
        else if (t.t === 'rparen') { d -= 1; if (d <= 0) { m += 1; break; } }
      }
      const end = toks[Math.min(m, toks.length) - 1]!.end;
      terms.push({ kind: 'expr', text: slice(runStart, end) });
      i = m;
      continue;
    }
    if (head.t !== 'atom') { i = k + 1; continue; }
    i = k + 1;
    if (head.field !== null) {
      terms.push({ kind: 'field', op: head.field, value: canonicalValue(head.field, head.value), negated, quoted: head.quoted });
      continue;
    }
    // Consecutive plain bare words read as one text filter.
    const last = terms[terms.length - 1];
    if (!negated && !head.quoted && last?.kind === 'text' && !last.negated && !last.quoted) {
      terms[terms.length - 1] = { ...last, value: `${last.value} ${head.value}` };
      continue;
    }
    terms.push({ kind: 'text', value: head.value, negated, quoted: head.quoted });
  }
  return { terms };
}

// ---------------------------------------------------------------------------
// Serializer
// ---------------------------------------------------------------------------

/** Quote a value for the query text, escaping `\` and `"`. */
export function quote(value: string): string {
  return `"${value.replace(/[\\"]/g, (c) => `\\${c}`)}"`;
}

/** Whether a field value can be written bare after `op:`. */
function bareFieldValue(value: string): boolean {
  return value !== '' && !/[\s()"]/u.test(value);
}

/** Whether a free-text word can be written bare (it must not lex as anything else). */
function bareWord(word: string): boolean {
  if (!bareFieldValue(word) || word.startsWith('-') || KEYWORDS.has(word)) return false;
  const colon = word.indexOf(':');
  return colon < 0 || !isOperator(word.slice(0, colon).toLowerCase());
}

export function serializeTerm(term: Term): string {
  if (term.kind === 'expr') return term.text.trim();
  const neg = term.negated ? '-' : '';
  if (term.kind === 'field') {
    if (term.value === '') return `${neg}${term.op}:${term.quoted ? '""' : ''}`;
    const value = term.quoted || !bareFieldValue(term.value) ? quote(term.value) : term.value;
    return `${neg}${term.op}:${value}`;
  }
  const words = term.value.split(/\s+/u).filter((w) => w !== '');
  if (words.length === 0) return '';
  if (!term.quoted && !term.negated && words.every(bareWord)) return words.join(' ');
  if (!term.quoted && term.negated && words.length === 1 && bareWord(words[0]!)) return `-${words[0]}`;
  return `${neg}${quote(term.value)}`;
}

/** The canonical query text: one space between terms. */
export function serializeQuery(query: SearchQuery): string {
  return query.terms.map(serializeTerm).filter((s) => s !== '').join(' ');
}

/** Rewrite input text into its canonical form. */
export function canonicalize(input: string): string {
  return serializeQuery(parseQuery(input));
}

// ---------------------------------------------------------------------------
// Editing helpers (pure; each returns a new query)
// ---------------------------------------------------------------------------

export function replaceTerm(query: SearchQuery, index: number, term: Term | null): SearchQuery {
  const terms = query.terms.slice();
  if (term === null) terms.splice(index, 1);
  else terms[index] = term;
  return { terms };
}

export function removeTerms(query: SearchQuery, indexes: readonly number[]): SearchQuery {
  const drop = new Set(indexes);
  return { terms: query.terms.filter((_, i) => !drop.has(i)) };
}

export function addTerm(query: SearchQuery, term: Term): SearchQuery {
  return { terms: [...query.terms, term] };
}

export function fieldTerm(op: Operator, value: string, negated = false): Term {
  return { kind: 'field', op, value: canonicalValue(op, value), negated, quoted: false };
}

/** Index of the first positive (not negated) `op:` term. */
export function findField(query: SearchQuery, op: Operator, accept: (value: string) => boolean = () => true): number {
  return query.terms.findIndex((t) => t.kind === 'field' && t.op === op && !t.negated && accept(t.value));
}

/** Set `op:value`, replacing the first positive term of that operator or adding one. */
export function setField(query: SearchQuery, op: Operator, value: string): SearchQuery {
  const at = findField(query, op);
  const term = fieldTerm(op, value);
  return at < 0 ? addTerm(query, term) : replaceTerm(query, at, term);
}

/** Set or clear one operator (`after`/`before`, `larger`/`smaller`, `in`). */
export function setOptionalField(query: SearchQuery, op: Operator, value: string | null): SearchQuery {
  const at = findField(query, op);
  if (value === null || value.trim() === '') return at < 0 ? query : replaceTerm(query, at, null);
  return setField(query, op, value.trim());
}

// ---------------------------------------------------------------------------
// The chip view
// ---------------------------------------------------------------------------

export type ChipKind =
  | 'from' | 'to' | 'cc' | 'subject' | 'body' | 'text' | 'filename' | 'tag'
  | 'attachment' | 'event' | 'unread' | 'flagged' | 'pinned'
  | 'date' | 'size' | 'scope' | 'other' | 'expr';

export interface Chip {
  /** Stable while the query keeps its shape: the kind and first term index. */
  readonly key: string;
  readonly kind: ChipKind;
  /** The terms this chip stands for (a date chip may own `after:` and `before:`). */
  readonly terms: readonly number[];
}

const TEXT_CHIPS: Partial<Record<Operator, ChipKind>> = {
  from: 'from', to: 'to', cc: 'cc', subject: 'subject', body: 'body', text: 'text', filename: 'filename', tag: 'tag',
};

/** The chips for a query, in term order. Pairs of range operators share one chip. */
export function chipsOf(query: SearchQuery): Chip[] {
  const chips: Chip[] = [];
  const claimed = new Set<number>();
  const pair = (kind: ChipKind, complement: Operator, i: number): void => {
    const other = query.terms.findIndex((t, j) => !claimed.has(j) && t.kind === 'field' && !t.negated && t.op === complement);
    const owned = other < 0 ? [i] : [i, other];
    owned.forEach((j) => claimed.add(j));
    chips.push({ key: `${kind}:${i}`, kind, terms: owned });
  };
  query.terms.forEach((term, i) => {
    if (claimed.has(i)) return;
    if (term.kind === 'expr') { claimed.add(i); chips.push({ key: `expr:${i}`, kind: 'expr', terms: [i] }); return; }
    if (term.kind === 'text') { claimed.add(i); chips.push({ key: `text:${i}`, kind: 'text', terms: [i] }); return; }
    const one = (kind: ChipKind): void => { claimed.add(i); chips.push({ key: `${kind}:${i}`, kind, terms: [i] }); };
    const text = TEXT_CHIPS[term.op];
    if (text) return one(text);
    switch (term.op) {
      case 'has':
        return one(term.value === 'attachment' ? 'attachment' : term.value === 'calendar' ? 'event' : 'other');
      case 'is':
        if (term.value === 'unread' || term.value === 'read') return one('unread');
        if (term.value === 'flagged' || term.value === 'unflagged') return one('flagged');
        if (term.value === 'pinned') return one('pinned');
        return one('other');
      case 'pinned':
        return one('pinned');
      case 'in':
        return term.negated ? one('other') : one('scope');
      case 'after': case 'before':
        if (term.negated) return one('other');
        claimed.add(i);
        return pair('date', term.op === 'after' ? 'before' : 'after', i);
      case 'larger': case 'smaller':
        if (term.negated) return one('other');
        claimed.add(i);
        return pair('size', term.op === 'larger' ? 'smaller' : 'larger', i);
      default:
        return one('other');
    }
  });
  return chips;
}

/** Flip a two-state chip: has:x ↔ -has:x, is:unread ↔ is:read, is:flagged ↔ is:unflagged. */
export function toggleTerm(term: Term): Term {
  if (term.kind !== 'field') return term.kind === 'text' ? { ...term, negated: !term.negated } : term;
  if (term.op === 'is') {
    const flip: Record<string, string> = { unread: 'read', read: 'unread', flagged: 'unflagged', unflagged: 'flagged' };
    const next = flip[term.value];
    if (next !== undefined) return { ...term, value: next, negated: false };
  }
  return { ...term, negated: !term.negated };
}

/** The value of the first `op` term among `indexes`, or ''. */
export function valueIn(query: SearchQuery, indexes: readonly number[], op: Operator): string {
  for (const i of indexes) {
    const t = query.terms[i];
    if (t?.kind === 'field' && t.op === op) return t.value;
  }
  return '';
}

/** Rewrite a range chip's two bounds (either may be empty to drop it). */
export function setRange(query: SearchQuery, chip: Chip, low: Operator, high: Operator, values: { low: string; high: string }): SearchQuery {
  const kept = removeTerms(query, chip.terms);
  const at = Math.min(...chip.terms);
  const added: Term[] = [];
  if (values.low.trim() !== '') added.push(fieldTerm(low, values.low.trim()));
  if (values.high.trim() !== '') added.push(fieldTerm(high, values.high.trim()));
  const terms = kept.terms.slice();
  terms.splice(Math.min(at, terms.length), 0, ...added);
  return { terms };
}

/** Valid date (YYYY-MM-DD) and size (`500k`, `1m`, `2g`, bytes) values. */
export const isDateValue = (v: string): boolean => /^\d{4}-\d{2}-\d{2}$/u.test(v.trim());
export const isSizeValue = (v: string): boolean => /^\d+\s*[kmg]?$/iu.test(v.trim());

// ---------------------------------------------------------------------------
// Folder scope (`in:`)
// ---------------------------------------------------------------------------

export interface ScopeMailbox { readonly id: string; readonly name: string; readonly role?: string | null }

/** How `in:` names a mailbox in the input: its name when no other mailbox shares
 *  it (case-insensitively), else its id. */
export function scopeValue(mailbox: ScopeMailbox, all: readonly ScopeMailbox[]): string {
  const same = all.filter((m) => m.name.toLowerCase() === mailbox.name.toLowerCase());
  return same.length === 1 ? mailbox.name : mailbox.id;
}

/** The mailbox an `in:` value names: an id, else a unique name, else a role. */
export function resolveScope(value: string, all: readonly ScopeMailbox[]): ScopeMailbox | null {
  const byId = all.find((m) => m.id === value);
  if (byId) return byId;
  const lower = value.toLowerCase();
  const byName = all.filter((m) => m.name.toLowerCase() === lower);
  if (byName.length === 1) return byName[0]!;
  const byRole = all.filter((m) => (m.role ?? '').toLowerCase() === lower);
  return byRole.length === 1 ? byRole[0]! : null;
}

/** The text the server runs: `in:` names resolved to mailbox ids (the engine's
 *  `in:` matches ids). Everything else is the input as typed. */
export function serverQueryText(input: string, all: readonly ScopeMailbox[]): string {
  const query = parseQuery(input);
  if (!query.terms.some((t) => t.kind === 'field' && t.op === 'in')) return input;
  return serializeQuery({
    terms: query.terms.map((t) => {
      if (t.kind !== 'field' || t.op !== 'in') return t;
      const box = resolveScope(t.value, all);
      return box && box.id !== t.value ? { ...t, value: box.id, quoted: false } : t;
    }),
  });
}

/** Set the folder scope: a mailbox, or null for all folders. */
export function setScope(query: SearchQuery, mailbox: ScopeMailbox | null, all: readonly ScopeMailbox[]): SearchQuery {
  return setOptionalField(query, 'in', mailbox === null ? null : scopeValue(mailbox, all));
}

// ---------------------------------------------------------------------------
// Offline (reduced cached-header search, offline/search.ts)
// ---------------------------------------------------------------------------

/** The reduced offline filter for a query, when every term has one; else the
 *  whole input as one substring, which is what offline search always did. */
export function offlineQueryOf(input: string): OfflineQuery {
  const query = parseQuery(input);
  const out: { -readonly [K in keyof OfflineQuery]: OfflineQuery[K] } = {};
  const texts: string[] = [];
  for (const term of query.terms) {
    if (term.kind === 'expr' || term.negated && !(term.kind === 'field' && term.op === 'has')) return { text: input };
    if (term.kind === 'text') { texts.push(term.value); continue; }
    const { op, value } = term;
    if ((op === 'from' || op === 'to' || op === 'subject' || op === 'body') && out[op] === undefined) out[op] = value;
    else if (op === 'text') texts.push(value);
    else if (op === 'has' && value === 'attachment') out.hasAttachment = !term.negated;
    else if (op === 'is' && (value === 'unread' || value === 'read')) out[value === 'unread' ? 'notKeyword' : 'hasKeyword'] = '$seen';
    else if (op === 'is' && value === 'flagged' && out.hasKeyword === undefined) out.hasKeyword = '$flagged';
    else return { text: input };
  }
  if (texts.length > 0) out.text = texts.join(' ');
  return out;
}
