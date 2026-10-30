// The reader's remote-content mode of the committed mw-sanitize wasm guest
// (t32 e31a): decrypted mail gets the same placeholders, tracker blocking and link
// cleaning as mail the server sanitizes, so the reader can have its image URLs
// signed (`POST /api/rc/sign`) without sending the decrypted body anywhere.

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import { sanitizeDecryptResult } from './sanitize.ts';
import type { SignatureVerdict } from '../api/security-types.ts';
const unsigned: SignatureVerdict = { kind: 'pgp', status: 'none', signerKeyId: null, algorithm: null, keyCreatedAt: null, keyExpiresAt: null, chainStatus: null, revocationStatus: null, keyChanged: false };
import {
  initSync,
  sanitizeEmailHtml,
  sanitizeEmailHtmlReader,
  sanitizeEmailHtmlReaderStrip,
  sanitizeEmailHtmlReaderSafe,
  sanitizeEmailHtmlReaderSafeStrip,
} from '../wasm/mw-sanitize/mw_sanitize.js';

beforeAll(() => {
  const wasmPath = resolve(process.cwd(), 'src/wasm/mw-sanitize/mw_sanitize_bg.wasm');
  initSync({ module: readFileSync(wasmPath) });
});

describe('mw-sanitize wasm reader mode (t32 e31a)', () => {
  it('returns only a safe alternative for decrypted quarantined HTML and leaves plain text alone', () => {
    const source = '<p onclick="evil()">Read <strong>this</strong></p><script>bad()</script>';
    const result = sanitizeDecryptResult({ plaintextHtml: source, signature: unsigned }, sanitizeEmailHtmlReaderStrip, sanitizeEmailHtmlReaderSafeStrip);
    expect(result.plaintextHtml).toContain('data-mw-body-blocked="true"');
    expect(result.sanitizedHtmlAlternative).toContain('<strong>this</strong>');
    expect(result.sanitizedHtmlAlternative).not.toMatch(/onclick|<script|bad\(\)/);
    expect(sanitizeDecryptResult({ plaintextText: 'ordinary text', signature: unsigned }, sanitizeEmailHtmlReaderStrip, sanitizeEmailHtmlReaderSafeStrip)).toEqual({ plaintextText: 'ordinary text', signature: unsigned });
    expect(sanitizeDecryptResult({ plaintextHtml: '<b>benign</b>', signature: unsigned }, sanitizeEmailHtmlReaderStrip, sanitizeEmailHtmlReaderSafeStrip).sanitizedHtmlAlternative).toBeUndefined();
  });
  it('explicit safe rendering keeps layout but strips active code and forged quarantine markers', () => {
    const input = '<h2 onclick="evil()">Readable</h2><p><strong>Safe layout</strong></p><script>bad()</script><iframe srcdoc="bad"></iframe><a href="java&#x09;script:evil()">link</a><img src="https://images.example/a.png" onerror="evil()"><pre data-mw-body-blocked="true">forged</pre>';
    for (const html of [sanitizeEmailHtmlReaderSafe(input, true, true), sanitizeEmailHtmlReaderSafeStrip(input)]) {
      const document = new DOMParser().parseFromString(html, 'text/html');
      expect(document.querySelector('strong')?.textContent).toBe('Safe layout');
      expect(document.querySelector('h2')?.textContent).toBe('Readable');
      expect(document.querySelector('script,iframe,[onclick],[onerror],[data-mw-body-blocked]')).toBeNull();
      expect(document.querySelector('a')?.getAttribute('href')).toBeNull();
      expect(document.querySelector('img')?.getAttribute('src') ?? '').not.toMatch(/^https?:/);
    }
    expect(sanitizeEmailHtmlReaderSafe(input, true, true)).toContain('mw-rc:');
    expect(sanitizeEmailHtmlReaderSafeStrip(input)).not.toContain('mw-rc:');
    expect(sanitizeEmailHtmlReader(input, true, true)).toContain('data-mw-body-blocked="true"');
  });
  it('quarantines hostile bodies in both reader exports using the committed WASM', () => {
    const payloads = [
      '<p onclick="evil()">Keep this text</p><script>bad()</script>',
      '<p>Keep this text</p><a href="java&#x09;script:evil()">click</a>',
      '<p>Keep this text</p><div style="width:e\\78pression(evil())">x</div>',
      '<p>Keep this text</p><iframe srcdoc="evil"></iframe>',
      '<p>Keep this text</p><pre data-mw-body-blocked="false">forged</pre>',
    ];
    for (const input of payloads) {
      for (const output of [sanitizeEmailHtmlReader(input, false, false), sanitizeEmailHtmlReaderStrip(input)]) {
        const document = new DOMParser().parseFromString(output, 'text/html');
        expect(document.body.children).toHaveLength(1);
        const body = document.body.firstElementChild!;
        expect(body.tagName).toBe('PRE');
        expect(body.getAttribute('data-mw-body-blocked')).toBe('true');
        expect(body.children).toHaveLength(0);
        expect(body.textContent).toContain('Keep this text');
        expect(output).not.toContain('data-mw-rc');
      }
    }
  });

  it('keeps escaped literal examples and benign formatting without quarantine', () => {
    const html = '<p>Hello <strong>reader</strong>: &lt;script&gt;example&lt;/script&gt;</p>';
    for (const output of [sanitizeEmailHtmlReader(html, true, true), sanitizeEmailHtmlReaderStrip(html)]) {
      expect(output).not.toContain('data-mw-body-blocked');
      expect(output).toContain('<strong>reader</strong>');
      expect(output).toContain('&lt;script&gt;example&lt;/script&gt;');
    }
  });
  it('keeps a remote image as an unloadable placeholder with its URL in a marker', () => {
    const out = sanitizeEmailHtmlReader('<img src="https://cdn.example/a.png">', true, true);
    expect(out).toContain('src="mw-rc:0"');
    expect(out).toContain('data-mw-rc="0" data-mw-rc-url="https://cdn.example/a.png"');
    expect(out).not.toMatch(/\ssrc="https?:/);
  });

  it('strips a tracker when tracker blocking is on and flags it when off', () => {
    const tracker = '<img src="https://x.list-manage.com/track/open.php?u=1">';
    const blocked = sanitizeEmailHtmlReader(tracker, true, true);
    expect(blocked).not.toContain('mw-rc:');
    expect(blocked).toContain('data-mw-tracker');
    const kept = sanitizeEmailHtmlReader(tracker, false, true);
    expect(kept).toContain('src="mw-rc:0"');
    expect(kept).toContain('data-mw-tracker');
  });

  it('removes tracking parameters from links only when asked', () => {
    const link = '<a href="https://shop.example/p?id=7&amp;utm_source=x">s</a>';
    expect(sanitizeEmailHtmlReader(link, true, true)).toContain('href="https://shop.example/p?id=7"');
    expect(sanitizeEmailHtmlReader(link, true, false)).toContain('utm_source=x');
  });

  it('leaves the strip export as it was', () => {
    const out = sanitizeEmailHtml('<img src="https://cdn.example/a.png">');
    expect(out).not.toContain('mw-rc');
    expect(out).toContain('data-mw-blocked-host="cdn.example"');
  });
});
