// Shared catalog readers for the i18n scripts (check-catalog.mjs, status.mjs).

import { readdir, readFile } from 'node:fs/promises';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

export const repoRoot = fileURLToPath(new URL('../../../', import.meta.url));
export const localesDir = join(repoRoot, 'apps/web/locales');
export const srcDir = join(repoRoot, 'apps/web/src');
export const statusPath = join(localesDir, 'TRANSLATION-STATUS.json');
export const SOURCE = 'en';

const localesTs = join(srcDir, 'i18n/locales.ts');

/** Recursively list files under `dir` whose name passes `filter`. */
export async function walk(dir, filter, out = []) {
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

/**
 * The shipped locales, read out of the `export const LOCALES = [...]` literal in
 * src/i18n/locales.ts rather than copied, so the scripts cannot report on a
 * different set from the one the app ships. Exits the process if unreadable.
 */
export async function shippedLocales(script) {
  const src = await readFile(localesTs, 'utf8');
  const decl = src.match(/export const LOCALES\s*=\s*\[([^\]]*)\]/);
  const locales = decl ? [...decl[1].matchAll(/(['"])([^'"]+)\1/g)].map((m) => m[2]) : [];
  if (locales.length === 0 || !locales.includes(SOURCE)) {
    console.error(
      `${script}: could not read a LOCALES array containing '${SOURCE}' from ${relative(repoRoot, localesTs)}`,
    );
    process.exit(1);
  }
  return locales;
}

/**
 * Parse the top-level Fluent message ids from a `.ftl` source. Message ids start
 * at column 0 (attributes and select variants are indented); Fluent TERMS begin
 * with `-` and are not t()-addressable. Comments and blank lines are skipped.
 */
export function parseFtlIds(source) {
  const ids = new Set();
  for (const rawLine of source.split(/\r?\n/)) {
    if (rawLine.length === 0 || /^\s/.test(rawLine)) continue;
    if (rawLine.startsWith('#') || rawLine.startsWith('-')) continue;
    const m = rawLine.match(/^([a-zA-Z][a-zA-Z0-9_-]*)\s*=/);
    if (m) ids.add(m[1]);
  }
  return ids;
}

/** All message ids defined for a locale (union across its module catalogs). */
export async function idsForLocale(locale) {
  const files = await walk(join(localesDir, locale), (n) => n.endsWith('.ftl'));
  const ids = new Set();
  for (const f of files) {
    for (const id of parseFtlIds(await readFile(f, 'utf8'))) ids.add(id);
  }
  return ids;
}

/** Read TRANSLATION-STATUS.json; `null` if the file does not exist. Throws on bad JSON. */
export async function readStatus() {
  let text;
  try {
    text = await readFile(statusPath, 'utf8');
  } catch {
    return null;
  }
  return JSON.parse(text);
}
