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

import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
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

let tauriDriver;

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
    if (nativeDriver) args.push('--native-driver', nativeDriver);
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
    // Give tauri-driver a moment to bind :4444 before the session connects.
    return new Promise((resolve) => setTimeout(resolve, 2_000));
  },

  onComplete() {
    if (tauriDriver) tauriDriver.kill();
  },
};
