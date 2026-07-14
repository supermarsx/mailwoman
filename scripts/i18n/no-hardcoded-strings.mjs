// no-hardcoded-strings gate (1.0 hardening, SPEC §24 / ROADMAP-1.0 L21).
//
// A heuristic scan that flags NEW user-facing string literals in the web `.tsx`
// source that are NOT routed through `t()` — i.e. text that would ship
// untranslated. e1–e4 wrapped every user-facing literal in `t()`; this gate keeps
// it that way: a newly-introduced bare JSX text node or a hardcoded aria-label /
// placeholder / title / alt FAILS the build.
//
// It is deliberately a lint-style heuristic, not a compiler: JSX is matched by
// pattern, and a curated allowlist (`hardcoded-allowlist.json`) carries the
// UNAVOIDABLE literals — brand names, symbols/glyphs, format tokens, technical
// acronyms, and text asserted verbatim by the security/honesty model. Anything
// flagged that is NOT in the allowlist is a new literal → fix it (wrap in `t()`)
// or, if genuinely untranslatable, add it to the allowlist with a reason.
//
// Scope: `apps/web/src/**/*.tsx` (JSX only), excluding tests and generated CSS.
//
// Usage: `node scripts/i18n/no-hardcoded-strings.mjs` (exit non-zero on a new literal).
//        `node scripts/i18n/no-hardcoded-strings.mjs --update` rewrites the
//        allowlist from the current findings (baseline maintenance — review the diff).

import { readdir, readFile, writeFile } from 'node:fs/promises';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = fileURLToPath(new URL('../../', import.meta.url));
const srcDir = join(repoRoot, 'apps/web/src');
const allowlistPath = join(repoRoot, 'scripts/i18n/hardcoded-allowlist.json');
const UPDATE = process.argv.includes('--update');

async function walk(dir, filter, out = []) {
  let entries;
  try {
    entries = await readdir(dir, { withFileTypes: true });
  } catch {
    return out;
  }
  for (const e of entries) {
    const p = join(dir, e.name);
    if (e.isDirectory()) {
      if (e.name === 'node_modules' || e.name === 'dist') continue;
      await walk(p, filter, out);
    } else if (filter(e.name)) {
      out.push(p);
    }
  }
  return out;
}

// A candidate is "translatable text" if it has at least one run of 2+ letters —
// i.e. a real word, not a symbol/glyph, number, single letter, or punctuation.
const hasWord = (s) => /[A-Za-zÀ-ɏ]{2,}/.test(s);

// Text nodes we never flag even before the allowlist: pure expressions, entities,
// and non-word decoration. These are structural, not user copy.
function isIgnorableText(s) {
  const t = s.trim();
  if (t.length === 0) return true;
  if (!hasWord(t)) return true; // symbols / glyphs / numbers / single letters
  if (/^\{.*\}$/.test(t)) return true; // a lone {expression}
  if (/^&[a-zA-Z]+;$/.test(t)) return true; // an HTML entity like &nbsp;
  // Code, not copy: a `.tsx` line-level `>text<` match frequently catches TS
  // generics / expressions (`=> Promise<void>` → "Promise", `x >= 1 && y.z` →
  // "= 1 && y.z"). Reject anything carrying code punctuation or property access —
  // genuine UI copy has none of these (a trailing sentence period is fine, an
  // INTERNAL letter.letter is a member access).
  if (/[=|&;()?`]|=>|[A-Za-z]\.[A-Za-z]/.test(t)) return true;
  return false;
}

/**
 * Extract candidate user-facing literals from one line of `.tsx`. Two kinds:
 *   - jsx-text: text sitting directly between JSX tags (`>Hello<`).
 *   - <attr>:   a hardcoded string-literal value for a user-facing attribute
 *               (aria-label / placeholder / title / alt). The `={...}` expression
 *               form is NOT flagged (that is dynamic / already `t()`-routed).
 * Both are heuristics operating per-line; multi-line JSX text is only partially
 * seen, which is acceptable for a "no NEW literal" ratchet.
 */
function candidatesInLine(line) {
  const found = [];

  // user-facing string attributes: aria-label="Foo"  title='Bar'
  const attrRe = /\b(aria-label|placeholder|title|alt|aria-description|aria-roledescription)\s*=\s*(['"])([^'"]*)\2/g;
  let m;
  while ((m = attrRe.exec(line)) !== null) {
    const text = m[3];
    if (hasWord(text)) found.push({ kind: m[1], text: text.trim() });
  }

  // JSX text nodes: >Some words< — reject fragments containing { } < (expressions
  // / nested tags) so we only catch genuine static copy. The `(?<!=)` lookbehind
  // drops TS arrow-return types (`=> Promise<T>` is not a JSX `>text<`).
  const textRe = /(?<!=)>([^<>{}]+)</g;
  while ((m = textRe.exec(line)) !== null) {
    const text = m[1];
    if (!isIgnorableText(text)) found.push({ kind: 'jsx-text', text: text.trim() });
  }

  return found;
}

// --- scan -------------------------------------------------------------------
const files = await walk(
  srcDir,
  (n) => n.endsWith('.tsx') && !/\.test\.tsx$/.test(n),
);

const findings = []; // { file, line, kind, text }
for (const f of files) {
  if (f.includes(join('src', 'i18n'))) continue; // i18n runtime, not UI copy
  const src = await readFile(f, 'utf8');
  src.split(/\r?\n/).forEach((line, i) => {
    for (const c of candidatesInLine(line)) {
      findings.push({ file: relative(repoRoot, f), line: i + 1, kind: c.kind, text: c.text });
    }
  });
}

// --- allowlist --------------------------------------------------------------
let allow = { strings: [] };
try {
  allow = JSON.parse(await readFile(allowlistPath, 'utf8'));
} catch {
  /* first run — no allowlist yet */
}
const allowed = new Set(allow.strings ?? []);

const violations = findings.filter((f) => !allowed.has(f.text));

if (UPDATE) {
  const uniq = [...new Set(findings.map((f) => f.text))].sort((a, b) => a.localeCompare(b));
  const out = {
    _comment:
      'Allowlist for no-hardcoded-strings.mjs — UNAVOIDABLE literals (brand names, ' +
      'glyphs/symbols, format tokens, technical acronyms, verbatim security/honesty ' +
      'copy). Everything else must go through t(). Regenerate with --update after review.',
    strings: uniq,
  };
  await writeFile(allowlistPath, JSON.stringify(out, null, 2) + '\n', 'utf8');
  console.log(`no-hardcoded-strings: wrote allowlist with ${uniq.length} entries -> ${relative(repoRoot, allowlistPath)}`);
  process.exit(0);
}

console.log(
  `no-hardcoded-strings: scanned ${files.length} .tsx files; ${findings.length} candidate literal(s), ` +
    `${allowed.size} allowlisted, ${violations.length} unallowlisted.`,
);

if (violations.length > 0) {
  console.error('\nno-hardcoded-strings: FAIL — new hardcoded user-facing literal(s) (wrap in t() or allowlist):');
  for (const v of violations.slice(0, 100)) {
    console.error(`  • [${v.kind}] "${v.text}"  (${v.file}:${v.line})`);
  }
  if (violations.length > 100) console.error(`  … and ${violations.length - 100} more`);
  process.exit(1);
}

console.log('no-hardcoded-strings: OK — no new hardcoded user-facing literals.');
