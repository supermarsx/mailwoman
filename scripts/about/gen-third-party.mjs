#!/usr/bin/env node
// Third-party list for the About screens (t32-e75; plan t32-cfg.md §4.11, DC9 with
// licence texts in scope).
//
//   node scripts/about/gen-third-party.mjs           write the three outputs
//   node scripts/about/gen-third-party.mjs --check   exit 1 when they are stale
//
// Inputs (all read-only, offline):
//   - `cargo metadata --format-version 1 --locked --offline` for the workspace and for
//     the standalone crates/mw-media-wasm (its module is embedded in mw-render). Only
//     crates reachable through normal or build dependencies from a shipped root are
//     listed; dev-dependencies and the first-party workspace crates are not.
//   - `pnpm licenses list --json --prod`, run in apps/web, plus each package's
//     package.json for its repository link. Packages pinned to one OS or CPU (the
//     prebuilt native binaries of optional dependencies) are left out: which of them
//     are installed depends on the host, and none of them is in the browser bundle.
//   - Licence files from the unpacked cargo registry sources and from
//     apps/web/node_modules. A package without one gets the standard text of its
//     licence from standard-texts.mjs, marked as such.
//
// Outputs (committed; `--check` compares them as parsed JSON):
//   apps/web/src/about/third-party.web.json         npm packages, fonts, icons
//   crates/mw-server/src/about/third-party.crates.json  Rust crates
//   apps/web/src/about/texts/<0-f>.json             each licence text once, by id,
//                                                   in 16 shards by the id's first hex
//                                                   digit, so opening one text loads ~85 KB
//
// No dependency beyond node itself.

import { execFileSync, execSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, readdirSync, statSync, writeFileSync, mkdirSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { STANDARD_TEXTS } from './standard-texts.mjs';

export const SCHEMA = 'mailwoman-third-party/1';

export const OUTPUTS = {
  web: 'apps/web/src/about/third-party.web.json',
  crates: 'crates/mw-server/src/about/third-party.crates.json',
  textsDir: 'apps/web/src/about/texts',
};

export const SHARDS = '0123456789abcdef'.split('');

/** Every output file as [path relative to the root, document]. */
export function outputFiles(out) {
  const shards = Object.fromEntries(SHARDS.map((h) => [h, {}]));
  for (const [id, text] of Object.entries(out.texts)) shards[id[0]][id] = text;
  return [
    [OUTPUTS.web, out.web],
    [OUTPUTS.crates, out.crates],
    ...SHARDS.map((h) => [`${OUTPUTS.textsDir}/${h}.json`, shards[h]]),
  ];
}

export const LOCKFILES = {
  cargo: ['Cargo.lock', 'crates/mw-media-wasm/Cargo.lock'],
  web: ['apps/web/pnpm-lock.yaml'],
};

// Shipped Rust roots, by workspace package name, and what each one ends up in.
//   server   the `mailwoman` server binary and its `mw-render` worker
//   web      crates compiled to wasm by scripts/build-wasm.sh into the web bundle
//   plugins  the first-party wasm components in plugins/dist
//   desktop / mobile  the Tauri shells (listed, tagged, so a reader can tell them apart)
export const ROOTS = {
  'mw-server': 'server',
  'mw-render': 'server',
  'mw-crypto': 'web',
  'mw-sanitize': 'web',
  'bridge-ews': 'plugins',
  'bridge-gmail': 'plugins',
  'bridge-graph': 'plugins',
  languagetool: 'plugins',
  'masked-email': 'plugins',
  'nextcloud-plugin': 'plugins',
  'spam-rspamd': 'plugins',
  'spam-spamassassin': 'plugins',
  'mailwoman-desktop': 'desktop',
  'mailwoman-mobile': 'mobile',
  // standalone workspace, embedded in mw-render with include_bytes!
  'mw-media-wasm': 'server',
};

export const ARTIFACT_ORDER = ['server', 'web', 'plugins', 'desktop', 'mobile'];

// Assets that are not packages: the bundled UI typeface, the fonts an operator may
// pull with `mailwoman fonts pull`, and the icons.
export const ASSETS = [
  {
    kind: 'font',
    name: 'Jost',
    version: '5.3.0',
    licence: 'OFL-1.1',
    repository: 'https://github.com/indestructible-type/Jost',
    homepage: 'https://fontsource.org/fonts/jost',
    files: ['apps/web/src/assets/fonts/jost/OFL.txt'],
    note: 'bundled',
  },
  {
    kind: 'font',
    name: 'Inter',
    licence: 'OFL-1.1',
    repository: 'https://github.com/rsms/inter',
    homepage: 'https://fonts.google.com/specimen/Inter',
    note: 'fonts-pull',
  },
  {
    kind: 'font',
    name: 'Newsreader',
    licence: 'OFL-1.1',
    repository: 'https://github.com/productiontype/Newsreader',
    homepage: 'https://fonts.google.com/specimen/Newsreader',
    note: 'fonts-pull',
  },
  {
    kind: 'font',
    name: 'JetBrains Mono',
    licence: 'OFL-1.1',
    repository: 'https://github.com/JetBrains/JetBrainsMono',
    homepage: 'https://fonts.google.com/specimen/JetBrains+Mono',
    note: 'fonts-pull',
  },
  {
    kind: 'icons',
    name: 'Mailwoman icons',
    licence: 'MIT',
    repository: 'https://github.com/supermarsx/mailwoman',
    files: ['license.md'],
    note: 'first-party',
  },
];

// SPDX licence and exception ids this generator knows. Anything else is flagged so a
// person looks at it before it ships.
export const KNOWN_LICENCES = new Set([
  '0BSD', 'Apache-2.0', 'BSD-1-Clause', 'BSD-2-Clause', 'BSD-3-Clause', 'BSL-1.0',
  'BlueOak-1.0.0', 'CC-BY-4.0', 'CC0-1.0', 'CDLA-Permissive-2.0', 'ISC', 'MIT', 'MIT-0',
  'MPL-2.0', 'OFL-1.1', 'Python-2.0', 'Unicode-3.0', 'Unicode-DFS-2016', 'Unlicense',
  'Zlib', 'bzip2-1.0.6', 'zlib-acknowledgement',
  // Only ever an alternative next to MIT/Apache-2.0 here (r-efi); the policy check
  // below still refuses it on its own.
  'LGPL-2.1-or-later',
]);
export const KNOWN_EXCEPTIONS = new Set(['LLVM-exception']);

const LICENCE_FILE = /^(licen[cs]e|copying|notice|unlicense|copyright)([-._][\w.-]*)?$/i;
const MAX_TEXT_BYTES = 256 * 1024;

// ---------------------------------------------------------------------------
// Pure helpers (exported for gen-third-party.test.mjs)
// ---------------------------------------------------------------------------

/** Tokenise and parse an SPDX expression. Returns null when it does not parse. */
export function parseSpdx(expr) {
  if (typeof expr !== 'string' || !expr.trim()) return null;
  // Cargo still accepts the old `MIT/Apache-2.0` form; it means OR.
  const src = expr.replace(/\s*\/\s*/g, ' OR ');
  const tokens = src.match(/\(|\)|[^\s()]+/g) ?? [];
  let i = 0;
  const peek = () => tokens[i];
  const primary = () => {
    const t = tokens[i++];
    if (t === undefined) throw new Error('end');
    if (t === '(') {
      const e = orExpr();
      if (tokens[i++] !== ')') throw new Error(')');
      return e;
    }
    if (/^(AND|OR|WITH|\))$/i.test(t)) throw new Error(t);
    let node = { id: t.replace(/\+$/, '') };
    if (peek() && peek().toUpperCase() === 'WITH') {
      i++;
      const ex = tokens[i++];
      if (!ex || ex === '(' || ex === ')') throw new Error('with');
      node = { id: node.id, exception: ex };
    }
    return node;
  };
  const andExpr = () => {
    const parts = [primary()];
    while (peek() && peek().toUpperCase() === 'AND') { i++; parts.push(primary()); }
    return parts.length === 1 ? parts[0] : { and: parts };
  };
  const orExpr = () => {
    const parts = [andExpr()];
    while (peek() && peek().toUpperCase() === 'OR') { i++; parts.push(andExpr()); }
    return parts.length === 1 ? parts[0] : { or: parts };
  };
  try {
    const tree = orExpr();
    return i === tokens.length ? tree : null;
  } catch {
    return null;
  }
}

/** Display form: the old slash form rewritten, one space around operators. */
export function normaliseLicence(expr) {
  if (typeof expr !== 'string') return '';
  const s = expr.trim().replace(/\s*\/\s*/g, ' OR ').replace(/\s+/g, ' ');
  // pnpm prints "(MIT OR Apache-2.0)"; drop one outer pair that spans the whole thing.
  if (s.startsWith('(') && s.endsWith(')') && parseSpdx(s.slice(1, -1))) {
    let depth = 0;
    let spans = true;
    for (let k = 0; k < s.length; k++) {
      if (s[k] === '(') depth++;
      else if (s[k] === ')') depth--;
      if (depth === 0 && k < s.length - 1) { spans = false; break; }
    }
    if (spans) return s.slice(1, -1);
  }
  return s;
}

/** Every licence id named in an expression (exceptions not included). */
export function licenceIds(tree) {
  if (!tree) return [];
  if (tree.id) return [tree.id];
  return [...new Set((tree.or ?? tree.and).flatMap(licenceIds))];
}

/** Ids and exceptions the generator does not know. Empty for a clean expression. */
export function unknownIds(expr) {
  const tree = parseSpdx(expr);
  if (!tree) return [String(expr ?? '(none)')];
  const out = [];
  const walk = (n) => {
    if (n.id) {
      if (!KNOWN_LICENCES.has(n.id)) out.push(n.id);
      if (n.exception && !KNOWN_EXCEPTIONS.has(n.exception)) out.push(n.exception);
    } else (n.or ?? n.and).forEach(walk);
  };
  walk(tree);
  return out;
}

/** Whether an expression is satisfiable under an allow-list (cargo-deny semantics). */
export function allowedBy(expr, allow) {
  const tree = parseSpdx(expr);
  if (!tree) return false;
  const ok = (n) => {
    if (n.id) return allow.has(n.exception ? `${n.id} WITH ${n.exception}` : n.id);
    if (n.or) return n.or.some(ok);
    return n.and.every(ok);
  };
  return ok(tree);
}

/**
 * An https URL in canonical form, or null. Accepts the usual package-manager
 * spellings of a repository (git+https://, github:owner/repo, owner/repo) and
 * rewrites them; anything else (http, git://, ssh, file) is refused.
 */
export function httpsLink(raw) {
  if (typeof raw !== 'string') return null;
  let s = raw.trim();
  if (!s) return null;
  const short = s.match(/^(github|gitlab|bitbucket):([\w.-]+\/[\w.-]+)$/i);
  if (short) {
    const host = { github: 'github.com', gitlab: 'gitlab.com', bitbucket: 'bitbucket.org' }[short[1].toLowerCase()];
    s = `https://${host}/${short[2]}`;
  } else if (/^[\w.-]+\/[\w.-]+$/.test(s)) {
    s = `https://github.com/${s}`;
  }
  s = s.replace(/^git\+https:/i, 'https:');
  // A plain-text or git:// link to one of the public forges names the same project
  // page over https; anywhere else the scheme is refused below.
  s = s.replace(/^(?:git\+)?(?:git|http|ssh):\/\/(?:git@)?(github\.com|gitlab\.com|bitbucket\.org|codeberg\.org)\//i, 'https://$1/');
  let url;
  try {
    url = new URL(s);
  } catch {
    return null;
  }
  if (url.protocol !== 'https:' || url.username || url.password) return null;
  if (/^(github\.com|gitlab\.com|bitbucket\.org|codeberg\.org)$/i.test(url.hostname)) {
    url.pathname = url.pathname.replace(/\.git$/i, '');
  }
  return url.href.replace(/\/$/, '');
}

/** Licence text as stored: no BOM, LF line ends, no trailing blanks. */
export function normaliseText(text) {
  return text
    .replace(/^﻿/, '')
    .replace(/\r\n?/g, '\n')
    .split('\n')
    .map((l) => l.replace(/[ \t]+$/, ''))
    .join('\n')
    .trim();
}

export function textId(text) {
  return createHash('sha256').update(text, 'utf8').digest('hex').slice(0, 16);
}

export function sha256File(path) {
  return createHash('sha256').update(normaliseText(readFileSync(path, 'utf8')), 'utf8').digest('hex');
}

/** Licence-like files at the top of a package directory, sorted. */
export function licenceFiles(dir) {
  if (!dir || !existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((f) => LICENCE_FILE.test(f))
    .filter((f) => {
      const st = statSync(join(dir, f));
      return st.isFile() && st.size > 0 && st.size <= MAX_TEXT_BYTES;
    })
    .sort();
}

/**
 * Crates reachable from the roots through normal and build dependencies, with the
 * set of artifacts each one ends up in. `metadata` is `cargo metadata` output.
 */
export function walkCargo(metadata, roots) {
  const pkgs = new Map(metadata.packages.map((p) => [p.id, p]));
  const nodes = new Map(metadata.resolve.nodes.map((n) => [n.id, n]));
  const reached = new Map(); // id -> Set(artifact)
  for (const id of metadata.workspace_members) {
    const artifact = roots[pkgs.get(id)?.name];
    if (!artifact) continue;
    const queue = [id];
    const seen = new Set(queue);
    while (queue.length) {
      const cur = queue.shift();
      if (!reached.has(cur)) reached.set(cur, new Set());
      reached.get(cur).add(artifact);
      for (const d of nodes.get(cur)?.deps ?? []) {
        const shipped = (d.dep_kinds ?? [{ kind: null }]).some((k) => k.kind === null || k.kind === 'build');
        if (shipped && !seen.has(d.pkg)) {
          seen.add(d.pkg);
          queue.push(d.pkg);
        }
      }
    }
  }
  const out = [];
  for (const [id, artifacts] of reached) {
    const p = pkgs.get(id);
    if (!p || p.source === null || p.source === undefined) continue; // first-party
    out.push({ pkg: p, artifacts: ARTIFACT_ORDER.filter((a) => artifacts.has(a)) });
  }
  return out;
}

/** Same artifact sets merged when one crate version is reached from two workspaces. */
function mergeCrates(lists) {
  const byKey = new Map();
  for (const { pkg, artifacts } of lists.flat()) {
    const key = `${pkg.name}@${pkg.version}`;
    const prev = byKey.get(key);
    if (prev) prev.artifacts = ARTIFACT_ORDER.filter((a) => prev.artifacts.includes(a) || artifacts.includes(a));
    else byKey.set(key, { pkg, artifacts });
  }
  return [...byKey.values()];
}

// ---------------------------------------------------------------------------
// Collection
// ---------------------------------------------------------------------------

class Texts {
  constructor() {
    this.map = new Map();
  }
  add(text) {
    const t = normaliseText(text);
    const id = textId(t);
    this.map.set(id, t);
    return id;
  }
  toJSON() {
    return Object.fromEntries([...this.map.entries()].sort(([a], [b]) => (a < b ? -1 : 1)));
  }
}

function registryDirs(cargoHome) {
  const base = join(cargoHome, 'registry', 'src');
  if (!existsSync(base)) return [];
  return readdirSync(base).map((d) => join(base, d)).sort();
}

function crateSourceDir(pkg, regDirs) {
  // Prefer the manifest cargo metadata reports; fall back to a registry lookup.
  const fromManifest = pkg.manifest_path ? dirname(pkg.manifest_path) : null;
  if (fromManifest && existsSync(fromManifest)) return fromManifest;
  for (const d of regDirs) {
    const cand = join(d, `${pkg.name}-${pkg.version}`);
    if (existsSync(cand)) return cand;
  }
  return null;
}

/**
 * Attach licence texts to an entry. Files from the package come first; with none,
 * the standard text of each licence id in the expression, flagged when the table
 * lacks one.
 */
function attachTexts(entry, dir, extraFiles, texts, flags) {
  const files = [...new Set([...(dir ? licenceFiles(dir) : []), ...extraFiles])];
  const out = [];
  for (const f of files) {
    const full = dir ? resolve(dir, f) : f;
    if (!existsSync(full)) continue;
    const st = statSync(full);
    if (!st.isFile() || st.size === 0 || st.size > MAX_TEXT_BYTES) continue;
    out.push({ file: dir ? relative(dir, full).replace(/\\/g, '/') : f.split(/[\\/]/).pop(), id: texts.add(readFileSync(full, 'utf8')) });
  }
  if (out.length === 0) {
    const ids = licenceIds(parseSpdx(entry.licence));
    for (const id of ids) if (STANDARD_TEXTS[id]) out.push({ standard: id, id: texts.add(STANDARD_TEXTS[id]) });
    // One text per alternative is enough for an OR; flag only when none is known,
    // or when an AND leaves a part without text.
    const tree = parseSpdx(entry.licence);
    const missing = ids.filter((id) => !STANDARD_TEXTS[id]);
    if (out.length === 0 || (missing.length && tree && !tree.or)) {
      flags.push(`${entry.name}@${entry.version ?? '-'}: no licence file and no standard text for ${missing.join(', ') || '(no licence)'}`);
    }
  }
  entry.texts = out;
}

function linkFields(entry, repository, homepage, flags) {
  const label = `${entry.name}@${entry.version ?? '-'}`;
  for (const [field, raw] of [['repository', repository], ['homepage', homepage]]) {
    if (raw === undefined || raw === null || raw === '') continue;
    const link = httpsLink(raw);
    if (link) entry[field] = link;
    else flags.push(`${label}: ${field} dropped (not https): ${raw}`);
  }
  if (entry.homepage && entry.homepage === entry.repository) delete entry.homepage;
}

function licenceField(entry, raw, flags) {
  const label = `${entry.name}@${entry.version ?? '-'}`;
  entry.licence = raw ? normaliseLicence(raw) : '';
  const unknown = entry.licence ? unknownIds(entry.licence) : ['(none)'];
  if (unknown.length) flags.push(`${label}: unknown licence ${unknown.join(', ')} in "${entry.licence}"`);
}

const sortEntries = (a, b) =>
  a.name.localeCompare(b.name, 'en') || String(a.version ?? '').localeCompare(String(b.version ?? ''), 'en', { numeric: true });

export function collectCrates({ metadatas, cargoHome, texts, flags }) {
  const regDirs = registryDirs(cargoHome);
  const merged = mergeCrates(metadatas.map((m) => walkCargo(m, ROOTS)));
  const entries = [];
  for (const { pkg, artifacts } of merged) {
    const e = { name: pkg.name, version: pkg.version };
    licenceField(e, pkg.license, flags);
    linkFields(e, pkg.repository, pkg.homepage, flags);
    const dir = crateSourceDir(pkg, regDirs);
    if (!dir) flags.push(`${e.name}@${e.version}: source not unpacked under ${cargoHome} (run cargo fetch --locked)`);
    const extra = pkg.license_file && dir ? [pkg.license_file] : [];
    attachTexts(e, dir, extra, texts, flags);
    e.artifacts = artifacts;
    entries.push(e);
  }
  return entries.sort(sortEntries);
}

export function collectNpm({ pnpmList, webDir, texts, flags }) {
  const entries = [];
  const seen = new Set();
  for (const items of Object.values(pnpmList)) {
    for (const item of items) {
      item.versions.forEach((version, k) => {
        const dir = item.paths?.[k] ?? item.paths?.[0];
        const key = `${item.name}@${version}`;
        if (seen.has(key)) return;
        let pj = {};
        try {
          pj = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf8'));
        } catch {
          flags.push(`${key}: package.json not readable under node_modules`);
        }
        if (pj.os || pj.cpu) return; // host-specific prebuilt binary, see header
        seen.add(key);
        const e = { name: item.name, version };
        licenceField(e, item.license ?? pj.license, flags);
        const repo = typeof pj.repository === 'string' ? pj.repository : pj.repository?.url;
        linkFields(e, repo, item.homepage ?? pj.homepage, flags);
        attachTexts(e, dir && existsSync(dir) ? dir : null, [], texts, flags);
        entries.push(e);
      });
    }
  }
  void webDir;
  return entries.sort(sortEntries);
}

export function collectAssets({ root, texts, flags }) {
  return ASSETS.map((a) => {
    const e = { kind: a.kind, name: a.name };
    if (a.version) e.version = a.version;
    licenceField(e, a.licence, flags);
    linkFields(e, a.repository, a.homepage, flags);
    attachTexts(e, null, (a.files ?? []).map((f) => join(root, f)), texts, flags);
    e.note = a.note;
    return e;
  });
}

function lockInfo(root, files) {
  return files.map((f) => ({ file: f, sha256: existsSync(join(root, f)) ? sha256File(join(root, f)) : null }));
}

function readDenyAllow(root) {
  const p = join(root, 'deny.toml');
  if (!existsSync(p)) return null;
  const src = readFileSync(p, 'utf8');
  const sec = src.match(/^\[licenses\][\s\S]*?^allow\s*=\s*\[([\s\S]*?)^\]/m);
  if (!sec) return null;
  return new Set([...sec[1].replace(/#.*$/gm, '').matchAll(/"([^"]+)"/g)].map((m) => m[1]));
}

/** The js-licenses job's --onlyAllow list in ci.yml (web side of the policy). */
function readJsAllow(root) {
  const p = join(root, '.github/workflows/ci.yml');
  if (!existsSync(p)) return null;
  const m = readFileSync(p, 'utf8').match(/--onlyAllow\s+"([^"]+)"/);
  return m ? new Set(m[1].split(';').map((s) => s.trim()).filter(Boolean)) : null;
}

/** Entries whose licence cannot be satisfied under a policy allow-list. */
export function policyConflicts(entries, allow) {
  if (!allow) return [];
  return entries.filter((e) => !allowedBy(e.licence, allow)).map((e) => `${e.name}@${e.version ?? '-'} (${e.licence})`);
}

// ---------------------------------------------------------------------------
// Inputs from tools
// ---------------------------------------------------------------------------

function cargoMetadata(root, manifest) {
  const args = ['metadata', '--format-version', '1', '--locked', '--offline'];
  if (manifest) args.push('--manifest-path', join(root, manifest));
  const out = execFileSync('cargo', args, { cwd: root, encoding: 'utf8', maxBuffer: 512 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'] });
  return JSON.parse(out);
}

function pnpmLicences(webDir) {
  // A fixed command line through the shell: on Windows pnpm is a .cmd shim, which
  // execFile cannot start without one.
  const out = execSync('pnpm licenses list --json --prod', {
    cwd: webDir,
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  return JSON.parse(out);
}

// ---------------------------------------------------------------------------
// Build + compare
// ---------------------------------------------------------------------------

export function build({ root, metadatas, pnpmList, cargoHome }) {
  const texts = new Texts();
  const flags = [];
  const crates = collectCrates({ metadatas, cargoHome, texts, flags });
  const npm = collectNpm({ pnpmList, webDir: join(root, 'apps/web'), texts, flags });
  const assets = collectAssets({ root, texts, flags });
  const generator = 'scripts/about/gen-third-party.mjs';
  const web = {
    schema: SCHEMA,
    generator,
    locks: lockInfo(root, LOCKFILES.web),
    texts: 'texts/<first character of the id>.json',
    packages: npm,
    assets,
  };
  const cratesDoc = {
    schema: SCHEMA,
    generator,
    locks: lockInfo(root, LOCKFILES.cargo),
    texts: 'apps/web/src/about/texts/<first character of the id>.json',
    artifacts: ARTIFACT_ORDER,
    packages: crates,
  };
  const policy = {
    cargoDeny: policyConflicts(crates, readDenyAllow(root)),
    jsLicenses: policyConflicts(npm, readJsAllow(root)),
  };
  return { web, crates: cratesDoc, texts: texts.toJSON(), flags, policy };
}

const serialise = (doc) => `${JSON.stringify(doc, null, 2)}\n`;

/** Human-readable differences between a committed document and a fresh one. */
export function diffDocs(name, committed, fresh) {
  if (committed === null) return [`${name}: missing`];
  if (JSON.stringify(committed) === JSON.stringify(fresh)) return [];
  const out = [];
  for (const l of fresh.locks ?? []) {
    const old = (committed.locks ?? []).find((c) => c.file === l.file);
    if (!old || old.sha256 !== l.sha256) out.push(`${name}: ${l.file} changed since the list was generated`);
  }
  const key = (e) => `${e.kind ?? ''}${e.name}@${e.version ?? ''}`;
  if (fresh.packages) {
    const a = new Map((committed.packages ?? []).map((e) => [key(e), JSON.stringify(e)]));
    const b = new Map(fresh.packages.map((e) => [key(e), JSON.stringify(e)]));
    for (const k of b.keys()) if (!a.has(k)) out.push(`${name}: added ${k}`);
    for (const k of a.keys()) if (!b.has(k)) out.push(`${name}: removed ${k}`);
    for (const [k, v] of b) if (a.has(k) && a.get(k) !== v) out.push(`${name}: changed ${k}`);
  }
  if (out.length === 0) out.push(`${name}: content differs`);
  return out;
}

function readJson(path) {
  try {
    return JSON.parse(readFileSync(path, 'utf8'));
  } catch {
    return null;
  }
}

function parseArgs(argv) {
  const opts = { check: false, root: null, quiet: false, pnpmJson: null };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--check') opts.check = true;
    else if (a === '--quiet') opts.quiet = true;
    else if (a === '--root') opts.root = argv[++i];
    else if (a === '--pnpm-json') opts.pnpmJson = argv[++i];
    else if (a === '--help' || a === '-h') opts.help = true;
    else throw new Error(`unknown argument ${a}`);
  }
  return opts;
}

export function main(argv = process.argv.slice(2)) {
  const opts = parseArgs(argv);
  if (opts.help) {
    console.log('usage: node scripts/about/gen-third-party.mjs [--check] [--root DIR] [--pnpm-json FILE] [--quiet]');
    console.log('  --root DIR        run against another checkout (default: this repository)');
    console.log('  --pnpm-json FILE  saved `pnpm licenses list --json --prod` output instead of running pnpm');
    return 0;
  }
  const root = resolve(opts.root ?? fileURLToPath(new URL('../../', import.meta.url)));
  const cargoHome = process.env.CARGO_HOME || join(homedir(), '.cargo');
  let metadatas;
  let pnpmList;
  try {
    metadatas = [cargoMetadata(root), cargoMetadata(root, 'crates/mw-media-wasm/Cargo.toml')];
    pnpmList = opts.pnpmJson ? JSON.parse(readFileSync(opts.pnpmJson, 'utf8')) : pnpmLicences(join(root, 'apps/web'));
  } catch (err) {
    console.error(`gen-third-party: could not read the dependency set: ${String(err.stderr || err.message).trim()}`);
    console.error('  needs `cargo fetch --locked` (offline metadata and sources) and apps/web/node_modules.');
    return opts.check ? 1 : 2;
  }
  const out = build({ root, metadatas, pnpmList, cargoHome });
  const files = outputFiles(out);

  const report = () => {
    if (opts.quiet) return;
    const bytes = files.map(([, doc]) => Buffer.byteLength(serialise(doc)));
    const shardBytes = bytes.slice(2);
    console.log(
      `crates ${out.crates.packages.length}, npm packages ${out.web.packages.length}, assets ${out.web.assets.length}, ` +
        `unique licence texts ${Object.keys(out.texts).length}; bytes web ${bytes[0]}, crates ${bytes[1]}, ` +
        `texts ${shardBytes.reduce((a, b) => a + b, 0)} in ${shardBytes.length} shards (largest ${Math.max(...shardBytes)})`,
    );
    for (const f of out.flags) console.log(`FLAG ${f}`);
    for (const c of out.policy.cargoDeny) console.log(`POLICY deny.toml [licenses].allow does not cover ${c}`);
    for (const c of out.policy.jsLicenses) console.log(`POLICY ci.yml js-licenses --onlyAllow does not cover ${c}`);
  };

  if (opts.check) {
    const problems = files.flatMap(([rel, doc]) => diffDocs(rel, readJson(join(root, rel)), doc));
    if (problems.length) {
      for (const p of problems.slice(0, 50)) console.error(`STALE ${p}`);
      if (problems.length > 50) console.error(`STALE … and ${problems.length - 50} more`);
      console.error('gen-third-party --check: FAIL. Run `node scripts/about/gen-third-party.mjs` and commit the result.');
      return 1;
    }
    report();
    console.log('gen-third-party --check: OK');
    return 0;
  }

  for (const [rel, doc] of files) {
    mkdirSync(dirname(join(root, rel)), { recursive: true });
    writeFileSync(join(root, rel), serialise(doc));
  }
  report();
  return 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  process.exitCode = main();
}
