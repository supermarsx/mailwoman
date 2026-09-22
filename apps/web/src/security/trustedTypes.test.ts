import { describe, expect, it } from 'vitest';
import { isAppScriptUrl } from './trustedTypes.ts';

// The unit half of the 26.20 t24-e13 fix. It pins the SHAPE of the rule; it
// cannot prove the policy is installed early enough or that the browser accepts
// it, because jsdom sends no CSP and has no Trusted Types. That half is
// apps/web/e2e/crypto-security.spec.ts's "Trusted Types" block, which runs in a
// real browser against the app as mw-server serves it.

const ORIGIN = 'http://localhost:8090';

describe('isAppScriptUrl — the app’s own script assets', () => {
  it('accepts Vite worker chunks under /assets/', () => {
    expect(isAppScriptUrl(`${ORIGIN}/assets/worker.entry-aor3xJE5.js`, ORIGIN)).toBe(true);
    expect(isAppScriptUrl(`${ORIGIN}/assets/index-X8vAaQ53.js`, ORIGIN)).toBe(true);
    expect(isAppScriptUrl(`${ORIGIN}/assets/chunk.mjs`, ORIGIN)).toBe(true);
  });

  it('accepts the two root-served scripts', () => {
    expect(isAppScriptUrl(`${ORIGIN}/sw.js`, ORIGIN)).toBe(true);
    expect(isAppScriptUrl(`${ORIGIN}/pdf.worker.mjs`, ORIGIN)).toBe(true);
  });

  it('accepts the same assets under a deploy prefix', () => {
    expect(isAppScriptUrl(`${ORIGIN}/mail/assets/worker.entry-x.js`, ORIGIN, '/mail')).toBe(true);
    expect(isAppScriptUrl(`${ORIGIN}/mail/sw.js`, ORIGIN, '/mail')).toBe(true);
    // …and refuses the un-prefixed path when a prefix is configured.
    expect(isAppScriptUrl(`${ORIGIN}/elsewhere/assets/x.js`, ORIGIN, '/mail')).toBe(false);
  });

  it('refuses a cross-origin script URL', () => {
    expect(isAppScriptUrl('https://cdn.evil.example/assets/worker.js', ORIGIN)).toBe(false);
    expect(isAppScriptUrl('//cdn.evil.example/assets/worker.js', ORIGIN)).toBe(false);
    // Same host, different scheme/port is still a different origin.
    expect(isAppScriptUrl('https://localhost:8090/assets/worker.js', ORIGIN)).toBe(false);
    expect(isAppScriptUrl('http://localhost:9999/assets/worker.js', ORIGIN)).toBe(false);
  });

  it('refuses the script-bearing pseudo-schemes', () => {
    expect(isAppScriptUrl('data:text/javascript,alert(1)', ORIGIN)).toBe(false);
    expect(isAppScriptUrl('blob:http://localhost:8090/f0e1-d2c3', ORIGIN)).toBe(false);
    expect(isAppScriptUrl('javascript:alert(1)', ORIGIN)).toBe(false);
  });

  it('refuses same-origin paths outside the app’s script assets', () => {
    expect(isAppScriptUrl(`${ORIGIN}/api/export/1`, ORIGIN)).toBe(false);
    expect(isAppScriptUrl(`${ORIGIN}/uploads/attacker.js`, ORIGIN)).toBe(false);
    // An upload nested under /assets/ is not a build chunk.
    expect(isAppScriptUrl(`${ORIGIN}/assets/user/attacker.js`, ORIGIN)).toBe(false);
    // Non-script extensions, and the bare directory.
    expect(isAppScriptUrl(`${ORIGIN}/assets/index.css`, ORIGIN)).toBe(false);
    expect(isAppScriptUrl(`${ORIGIN}/assets/`, ORIGIN)).toBe(false);
  });

  it('normalizes traversal before testing the prefix', () => {
    // Resolves to /uploads/attacker.js, which is not an app script asset.
    expect(isAppScriptUrl(`${ORIGIN}/assets/../uploads/attacker.js`, ORIGIN)).toBe(false);
  });

  it('ignores a query string or fragment when checking the extension', () => {
    expect(isAppScriptUrl(`${ORIGIN}/assets/worker.entry-x.js?worker&type=module`, ORIGIN)).toBe(
      true,
    );
    // …and a query cannot smuggle a script extension onto a non-script path.
    expect(isAppScriptUrl(`${ORIGIN}/uploads/evil?x=.js`, ORIGIN)).toBe(false);
  });
});
