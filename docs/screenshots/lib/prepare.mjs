// Inputs of a capture run: the web client built from the current source, private
// copies of the server and mock binaries, and a capture slot (at most four runs at
// once on one machine).
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, renameSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { REPO, SCRATCH, WEB_APP, log, sleep } from './paths.mjs';

const EXE = process.platform === 'win32' ? '.exe' : '';

// ── web client ─────────────────────────────────────────────────────────────────

const WEB_INPUTS = ['src', 'locales', 'public', 'index.html', 'vite.config.ts', 'package.json', 'scripts'];

function walk(dir, out) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) walk(path, out);
    else out.push(path);
  }
}

/** A hash of every web input file's path, size and modification time. */
export function webSourceHash() {
  const files = [];
  for (const input of WEB_INPUTS) {
    const path = join(WEB_APP, input);
    if (!existsSync(path)) continue;
    if (statSync(path).isDirectory()) walk(path, files);
    else files.push(path);
  }
  files.sort();
  const hash = createHash('sha256');
  for (const file of files) {
    const s = statSync(file);
    hash.update(`${relative(WEB_APP, file)}\0${s.size}\0${s.mtimeMs}\n`);
  }
  return hash.digest('hex').slice(0, 16);
}

/**
 * The directory of a web build of the current source, built on first use into
 * SCRATCH/web/<hash> (never apps/web/dist, which other work uses). `given` (from
 * --web-dir) skips the build.
 */
export async function ensureWeb(given) {
  if (given) {
    if (!existsSync(join(given, 'index.html'))) throw new Error(`--web-dir ${given} has no index.html`);
    return { dir: given, hash: 'given' };
  }
  const hash = webSourceHash();
  const dir = join(SCRATCH, 'web', hash);
  if (existsSync(join(dir, 'index.html'))) return { dir, hash };
  const lock = `${dir}.lock`;
  mkdirSync(join(SCRATCH, 'web'), { recursive: true });
  for (;;) {
    try {
      mkdirSync(lock);
      break;
    } catch {
      if (existsSync(join(dir, 'index.html'))) return { dir, hash };
      log(`waiting for another run's web build (${lock})`);
      await sleep(5000);
    }
  }
  try {
    const tmp = `${dir}.tmp-${process.pid}`;
    log(`building the web client from source into ${dir} (about two minutes)`);
    const vite = join(WEB_APP, 'node_modules', 'vite', 'bin', 'vite.js');
    const result = spawnSync(process.execPath, [vite, 'build', '--outDir', tmp, '--emptyOutDir', '--logLevel', 'warn'], {
      cwd: WEB_APP,
      stdio: 'inherit',
    });
    if (result.status !== 0) {
      rmSync(tmp, { recursive: true, force: true });
      // Other work may leave the source tree mid-edit. Fall back to the newest
      // earlier build rather than failing the run, and say so.
      const previous = readdirSync(join(SCRATCH, 'web'), { withFileTypes: true })
        .filter((d) => d.isDirectory() && /^[0-9a-f]{16}$/.test(d.name) && existsSync(join(SCRATCH, 'web', d.name, 'index.html')))
        .map((d) => ({ name: d.name, time: statSync(join(SCRATCH, 'web', d.name, 'index.html')).mtimeMs }))
        .sort((a, b) => b.time - a.time)[0];
      if (!previous || process.env.MW_SHOTS_NO_FALLBACK) throw new Error('vite build of the current source failed');
      log(`WARNING: the current web source does not build; using the previous build ${previous.name} from ${new Date(previous.time).toISOString()}`);
      return { dir: join(SCRATCH, 'web', previous.name), hash: `${previous.name} (fallback)` };
    }
    renameSync(tmp, dir);
    return { dir, hash };
  } finally {
    rmSync(lock, { recursive: true, force: true });
  }
}

// ── binaries ───────────────────────────────────────────────────────────────────

/** The first existing candidate. */
function pick(what, candidates) {
  const found = candidates.filter(Boolean).find((p) => existsSync(p));
  if (!found) throw new Error(`no ${what} binary; looked at ${candidates.filter(Boolean).join(', ')}`);
  return found;
}

/** Copy `source` once into SCRATCH/bin (keyed by size and time) and return the copy. */
function privateCopy(source) {
  const s = statSync(source);
  const dir = join(SCRATCH, 'bin');
  mkdirSync(dir, { recursive: true });
  const name = `${s.size}-${Math.round(s.mtimeMs)}-${source.split(/[\\/]/).pop()}`;
  const copy = join(dir, name);
  if (!existsSync(copy)) {
    const tmp = `${copy}.tmp-${process.pid}`;
    copyFileSync(source, tmp);
    try {
      renameSync(tmp, copy);
    } catch {
      rmSync(tmp, { force: true }); // another run copied it first
    }
  }
  return { path: copy, source, built: new Date(s.mtimeMs).toISOString() };
}

/** The server binary (MW_SHOTS_SERVER_BIN, else target/debug/mailwoman), copied. */
export function serverBinary() {
  return privateCopy(pick('server', [process.env.MW_SHOTS_SERVER_BIN, join(REPO, 'target', 'debug', `mailwoman${EXE}`)]));
}

/**
 * The mock JMAP binary (MW_SHOTS_MOCK_BIN, else target/t40/debug, the build that
 * honours Email/query text and MW_MOCK_USER), copied. target/debug's older mock
 * ignores both: every opened message then shows "Superseded by a newer message".
 */
export function mockBinary() {
  const found = privateCopy(
    pick('mock JMAP', [
      process.env.MW_SHOTS_MOCK_BIN,
      join(REPO, 'target', 't40', 'debug', `mw-mock-jmap${EXE}`),
      join(REPO, 'target', 'debug', `mw-mock-jmap${EXE}`),
    ]),
  );
  if (!found.source.includes(`${join('target', 't40')}`) && !process.env.MW_SHOTS_MOCK_BIN) {
    log('warning: using target/debug mw-mock-jmap; build the current one with: $env:CARGO_TARGET_DIR="target\\t40"; cargo build -p mw-mock-jmap');
  }
  return found;
}

// ── capture slots ──────────────────────────────────────────────────────────────

export const MAX_SLOTS = 4;

function alive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return error.code === 'EPERM';
  }
}

/**
 * Take one of MAX_SLOTS capture slots (SCRATCH/slots/<n>), waiting while all are
 * taken. A slot whose owner process is gone is reclaimed. Returns a release function.
 */
export async function takeSlot() {
  const root = join(SCRATCH, 'slots');
  mkdirSync(root, { recursive: true });
  let announced = false;
  for (;;) {
    for (let n = 1; n <= MAX_SLOTS; n += 1) {
      const slot = join(root, String(n));
      try {
        mkdirSync(slot);
      } catch {
        let owner = NaN;
        try {
          owner = Number(readFileSync(join(slot, 'pid'), 'utf8'));
        } catch {
          /* being written */
        }
        if (Number.isFinite(owner) && owner > 0 && !alive(owner)) rmSync(slot, { recursive: true, force: true });
        continue;
      }
      writeFileSync(join(slot, 'pid'), String(process.pid));
      return () => rmSync(slot, { recursive: true, force: true });
    }
    if (!announced) log(`all ${MAX_SLOTS} capture slots are taken (${root}); waiting`);
    announced = true;
    await sleep(5000);
  }
}
