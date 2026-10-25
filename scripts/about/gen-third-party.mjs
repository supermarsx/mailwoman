#!/usr/bin/env node
// Third-party list for the About screens (t32-e75; plan t32-cfg.md §4.11 with review
// amendments A12/A28; DC9 with licence texts in scope).
//
//   node scripts/about/gen-third-party.mjs                 regenerate the list
//   node scripts/about/gen-third-party.mjs --check         exit 1 when it is stale or a rule fails
//   node scripts/about/gen-third-party.mjs --image TAG     also re-read the runtime image's
//                                                          Debian packages from a local image
//
// Run it after `vite build` in apps/web: the web list is what rollup put in the bundle,
// recorded by apps/web/scripts/about-bundled-packages.mjs.
//
// Sections of apps/web/src/about/third-party.json:
//   rust        crates linked into what ships, through normal dependencies only:
//               mailwoman + mw-render for x86_64/aarch64 Linux (glibc) and x86_64 musl,
//               mw-crypto + mw-sanitize for wasm32 (web bundle), the plugins for
//               wasm32-wasip2 (plugins/dist), mw-media-wasm (embedded in mw-render).
//               Each `cargo metadata --filter-platform <target>` so a dependency of
//               another platform is not listed.
//   shells      crates only in the Tauri desktop and mobile apps (their own targets)
//   web         npm packages rollup included in the web bundle
//   assets      the bundled Jost face, the fonts `mailwoman fonts pull` fetches, the
//               icons and theme textures (first-party)
//   runtimeImage  what the server images add: the distroless Debian packages of the
//               `runtime` image with their /usr/share/doc/*/copyright files, and for
//               the FROM-scratch `runtime-static` image the statically linked musl and
//               Alpine's CA bundle
//   buildTools  proc-macros, build scripts and their dependencies: run while building,
//               not shipped
// Licence texts: each package's LICENSE*/COPYING*/NOTICE*/COPYRIGHT* files (NOTICE kept),
// stored once by hash in apps/web/src/about/texts/<first hex digit>.json; a package with
// no file gets the SPDX standard text from scripts/about/spdx/<id>.txt, marked so.
//
// --check fails when: a lockfile changed since generation; the list differs from a fresh
// run (a shipped crate, bundled package or asset added, removed or changed); an entry has
// no licence text; a licence in rust/shells/web/buildTools is outside deny.toml's
// [licenses] allow list (the same policy `cargo deny` applies); a font, image or other
// file under apps/web/src/assets or apps/web/public is not accounted for; the bundle
// record is missing; or the Dockerfile names a different runtime image than the list was
// made from. Assets and runtime-image packages are listed but not held to deny.toml:
// fonts are not code, and the Debian packages are the base image's, not dependencies.
//
// spdx/: MIT, ISC, BSD-2-Clause, BSD-3-Clause and Zlib are the SPDX templates (copyright
// line left as the placeholder); Apache-2.0, MPL-2.0, BSL-1.0, Unicode-3.0 and CC0-1.0 are
// the texts as shipped unchanged by crates in the registry (ammonia 4.1.4, cssparser
// 0.36.0, ryu 1.0.23, unicode-ident 1.0.27, constant_time_eq 0.4.2); OFL-1.1 is the licence
// part of apps/web/src/assets/fonts/jost/OFL.txt.
//
// No dependency beyond node itself. cargo, and docker for --image, are run read-only.

import { execFile, execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { homedir, tmpdir } from 'node:os';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { promisify } from 'node:util';

export const SCHEMA = 'mailwoman-third-party/2';
export const OUTPUT = 'apps/web/src/about/third-party.json';
export const TEXTS_DIR = 'apps/web/src/about/texts';
export const SHARDS = '0123456789abcdef'.split('');
// Outside dist/ on purpose: everything in dist/ is embedded in the server and served.
export const BUNDLE_RECORD = 'apps/web/node_modules/.cache/mailwoman-about/bundled-packages.json';
export const LOCKFILES = {
  cargoLock: 'Cargo.lock',
  mediaWasmLock: 'crates/mw-media-wasm/Cargo.lock',
  pnpmLock: 'apps/web/pnpm-lock.yaml',
};
export const SECTIONS = ['rust', 'shells', 'web', 'assets', 'runtimeImage', 'buildTools'];
const POLICY_SECTIONS = ['rust', 'shells', 'web', 'buildTools'];

const LINUX = ['x86_64-unknown-linux-gnu', 'aarch64-unknown-linux-gnu', 'x86_64-unknown-linux-musl'];
const DESKTOP = ['x86_64-pc-windows-msvc', 'x86_64-apple-darwin', 'aarch64-apple-darwin', 'x86_64-unknown-linux-gnu'];
const MOBILE = ['aarch64-linux-android', 'armv7-linux-androideabi', 'x86_64-linux-android', 'aarch64-apple-ios'];
const PLUGINS = ['bridge-ews', 'bridge-gmail', 'bridge-graph', 'languagetool', 'masked-email', 'nextcloud-plugin', 'spam-rspamd', 'spam-spamassassin'];

// One `cargo metadata` per (manifest, platform); `roots` names the workspace packages
// walked in it and the artifact each ends up in.
export const CARGO_WALKS = [
  { manifest: null, platforms: LINUX, roots: { 'mw-server': 'server', 'mw-render': 'server' } },
  { manifest: null, platforms: ['wasm32-unknown-unknown'], roots: { 'mw-crypto': 'web', 'mw-sanitize': 'web' } },
  { manifest: null, platforms: ['wasm32-wasip2'], roots: Object.fromEntries(PLUGINS.map((p) => [p, 'plugins'])) },
  { manifest: 'crates/mw-media-wasm/Cargo.toml', platforms: ['wasm32-unknown-unknown'], roots: { 'mw-media-wasm': 'server' } },
  { manifest: null, platforms: DESKTOP, roots: { 'mailwoman-desktop': 'desktop' } },
  { manifest: null, platforms: MOBILE, roots: { 'mailwoman-mobile': 'mobile' } },
];
export const ARTIFACT_ORDER = ['server', 'web', 'plugins', 'desktop', 'mobile'];
const SHIPPED_ARTIFACTS = new Set(['server', 'web', 'plugins']);

// Assets. `covers` lists the files (prefix match) an entry accounts for.
export const ASSETS = [
  {
    kind: 'font',
    name: 'Jost',
    versionFrom: 'apps/web/src/assets/fonts/jost/SOURCE.txt',
    licence: 'OFL-1.1',
    repository: 'https://github.com/indestructible-type/Jost',
    homepage: 'https://fontsource.org/fonts/jost',
    files: ['apps/web/src/assets/fonts/jost/OFL.txt'],
    covers: ['apps/web/src/assets/fonts/jost/'],
    note: 'bundled',
  },
  { kind: 'font', name: 'Inter', licence: 'OFL-1.1', repository: 'https://github.com/rsms/inter', homepage: 'https://fonts.google.com/specimen/Inter', note: 'fonts-pull' },
  { kind: 'font', name: 'Newsreader', licence: 'OFL-1.1', repository: 'https://github.com/productiontype/Newsreader', homepage: 'https://fonts.google.com/specimen/Newsreader', note: 'fonts-pull' },
  { kind: 'font', name: 'JetBrains Mono', licence: 'OFL-1.1', repository: 'https://github.com/JetBrains/JetBrainsMono', homepage: 'https://fonts.google.com/specimen/JetBrains+Mono', note: 'fonts-pull' },
  {
    kind: 'icons',
    name: 'Mailwoman icons',
    licence: 'MIT',
    repository: 'https://github.com/supermarsx/mailwoman',
    files: ['license.md'],
    covers: ['apps/web/public/icons/', 'apps/web/public/favicon.svg', 'apps/web/public/favicon.ico', 'apps/web/public/apple-touch-icon.png'],
    note: 'first-party',
  },
  {
    kind: 'image',
    name: 'Theme textures',
    licence: 'MIT',
    repository: 'https://github.com/supermarsx/mailwoman',
    files: ['license.md'],
    covers: ['apps/web/public/themes/'],
    note: 'first-party',
  },
];
// Files in apps/web/public copied from a package rather than bundled by rollup.
export const VENDORED_PUBLIC = { 'apps/web/public/pdf.worker.mjs': 'pdfjs-dist' };
// First-party code and configuration in apps/web/public (not assets).
export const FIRST_PARTY_PUBLIC = ['apps/web/public/sw.js', 'apps/web/public/manifest.webmanifest'];
const ASSET_EXT = /\.(woff2?|ttf|otf|eot|png|jpe?g|gif|webp|avif|svg|ico|bmp)$/i;

// The FROM-scratch image's non-Rust contents (Dockerfile stage runtime-static).
const STATIC_IMAGE_ENTRIES = [
  { name: 'musl', licence: 'MIT', homepage: 'https://musl.libc.org', note: 'static-libc' },
  { name: 'ca-certificates', licence: 'MPL-2.0 AND MIT', homepage: 'https://gitlab.alpinelinux.org/alpine/ca-certificates', note: 'alpine-ca' },
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

/** SPDX standard texts by id, from scripts/about/spdx/<id>.txt. */
export const STANDARD_TEXTS = (() => {
  const dir = fileURLToPath(new URL('./spdx/', import.meta.url));
  const out = {};
  for (const f of existsSync(dir) ? readdirSync(dir) : []) {
    if (f.endsWith('.txt')) out[f.slice(0, -4)] = readFileSync(join(dir, f), 'utf8');
  }
  return out;
})();

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
  // package.json sometimes says "(MIT OR Apache-2.0)"; drop one outer pair that spans it all.
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

/** The `License:` short names of a Debian (DEP-5) copyright file, in order, once each. */
export function debianLicences(copyright) {
  const names = [];
  for (const m of copyright.matchAll(/^License:[ \t]*(\S[^\n]*)$/gm)) {
    const name = m[1].trim();
    if (!names.includes(name)) names.push(name);
  }
  return names;
}

/** `FROM <ref> AS <stage>` lines of a Dockerfile, by stage name. */
export function dockerStages(src) {
  const out = {};
  for (const m of src.matchAll(/^FROM\s+(\S+)\s+AS\s+(\S+)\s*$/gim)) out[m[2]] = m[1];
  return out;
}

const isProcMacro = (pkg) => (pkg?.targets ?? []).some((t) => (t.kind ?? []).includes('proc-macro'));

/**
 * Walk one `cargo metadata` graph from the roots. Normal edges into a non-proc-macro
 * crate are shipped; a proc-macro, or anything behind a build edge, starts the build-
 * tool walk, which follows normal and build edges. Workspace crates are walked through
 * but not returned. Result: { shipped, tools }: Map(id -> { pkg, artifacts:Set }).
 */
export function walkCargo(metadata, roots) {
  const pkgs = new Map(metadata.packages.map((p) => [p.id, p]));
  const nodes = new Map(metadata.resolve.nodes.map((n) => [n.id, n]));
  const shipped = new Map();
  const tools = new Map();
  const mark = (map, id, artifact) => {
    if (!map.has(id)) map.set(id, { pkg: pkgs.get(id), artifacts: new Set() });
    const had = map.get(id).artifacts.has(artifact);
    map.get(id).artifacts.add(artifact);
    return !had;
  };
  const kinds = (d) => (d.dep_kinds ?? [{ kind: null }]).map((k) => k.kind);
  for (const rootId of metadata.workspace_members) {
    const artifact = roots[pkgs.get(rootId)?.name];
    if (!artifact) continue;
    const queue = [rootId];
    const toolQueue = [];
    mark(shipped, rootId, artifact);
    while (queue.length) {
      const cur = queue.shift();
      for (const d of nodes.get(cur)?.deps ?? []) {
        const k = kinds(d);
        if (k.includes(null)) {
          if (isProcMacro(pkgs.get(d.pkg))) toolQueue.push(d.pkg);
          else if (mark(shipped, d.pkg, artifact)) queue.push(d.pkg);
        }
        if (k.includes('build')) toolQueue.push(d.pkg);
      }
    }
    const seen = new Set();
    while (toolQueue.length) {
      const cur = toolQueue.shift();
      if (seen.has(cur)) continue;
      seen.add(cur);
      mark(tools, cur, artifact);
      for (const d of nodes.get(cur)?.deps ?? []) {
        const k = kinds(d);
        if (k.includes(null) || k.includes('build')) toolQueue.push(d.pkg);
      }
    }
  }
  const external = (map) => new Map([...map].filter(([, v]) => v.pkg && v.pkg.source !== null && v.pkg.source !== undefined));
  return { shipped: external(shipped), tools: external(tools) };
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
  put(id, text) {
    this.map.set(id, text);
  }
  toJSON() {
    return Object.fromEntries([...this.map.entries()].sort(([a], [b]) => (a < b ? -1 : 1)));
  }
}

const label = (e) => `${e.name}@${e.version ?? '-'}`;
const sortEntries = (a, b) =>
  a.name.localeCompare(b.name, 'en') || String(a.version ?? '').localeCompare(String(b.version ?? ''), 'en', { numeric: true });

function registryDirs(cargoHome) {
  const base = join(cargoHome, 'registry', 'src');
  if (!existsSync(base)) return [];
  return readdirSync(base).map((d) => join(base, d)).sort();
}

function crateSourceDir(pkg, regDirs) {
  const fromManifest = pkg.manifest_path ? dirname(pkg.manifest_path) : null;
  if (fromManifest && existsSync(fromManifest)) return fromManifest;
  for (const d of regDirs) {
    const cand = join(d, `${pkg.name}-${pkg.version}`);
    if (existsSync(cand)) return cand;
  }
  return null;
}

/** Licence texts: the package's own files first; with none, the standard text of each id. */
function attachTexts(entry, dir, extraFiles, texts) {
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
    for (const id of licenceIds(parseSpdx(entry.licence))) {
      if (STANDARD_TEXTS[id]) out.push({ standard: id, id: texts.add(STANDARD_TEXTS[id]) });
    }
  }
  entry.texts = out;
}

function linkFields(entry, repository, homepage, flags) {
  for (const [field, raw] of [['repository', repository], ['homepage', homepage]]) {
    if (raw === undefined || raw === null || raw === '') continue;
    const link = httpsLink(raw);
    if (link) entry[field] = link;
    else flags.push(`${label(entry)}: ${field} dropped (not https): ${raw}`);
  }
  if (entry.homepage && entry.homepage === entry.repository) delete entry.homepage;
}

function licenceField(entry, raw, flags) {
  entry.licence = raw ? normaliseLicence(raw) : '';
  const unknown = entry.licence ? unknownIds(entry.licence) : ['(none)'];
  if (unknown.length) flags.push(`${label(entry)}: unknown licence ${unknown.join(', ')} in "${entry.licence}"`);
}

function crateEntry(pkg, artifacts, regDirs, texts, flags) {
  const e = { kind: 'crate', name: pkg.name, version: pkg.version };
  licenceField(e, pkg.license, flags);
  linkFields(e, pkg.repository, pkg.homepage, flags);
  const dir = crateSourceDir(pkg, regDirs);
  if (!dir) flags.push(`${label(e)}: source not unpacked (run cargo fetch --locked)`);
  attachTexts(e, dir, pkg.license_file && dir ? [pkg.license_file] : [], texts);
  e.artifacts = ARTIFACT_ORDER.filter((a) => artifacts.has(a));
  return e;
}

export function collectCrates({ cargo, cargoHome, texts, flags }) {
  const regDirs = registryDirs(cargoHome);
  const shipped = new Map(); // name@version -> { pkg, artifacts:Set }
  const tools = new Map();
  const merge = (into, from) => {
    for (const { pkg, artifacts } of from.values()) {
      const key = `${pkg.name}@${pkg.version}`;
      if (!into.has(key)) into.set(key, { pkg, artifacts: new Set() });
      for (const a of artifacts) into.get(key).artifacts.add(a);
    }
  };
  for (const { metadata, roots } of cargo) {
    const w = walkCargo(metadata, roots);
    merge(shipped, w.shipped);
    merge(tools, w.tools);
  }
  const rust = [];
  const shells = [];
  for (const { pkg, artifacts } of shipped.values()) {
    const e = crateEntry(pkg, artifacts, regDirs, texts, flags);
    ([...artifacts].some((a) => SHIPPED_ARTIFACTS.has(a)) ? rust : shells).push(e);
  }
  const buildTools = [];
  for (const [key, { pkg, artifacts }] of tools) {
    if (shipped.has(key)) continue;
    buildTools.push(crateEntry(pkg, artifacts, regDirs, texts, flags));
  }
  return { rust: rust.sort(sortEntries), shells: shells.sort(sortEntries), buildTools: buildTools.sort(sortEntries) };
}

function npmDir(root, name, version, recorded) {
  if (recorded && existsSync(join(recorded, 'package.json'))) return recorded;
  const nm = join(root, 'apps/web/node_modules');
  const store = join(nm, '.pnpm', `${name.replace('/', '+')}@${version}`, 'node_modules', name);
  if (existsSync(join(store, 'package.json'))) return store;
  const top = join(nm, name);
  return existsSync(join(top, 'package.json')) ? top : null;
}

export function collectWeb({ root, bundled, texts, flags, problems }) {
  if (!bundled || !Array.isArray(bundled.packages)) {
    problems.push(`bundle record missing (${BUNDLE_RECORD}): run \`vite build\` in apps/web first`);
    return [];
  }
  const entries = [];
  const seen = new Set();
  for (const rec of bundled.packages) {
    const key = `${rec.name}@${rec.version}`;
    if (seen.has(key)) continue;
    seen.add(key);
    const dir = npmDir(root, rec.name, rec.version, rec.dir);
    let pj = {};
    if (dir) pj = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf8'));
    else problems.push(`${key}: bundled, but its package.json is not under apps/web/node_modules`);
    const e = { kind: 'npm', name: rec.name, version: rec.version };
    const lic = typeof pj.license === 'string' ? pj.license : pj.license?.type;
    licenceField(e, lic, flags);
    const repo = typeof pj.repository === 'string' ? pj.repository : pj.repository?.url;
    linkFields(e, repo, pj.homepage, flags);
    attachTexts(e, dir, [], texts);
    entries.push(e);
  }
  return entries.sort(sortEntries);
}

export function collectAssets({ root, texts, flags }) {
  return ASSETS.map((a) => {
    const e = { kind: a.kind, name: a.name };
    if (a.versionFrom) {
      const m = readFileSync(join(root, a.versionFrom), 'utf8').match(/^Package:\s+\S+\s+(\d[\w.+-]*)/m);
      if (m) e.version = m[1];
      else flags.push(`${a.name}: no "Package: <name> <version>" line in ${a.versionFrom}`);
    }
    licenceField(e, a.licence, flags);
    linkFields(e, a.repository, a.homepage, flags);
    attachTexts(e, null, (a.files ?? []).map((f) => join(root, f)), texts);
    e.note = a.note;
    return e;
  });
}

function listFiles(dir) {
  if (!existsSync(dir)) return [];
  return readdirSync(dir, { withFileTypes: true }).flatMap((d) =>
    d.isDirectory() ? listFiles(join(dir, d.name)) : [join(dir, d.name)],
  );
}

/** Fonts and images under src/assets, and every file under public, must be accounted for. */
export function uncoveredFiles(root, web) {
  const rel = (p) => relative(root, p).replace(/\\/g, '/');
  const covers = ASSETS.flatMap((a) => a.covers ?? []);
  const covered = (p) => covers.some((c) => (c.endsWith('/') ? p.startsWith(c) : p === c));
  const problems = [];
  const assetFiles = listFiles(join(root, 'apps/web/src/assets')).map(rel).filter((p) => ASSET_EXT.test(p));
  const publicFiles = listFiles(join(root, 'apps/web/public')).map(rel);
  for (const p of [...assetFiles, ...publicFiles].sort()) {
    if (covered(p) || FIRST_PARTY_PUBLIC.includes(p)) continue;
    const pkg = VENDORED_PUBLIC[p];
    if (pkg) {
      if (!web.some((e) => e.name === pkg)) problems.push(`${p}: a copy from ${pkg}, which is not in the web list`);
      continue;
    }
    problems.push(`${p}: not covered by any entry`);
  }
  return problems;
}

/**
 * Runtime image entries. `image` (from --image) refreshes the Debian packages; without
 * it the previous list's are kept while the Dockerfile names the same base image.
 */
export function collectRuntimeImage({ root, image, previous, texts, flags, problems }) {
  const stages = existsSync(join(root, 'Dockerfile')) ? dockerStages(readFileSync(join(root, 'Dockerfile'), 'utf8')) : {};
  const images = {
    runtime: { from: stages.runtime ?? null },
    runtimeStatic: { from: 'scratch', buildFrom: stages['build-musl'] ?? null },
  };
  let debs = [];
  if (image) {
    images.runtime.readFrom = image.readFrom;
    for (const p of image.packages) {
      const e = { kind: 'os', name: p.name, version: p.version };
      e.licence = debianLicences(p.copyright).join(' AND ');
      linkFields(e, null, p.homepage, flags);
      e.texts = p.copyright ? [{ file: `/usr/share/doc/${p.name}/copyright`, id: texts.add(p.copyright) }] : [];
      e.images = ['runtime'];
      debs.push(e);
    }
  } else if (previous?.doc?.images?.runtime?.from && previous.doc.images.runtime.from === images.runtime.from) {
    images.runtime.readFrom = previous.doc.images.runtime.readFrom;
    debs = (previous.doc.runtimeImage ?? []).filter((e) => e.images?.includes('runtime'));
    for (const e of debs) {
      for (const t of e.texts ?? []) {
        if (previous.texts?.[t.id] !== undefined) texts.put(t.id, previous.texts[t.id]);
        else problems.push(`${label(e)}: licence text ${t.id} missing from the previous list; regenerate with --image`);
      }
    }
  } else if (previous?.doc?.images?.runtime?.from) {
    problems.push(
      `runtime image changed (Dockerfile: ${images.runtime.from}; list made from ${previous.doc.images.runtime.from}): ` +
        'regenerate with --image <tag built from this Dockerfile>',
    );
  } else {
    problems.push('no runtime image entries yet: regenerate with --image <tag built from this Dockerfile>');
  }
  if (images.runtimeStatic.buildFrom === null) problems.push('Dockerfile has no `AS build-musl` stage; update STATIC_IMAGE_ENTRIES');
  const statics = STATIC_IMAGE_ENTRIES.map((s) => {
    const e = { kind: 'os', name: s.name };
    licenceField(e, s.licence, flags);
    linkFields(e, null, s.homepage, flags);
    attachTexts(e, null, [], texts);
    e.images = ['runtime-static'];
    e.note = s.note;
    return e;
  });
  return { images, runtimeImage: [...debs, ...statics].sort(sortEntries) };
}

function readDenyAllow(root) {
  const p = join(root, 'deny.toml');
  if (!existsSync(p)) return null;
  const sec = readFileSync(p, 'utf8').match(/^\[licenses\][\s\S]*?^allow\s*=\s*\[([\s\S]*?)^\]/m);
  if (!sec) return null;
  return new Set([...sec[1].replace(/#.*$/gm, '').matchAll(/"([^"]+)"/g)].map((m) => m[1]));
}

// ---------------------------------------------------------------------------
// Build + compare
// ---------------------------------------------------------------------------

/**
 * Everything from already-gathered inputs; no tools run here.
 *   cargo:    [{ metadata, roots }]   one per `cargo metadata` call
 *   bundled:  the bundle record, or null
 *   image:    { readFrom, packages:[{ name, version, homepage, copyright }] } or null
 *   previous: { doc, texts } of the committed list, or null
 * Returns { doc, texts, flags (informational), problems (fail --check) }.
 */
export function build({ root, cargo, bundled, cargoHome, image = null, previous = null }) {
  const texts = new Texts();
  const flags = [];
  const problems = [];
  const { rust, shells, buildTools } = collectCrates({ cargo, cargoHome, texts, flags });
  const web = collectWeb({ root, bundled, texts, flags, problems });
  const assets = collectAssets({ root, texts, flags });
  const { images, runtimeImage } = collectRuntimeImage({ root, image, previous, texts, flags, problems });
  const generatedFrom = Object.fromEntries(
    Object.entries(LOCKFILES).map(([k, f]) => [k, existsSync(join(root, f)) ? sha256File(join(root, f)) : null]),
  );
  const doc = {
    schema: SCHEMA,
    generator: 'scripts/about/gen-third-party.mjs',
    generatedFrom,
    images,
    texts: 'texts/<first character of the id>.json',
    rust,
    shells,
    web,
    assets,
    runtimeImage,
    buildTools,
  };

  for (const s of SECTIONS) for (const e of doc[s]) if (e.texts.length === 0) problems.push(`${label(e)}: no licence text`);
  const allow = readDenyAllow(root);
  if (!allow) problems.push("deny.toml has no [licenses] allow list to check against");
  else {
    for (const s of POLICY_SECTIONS) {
      for (const e of doc[s]) {
        if (!allowedBy(e.licence, allow)) problems.push(`${label(e)}: licence ${e.licence || '(none)'} is outside deny.toml's [licenses] allow list`);
      }
    }
  }
  problems.push(...uncoveredFiles(root, web));
  return { doc, texts: texts.toJSON(), flags, problems };
}

/** Every output file as [path relative to the root, document]. */
export function outputFiles(out) {
  const shards = Object.fromEntries(SHARDS.map((h) => [h, {}]));
  for (const [id, text] of Object.entries(out.texts)) shards[id[0]][id] = text;
  return [[OUTPUT, out.doc], ...SHARDS.map((h) => [`${TEXTS_DIR}/${h}.json`, shards[h]])];
}

const serialise = (doc) => `${JSON.stringify(doc, null, 2)}\n`;

/** Human-readable differences between a committed document and a fresh one. */
export function diffDocs(name, committed, fresh) {
  if (committed === null || committed === undefined) return [`${name}: missing`];
  if (JSON.stringify(committed) === JSON.stringify(fresh)) return [];
  const out = [];
  for (const [k, sha] of Object.entries(fresh.generatedFrom ?? {})) {
    if (committed.generatedFrom?.[k] !== sha) out.push(`${name}: ${LOCKFILES[k] ?? k} changed since the list was generated`);
  }
  const key = (e) => `${e.kind === 'crate' || e.kind === 'npm' ? '' : `${e.kind}:`}${e.name}@${e.version ?? ''}`;
  for (const s of SECTIONS) {
    if (!fresh[s]) continue;
    const a = new Map((committed[s] ?? []).map((e) => [key(e), JSON.stringify(e)]));
    const b = new Map(fresh[s].map((e) => [key(e), JSON.stringify(e)]));
    for (const k of b.keys()) if (!a.has(k)) out.push(`${name}: ${s}: added ${k}`);
    for (const k of a.keys()) if (!b.has(k)) out.push(`${name}: ${s}: removed ${k}`);
    for (const [k, v] of b) if (a.has(k) && a.get(k) !== v) out.push(`${name}: ${s}: changed ${k}`);
  }
  if (out.length === 0) out.push(`${name}: content differs`);
  return out;
}

// ---------------------------------------------------------------------------
// Inputs from tools
// ---------------------------------------------------------------------------

const execFileP = promisify(execFile);

async function cargoMetadata(root, manifest, platform) {
  const args = ['metadata', '--format-version', '1', '--locked', '--offline', '--filter-platform', platform];
  if (manifest) args.push('--manifest-path', join(root, manifest));
  const { stdout } = await execFileP('cargo', args, { cwd: root, encoding: 'utf8', maxBuffer: 512 * 1024 * 1024 });
  return JSON.parse(stdout);
}

/** Debian packages of a local image: name, version and homepage from dpkg status.d, copyright text. */
export function readImage(ref) {
  const docker = (...a) => execFileSync('docker', a, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
  const id = docker('create', ref);
  const tmp = mkdtempSync(join(tmpdir(), 'mw-tp-image-'));
  try {
    docker('cp', `${id}:/var/lib/dpkg/status.d`, join(tmp, 'status.d'));
    const packages = [];
    for (const f of readdirSync(join(tmp, 'status.d')).filter((n) => !n.includes('.')).sort()) {
      const status = readFileSync(join(tmp, 'status.d', f), 'utf8');
      const field = (n) => status.match(new RegExp(`^${n}:\\s*(.+)$`, 'm'))?.[1].trim();
      const name = field('Package') ?? f;
      let copyright = '';
      try {
        docker('cp', '-L', `${id}:/usr/share/doc/${name}/copyright`, join(tmp, `${name}.copyright`));
        copyright = readFileSync(join(tmp, `${name}.copyright`), 'utf8');
      } catch {
        // no copyright file: the entry gets none, and --check reports it
      }
      packages.push({ name, version: field('Version'), homepage: field('Homepage'), copyright });
    }
    return { readFrom: ref, packages };
  } finally {
    try {
      docker('rm', id);
    } catch {
      // already gone
    }
    rmSync(tmp, { recursive: true, force: true });
  }
}

function readJson(path) {
  try {
    return JSON.parse(readFileSync(path, 'utf8'));
  } catch {
    return null;
  }
}

function readPrevious(root) {
  const doc = readJson(join(root, OUTPUT));
  if (!doc) return null;
  const texts = Object.assign({}, ...SHARDS.map((h) => readJson(join(root, TEXTS_DIR, `${h}.json`)) ?? {}));
  return { doc, texts };
}

function parseArgs(argv) {
  const opts = { check: false, root: null, quiet: false, bundled: null, image: null };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--check') opts.check = true;
    else if (a === '--quiet') opts.quiet = true;
    else if (a === '--root') opts.root = argv[++i];
    else if (a === '--bundled') opts.bundled = argv[++i];
    else if (a === '--image') opts.image = argv[++i];
    else if (a === '--help' || a === '-h') opts.help = true;
    else throw new Error(`unknown argument ${a}`);
  }
  return opts;
}

export async function main(argv = process.argv.slice(2)) {
  const opts = parseArgs(argv);
  if (opts.help) {
    console.log('usage: node scripts/about/gen-third-party.mjs [--check] [--image TAG] [--root DIR] [--bundled FILE] [--quiet]');
    console.log('  --check         write nothing; exit 1 when the list is stale or a rule fails');
    console.log('  --image TAG     re-read the runtime image packages from a local image built from this Dockerfile');
    console.log('  --root DIR      run against another checkout (default: this repository)');
    console.log(`  --bundled FILE  bundle record (default: <root>/${BUNDLE_RECORD}, written by vite build)`);
    return 0;
  }
  const root = resolve(opts.root ?? fileURLToPath(new URL('../../', import.meta.url)));
  const cargoHome = process.env.CARGO_HOME || join(homedir(), '.cargo');
  let cargo;
  let image = null;
  try {
    const calls = CARGO_WALKS.flatMap((w) => w.platforms.map((p) => ({ w, p })));
    const metas = await Promise.all(calls.map(({ w, p }) => cargoMetadata(root, w.manifest, p)));
    cargo = metas.map((metadata, i) => ({ metadata, roots: calls[i].w.roots }));
    if (opts.image) image = readImage(opts.image);
  } catch (err) {
    console.error(`gen-third-party: could not read an input: ${String(err.stderr || err.message).trim()}`);
    console.error('  needs `cargo fetch --locked` (offline metadata and sources), and docker for --image.');
    return 1;
  }
  const bundled = readJson(opts.bundled ? resolve(opts.bundled) : join(root, BUNDLE_RECORD));
  const out = build({ root, cargo, bundled, cargoHome, image, previous: readPrevious(root) });
  const files = outputFiles(out);

  const report = () => {
    if (opts.quiet) return;
    const bytes = files.map(([, doc]) => Buffer.byteLength(serialise(doc)));
    const shardBytes = bytes.slice(1);
    const n = (s) => `${s} ${out.doc[s].length}`;
    console.log(
      `${SECTIONS.map(n).join(', ')}; unique licence texts ${Object.keys(out.texts).length}; ` +
        `bytes list ${bytes[0]}, texts ${shardBytes.reduce((a, b) => a + b, 0)} in ${shardBytes.length} shards (largest ${Math.max(...shardBytes)})`,
    );
    for (const f of out.flags) console.log(`FLAG ${f}`);
  };
  const fail = (lines, what) => {
    for (const p of lines.slice(0, 60)) console.error(`${what} ${p}`);
    if (lines.length > 60) console.error(`${what} … and ${lines.length - 60} more`);
  };

  if (opts.check) {
    const stale = files.flatMap(([rel, doc]) => diffDocs(rel, readJson(join(root, rel)), doc));
    if (stale.length || out.problems.length) {
      fail(out.problems, 'PROBLEM');
      fail(stale, 'STALE');
      console.error('gen-third-party --check: FAIL. Fix the problems, run `node scripts/about/gen-third-party.mjs` and commit the result.');
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
  if (out.problems.length) {
    fail(out.problems, 'PROBLEM');
    console.error('gen-third-party: written, but the problems above fail --check.');
    return 1;
  }
  return 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  main().then((code) => {
    process.exitCode = code;
  });
}
