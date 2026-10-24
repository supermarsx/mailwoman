// Live desktop-shell E2E (plan §3 e9 / §7 DoD). Runs against the REAL built
// Mailwoman desktop app through tauri-driver + WebView2 (Windows). One WebDriver
// session, ordered assertions:
//
//   1. boot        — the shell launches (implying the §7.4 integrity gate passed,
//                    since a mismatch aborts launch), the shared SPA reaches its
//                    first screen on a fresh profile — the sign-in form of the
//                    server compiled into the build (MW_SHELL_SERVER_URL), with no
//                    server-address field or any other server input (t32), not the
//                    boot-error card or an empty #root — and the frozen
//                    `__MW_CONFIG__` handshake is present (desktop / native /
//                    capabilities / serverUrl).
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
//   7. unreachable — with the server stopped, a reload shows the boot-error card
//                    with Retry and "contact your administrator", and no Change
//                    server control or server field (t32).
//
// The SPA reaches the OS through Tauri's IPC; we invoke the same `mw_*` commands
// the SPA's platform/tauri.ts invokes, via `window.__TAURI_INTERNALS__.invoke`.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

// The server the app was built for; wdio.conf.mjs starts it there.
const SERVER_URL = process.env.MW_E2E_SERVER_URL || 'http://127.0.0.1:37930';

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

/** True once the SPA has rendered into #root. */
async function spaMounted() {
  return browser.execute(() => {
    const root = document.getElementById('root');
    return !!root && root.children.length > 0;
  });
}

/**
 * Which screen the SPA is showing. A fresh profile is signed out, so the first
 * screen is the sign-in form (Login.tsx) of the server compiled into the build. It
 * must have no server-address field (t32): `serverField` and `serverInputs` list
 * anything that looks like one. The boot-error card (AsyncState.tsx,
 * `.async-state--error`) means the SPA mounted and then failed to start; children
 * in #root alone cannot tell the two apart.
 */
async function firstScreen() {
  return browser.execute(() => {
    const root = document.getElementById('root');
    if (!root || root.children.length === 0) return { kind: 'empty' };
    const notConfigured = root.querySelector('[data-testid="shell-not-configured"]');
    if (notConfigured) return { kind: 'not-configured', text: notConfigured.textContent.trim() };
    const form = root.querySelector('form.login__card');
    if (form) {
      const inputs = [...form.querySelectorAll('input')];
      const labelOf = (i) => (i.closest('label')?.textContent ?? '').trim();
      return {
        kind: 'login',
        visible: form.getClientRects().length > 0,
        serverField: !!root.querySelector('[data-testid="login-server-address"]'),
        inputTypes: inputs.map((i) => i.type).sort(),
        serverInputs: inputs
          .filter((i) => i.type === 'url' || /server|url|https?:/i.test(`${labelOf(i)} ${i.placeholder}`))
          .map((i) => `${i.type} ${labelOf(i)} ${i.placeholder}`.trim()),
      };
    }
    const error = root.querySelector('.async-state--error');
    if (error) {
      return {
        kind: 'error',
        text: error.textContent.trim().slice(0, 200),
        contactAdmin: root.querySelector('[data-testid="boot-contact-administrator"]')?.textContent.trim() ?? null,
        changeServer: !!root.querySelector('[data-testid="boot-change-server"]'),
        serverField: !!root.querySelector('[data-testid="login-server-address"]'),
        buttons: [...root.querySelectorAll('button')].map((b) => b.getAttribute('data-testid') || b.textContent.trim()),
        lang: document.documentElement.lang,
      };
    }
    if (root.querySelector('[data-testid="async-pending"]')) return { kind: 'pending' };
    return { kind: 'other', text: root.textContent.trim().slice(0, 200) };
  });
}

/**
 * What the WebView is showing instead of the SPA. CI run 39895491554 committed
 * `http://tauri.localhost/` as an empty-looking `text/plain` document while a service
 * worker from an earlier launch was registered; the driver log cannot say who answered.
 * This records it: the HTTP status and delivery of the navigation (`responseStatus` 400
 * with no body is wry rejecting the request; 200 is the shell's asset handler), the
 * document's MIME type and start, and the service worker in control.
 */
async function documentState() {
  return browser.executeAsync((done) => {
    const nav = performance.getEntriesByType('navigation')[0];
    const sw = navigator.serviceWorker;
    const state = {
      url: location.href,
      contentType: document.contentType,
      readyState: document.readyState,
      title: document.title,
      rootPresent: !!document.getElementById('root'),
      html: document.documentElement ? document.documentElement.outerHTML.slice(0, 400) : null,
      navigation: nav
        ? {
            responseStatus: nav.responseStatus,
            deliveryType: nav.deliveryType,
            workerStart: nav.workerStart,
            transferSize: nav.transferSize,
            encodedBodySize: nav.encodedBodySize,
            decodedBodySize: nav.decodedBodySize,
          }
        : null,
      serviceWorkerController: sw && sw.controller ? sw.controller.scriptURL : null,
    };
    if (!sw) {
      done(state);
      return;
    }
    sw.getRegistrations().then(
      (regs) => done({ ...state, serviceWorkerRegistrations: regs.map((r) => r.scope) }),
      (err) => done({ ...state, serviceWorkerRegistrations: `error: ${err}` }),
    );
  });
}

/**
 * Separates "the service worker's answer is wrong" from "the shell's answer is wrong":
 * unregister every worker, reload, and report whether the SPA mounts without one. The
 * test still fails afterwards; this only prints the result.
 */
async function mountsWithoutServiceWorker() {
  const removed = await browser.executeAsync((done) => {
    const sw = navigator.serviceWorker;
    if (!sw) {
      done(0);
      return;
    }
    sw.getRegistrations()
      .then((regs) => Promise.all(regs.map((r) => r.unregister())).then(() => regs.length))
      .then(done, () => done(-1));
  });
  if (removed <= 0) return { removed, mounted: null };
  await browser.execute(() => location.reload());
  const mounted = await browser
    .waitUntil(spaMounted, { timeout: 15_000 })
    .then(() => true, () => false);
  const after = await documentState().catch((e) => `unavailable: ${e.message}`);
  return { removed, mounted, after };
}

describe('Mailwoman desktop shell — live (tauri-driver / WebView2)', () => {
  before(async () => {
    // Wait for the first screen before driving anything. The boot-error card is final
    // (it waits for Retry), so it ends the wait as early as the sign-in form does.
    let screen = { kind: 'empty' };
    try {
      await browser
        .waitUntil(
          async () => {
            screen = await firstScreen();
            return screen.kind === 'login' || screen.kind === 'error';
          },
          { timeout: 30_000 },
        )
        .catch(() => undefined);
      if (screen.kind === 'empty') throw new Error('SPA did not mount into #root');
      if (screen.kind !== 'login') {
        throw new Error(`SPA did not reach the sign-in form of ${SERVER_URL}: ${JSON.stringify(screen)}`);
      }
    } catch (err) {
      const state = await documentState().catch((e) => `unavailable: ${e.message}`);
      // eslint-disable-next-line no-console
      console.log(`[e2e] document instead of the SPA: ${JSON.stringify(state, null, 2)}`);
      const retry = await mountsWithoutServiceWorker().catch((e) => `unavailable: ${e.message}`);
      // eslint-disable-next-line no-console
      console.log(`[e2e] after unregistering service workers: ${JSON.stringify(retry, null, 2)}`);
      throw err;
    }
  });

  it('boots past the integrity gate, loads the SPA, and injects __MW_CONFIG__', async () => {
    // The window only exists because verify_bundle_integrity() passed in setup
    // (a mismatch/missing asset returns Err and aborts launch, §7.4 / risk #9).
    assert.equal(await browser.getTitle(), 'Mailwoman');

    const screen = await firstScreen();
    assert.equal(screen.kind, 'login', `first screen is the sign-in form: ${JSON.stringify(screen)}`);
    assert.equal(screen.visible, true, 'the sign-in form is rendered');
    // t32: the server is fixed by the build; the user cannot enter one.
    assert.equal(screen.serverField, false, 'no [data-testid=login-server-address] field');
    assert.deepEqual(screen.serverInputs, [], 'no server or URL input of any kind');
    // The address field is `type=text` (a deployment may sign in with a bare user name, t32-h9).
    assert.deepEqual(screen.inputTypes, ['password', 'text'], 'only the email address and the password');

    const cfg = await browser.execute(() => window.__MW_CONFIG__ ?? null);
    assert.ok(cfg, '__MW_CONFIG__ present');
    assert.equal(cfg.platform.kind, 'desktop', 'platform.kind === desktop');
    assert.equal(cfg.native, true, 'native === true');
    assert.equal(cfg.capabilities, true, 'capabilities === true');
    assert.equal(cfg.serverUrl, SERVER_URL, 'serverUrl is the address compiled into the build');
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

  // Last: it stops the server for the rest of the session.
  it('an unreachable server shows Retry and the administrator line, with no way to change the server', async () => {
    const pidFile = process.env.MW_E2E_SERVER_PID_FILE;
    assert.ok(pidFile, 'wdio.conf.mjs recorded the server pid');
    process.kill(Number(readFileSync(pidFile, 'utf8')));
    await browser.waitUntil(
      () => fetch(`${SERVER_URL}/healthz`).then(() => false, () => true),
      { timeout: 15_000, timeoutMsg: 'the server still answers after being stopped' },
    );

    await browser.execute(() => location.reload());
    let screen = { kind: 'empty' };
    await browser.waitUntil(
      async () => {
        screen = await firstScreen();
        return screen.kind === 'error';
      },
      { timeout: 30_000, timeoutMsg: 'no boot-error card' },
    ).catch(() => undefined);
    assert.equal(screen.kind, 'error', `the boot-error card: ${JSON.stringify(screen)}`);
    assert.deepEqual(screen.buttons, ['async-retry'], 'Retry is the only control');
    assert.equal(screen.changeServer, false, 'no Change server control');
    assert.equal(screen.serverField, false, 'no server-address field');
    assert.ok(screen.contactAdmin, 'the administrator line is shown');
    if (screen.lang === 'en') {
      assert.equal(screen.contactAdmin, 'If this keeps happening, contact your administrator.');
    }
  });
});
