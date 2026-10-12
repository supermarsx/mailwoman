// WebdriverIO config for the Mailwoman desktop shell live E2E (plan §3 e9).
//
// This drives the REAL built desktop app through `tauri-driver`, the official
// Tauri WebDriver bridge (a thin proxy in front of the platform's native
// WebView driver: MSEdgeDriver on Windows/WebView2, WebKitWebDriver on Linux).
// The specs assert the shell boots past the §7.4 integrity gate, loads the
// shared SPA, injects `__MW_CONFIG__`, and that every `mw_*` capability command
// the SPA's `platform/tauri.ts` invokes actually round-trips against the OS.
//
// PREREQUISITES (documented for CI in ./README.md):
//   1. A built desktop binary: `target/{release,debug}/mailwoman-desktop.exe`
//      (built by `scripts/build-shells.*` or `cargo build -p mailwoman-desktop`).
//      Override with MW_DESKTOP_BIN.
//   2. `tauri-driver` on PATH or at ~/.cargo/bin (`cargo install tauri-driver`).
//      Override with TAURI_DRIVER.
//   3. A native WebDriver matching the installed WebView:
//        - Windows: `msedgedriver.exe` whose version matches the WebView2
//          runtime (e.g. 146.x). Point MSEDGEDRIVER at it (tauri-driver's
//          `--native-driver`), or put it on PATH.
//        - Linux:   `WebKitWebDriver` on PATH (tauri-driver finds it).
//
// tauri-driver is spawned once in onPrepare and torn down in onComplete; it
// launches/quits a fresh app process per WebDriver session.
//
// OPTIONAL (Windows; README "When the session does not open"):
//   MW_E2E_WEBVIEW_USER_DATA_FOLDER  the WebView2 user data folder msedgedriver reads the
//                                    shell's debug port from, in place of a temporary one.

import { spawn } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
// apps/desktop/e2e -> repo root
const repoRoot = path.resolve(__dirname, '..', '..', '..');

const isWin = process.platform === 'win32';
const exeName = isWin ? 'mailwoman-desktop.exe' : 'mailwoman-desktop';

/** Resolve the built desktop binary (prefer release, fall back to debug). */
function resolveApplication() {
  if (process.env.MW_DESKTOP_BIN) return process.env.MW_DESKTOP_BIN;
  for (const profile of ['release', 'debug']) {
    const candidate = path.join(repoRoot, 'target', profile, exeName);
    if (existsSync(candidate)) return candidate;
  }
  throw new Error(
    `desktop binary not found under target/{release,debug}/${exeName}; ` +
      `build it (scripts/build-shells) or set MW_DESKTOP_BIN`,
  );
}

/** Resolve tauri-driver. */
function resolveTauriDriver() {
  if (process.env.TAURI_DRIVER) return process.env.TAURI_DRIVER;
  const bin = isWin ? 'tauri-driver.exe' : 'tauri-driver';
  return path.resolve(os.homedir(), '.cargo', 'bin', bin);
}

const application = resolveApplication();
const tauriDriverBin = resolveTauriDriver();
// MSEDGEDRIVER (Windows) -> tauri-driver `--native-driver`. On Linux the native
// WebKitWebDriver is found on PATH, so this stays empty.
const nativeDriver = process.env.MSEDGEDRIVER;
// msedgedriver `webviewOptions.userDataFolder`. Unset, the driver creates a temporary
// folder and hands it to the shell through WEBVIEW2_USER_DATA_FOLDER.
const webviewUserDataFolder = process.env.MW_E2E_WEBVIEW_USER_DATA_FOLDER;

// Session-start diagnostics (Windows). `logs/` is git-ignored.
const verbose = isWin && Boolean(nativeDriver);
const logsDir = path.join(__dirname, 'logs');
const driverLog = path.join(logsDir, 'msedgedriver.log');
const snapshotScript = path.join(__dirname, 'diagnostics', 'session-snapshot.ps1');
// msedgedriver waits 60 s for the shell's debug port; a healthy session opens in a few
// seconds. One snapshot inside that wait shows what the shell is doing instead.
const SNAPSHOT_DELAY_MS = 25_000;
const CRLF = String.fromCharCode(13, 10);

let tauriDriver;
let snapshotTimer;
let snapshot = '';

/**
 * tauri-driver owns the msedgedriver process, so msedgedriver's verbose log is obtained
 * by handing tauri-driver a wrapper script as the native driver.
 */
function verboseNativeDriver(driver) {
  mkdirSync(logsDir, { recursive: true });
  const wrapper = path.join(logsDir, 'msedgedriver-verbose.cmd');
  const run = `"${driver}" --verbose --readable-timestamp --log-path="${driverLog}" %*`;
  writeFileSync(wrapper, ['@echo off', run, ''].join(CRLF));
  return wrapper;
}

/** Record the shell's processes, windows and WebView2 state (diagnostics/session-snapshot.ps1). */
function takeSnapshot() {
  const ps = spawn(
    'powershell.exe',
    ['-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', snapshotScript, '-ExeName', exeName],
    { stdio: ['ignore', 'pipe', 'pipe'] },
  );
  ps.stdout.on('data', (chunk) => (snapshot += chunk));
  ps.stderr.on('data', (chunk) => (snapshot += chunk));
  ps.on('error', (err) => (snapshot += `snapshot failed to start: ${err.message}`));
}

/** Print what a failed run recorded about session start. Quiet on a passing run. */
function printSessionDiagnostics() {
  const seconds = SNAPSHOT_DELAY_MS / 1000;
  const out = ['---- session diagnostics (README: "When the session does not open") ----'];
  if (snapshot) {
    out.push(`${seconds} s after tauri-driver started:`);
    out.push(...snapshot.trimEnd().split(/\r?\n/).map((line) => `  ${line}`));
  } else {
    out.push(`no snapshot: the run ended within ${seconds} s of tauri-driver starting`);
  }
  if (verbose && existsSync(driverLog)) {
    // Top-level entries only; request and response bodies are the indented lines.
    const entries = readFileSync(driverLog, 'utf8')
      .split(/\r?\n/)
      .filter((line) => line.startsWith('[') && !line.includes('Populating '));
    const shown = entries.slice(0, 30);
    out.push(`msedgedriver log, first ${shown.length} of ${entries.length} entries (${driverLog}):`);
    out.push(...shown.map((line) => `  ${line.slice(0, 600)}`));
  } else {
    out.push('no msedgedriver log: it is written only when MSEDGEDRIVER names the driver');
  }
  for (const line of out) {
    // eslint-disable-next-line no-console
    console.log(`[e2e] ${line}`);
  }
}

export const config = {
  runner: 'local',
  hostname: '127.0.0.1',
  port: 4444,
  path: '/',

  specs: ['./specs/**/*.e2e.mjs'],
  maxInstances: 1,

  capabilities: [
    {
      // tauri-driver reads `tauri:options.application` to launch the shell.
      'tauri:options': {
        application,
        // tauri-driver forwards this as msedgedriver's `ms:edgeOptions.webviewOptions`.
        ...(webviewUserDataFolder
          ? { webviewOptions: { userDataFolder: webviewUserDataFolder } }
          : {}),
      },
    },
  ],

  logLevel: 'warn',
  bail: 0,
  waitforTimeout: 20_000,
  connectionRetryTimeout: 120_000,
  connectionRetryCount: 3,

  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    // Native launch + real OS calls (keychain/DPAPI, capture, a spawned
    // mw-server) are slower than a browser test; keep a generous ceiling.
    timeout: 120_000,
  },

  // Spawn tauri-driver once for the run; it manages the app process per session.
  onPrepare() {
    const args = [];
    if (nativeDriver) {
      args.push('--native-driver', verbose ? verboseNativeDriver(nativeDriver) : nativeDriver);
    }
    if (verbose) {
      // eslint-disable-next-line no-console
      console.log(`[e2e] msedgedriver verbose log: ${driverLog}`);
    }
    if (webviewUserDataFolder) {
      // eslint-disable-next-line no-console
      console.log(`[e2e] WebView2 user data folder: ${webviewUserDataFolder}`);
    }
    // eslint-disable-next-line no-console
    console.log(
      `[e2e] launching tauri-driver (${tauriDriverBin})` +
        (nativeDriver ? ` --native-driver ${nativeDriver}` : '') +
        `\n[e2e] application: ${application}`,
    );
    tauriDriver = spawn(tauriDriverBin, args, {
      stdio: [null, process.stdout, process.stderr],
    });
    tauriDriver.on('error', (err) => {
      // eslint-disable-next-line no-console
      console.error('[e2e] tauri-driver failed to start:', err);
      process.exit(1);
    });
    if (isWin) snapshotTimer = setTimeout(takeSnapshot, SNAPSHOT_DELAY_MS);
    // Give tauri-driver a moment to bind :4444 before the session connects.
    return new Promise((resolve) => setTimeout(resolve, 2_000));
  },

  onComplete(exitCode) {
    clearTimeout(snapshotTimer);
    if (tauriDriver) tauriDriver.kill();
    if (isWin && exitCode !== 0) printSessionDiagnostics();
  },
};
