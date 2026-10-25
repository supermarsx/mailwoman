// The reader's remote-content mode of the committed mw-sanitize wasm guest
// (t32 e31a): decrypted mail gets the same placeholders, tracker blocking and link
// cleaning as mail the server sanitizes, so the reader can have its image URLs
// signed (`POST /api/rc/sign`) without sending the decrypted body anywhere.

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import {
  initSync,
  sanitizeEmailHtml,
  sanitizeEmailHtmlReader,
} from '../wasm/mw-sanitize/mw_sanitize.js';

beforeAll(() => {
  const wasmPath = resolve(process.cwd(), 'src/wasm/mw-sanitize/mw_sanitize_bg.wasm');
  initSync({ module: readFileSync(wasmPath) });
});

describe('mw-sanitize wasm reader mode (t32 e31a)', () => {
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
