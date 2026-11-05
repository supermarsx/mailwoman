import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { authorizedPdfBlob, safePdfLink, validPdfMessage, PDF_CHANNEL, PDF_FRAME_ORIGIN, PDF_FRAME_SANDBOX } from './pdfBridge.ts';
describe('PDF link and frame boundary', () => {
  it('only reads attachment blobs created by the current origin', () => {
    expect(authorizedPdfBlob(`blob:${location.origin}/authorized`)).toBe(true);
    for (const value of ['https://outside.test/pdf', '/api/attachment', 'blob:https://outside.test/id', 'data:application/pdf;base64,AAAA', 'blob:broken']) expect(authorizedPdfBlob(value)).toBe(false);
  });
  it('admits supported explicit destinations and rejects executable, local, authenticated or malformed URLs', () => {
    for (const value of ['https://example.test/a?x=1', 'http://example.test/', 'mailto:person@example.test', 'tel:+351123456789']) expect(safePdfLink(value)).toBe(value);
    for (const value of ['javascript:alert(1)', 'data:text/html,test', 'file:///private', 'blob:https://example.test/a', '/api/session', 'https://user:pass@example.test', 'https://example.test/\nsecret', {}, null]) expect(safePdfLink(value)).toBeNull();
  });
  it('requires both the owning frame and exact origin for the named protocol', () => {
    const event = new MessageEvent('message', { source: window, origin: PDF_FRAME_ORIGIN, data: { channel: PDF_CHANNEL } });
    expect(validPdfMessage(event, window, PDF_FRAME_ORIGIN)).toBe(true);
    expect(validPdfMessage(event, null, PDF_FRAME_ORIGIN)).toBe(false);
    expect(validPdfMessage(event, window, 'https://other.test')).toBe(false);
    expect(validPdfMessage(new MessageEvent('message', { source: window, origin: location.origin, data: { channel: PDF_CHANNEL } }), window, PDF_FRAME_ORIGIN)).toBe(false);
    expect(validPdfMessage(new MessageEvent('message', { source: window, origin: PDF_FRAME_ORIGIN, data: { channel: 'other' } }), window, PDF_FRAME_ORIGIN)).toBe(false);
  });
  it('sandboxes the frame to scripts only: an opaque origin, never allow-same-origin', () => {
    const tokens = PDF_FRAME_SANDBOX.split(' ');
    expect(tokens).toContain('allow-scripts');
    expect(tokens).not.toContain('allow-same-origin');
    expect(tokens).not.toContain('allow-top-navigation');
    expect(tokens).not.toContain('allow-popups');
  });
});

// The frame side is a classic module with top-level effects, so its protocol is
// pinned at the source: what it must do and must not do once it has no host access.
describe('PDF frame bootstrap contract (opaque origin)', () => {
  const source = readFileSync(resolve(process.cwd(), 'public/pdfjs/bootstrap.mjs'), 'utf8');
  it('never reads the host page and answers only the window that embeds it', () => {
    expect(source).not.toMatch(/window\.parent\.document/);
    expect(source).toContain('event.source !== host');
    expect(source).toContain("hostOrigin ?? '*'");
    expect(source).toContain('event.origin !== hostOrigin');
    expect(source).not.toMatch(/location\.origin/);
  });
  it('keeps its own state: in-memory storage, options set locally, one nonce per frame', () => {
    expect(source).toContain("defineProperty(window, 'localStorage'");
    expect(source).toContain("document.addEventListener('webviewerloaded', configure)");
    expect(source).toContain('message.nonce === nonce');
    expect(source).toContain('message.instance !== instance');
  });
  it('hands the worker and its data files to PDF.js as absolute URLs', () => {
    for (const key of ['cMapUrl', 'standardFontDataUrl', 'wasmUrl', 'iccUrl', 'workerSrc']) expect(source).toContain(key);
    expect(source).toContain("new URL('./web/', import.meta.url)");
  });
});
