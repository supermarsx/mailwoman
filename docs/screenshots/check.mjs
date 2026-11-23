#!/usr/bin/env node
// Consistency of manifests, images and pages. Exit code 1 on any error.
//
//   node docs/screenshots/check.mjs [--area <area>[,<area>]]
//
// Checks: manifest fields and seed names; every manifest entry has its image and
// is referenced from its page with the right class (.mw-shot for full shots,
// .mw-crop for crops) and non-empty alt text; an entry with callouts is followed
// on its page by a {.mw-callouts} legend; every /screenshots/<area>/ image a page
// uses has a manifest entry; no image without an entry; size budgets; every page
// with `screen: true` front matter has an orientation shot.
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';
import { META, OUTPUT, REPO } from './lib/paths.mjs';
import { BUDGET, kindOf, loadManifests, outputOf, urlOf, validateAll } from './lib/manifest.mjs';
import { loadSeedDefs } from './lib/seed.mjs';

const DOCS = join(REPO, 'docs');
const SKIP_DIRS = new Set(['node_modules', '.vitepress', 'public']);

function parseArgs(argv) {
  const opts = { areas: null };
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === '--area' || argv[i] === '--manifest') opts.areas = argv[++i].split(',');
    else throw new Error(`unknown argument ${argv[i]}`);
  }
  return opts;
}

function markdownFiles(dir, out = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      if (!SKIP_DIRS.has(entry.name)) markdownFiles(join(dir, entry.name), out);
    } else if (entry.name.endsWith('.md')) out.push(join(dir, entry.name));
  }
  return out;
}

const IMAGE = /!\[([^\]]*)\]\((\/screenshots\/[^)\s]+)\)(\{[^}\n]*\})?/g;

function main() {
  const opts = parseArgs(process.argv.slice(2));
  const errors = [];
  const warnings = [];
  const all = loadManifests();
  const manifests = opts.areas ? all.filter((m) => opts.areas.includes(m.area)) : all;
  if (opts.areas) {
    for (const a of opts.areas) if (!all.some((m) => m.area === a)) errors.push(`no manifest for area '${a}'`);
  }
  errors.push(...validateAll(manifests));

  // Seeds.
  let seeds;
  try {
    seeds = loadSeedDefs();
  } catch (error) {
    errors.push(error.message);
    seeds = new Map();
  }
  for (const { area, entries } of manifests) {
    for (const e of entries) {
      for (const name of e.seed ?? []) {
        const def = seeds.get(name);
        if (!def) errors.push(`${area}/${e.id}: unknown seed '${name}'`);
        else if (def.stacks && !def.stacks.includes(e.stack)) errors.push(`${area}/${e.id}: seed '${name}' does not run on stack ${e.stack}`);
      }
    }
  }

  // Image references in pages.
  const refs = new Map(); // url -> [{ page, alt, cls, index, text }]
  const pages = new Map(); // docs-relative path -> text
  for (const file of markdownFiles(DOCS)) {
    const rel = relative(DOCS, file).replace(/\\/g, '/');
    // Code examples (fenced blocks, inline code) are not image uses: blank them
    // out, keeping offsets, so the legend search below still lines up.
    const text = readFileSync(file, 'utf8')
      .replace(/^[ \t]*(`{3,}|~{3,})[^\n]*\n[\s\S]*?^[ \t]*\1[^\n]*$/gm, (m) => m.replace(/[^\n]/g, ' '))
      .replace(/`[^`\n]+`/g, (m) => ' '.repeat(m.length));
    pages.set(rel, text);
    for (const m of text.matchAll(IMAGE)) {
      const [, alt, url, attrs = ''] = m;
      if (!refs.has(url)) refs.set(url, []);
      refs.get(url).push({ page: rel, alt: alt.trim(), cls: attrs, index: m.index, text });
    }
    // Screen pages need an orientation shot.
    const front = /^---\r?\n([\s\S]*?)\r?\n---/.exec(text)?.[1] ?? '';
    const inScope = !opts.areas || manifests.some((m) => m.entries.some((e) => e.page === rel));
    if (inScope && /^screen:\s*true\s*$/m.test(front) &&!/\{[^}]*\.mw-shot[^}]*\}/.test(text)) {
      errors.push(`${rel}: has 'screen: true' but no orientation shot ({.mw-shot})`);
    }
  }
  const entryByUrl = new Map();
  for (const { area, entries } of all) for (const e of entries) entryByUrl.set(urlOf(area, e.id), { area, e });

  for (const [url, uses] of refs) {
    const area = url.split('/')[2];
    if (!/^\/screenshots\/[^/]+\/[^/]+\.webp$/.test(url)) continue; // not a harness image
    if (opts.areas && !opts.areas.includes(area)) continue;
    const owner = entryByUrl.get(url);
    for (const use of uses) {
      if (!use.alt) errors.push(`${use.page}: image ${url} has no alt text`);
      if (!owner) errors.push(`${use.page}: ${url} has no manifest entry`);
      else {
        const want = kindOf(owner.e) === 'full' ? 'mw-shot' : 'mw-crop';
        if (!use.cls.includes(`.${want}`)) errors.push(`${use.page}: ${url} needs {.${want}}`);
      }
    }
  }

  // Entries -> images and pages; budgets.
  let total = 0;
  const sizes = [];
  for (const { area, entries } of manifests) {
    for (const e of entries) {
      const file = outputOf(area, e.id);
      const url = urlOf(area, e.id);
      if (!existsSync(file)) {
        errors.push(`${area}/${e.id}: image missing (${relative(REPO, file)}); run capture.mjs --only ${area}/${e.id}`);
        continue;
      }
      const bytes = statSync(file).size;
      const kind = kindOf(e);
      sizes.push(bytes);
      if (bytes > BUDGET[kind]) errors.push(`${area}/${e.id}: ${Math.round(bytes / 1024)} KB is over the ${kind} budget of ${BUDGET[kind] / 1024} KB`);
      if (e.page === null) {
        errors.push(`${area}/${e.id}: no page; every image is used on a page`);
        continue;
      }
      if (!pages.has(e.page)) {
        errors.push(`${area}/${e.id}: page docs/${e.page} does not exist`);
        continue;
      }
      const uses = (refs.get(url) ?? []).filter((u) => u.page === e.page);
      if (!uses.length) errors.push(`${area}/${e.id}: not referenced from docs/${e.page}`);
      if (e.callouts?.length && uses.length) {
        const after = uses[0].text.slice(uses[0].index);
        const legend = /\{\s*\.mw-callouts\s*\}/.exec(after);
        if (!legend) errors.push(`${area}/${e.id}: has callouts but docs/${e.page} has no {.mw-callouts} legend after the image`);
        else {
          const items = after.slice(0, legend.index).split(/\r?\n/).filter((l) => /^\s*\d+\.\s/.test(l)).length;
          const max = Math.max(...e.callouts.map((c) => c.n));
          if (items < max) errors.push(`${area}/${e.id}: the legend lists ${items} items for callouts up to ${max}`);
        }
      }
    }
  }

  // Images of one area should come from one web build (warning only).
  for (const { area, entries } of manifests) {
    const file = join(META, `${area}.json`);
    if (!existsSync(file)) continue;
    const meta = JSON.parse(readFileSync(file, 'utf8')).entries ?? {};
    const builds = new Map();
    for (const e of entries) {
      const m = meta[e.id];
      if (!m?.web) continue;
      const key = String(m.web);
      if (!builds.has(key)) builds.set(key, { revision: m.webRevision ?? 'unknown', builtAt: m.webBuiltAt ?? 'unknown', ids: [] });
      builds.get(key).ids.push(e.id);
      if (m.webFallback) warnings.push(`${area}/${e.id}: captured from a fallback web build (${key}) because the source did not build then`);
    }
    if (builds.size > 1) {
      const list = [...builds].map(([hash, b]) => `${hash} (source ${b.revision}, built ${b.builtAt}): ${b.ids.join(', ')}`).join('; ');
      warnings.push(`${area}: images come from ${builds.size} different web builds; recapture the older ones (--manifest ${area}) so they match: ${list}`);
    }
  }

  // Orphans and the overall budget.
  if (existsSync(OUTPUT)) {
    for (const dir of readdirSync(OUTPUT, { withFileTypes: true })) {
      if (!dir.isDirectory()) continue;
      for (const f of readdirSync(join(OUTPUT, dir.name))) {
        const url = `/screenshots/${dir.name}/${f}`;
        const bytes = statSync(join(OUTPUT, dir.name, f)).size;
        total += bytes;
        if (opts.areas && !opts.areas.includes(dir.name)) continue;
        if (!entryByUrl.has(url)) errors.push(`${relative(REPO, join(OUTPUT, dir.name, f))}: no manifest entry (remove the file or add the entry)`);
      }
    }
  }
  if (total > BUDGET.total) errors.push(`all screenshots together are ${(total / 1048576).toFixed(1)} MB, over ${BUDGET.total / 1048576} MB`);

  for (const w of warnings) console.log(`warning: ${w}`);
  for (const e of errors) console.log(`error: ${e}`);
  const n = manifests.reduce((k, m) => k + m.entries.length, 0);
  console.log(
    `${n} entries in ${manifests.length} manifest(s); ${sizes.length} images, ${Math.round(sizes.reduce((a, b) => a + b, 0) / 1024)} KB checked; all screenshots ${(total / 1048576).toFixed(2)} MB; ${errors.length} error(s)`,
  );
  return errors.length ? 1 : 0;
}

try {
  process.exitCode = main();
} catch (error) {
  console.error(error.message);
  process.exitCode = 2;
}
