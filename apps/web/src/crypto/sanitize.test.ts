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
    expect(clean).not.toContain('tracker.evil');
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

describe('looksLikeHtml', () => {
  it('detects opening and closing tags, not stray angle brackets', () => {
    expect(looksLikeHtml('<p>hi</p>')).toBe(true);
    expect(looksLikeHtml('<div class="x">')).toBe(true);
    expect(looksLikeHtml('<script>x</script>')).toBe(true);
    expect(looksLikeHtml('a < b and c > d')).toBe(false);
    expect(looksLikeHtml('just text')).toBe(false);
  });
});
