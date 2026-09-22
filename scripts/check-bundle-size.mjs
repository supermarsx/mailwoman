// Bundle-size budget gate (SPEC §16 / plan §3 e7, e8 asserts it in CI).
//
// Budgets: the THIN desktop shell (SPA + Tauri runtime, NO engine) < 21 MB; the
// SELF-CONTAINED desktop (thin shell + the bundled sibling `mw-server`) < 126 MB.
// These are the 26.20 measured revision of the original 10 MB / 40 MB numbers —
// measured × 1.15, the same policy §23 got in 26.9. Basis and measurements:
// docs/perf/size-budget-revision.md.
// The thin shell carries only the SPA + WebView glue; the engine appears solely as
// the bundled `mw-server` resource in self-contained mode.
//
// This measures the built desktop binary (after `tauri build --no-bundle`) and, when
// the self-contained resource is present, the binary + bundled server. e7 uses it in
// build-shells as a fast local check; e8 runs it as the authoritative CI gate.

import { statSync, existsSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(fileURLToPath(new URL('.', import.meta.url)), '..');
const MB = 1024 * 1024;
const THIN_BUDGET = 21 * MB;
const SELF_CONTAINED_BUDGET = 126 * MB;

const exeCandidates = [
  join(root, 'target', 'release', 'mailwoman-desktop.exe'),
  join(root, 'target', 'release', 'mailwoman-desktop'),
];
const exe = exeCandidates.find((p) => existsSync(p));
if (!exe) {
  console.error(
    `check-bundle-size: no desktop binary found in target/release — run \`tauri build\` first.`,
  );
  process.exit(1);
}

const exeSize = statSync(exe).size;

// The bundled sibling server (self-contained mode) lives in the shell resources.
const resDir = join(root, 'apps', 'desktop', 'src-tauri', 'resources');
let serverSize = 0;
if (existsSync(resDir)) {
  for (const name of readdirSync(resDir)) {
    if (/^mw-server(\.exe)?$/.test(name)) {
      serverSize = statSync(join(resDir, name)).size;
    }
  }
}

const fmt = (n) => `${(n / MB).toFixed(2)} MB`;
let failed = false;

console.log(`check-bundle-size: thin desktop binary ${fmt(exeSize)} (budget ${fmt(THIN_BUDGET)})`);
if (exeSize > THIN_BUDGET) {
  console.error(`  ✗ thin shell exceeds the §16 ${fmt(THIN_BUDGET)} budget`);
  failed = true;
}

if (serverSize > 0) {
  const total = exeSize + serverSize;
  console.log(
    `check-bundle-size: self-contained (shell + mw-server ${fmt(serverSize)}) = ${fmt(total)} (budget ${fmt(SELF_CONTAINED_BUDGET)})`,
  );
  if (total > SELF_CONTAINED_BUDGET) {
    console.error(`  ✗ self-contained bundle exceeds the §16 ${fmt(SELF_CONTAINED_BUDGET)} budget`);
    failed = true;
  }
} else {
  console.log(`check-bundle-size: no bundled mw-server resource (thin build) — self-contained budget N/A`);
}

process.exit(failed ? 1 : 0);
