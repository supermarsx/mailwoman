// Live desktop-shell E2E (plan §3 e9 / §7 DoD). Runs against the REAL built
// Mailwoman desktop app through tauri-driver + WebView2 (Windows). One WebDriver
// session, ordered assertions:
//
//   1. boot        — the shell launches (implying the §7.4 integrity gate passed,
//                    since a mismatch aborts launch), the shared SPA mounts, and
//                    the frozen `__MW_CONFIG__` handshake is present (desktop /
//                    native / capabilities).
//   2. keychain    — the OS keychain (DPAPI on Windows) round-trips through the
//                    `mw_keychain_*` commands: set -> get -> delete -> get(null).
//   3. capture     — `mw_set_capture_protection({enabled})` reports {supported:true}
//                    on Windows (WDA_EXCLUDEFROMCAPTURE), then disables cleanly.
//   4. notification— `mw_notify(...)` fires a native toast without error (the
//                    action-button round-trip is documented in README as OS-limited
//                    on desktop; the choke-point event bridge is wired).
//   5. badge       — `mw_set_badge_count` accepts a count and clears to 0.
//   6. self-cont.  — `mw_self_contained_status` is "off" at boot; if the bundled
//                    mw-server resource is present, `mw_start_local_server` spawns it
//                    on loopback (blocking until /healthz 200) and status -> "ready",
//                    then stop -> "off". (Otherwise skipped with a clear note; the
//                    Rust integration test proves the spawn live regardless.)
//
// The SPA reaches the OS through Tauri's IPC; we invoke the same `mw_*` commands
// the SPA's platform/tauri.ts invokes, via `window.__TAURI_INTERNALS__.invoke`.

import assert from 'node:assert/strict';

/**
 * Invoke a Tauri command in the WebView exactly as the SPA does, awaiting the
 * IPC Promise. Returns `{ ok }` on success or throws with the command's error.
 */
async function invoke(command, args = {}) {
  const result = await browser.executeAsync(
    (command, args, done) => {
      const internals = window.__TAURI_INTERNALS__;
      if (!internals || typeof internals.invoke !== 'function') {
        done({ __err: 'window.__TAURI_INTERNALS__.invoke is unavailable' });
        return;
      }
      Promise.resolve(internals.invoke(command, args)).then(
        (value) => done({ __ok: value }),
        (err) => done({ __err: String(err && err.message ? err.message : err) }),
      );
    },
    command,
    args,
  );
  if (result && Object.prototype.hasOwnProperty.call(result, '__ok')) return result.__ok;
  throw new Error(`invoke(${command}) failed: ${result ? result.__err : 'no result'}`);
}

describe('Mailwoman desktop shell — live (tauri-driver / WebView2)', () => {
  before(async () => {
    // The SPA mounts into #root; wait for it to render before driving anything.
    await browser.waitUntil(
      async () =>
        browser.execute(() => {
          const root = document.getElementById('root');
          return !!root && root.children.length > 0;
        }),
      { timeout: 30_000, timeoutMsg: 'SPA did not mount into #root' },
    );
  });

  it('boots past the integrity gate, loads the SPA, and injects __MW_CONFIG__', async () => {
    // The window only exists because verify_bundle_integrity() passed in setup
    // (a mismatch/missing asset returns Err and aborts launch, §7.4 / risk #9).
    assert.equal(await browser.getTitle(), 'Mailwoman');

    const rootChildren = await browser.execute(
      () => document.getElementById('root')?.children.length ?? 0,
    );
    assert.ok(rootChildren > 0, 'SPA #root has rendered content');

    const cfg = await browser.execute(() => window.__MW_CONFIG__ ?? null);
    assert.ok(cfg, '__MW_CONFIG__ present');
    assert.equal(cfg.platform.kind, 'desktop', 'platform.kind === desktop');
    assert.equal(cfg.native, true, 'native === true');
    assert.equal(cfg.capabilities, true, 'capabilities === true');
    assert.ok(typeof cfg.platform.os === 'string' && cfg.platform.os.length > 0);
  });

  it('round-trips the OS keychain (DPAPI) via mw_keychain_*', async () => {
    const service = 'mailwoman.e2e';
    const key = 'e2e-token';
    const value = `secret-${Date.now()}`;

    await invoke('mw_keychain_set', { service, key, value });
    const got = await invoke('mw_keychain_get', { service, key });
    assert.equal(got, value, 'keychain get returns the stored secret');

    await invoke('mw_keychain_delete', { service, key });
    const afterDelete = await invoke('mw_keychain_get', { service, key });
    assert.equal(afterDelete, null, 'keychain get returns null after delete');
  });

  it('enables + disables screen-capture protection (Windows WDA_EXCLUDEFROMCAPTURE)', async () => {
    const on = await invoke('mw_set_capture_protection', { enabled: true });
    assert.ok(on && typeof on.supported === 'boolean', 'returns { supported }');
    // On Windows desktop the native content-protection is real → supported:true.
    // (Linux/browser honestly report false and keep the V4 watermark.)
    if (process.platform === 'win32') {
      assert.equal(on.supported, true, 'capture protection supported on Windows');
    }
    const off = await invoke('mw_set_capture_protection', { enabled: false });
    assert.ok(off && typeof off.supported === 'boolean');
  });

  it('fires a native notification via mw_notify', async () => {
    // Resolves (the toast is dispatched to the OS). The action-button delivery
    // back to the SPA is OS/plugin-limited on desktop (see README); the notify
    // path itself is the live acceptance.
    await invoke('mw_notify', {
      input: {
        id: 'e2e-note',
        title: 'Mailwoman E2E',
        body: 'live notification from the desktop shell',
        actions: [],
      },
    });
  });

  it('accepts a badge count via mw_set_badge_count', async () => {
    await invoke('mw_set_badge_count', { count: 3 });
    await invoke('mw_set_badge_count', { count: 0 });
  });

  // Regular function (not an arrow) so Mocha's `this.skip()` is available when the
  // bundled server resource is absent (e.g. a dev build with no resources/ dir).
  it('self-contained mode: spawns the bundled mw-server on loopback (or documents the resource gap)', async function () {
    const initial = await invoke('mw_self_contained_status');
    assert.equal(initial, 'off', 'self-contained status is off at boot');

    let url;
    try {
      url = await invoke('mw_start_local_server');
    } catch (err) {
      // The non-bundled dev binary has no resources/mw-server next to it; the
      // Rust integration test (cargo test -p mailwoman-desktop) proves the live
      // spawn + /healthz + JMAP round-trip. Skip here rather than fake it.
      if (/mw-server (binary )?(not found|missing)|cannot resolve bundled/i.test(String(err))) {
        // eslint-disable-next-line no-console
        console.log(`[e2e] self-contained spawn SKIPPED (bundled server absent): ${err}`);
        this.skip();
        return;
      }
      throw err;
    }

    // eslint-disable-next-line no-console
    console.log(`[e2e] self-contained mw-server LIVE on ${url}`);
    assert.match(url, /^http:\/\/127\.0\.0\.1:\d+$/, 'returns a loopback URL');
    const ready = await invoke('mw_self_contained_status');
    assert.equal(ready, 'ready', 'status is ready after a successful start');

    await invoke('mw_stop_local_server');
    const stopped = await invoke('mw_self_contained_status');
    assert.equal(stopped, 'off', 'status is off after stop');
  });
});
