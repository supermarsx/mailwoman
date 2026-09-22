import { test, expect } from '@playwright/test';

/**
 * Trusted Types × Web Workers, under the CSP mw-server actually sends
 * (26.20 t24-e13).
 *
 * ── Why this spec exists ───────────────────────────────────────────────────
 * 26.17 shipped `require-trusted-types-for 'script'` alongside a `default` policy
 * that exposed only `createHTML`. `new Worker(url)` takes a **TrustedScriptURL**,
 * so with no `createScriptURL` on the default policy the browser refused every
 * worker the app constructs:
 *
 *   Failed to construct 'Worker': This document requires 'TrustedScriptURL'
 *   assignment and no 'default' policy for 'TrustedScriptURL' has been defined.
 *
 * PGP, S/MIME, the in-worker sanitizer, the PDF.js viewer worker and the offline
 * service worker were all dead in the shipped app, and NOTHING caught it: vitest
 * runs in jsdom and the vite dev server sends no CSP, so the defect is only
 * reachable through the app as mw-server serves it. That is precisely what this
 * spec does — every assertion below runs in a real Chromium against :8090.
 *
 * The first test asserts the CSP header is actually present and enforcing, so the
 * rest cannot pass vacuously against a server that stopped sending it.
 *
 * Lives in the `crypto` project (`crypto-*.spec.ts` → the CI `e2e-crypto` job),
 * next to the features that were broken.
 */

/** The `TrustedScriptURL` sink, exercised in the page. Resolves to `'ok'` or the
 *  thrown error's message — never rejects, so a refusal is assertable text. */
const CONSTRUCT_WORKER = (url: string): string => `
  (() => {
    try {
      const w = new Worker(${JSON.stringify(url)});
      w.terminate();
      return 'ok';
    } catch (e) {
      return String(e && e.message ? e.message : e);
    }
  })()
`;

test.describe('Trusted Types default policy (served CSP)', () => {
  test('the shell is served with Trusted Types ENFORCED', async ({ page }) => {
    const res = await page.goto('/');
    expect(res, 'the shell must be reachable').not.toBeNull();
    const csp = res?.headers()['content-security-policy'] ?? '';
    // If this ever stops matching, every other assertion here is vacuous.
    expect(csp, 'shell CSP must enforce Trusted Types').toContain(
      "require-trusted-types-for 'script'",
    );
    // The sibling directives the worker path depends on.
    expect(csp).toContain("script-src 'self'");
    expect(csp).toContain('worker-src');
  });

  test('the app boots with NO TrustedScriptURL error', async ({ page }) => {
    const pageErrors: string[] = [];
    page.on('pageerror', (e) => pageErrors.push(e.message));
    // Collect CSP violations from the page itself; `securitypolicyviolation` is
    // the only way to see a refused INLINE STYLE, which raises no page error and
    // no console error in every browser (t24-e12 found ours only in a trace).
    await page.addInitScript(() => {
      (globalThis as unknown as { __mwCspViolations: string[] }).__mwCspViolations = [];
      document.addEventListener('securitypolicyviolation', (e) => {
        (globalThis as unknown as { __mwCspViolations: string[] }).__mwCspViolations.push(
          `${e.violatedDirective} @ ${e.sourceFile}:${e.lineNumber}`,
        );
      });
    });
    await page.goto('/');
    // The SPA rendered at all — proves createHTML is in place for Solid's boot
    // `template().innerHTML` write.
    await expect(page.locator('#root')).not.toBeEmpty();
    // …and the default policy is installed with the callbacks the app needs.
    //
    // These CALL the callbacks rather than checking `typeof p.createX === 'function'`.
    // A TrustedTypePolicy exposes all three names on its prototype whichever
    // callbacks were supplied, and an unsupplied one throws when invoked — so a
    // shape check passes identically before and after this fix and proves nothing.
    // Behaviour is the only thing that distinguishes them.
    const behaviour = await page.evaluate(() => {
      const tt = (window as unknown as { trustedTypes?: { defaultPolicy: unknown } }).trustedTypes;
      if (tt === undefined) return { present: false, html: '', scriptUrl: '', scriptThrew: false };
      const p = tt.defaultPolicy as {
        createHTML(s: string): unknown;
        createScriptURL(s: string): unknown;
        createScript(s: string): unknown;
      } | null;
      if (p === null) return { present: false, html: '', scriptUrl: '', scriptThrew: false };
      const call = (f: () => unknown): string => {
        try {
          return String(f());
        } catch (e) {
          return `THREW: ${String(e)}`;
        }
      };
      let scriptThrew = false;
      try {
        p.createScript('globalThis.__mw_tt_eval = 1');
      } catch {
        scriptThrew = true;
      }
      return {
        present: true,
        html: call(() => p.createHTML('<b>ok</b>')),
        scriptUrl: call(() => p.createScriptURL('/assets/worker.entry-test.js')),
        scriptThrew,
      };
    });
    expect(behaviour.present, 'a default policy must be registered').toBe(true);
    expect(behaviour.html, 'createHTML passes app HTML through').toBe('<b>ok</b>');
    expect(behaviour.scriptUrl, 'createScriptURL is what unblocks Web Workers').toBe(
      '/assets/worker.entry-test.js',
    );
    // Still fail-closed: the app has no string-to-code sink and must not gain one.
    expect(behaviour.scriptThrew, 'createScript must stay unsupplied (no eval sink)').toBe(true);

    expect(
      pageErrors.filter((m) => /TrustedScriptURL|Trusted Type/i.test(m)),
      'boot must raise no Trusted Types error',
    ).toEqual([]);

    // No directive of the shell CSP may be violated by the app's own boot. The
    // `style-src` half of this caught nothing before the fix only because the
    // offending screens are behind login — the authoritative guard for literal
    // style attributes across the WHOLE bundle is gate 3 of
    // `apps/web/scripts/check-size.mjs`, which reads the built output.
    const violations = await page.evaluate(
      () => (globalThis as unknown as { __mwCspViolations: string[] }).__mwCspViolations,
    );
    expect(violations, 'the app must not violate its own CSP at boot').toEqual([]);
  });

  test('a Worker from an app-owned script URL is CONSTRUCTED', async ({ page }) => {
    await page.goto('/');
    // `/sw.js` is one of the two root-served app scripts the policy allows, and is
    // a real, same-origin, loadable classic worker script — so this exercises the
    // whole sink (policy → TrustedScriptURL → Worker construction), not a stub.
    const result = await page.evaluate(CONSTRUCT_WORKER('/sw.js'));
    expect(result, 'the app’s own worker script must be allowed').toBe('ok');
  });

  test('a FOREIGN script URL is still REFUSED', async ({ page }) => {
    await page.goto('/');
    for (const hostile of [
      'https://cdn.evil.example/worker.js',
      '//cdn.evil.example/worker.js',
      'data:text/javascript,postMessage(1)',
    ]) {
      const result = await page.evaluate(CONSTRUCT_WORKER(hostile));
      expect(result, `${hostile} must be refused`).not.toBe('ok');
      // Refused by OUR policy (it throws a named TypeError), not merely by the
      // network — so the narrowness is what is being proven here.
      expect(result).toMatch(/refused to load script URL|Trusted/i);
    }
  });

  test('a same-origin NON-asset path is refused too', async ({ page }) => {
    await page.goto('/');
    // Same origin is not sufficient: the path must be an app script asset. This is
    // the assertion that would fail if the policy were relaxed to a passthrough.
    const result = await page.evaluate(CONSTRUCT_WORKER('/api/export/1'));
    expect(result).not.toBe('ok');
    expect(result).toMatch(/refused to load script URL/i);
  });
});
