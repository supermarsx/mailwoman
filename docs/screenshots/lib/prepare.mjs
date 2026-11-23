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

/** The marker a finished build carries; a folder without it (and without the
 * index.html of builds made before the marker existed) is never used. */
const BUILD_INFO = 'mw-shots-build.json';

function git(...args) {
  try {
    return spawnSync('git', args, { cwd: REPO, encoding: 'utf8' }).stdout.trim();
  } catch {
    return '';
  }
}

/** Source revision of apps/web: HEAD, plus "+local changes" when the tree differs. */
function webRevision() {
  const head = git('rev-parse', '--short', 'HEAD') || 'unknown';
  const dirty = git('status', '--porcelain', '--', 'apps/web/src', 'apps/web/locales', 'apps/web/public', 'apps/web/index.html', 'apps/web/vite.config.ts', 'apps/web/package.json');
  return dirty ? `${head}+local changes` : head;
}

/**
 * What is known about the build in `dir`: { dir, hash, revision, builtAt,
 * fallback }, or null when `dir` holds no finished build.
 */
export function readBuild(dir, hash) {
  try {
    const info = JSON.parse(readFileSync(join(dir, BUILD_INFO), 'utf8'));
    return { dir, hash: info.hash ?? hash, revision: info.revision ?? 'unknown', builtAt: info.builtAt ?? null, fallback: false };
  } catch {
    // A build from before the marker: it was renamed into place only when complete.
    if (!existsSync(join(dir, 'index.html'))) return null;
    return { dir, hash, revision: 'unknown', builtAt: new Date(statSync(join(dir, 'index.html')).mtimeMs).toISOString(), fallback: false };
  }
}

function alivePid(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return error.code === 'EPERM';
  }
}

/** Take the build lock for `hash` (a folder holding the owner's pid); a lock whose owner is gone is taken over. */
async function takeBuildLock(lock, done) {
  let announced = false;
  for (;;) {
    const finished = done();
    if (finished) return { finished };
    try {
      mkdirSync(lock);
      writeFileSync(join(lock, 'pid'), String(process.pid));
      return { release: () => rmSync(lock, { recursive: true, force: true }) };
    } catch {
      let owner = NaN;
      try {
        owner = Number(readFileSync(join(lock, 'pid'), 'utf8'));
      } catch {
        /* being written; or a lock from before pids were recorded */
        try {
          if (Date.now() - statSync(lock).mtimeMs > 15 * 60_000) owner = 0;
        } catch {
          /* gone */
        }
      }
      if (owner === 0 || (Number.isFinite(owner) && owner > 0 && !alivePid(owner))) {
        log(`taking over an abandoned web build lock (${lock})`);
        rmSync(lock, { recursive: true, force: true });
        continue;
      }
      if (!announced) log(`another run is building this web source; waiting for it (${lock})`);
      announced = true;
      await sleep(3000);
    }
  }
}

/** The newest finished build other than `except`, for the fallback. */
function newestBuild(except) {
  const root = join(SCRATCH, 'web');
  return readdirSync(root, { withFileTypes: true })
    .filter((d) => d.isDirectory() && /^[0-9a-f]{16}$/.test(d.name) && d.name !== except)
    .map((d) => readBuild(join(root, d.name), d.name))
    .filter(Boolean)
    .sort((a, b) => String(b.builtAt).localeCompare(String(a.builtAt)))[0];
}

/**
 * The web client built from the current source, in SCRATCH/web/<hash> (never
 * apps/web/dist, which other work uses). Safe when several runs start at once:
 * one run per source hash builds, under a lock, into a private temporary folder
 * that is renamed into place only when complete (and only if no finished build
 * is there yet); the others wait and reuse it. `given` (--web-dir) skips the
 * build. Returns { dir, hash, revision, builtAt, fallback }.
 */
export async function ensureWeb(given) {
  if (given) {
    if (!existsSync(join(given, 'index.html'))) throw new Error(`--web-dir ${given} has no index.html`);
    const known = readBuild(given, given.split(/[\\/]/).filter(Boolean).pop());
    return { ...known, dir: given, given: true };
  }
  const root = join(SCRATCH, 'web');
  mkdirSync(root, { recursive: true });
  const hash = webSourceHash();
  const dir = join(root, hash);
  const ready = () => readBuild(dir, hash);
  const lock = await takeBuildLock(`${dir}.lock`, ready);
  if (lock.finished) return lock.finished;
  const tmp = `${dir}.tmp-${process.pid}-${Date.now()}`;
  try {
    // Another run may have finished between our check and taking the lock.
    const already = ready();
    if (already) return already;
    const revision = webRevision();
    log(`building the web client (source ${revision}, hash ${hash}) into ${dir} (about two minutes)`);
    const vite = join(WEB_APP, 'node_modules', 'vite', 'bin', 'vite.js');
    const result = spawnSync(process.execPath, [vite, 'build', '--outDir', tmp, '--emptyOutDir', '--logLevel', 'warn'], {
      cwd: WEB_APP,
      stdio: 'inherit',
    });
    if (result.status !== 0) {
      // Other work may leave the source tree mid-edit. Fall back to the newest
      // earlier build rather than failing the run, and say so.
      const previous = newestBuild(hash);
      if (!previous || process.env.MW_SHOTS_NO_FALLBACK) throw new Error('vite build of the current source failed');
      log(`WARNING: the current web source does not build; using the earlier build ${previous.hash} (source ${previous.revision}, built ${previous.builtAt})`);
      return { ...previous, fallback: true };
    }
    const info = { hash, revision, builtAt: new Date().toISOString() };
    writeFileSync(join(tmp, BUILD_INFO), `${JSON.stringify(info, null, 2)}\n`);
    if (!ready()) {
      try {
        renameSync(tmp, dir);
      } catch (error) {
        // A folder is in the way: use it if it is a finished build, else report.
        if (!ready()) throw new Error(`cannot move the web build into ${dir}: ${error.message}`);
      }
    }
    return ready();
  } finally {
    rmSync(tmp, { recursive: true, force: true });
    lock.release();
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
