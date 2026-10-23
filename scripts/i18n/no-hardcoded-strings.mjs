// no-hardcoded-strings gate (SPEC §24; rewritten for t32-admin §2.9).
//
// Finds user-visible strings in the web app that do not go through t(), so they
// would ship untranslated. Two scans run:
//
//   1. AST scan (lib/hardcoded.mjs, TypeScript compiler API) over
//      apps/web/src/**/*.{ts,tsx}: JSX text including multi-line text, string
//      values of user-visible JSX attributes, `label:` / `description:` /
//      `title:`-style properties in .ts tables, alert/confirm/prompt/Notification
//      text, and `toLocale*String()` / `Intl.*Format(undefined)` calls that
//      format in the browser locale instead of the app locale (use
//      src/i18n/format.ts).
//   2. Line scan (the pre-t32 heuristic): single-line JSX text and quoted
//      aria-label / placeholder / title / alt values in .tsx files.
//
// Modes:
//   default    the line scan FAILS on a new literal, exactly as before t32; the
//              AST findings are printed as warnings and do not fail. This is the
//              report-only phase while t32 lanes still edit the web app.
//   --strict   (or MW_I18N_STRICT=1) every AST finding not allowlisted FAILS.
//              t32 stage A4 makes this the default.
//
// Accepting a literal: either a `// i18n-ignore: <reason>` comment on the same
// line or the line above, or an entry `{ "text", "reason" }` in
// hardcoded-allowlist.json (matched on the exact text, any file). Every
// allowlist entry needs a reason.
//
// The AST scan needs the `typescript` package from apps/web/node_modules (it is
// a devDependency). Without it the default mode prints a warning and runs only
// the line scan; --strict fails.
//
// Usage: node scripts/i18n/no-hardcoded-strings.mjs [--strict] [--src <dir>] [--list]
//   --src <dir>  scan another directory instead of apps/web/src (seeded tests)
//   --list       print every AST finding, not only the first 100

import { readdir, readFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { analyzeSource, hasWord } from './lib/hardcoded.mjs';

const repoRoot = fileURLToPath(new URL('../../', import.meta.url));
const argv = process.argv.slice(2);
const argValue = (flag) => {
  const i = argv.indexOf(flag);
  return i >= 0 ? argv[i + 1] : undefined;
};
const STRICT = argv.includes('--strict') || process.env.MW_I18N_STRICT === '1';
const LIST = argv.includes('--list');
const srcDir = resolve(argValue('--src') ?? join(repoRoot, 'apps/web/src'));
const allowlistPath = join(repoRoot, 'scripts/i18n/hardcoded-allowlist.json');

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

/** Files that hold no shipped UI copy: tests, test helpers, mock backends, styles, declarations, the i18n runtime. */
function outOfScope(file) {
  const rel = relative(srcDir, file).split(sep).join('/');
  return (
    /\.test\.tsx?$/.test(rel) ||
    /\.css\.ts$/.test(rel) ||
    /\.d\.ts$/.test(rel) ||
    rel.startsWith('i18n/') ||
    rel.startsWith('test/') ||
    rel.includes('/__fixtures__/') ||
    /(^|\/)mock\.tsx?$/.test(rel)
  );
}

// --- allowlist --------------------------------------------------------------
let allowFile = { entries: [] };
try {
  allowFile = JSON.parse(await readFile(allowlistPath, 'utf8'));
} catch (e) {
  console.error(`no-hardcoded-strings: cannot read ${relative(repoRoot, allowlistPath)}: ${e.message}`);
  process.exit(1);
}
const entries = allowFile.entries ?? [];
const missingReason = entries.filter((e) => typeof e.reason !== 'string' || e.reason.trim().length === 0);
if (missingReason.length > 0) {
  console.error('no-hardcoded-strings: FAIL — allowlist entries without a reason:');
  for (const e of missingReason) console.error(`  • "${e.text}"`);
  process.exit(1);
}
const allowed = new Set(entries.map((e) => e.text));
const usedAllow = new Set();
const isAllowed = (text) => {
  if (allowed.has(text)) {
    usedAllow.add(text);
    return true;
  }
  return false;
};

const files = await walk(srcDir, (n) => n.endsWith('.ts') || n.endsWith('.tsx'));
const inScope = files.filter((f) => !outOfScope(f));
const sources = new Map();
for (const f of inScope) sources.set(f, await readFile(f, 'utf8'));
const rel = (f) => relative(repoRoot, f).split(sep).join('/');

// --- 2. line scan (pre-t32 heuristic, still blocking in the default mode) ----
function isIgnorableText(s) {
  const t = s.trim();
  if (t.length === 0) return true;
  if (!hasWord(t)) return true;
  if (/^\{.*\}$/.test(t)) return true;
  if (/^&[a-zA-Z]+;$/.test(t)) return true;
  // A `>text<` match on a code line is often a generic or an expression.
  if (/[=|&;()?`]|=>|[A-Za-z]\.[A-Za-z]/.test(t)) return true;
  return false;
}

function lineCandidates(line) {
  const found = [];
  const attrRe = /\b(aria-label|placeholder|title|alt|aria-description|aria-roledescription)\s*=\s*(['"])([^'"]*)\2/g;
  let m;
  while ((m = attrRe.exec(line)) !== null) {
    if (hasWord(m[3])) found.push({ kind: m[1], text: m[3].trim() });
  }
  const textRe = /(?<!=)>([^<>{}]+)</g;
  while ((m = textRe.exec(line)) !== null) {
    if (!isIgnorableText(m[1])) found.push({ kind: 'jsx-text', text: m[1].trim() });
  }
  return found;
}

const lineFindings = [];
for (const [f, src] of sources) {
  if (!f.endsWith('.tsx')) continue;
  src.split(/\r?\n/).forEach((line, i) => {
    // Comments are not UI; i18n-ignore accepts the site.
    if (/^\s*(\/\/|\/\*|\*)/.test(line) || /\/\/\s*i18n-ignore:\s*\S/.test(line)) return;
    for (const c of lineCandidates(line)) {
      if (!isAllowed(c.text)) lineFindings.push({ file: rel(f), line: i + 1, kind: c.kind, text: c.text });
    }
  });
}

// --- 1. AST scan --------------------------------------------------------------
let ts = null;
try {
  ts = createRequire(join(repoRoot, 'apps/web/package.json'))('typescript');
} catch {
  ts = null;
}

const astFindings = [];
if (ts !== null) {
  for (const [f, src] of sources) {
    for (const finding of analyzeSource(ts, f, src)) {
      if (!isAllowed(finding.text)) astFindings.push({ file: rel(f), ...finding });
    }
  }
}

// --- report -----------------------------------------------------------------
const byKind = new Map();
for (const f of astFindings) {
  const k = f.kind.split(':')[0];
  byKind.set(k, (byKind.get(k) ?? 0) + 1);
}
const fmt = (f) => `  • [${f.kind}${f.multiline ? ', multi-line' : ''}] "${f.text}"  (${f.file}:${f.line})`;

console.log(
  `no-hardcoded-strings: ${inScope.length} files in scope under ${relative(repoRoot, srcDir) || srcDir}; ` +
    `${entries.length} allowlist entries; mode ${STRICT ? 'strict' : 'default (AST findings are warnings)'}.`,
);

if (ts === null) {
  const msg =
    'no-hardcoded-strings: the AST scan did not run: `typescript` is not installed in apps/web/node_modules ' +
    '(run `pnpm install` in apps/web).';
  if (STRICT) {
    console.error(`${msg}\nno-hardcoded-strings: FAIL — strict mode needs the AST scan.`);
    process.exit(1);
  }
  console.warn(`WARN ${msg} Only the line scan ran.`);
} else {
  const summary = [...byKind].map(([k, n]) => `${k} ${n}`).join(', ') || 'none';
  console.log(`no-hardcoded-strings: AST scan found ${astFindings.length} unallowlisted literal(s): ${summary}.`);
  if (astFindings.length > 0) {
    const shown = LIST ? astFindings : astFindings.slice(0, 100);
    const out = STRICT ? console.error : console.warn;
    out(STRICT ? '\nAST findings (FAIL in strict mode):' : '\nWARN AST findings (report-only until t32 stage A4):');
    for (const f of shown) out(fmt(f));
    if (shown.length < astFindings.length) out(`  … and ${astFindings.length - shown.length} more (--list prints all)`);
  }
}

const stale = entries.filter((e) => !usedAllow.has(e.text));
if (stale.length > 0 && argValue('--src') === undefined) {
  console.log(`\nno-hardcoded-strings: ${stale.length} allowlist entr${stale.length === 1 ? 'y is' : 'ies are'} no longer used:`);
  for (const e of stale) console.log(`  • "${e.text}"`);
}

if (lineFindings.length > 0) {
  console.error('\nno-hardcoded-strings: FAIL — hard-coded user-visible literal(s) (wrap in t() or allowlist with a reason):');
  for (const v of lineFindings.slice(0, 100)) console.error(`  • [${v.kind}] "${v.text}"  (${v.file}:${v.line})`);
  if (lineFindings.length > 100) console.error(`  … and ${lineFindings.length - 100} more`);
  process.exit(1);
}
if (STRICT && astFindings.length > 0) {
  console.error(`\nno-hardcoded-strings: FAIL — ${astFindings.length} literal(s) not routed through t() (strict mode).`);
  process.exit(1);
}
console.log(
  STRICT
    ? '\nno-hardcoded-strings: OK — no hard-coded user-visible literals (strict).'
    : '\nno-hardcoded-strings: OK — the line scan found no new literal; AST findings above are warnings.',
);
