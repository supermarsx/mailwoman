// The §1.3 torture assertion, CLIENT-SIDE: decrypted E2EE HTML carrying a
// <script>/onclick is sanitized by the IN-WORKER `mw-sanitize` wasm — the same
// assertion the server sanitizer passes, now proven against the real wasm build the
// crypto worker loads (`sanitizeEmailHtml`) driven through the exact `worker.entry.ts`
// routing (`sanitizeDecryptResult`). Loads the committed wasm bytes synchronously via
// wasm-pack's `initSync` (jsdom cannot host a Worker, but it can run the wasm).

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import { initSync, sanitizeEmailHtml } from '../wasm/mw-sanitize/mw_sanitize.js';
import { looksLikeHtml, sanitizeDecryptResult } from './sanitize.ts';
import type { SignatureVerdict } from '../api/security-types.ts';

const SIG: SignatureVerdict = {
  kind: 'pgp',
  status: 'none',
  signerKeyId: null,
  algorithm: null,
  keyCreatedAt: null,
  keyExpiresAt: null,
  chainStatus: null,
  revocationStatus: null,
  keyChanged: false,
};

beforeAll(() => {
  // Instantiate the real wasm-pack bundle from the committed bytes (no fetch). Vitest
  // runs with cwd = apps/web (the vite config dir), so resolve from there.
  const wasmPath = resolve(process.cwd(), 'src/wasm/mw-sanitize/mw_sanitize_bg.wasm');
  initSync({ module: readFileSync(wasmPath) });
});

describe('in-worker mw-sanitize wasm (plan §1.3)', () => {
  it('strips <script> and event handlers from decrypted HTML (real wasm)', () => {
    const dirty = '<p onclick="steal()">hello</p><script>window.__pwned=1</script>';
    const clean = sanitizeEmailHtml(dirty);
    expect(clean).not.toContain('<script');
    expect(clean).not.toContain('__pwned');
    expect(clean).not.toContain('onclick');
    expect(clean).not.toContain('steal()');
    expect(clean).toContain('hello');
  });

  it('neutralizes javascript: URLs and remote images (real wasm)', () => {
    const clean = sanitizeEmailHtml(
      '<a href="javascript:alert(1)">x</a><img src="https://tracker.evil/p.gif">',
    );
    expect(clean).not.toContain('javascript:');
    // The host survives ONLY inside the hidden `data-mw-blocked-host` breadcrumb
    // the sanitizer appends on purpose (t16 S9) — it "never carries a loadable
    // URL". This assertion used to be a flat `not.toContain('tracker.evil')`,
    // which passed only because the committed wasm guest predated 26.16 and
    // emitted no marker at all; rebuilding the guest (t24-e13) exposed it. The
    // property that matters is that nothing can LOAD from the host, so assert
    // that, exactly as e2e/sanitizer.spec.ts now does.
    expect(clean).toMatch(/data-mw-blocked-host="tracker\.evil"/);
    expect(clean.replace(/\sdata-mw-blocked-host="[^"]*"/g, '')).not.toContain('tracker.evil');
    expect(clean).not.toMatch(/(?:src|href)\s*=\s*["'][^"']*tracker\.evil/i);
  });

  it('routes decrypted HTML through the worker wiring and sanitizes it (script stripped)', () => {
    // The exact path worker.entry.ts runs: raw wasm-crypto decrypt result → route.
    const out = sanitizeDecryptResult(
      { plaintextText: '<div><script>alert(1)</script><b>secret</b></div>', signature: SIG },
      sanitizeEmailHtml,
    );
    expect(out.plaintextHtml).toBeDefined();
    expect(out.plaintextText).toBeUndefined();
    expect(out.plaintextHtml).not.toContain('<script');
    expect(out.plaintextHtml).not.toContain('alert(1)');
    expect(out.plaintextHtml).toContain('secret');
    expect(out.signature).toBe(SIG);
  });

  it('keeps non-HTML decrypted plaintext as escaped text (renders escaped downstream)', () => {
    const out = sanitizeDecryptResult(
      { plaintextText: 'plain body: 1 < 2 and 3 > 4', signature: SIG },
      sanitizeEmailHtml,
    );
    expect(out.plaintextText).toBe('plain body: 1 < 2 and 3 > 4');
    expect(out.plaintextHtml).toBeUndefined();
  });

  it('carries the subject through when present', () => {
    const out = sanitizeDecryptResult(
      { plaintextText: 'hi', subject: 'Protected subject', signature: SIG },
      sanitizeEmailHtml,
    );
    expect(out.subject).toBe('Protected subject');
  });
});

// ── Freshness of the committed guest (t24-e13) ─────────────────────────────
//
// `mw_sanitize_bg.wasm` is a COMMITTED binary, so it can silently fall behind
// `crates/mw-sanitize/src`. It had: the guest shipping in 26.19 was built before
// `db57bc0` (the CSS parse + property allowlist + selector namespacing: drop
// positioning and `@import`, drop external `url()`, clamp `z-index`) and before
// `06628e5` (26.16 tracker markers), so decrypted E2EE mail — the ONLY consumer of
// the in-worker sanitizer — was getting a weaker policy than server-sanitized mail.
//
// These assert behaviour that ONLY the current sanitizer has, so a stale guest
// fails the suite instead of passing quietly. They deliberately do not test string
// markers in the binary: `"z-index"` and the at-rule names are match arms, which
// rustc compiles to inline byte comparisons and never emits as data, so grepping
// the `.wasm` for them reports 0 on a perfectly fresh build. Behaviour is the only
// honest detector.
describe('committed mw-sanitize guest is CURRENT with crates/mw-sanitize/src', () => {
  it('clamps z-index (db57bc0) — MAX_Z_INDEX = 1000', () => {
    const out = sanitizeEmailHtml('<div style="z-index:999999">x</div>');
    expect(out).not.toContain('999999');
    expect(out).toMatch(/z-index:\s*1000/);
  });

  it('drops @import and keeps @media (db57bc0)', () => {
    const out = sanitizeEmailHtml(
      '<style>@import url("//evil.example/x.css"); @media screen { p { color: red } }</style><p>hi</p>',
    );
    expect(out).not.toContain('@import');
    expect(out).not.toContain('evil.example');
    expect(out).toContain('@media');
  });

  it('drops positioning and external url() from inline style (db57bc0)', () => {
    const fixed = sanitizeEmailHtml('<div style="position:fixed;top:0">x</div>');
    expect(fixed).not.toContain('fixed');
    const bg = sanitizeEmailHtml('<div style="background:url(https://evil.example/t.png)">x</div>');
    expect(bg).not.toContain('evil.example');
  });

  it('namespaces <style> selectors under .mw-email-body (db57bc0)', () => {
    const out = sanitizeEmailHtml('<style>p { color: red }</style><p>hi</p>');
    expect(out).toContain('mw-email-body');
  });

  it('emits the blocked-remote-image marker (06628e5 / t16 S9)', () => {
    const out = sanitizeEmailHtml('<img src="https://tracker.evil.example/p.gif">');
    // The marker reports what was blocked and never carries a loadable URL.
    expect(out).toContain('data-mw-blocked-host="tracker.evil.example"');
    expect(out).not.toMatch(/src\s*=\s*["'][^"']*tracker\.evil/i);
  });
});

describe('looksLikeHtml', () => {
  it('detects opening and closing tags, not stray angle brackets', () => {
    expect(looksLikeHtml('<p>hi</p>')).toBe(true);
    expect(looksLikeHtml('<div class="x">')).toBe(true);
    expect(looksLikeHtml('<script>x</script>')).toBe(true);
    expect(looksLikeHtml('a < b and c > d')).toBe(false);
    expect(looksLikeHtml('just text')).toBe(false);
  });
});
