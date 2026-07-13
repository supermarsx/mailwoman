// Build-time UI-bundle integrity emitter (SPEC §7.4 / plan §2.2, §3 e0/e7).
//
// Computes a deterministic SHA-256 over the built SPA (apps/web/dist) and writes
// `bundle-hash.json` into each shell's src-tauri dir. The shell records this at
// build time and verifies the loaded bundle's hash matches BEFORE pointing at any
// server (tamper gate, risk #9). e7 wires the shell-side verification
// (`verify_bundle_integrity` in the shell lib.rs) against this file.
//
// e0 ships the emitter + the frozen output shape; run it after `pnpm -C apps/web
// build` and before `tauri build`. Deterministic: files are sorted, so the hash is
// stable across machines for identical bytes.

import { createHash } from 'node:crypto';
import { readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(fileURLToPath(new URL('.', import.meta.url)), '..');
const distDir = join(root, 'apps', 'web', 'dist');
const targets = [
  join(root, 'apps', 'desktop', 'src-tauri', 'bundle-hash.json'),
  join(root, 'apps', 'mobile', 'src-tauri', 'bundle-hash.json'),
];

/** Recursively list every file under `dir`, POSIX-relative to `distDir`, sorted. */
function listFiles(dir) {
  const out = [];
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) out.push(...listFiles(full));
    else out.push(full);
  }
  return out.sort();
}

function main() {
  let files;
  try {
    files = listFiles(distDir);
  } catch {
    console.error(`emit-bundle-hash: ${distDir} not found — run \`pnpm -C apps/web build\` first.`);
    process.exit(1);
  }

  // Hash each file's relative path + bytes so both content and layout are covered.
  const overall = createHash('sha256');
  const perFile = {};
  for (const file of files) {
    const rel = relative(distDir, file).split(sep).join('/');
    const bytes = readFileSync(file);
    const fileHash = createHash('sha256').update(bytes).digest('hex');
    perFile[rel] = fileHash;
    overall.update(rel).update('\0').update(fileHash).update('\n');
  }

  const manifest = {
    algorithm: 'sha256',
    // The single value the shell compares at launch.
    bundleHash: overall.digest('hex'),
    fileCount: files.length,
    emittedAt: new Date().toISOString(),
    files: perFile,
  };
  const json = `${JSON.stringify(manifest, null, 2)}\n`;
  for (const target of targets) {
    writeFileSync(target, json);
    console.log(`emit-bundle-hash: wrote ${relative(root, target)} (${manifest.bundleHash.slice(0, 16)}…, ${manifest.fileCount} files)`);
  }
}

main();
