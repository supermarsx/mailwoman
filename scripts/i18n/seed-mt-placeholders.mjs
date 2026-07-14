// MT-seed placeholder generator (1.0 hardening, SPEC §24 / ROADMAP-1.0 L21).
//
// Ensures every non-`en` locale has a catalog file for every `en` module, each
// carrying an unmistakable `# MT-SEED — NOT REVIEWED` header. The 11 non-`en`
// locales are populated by HUMAN translators via Weblate (out of autonomous
// scope); these stubs give Weblate the full component × language matrix so each
// shows as "needs translation" (0% translated) — NOT as finished work.
//
// Honesty over convenience: the stubs are EMPTY of messages (comments only). At
// runtime every missing key falls back to the `en` source (registry.ts), so an
// empty stub is safe and truthful — seeding copied English as if it were a
// translation would misreport coverage and mislead reviewers.
//
// Idempotent + SAFE: a file that already holds real message ids (a translator has
// started it) is left untouched; only missing files and comment-only stubs are
// (re)written. Run after adding a new `en/<module>.ftl`:
//   node scripts/i18n/seed-mt-placeholders.mjs

import { readdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = fileURLToPath(new URL('../../', import.meta.url));
const localesDir = join(repoRoot, 'apps/web/locales');

// Mirror of src/i18n/locales.ts LOCALES (en is the source; the rest are seeded).
const NON_EN = ['de', 'fr', 'es', 'pt-BR', 'nl', 'it', 'pl', 'ru', 'uk', 'zh', 'ja'];

/** Does a `.ftl` source contain at least one real (top-level) message id? */
function hasMessages(source) {
  for (const line of source.split(/\r?\n/)) {
    if (line.length === 0 || /^\s/.test(line)) continue;
    if (line.startsWith('#') || line.startsWith('-')) continue;
    if (/^[a-zA-Z][a-zA-Z0-9_-]*\s*=/.test(line)) return true;
  }
  return false;
}

function stub(locale, module) {
  return (
    `# MT-SEED — NOT REVIEWED\n` +
    `# Locale: ${locale} · Module: ${module}. Source of truth: ../en/${module}.ftl\n` +
    `#\n` +
    `# PLACEHOLDER awaiting human translation via Weblate. This catalog holds NO\n` +
    `# reviewed translations — every key falls back to the en source at runtime until\n` +
    `# a translator fills it in, so Weblate shows it as needing translation (0%).\n` +
    `# Do NOT hand-edit or paste raw machine output as final: translate in Weblate.\n`
  );
}

const enModules = (await readdir(join(localesDir, 'en')))
  .filter((f) => f.endsWith('.ftl'))
  .map((f) => f.replace(/\.ftl$/, ''))
  .sort();

let created = 0;
let refreshed = 0;
let skipped = 0;

for (const locale of NON_EN) {
  const dir = join(localesDir, locale);
  for (const module of enModules) {
    const path = join(dir, `${module}.ftl`);
    let existing = null;
    try {
      existing = await readFile(path, 'utf8');
    } catch {
      /* missing */
    }
    if (existing !== null && hasMessages(existing)) {
      skipped += 1; // a translator has real content here — never clobber it
      continue;
    }
    const wantsHeader = existing === null || !existing.startsWith('# MT-SEED — NOT REVIEWED');
    if (!wantsHeader) {
      skipped += 1;
      continue;
    }
    await writeFile(path, stub(locale, module), 'utf8');
    if (existing === null) created += 1;
    else refreshed += 1;
  }
}

console.log(
  `seed-mt-placeholders: ${enModules.length} en modules × ${NON_EN.length} locales — ` +
    `created ${created}, refreshed ${refreshed}, left ${skipped} untouched.`,
);
