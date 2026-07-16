// Web-side Sieve rendering, lint, highlight, and dry-run for the rules module
// (audit #1, SPEC §6.1/§10.5).
//
// The SERVER is authoritative: it generates Sieve from `mw_sieve::Rule` and
// uploads it via ManageSieve. This module renders the SAME constrained subset in
// TypeScript purely for the raw-editor preview, syntax highlighting, the lint
// surface, and the dry-run — mirroring `crates/mw-sieve/src/{codegen,eval,lint}.rs`
// for the `MailRule` (from/to/subject/thread · is/contains · move/tag/stop/
// archive/suppressNotify) surface. It never touches the wire.

import type { MailRule, MailRuleAction, MailRuleCondition } from '../../api/crypto-types.ts';

// ── codegen (MailRule[] → Sieve source) ──────────────────────────────────────

/** Quote a Sieve string literal (RFC 5228 §2.4.2): escape `\` and `"`, fold newlines. */
function quote(s: string): string {
  let out = '"';
  for (const ch of s) {
    if (ch === '\\') out += '\\\\';
    else if (ch === '"') out += '\\"';
    else if (ch === '\r' || ch === '\n') out += ' ';
    else out += ch;
  }
  return out + '"';
}

const matchTag = (op: MailRuleCondition['op']): string => (op === 'is' ? ':is' : ':contains');

function renderCondition(c: MailRuleCondition): { test: string | null; ext: string[] } {
  switch (c.type) {
    case 'from':
      return { test: `address ${matchTag(c.op)} "from" ${quote(c.value)}`, ext: [] };
    case 'to':
      return { test: `address ${matchTag(c.op)} "to" ${quote(c.value)}`, ext: [] };
    case 'subject':
      return { test: `header ${matchTag(c.op)} "subject" ${quote(c.value)}`, ext: [] };
    case 'thread':
      // No portable Sieve test for a thread id — engine-only (see runsAtFor).
      return { test: null, ext: [] };
    default:
      return { test: null, ext: [] };
  }
}

function renderAction(a: MailRuleAction): { line: string | null; ext: string[] } {
  switch (a.type) {
    case 'move':
      return { line: `fileinto ${quote(a.value ?? '')};`, ext: ['fileinto'] };
    case 'archive':
      return { line: `fileinto "Archive";`, ext: ['fileinto'] };
    case 'tag':
      return { line: `addflag ${quote(a.value ?? '')};`, ext: ['imap4flags'] };
    case 'stop':
      return { line: 'stop;', ext: [] };
    case 'suppressNotify':
      // Silence is an engine-side flag with no Sieve action (mirrors convert.rs).
      return { line: '# suppress-notify (engine-side)', ext: [] };
    default:
      return { line: null, ext: [] };
  }
}

const sanitizeComment = (s: string): string => s.replace(/[\r\n]/g, ' ');

/**
 * Render a rule set to Sieve source, mirroring `codegen::generate` for the
 * `MailRule` subset. Disabled rules are skipped; the required extensions are
 * collected into one leading `require`.
 */
export function rulesToSieve(rules: MailRule[]): string {
  const extensions = new Set<string>();
  const blocks: string[] = [];

  for (const rule of rules.filter((r) => r.enabled)) {
    let block = `# rule: ${sanitizeComment(rule.name)}\n`;
    const tests: string[] = [];
    for (const c of rule.conditions) {
      const { test, ext } = renderCondition(c);
      ext.forEach((e) => extensions.add(e));
      if (test !== null) tests.push(test);
    }
    const actionLines: string[] = [];
    for (const a of rule.actions) {
      const { line, ext } = renderAction(a);
      ext.forEach((e) => extensions.add(e));
      if (line !== null) actionLines.push(line);
    }

    if (tests.length === 0) {
      block += actionLines.map((l) => `${l}\n`).join('');
    } else {
      const guard = tests.length === 1 ? tests[0] : `${rule.matchAll ? 'allof' : 'anyof'} (${tests.join(', ')})`;
      block += `if ${guard} {\n${actionLines.map((l) => `    ${l}\n`).join('')}}\n`;
    }
    blocks.push(block);
  }

  let out = '';
  if (extensions.size > 0) {
    const quoted = [...extensions].sort().map(quote).join(', ');
    out += `require [${quoted}];\n`;
    if (blocks.length > 0) out += '\n';
  }
  out += blocks.join('\n');
  return out;
}

/**
 * Where a rule actually runs. A rule that can be fully expressed in Sieve
 * (all conditions/actions have a Sieve surface) can be uploaded to the mail
 * server; anything using `thread`/`suppressNotify` is engine-only. The server's
 * `runsAt` is authoritative once persisted — this mirrors the classification for
 * the pre-save indicator.
 */
export function runsAtFor(rule: Pick<MailRule, 'conditions' | 'actions'>): 'server-sieve' | 'engine' {
  const engineOnly =
    rule.conditions.some((c) => c.type === 'thread') || rule.actions.some((a) => a.type === 'suppressNotify');
  return engineOnly ? 'engine' : 'server-sieve';
}

// ── lint (mirrors lint.rs — bracket/quote balance + require coverage) ─────────

const REQUIRING_COMMANDS: Record<string, string> = {
  fileinto: 'fileinto',
  addflag: 'imap4flags',
  setflag: 'imap4flags',
  removeflag: 'imap4flags',
  hasflag: 'imap4flags',
  vacation: 'vacation',
  reject: 'reject',
};

/** Lint raw Sieve text, returning human-readable diagnostics (empty = clean). */
export function lintSieve(input: string): string[] {
  const diags: string[] = [];
  const toks = tokenizeSieve(input);

  // Bracket balance.
  const stack: string[] = [];
  const opener: Record<string, string> = { '}': '{', ')': '(', ']': '[' };
  for (const tok of toks) {
    if (tok.kind !== 'punct') continue;
    if (tok.text === '{' || tok.text === '(' || tok.text === '[') stack.push(tok.text);
    else {
      const want = opener[tok.text];
      if (want !== undefined && stack.pop() !== want) {
        diags.push(`unbalanced \`${tok.text}\` — no matching \`${want}\``);
      }
    }
  }
  for (const open of stack) {
    const close = open === '{' ? '}' : open === '(' ? ')' : ']';
    diags.push(`unclosed \`${open}\` — missing \`${close}\``);
  }

  // Unterminated string (an odd count of unescaped quotes).
  if (hasUnterminatedString(input)) diags.push('unterminated string literal (unescaped `"`)');

  // Required-extension coverage.
  const required = collectRequired(toks);
  for (const t of toks) {
    if (t.kind !== 'keyword' && t.kind !== 'ident') continue;
    const cap = REQUIRING_COMMANDS[t.text];
    if (cap !== undefined && !t.text.startsWith(':') && !required.has(cap)) {
      diags.push(`\`${t.text}\` used but extension \`${cap}\` is not in \`require\``);
    }
  }
  return [...new Set(diags)];
}

function collectRequired(toks: Token[]): Set<string> {
  const set = new Set<string>();
  let inRequire = false;
  for (const tok of toks) {
    if (tok.kind === 'keyword' && tok.text === 'require') {
      inRequire = true;
    } else if (inRequire) {
      if (tok.kind === 'punct' && tok.text === ';') inRequire = false;
      else if (tok.kind === 'string') set.add(unquote(tok.text));
    }
  }
  return set;
}

function hasUnterminatedString(input: string): boolean {
  let i = 0;
  const n = input.length;
  while (i < n) {
    const ch = input.charAt(i);
    if (ch === '#') {
      while (i < n && input.charAt(i) !== '\n') i++;
    } else if (ch === '"') {
      i++;
      let closed = false;
      while (i < n) {
        if (input.charAt(i) === '\\' && i + 1 < n) i += 2;
        else if (input.charAt(i) === '"') {
          closed = true;
          i++;
          break;
        } else i++;
      }
      if (!closed) return true;
    } else i++;
  }
  return false;
}

// ── tokenizer / syntax highlighting ──────────────────────────────────────────

export type TokenKind = 'keyword' | 'ident' | 'tag' | 'string' | 'number' | 'comment' | 'punct' | 'text';

export interface Token {
  kind: TokenKind;
  text: string;
}

const KEYWORDS = new Set([
  'require',
  'if',
  'elsif',
  'else',
  'allof',
  'anyof',
  'address',
  'header',
  'body',
  'size',
  'hasflag',
  'fileinto',
  'addflag',
  'setflag',
  'removeflag',
  'redirect',
  'vacation',
  'keep',
  'discard',
  'stop',
  'reject',
]);

const unquote = (lit: string): string => lit.slice(1, -1).replace(/\\(.)/g, '$1');

/**
 * Tokenize Sieve text into typed spans for highlighting. Whitespace becomes
 * `text` tokens so the highlighter can reassemble the source byte-for-byte.
 * Never throws — malformed input just tokenizes best-effort (the lint surface
 * reports the problems).
 */
export function tokenizeSieve(input: string): Token[] {
  const toks: Token[] = [];
  let i = 0;
  const n = input.length;
  const isDelim = (c: string): boolean => /[\s"{}()[\];,#]/.test(c);

  while (i < n) {
    const ch = input.charAt(i);
    if (/\s/.test(ch)) {
      let j = i;
      while (j < n && /\s/.test(input.charAt(j))) j++;
      toks.push({ kind: 'text', text: input.slice(i, j) });
      i = j;
    } else if (ch === '#') {
      let j = i;
      while (j < n && input.charAt(j) !== '\n') j++;
      toks.push({ kind: 'comment', text: input.slice(i, j) });
      i = j;
    } else if (ch === '"') {
      let j = i + 1;
      while (j < n) {
        if (input.charAt(j) === '\\' && j + 1 < n) j += 2;
        else if (input.charAt(j) === '"') {
          j++;
          break;
        } else j++;
      }
      toks.push({ kind: 'string', text: input.slice(i, j) });
      i = j;
    } else if ('{}()[];,'.includes(ch)) {
      toks.push({ kind: 'punct', text: ch });
      i++;
    } else {
      let j = i;
      while (j < n && !isDelim(input.charAt(j))) j++;
      const word = input.slice(i, j);
      let kind: TokenKind;
      if (word.startsWith(':')) kind = 'tag';
      else if (/^\d+$/.test(word)) kind = 'number';
      else if (KEYWORDS.has(word)) kind = 'keyword';
      else kind = 'ident';
      toks.push({ kind, text: word });
      i = j;
    }
  }
  return toks;
}

// ── dry-run evaluator (mirrors eval.rs for the MailRule subset) ───────────────

/** A sample message the dry-run matches rules against. */
export interface SampleMessage {
  from: string;
  to: string;
  subject: string;
}

/** The dry-run outcome for one rule against the sample. */
export interface DryRunResult {
  ruleId: string;
  ruleName: string;
  matched: boolean;
  /** Human-readable action summaries that would run (empty when not matched). */
  actions: string[];
  /** `true` once a prior matched rule ran `stop` — this rule is not reached. */
  shortCircuited: boolean;
}

function fieldValue(msg: SampleMessage, type: MailRuleCondition['type']): string {
  switch (type) {
    case 'from':
      return msg.from;
    case 'to':
      return msg.to;
    case 'subject':
      return msg.subject;
    default:
      return ''; // `thread` has no sample field.
  }
}

function conditionMatches(c: MailRuleCondition, msg: SampleMessage): boolean {
  if (c.type === 'thread') return false; // not modelled in the dry-run sample.
  const hay = fieldValue(msg, c.type).toLowerCase();
  const needle = c.value.toLowerCase();
  return c.op === 'is' ? hay === needle : hay.includes(needle);
}

function ruleMatches(rule: MailRule, msg: SampleMessage): boolean {
  if (rule.conditions.length === 0) return true; // unconditional rule always fires.
  const results = rule.conditions.map((c) => conditionMatches(c, msg));
  return rule.matchAll ? results.every(Boolean) : results.some(Boolean);
}

function actionSummary(a: MailRuleAction): string {
  switch (a.type) {
    case 'move':
      return `move to ${a.value ?? ''}`;
    case 'archive':
      return 'archive';
    case 'tag':
      return `tag ${a.value ?? ''}`;
    case 'stop':
      return 'stop processing';
    case 'suppressNotify':
      return 'suppress notification';
    default:
      return a.type;
  }
}

/** Evaluate the enabled rules against a sample message, in order, honouring `stop`. */
export function dryRun(rules: MailRule[], msg: SampleMessage): DryRunResult[] {
  const out: DryRunResult[] = [];
  let stopped = false;
  for (const rule of rules) {
    if (!rule.enabled) continue;
    if (stopped) {
      out.push({ ruleId: rule.id, ruleName: rule.name, matched: false, actions: [], shortCircuited: true });
      continue;
    }
    const matched = ruleMatches(rule, msg);
    const actions = matched ? rule.actions.map(actionSummary) : [];
    out.push({ ruleId: rule.id, ruleName: rule.name, matched, actions, shortCircuited: false });
    if (matched && rule.actions.some((a) => a.type === 'stop')) stopped = true;
  }
  return out;
}
