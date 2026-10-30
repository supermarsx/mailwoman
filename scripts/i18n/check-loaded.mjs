// i18n gate (t112): no message id renders raw.
//
// Fails when an id in a message-id position is missing from the en catalog, or
// when an en id is used in a file where no catalog defining it is guaranteed to
// be loaded (see lib/loaded.mjs for how "guaranteed" is worked out from the
// import graph). The runtime shows the id itself in both cases, and the unit
// tests cannot see either: src/test/setup.ts registers every en catalog.
//
// Fix a finding by adding the id to the en catalog (and pt-PT), by moving it to
// a catalog the screen already loads, or by calling loadCatalog('<module>')
// where the id is shown. A file whose t() calls run only on behalf of a part
// that loads the catalog goes in scripts/i18n/loaded-allowlist.json with the
// reason; a single line takes `// i18n-ignore-id: <reason>` on it or above it.
//
// Usage: node scripts/i18n/check-loaded.mjs   (needs `typescript` in apps/web/node_modules)

import { readFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { localesDir, repoRoot, srcDir, SOURCE } from './lib/catalog.mjs';
import { checkLoaded } from './lib/loaded.mjs';

let ts;
try {
  ts = createRequire(join(repoRoot, 'apps/web/package.json'))('typescript');
} catch {
  console.error('check-loaded: FAIL — `typescript` is not installed in apps/web/node_modules (run pnpm -C apps/web install).');
  process.exit(1);
}

const allowlistPath = join(repoRoot, 'scripts/i18n/loaded-allowlist.json');
const allowlist = JSON.parse(await readFile(allowlistPath, 'utf8')).entries ?? [];
const { missing, unloaded, stale, fileCount } = await checkLoaded(ts, { srcDir, enDir: join(localesDir, SOURCE), repoRoot, allowlist });
console.log(`check-loaded: scanned ${fileCount} source files.`);

if (missing.length > 0) {
  console.error(`\ncheck-loaded: ${missing.length} message id(s) are not in the en catalog:`);
  for (const m of missing) console.error(`  • ${m.id}  (${m.file}:${m.line}, ${m.why})`);
}
if (unloaded.length > 0) {
  console.error(`\ncheck-loaded: ${unloaded.length} id(s) are used where their catalog is not loaded:`);
  for (const u of unloaded) console.error(`  • ${u.id}  (${u.file}:${u.line}) defined in ${u.modules.join(', ')}.ftl`);
}
if (stale.length > 0) {
  console.error(`
check-loaded: ${stale.length} loaded-allowlist.json entr(y/ies) are stale (file gone, no reason, or no longer uses those catalogs):`);
  for (const e of stale) console.error(`  • ${e.file} (${(e.catalogs ?? []).join(', ')})`);
}
if (missing.length > 0 || unloaded.length > 0 || stale.length > 0) {
  console.error('\ncheck-loaded: FAIL — these would show as raw message ids.');
  process.exit(1);
}
console.log('check-loaded: OK — every message id is in en and its catalog is loaded where it is used.');
