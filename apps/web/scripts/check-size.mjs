// Gzipped entry-bundle budget gate (SPEC §5.2: < 250 KB gzip).
// Run after `vite build`. Exits non-zero if the largest JS entry exceeds budget.
import { readdir, readFile, stat } from 'node:fs/promises';
import { gzipSync } from 'node:zlib';
import { join } from 'node:path';

const BUDGET_BYTES = 250 * 1024;
const assetsDir = new URL('../dist/assets/', import.meta.url);

let entries;
try {
  entries = await readdir(assetsDir);
} catch {
  console.error('check-size: dist/assets not found — run `pnpm build` first');
  process.exit(1);
}

const jsFiles = entries.filter((f) => f.endsWith('.js'));
if (jsFiles.length === 0) {
  console.error('check-size: no JS assets found in dist/assets');
  process.exit(1);
}

let worst = 0;
let worstName = '';
for (const f of jsFiles) {
  const path = join(assetsDir.pathname.replace(/^\/(?=[A-Za-z]:)/, ''), f);
  const buf = await readFile(path);
  const gz = gzipSync(buf).length;
  const raw = (await stat(path)).size;
  console.log(`  ${f}: ${(raw / 1024).toFixed(1)} KB raw, ${(gz / 1024).toFixed(1)} KB gzip`);
  if (gz > worst) {
    worst = gz;
    worstName = f;
  }
}

console.log(
  `check-size: largest entry ${worstName} = ${(worst / 1024).toFixed(1)} KB gzip (budget ${BUDGET_BYTES / 1024} KB)`,
);
if (worst > BUDGET_BYTES) {
  console.error('check-size: OVER BUDGET');
  process.exit(1);
}
console.log('check-size: OK');
