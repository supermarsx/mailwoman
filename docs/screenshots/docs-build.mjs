#!/usr/bin/env node
// Build the documentation site while holding the shared docs-build lock, so two
// writers never run VitePress over the same docs/.vitepress/cache at once.
//
//   node docs/screenshots/docs-build.mjs            # output in a scratch folder
//   node docs/screenshots/docs-build.mjs --out <dir>
//
// The lock is the directory docs/.vitepress/.docs-build.lock (created atomically,
// holding the owner's pid); a lock whose owner process is gone is taken over.
// VitePress fails the build on a dead internal link, so this is also the link check.
import { spawnSync } from 'node:child_process';
import { mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { REPO, SCRATCH, log, sleep } from './lib/paths.mjs';

const DOCS = join(REPO, 'docs');
const LOCK = join(DOCS, '.vitepress', '.docs-build.lock');

function alive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return error.code === 'EPERM';
  }
}

async function takeLock() {
  let announced = false;
  for (;;) {
    try {
      mkdirSync(LOCK);
      writeFileSync(join(LOCK, 'pid'), String(process.pid));
      return () => rmSync(LOCK, { recursive: true, force: true });
    } catch {
      let owner = NaN;
      try {
        owner = Number(readFileSync(join(LOCK, 'pid'), 'utf8'));
      } catch {
        /* being written */
      }
      if (Number.isFinite(owner) && owner > 0 && !alive(owner)) {
        rmSync(LOCK, { recursive: true, force: true });
        continue;
      }
      if (!announced) log(`another docs build holds ${LOCK}; waiting`);
      announced = true;
      await sleep(3000);
    }
  }
}

const outArg = process.argv.indexOf('--out');
const out = outArg > 0 ? process.argv[outArg + 1] : join(SCRATCH, `docs-dist-${process.pid}`);
const release = await takeLock();
let status = 1;
try {
  log(`building the docs into ${out}`);
  const result = spawnSync(process.execPath, [join(DOCS, 'node_modules', 'vitepress', 'bin', 'vitepress.js'), 'build', '.', '--outDir', out, '--cacheDir', `${out}-cache`], {
    cwd: DOCS,
    stdio: 'inherit',
  });
  status = result.status ?? 1;
} finally {
  release();
  rmSync(`${out}-cache`, { recursive: true, force: true });
  if (outArg < 0) rmSync(out, { recursive: true, force: true });
}
log(status === 0 ? 'docs build passed' : `docs build FAILED (exit ${status})`);
process.exit(status);
