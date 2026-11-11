import {pdfDownload} from './pdfBridge.ts';
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { authorizedPdfBlob, pdfFailureContext, pdfFailureLines, pdfFrameError, pdfHostFailure, PDF_ERROR_BOUNDARY, safePdfLink, validPdfMessage, PDF_CHANNEL, PDF_FRAME_ORIGIN, PDF_FRAME_SANDBOX } from './pdfBridge.ts';
describe('PDF link and frame boundary', () => {
  it('accepts download bytes and sanitizes names without accepting document URLs',()=>{
    const bytes=new Uint8Array([37,80,68,70]).buffer;
    expect(pdfDownload({bytes,name:'../../report.pdf',mime:'application/pdf'})).toEqual({bytes,name:'.._.._report.pdf',mime:'application/pdf'});
    expect(pdfDownload({bytes:new ArrayBuffer(0)})).toBeNull();expect(pdfDownload({bytes:'https://outside.test/pdf'})).toBeNull();
    expect(pdfDownload({bytes,name:'report.pdf',mime:'text/html; script=bad'})?.mime).toBe('application/octet-stream');
  });
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
describe('PDF frame errors', () => {
  it('turns the frame error message into a bounded Error for the reporter', () => {
    const e = pdfFrameError({ name: 'InvalidPDFException', message: 'x'.repeat(1000), stack: 'InvalidPDFException: bad\n    at a (pdf.mjs:1:1)' });
    expect(e.name).toBe('InvalidPDFException');
    expect(e.message.length).toBe(300);
    expect(e.stack).toContain('pdf.mjs');
    const bare = pdfFrameError(null);
    expect(bare.message).toBe('PDF viewer failed');
    expect(PDF_ERROR_BOUNDARY).toBe('Reader/AttachmentViewer/pdf');
  });
});

describe('PDF failure details', () => {
  it('carries the stage, pdf.js version, worker status and polyfilled built-ins, bounded and without control characters', () => {
    const e = pdfFrameError({ name: 'TypeError', message: 'x is not a function', stage: 'viewer', pdfjs: '6.1.200', polyfilled: 'Promise.try,Map.prototype.getOrInsertComputed', worker: 'main-thread: error event: boom\nsecret' });
    expect(e.pdf).toEqual({ stage: 'viewer', name: 'TypeError', message: 'x is not a function', pdfjs: '6.1.200', polyfilled: 'Promise.try,Map.prototype.getOrInsertComputed', worker: 'main-thread: error event: boom secret' });
    expect(pdfFrameError({ stage: '<img onerror=1>' }).pdf.stage).toBe('unknown');
    expect(pdfFrameError({ pdfjs: '6.1.200<script>' }).pdf.pdfjs).toBe('6.1.200script');
  });
  it('lists the lines a user can copy into a report, and a one-line context for the reporter', () => {
    const failure = pdfHostFailure('fetch', new Error('PDF unavailable (HTTP 502)'));
    expect(pdfFailureLines(failure, 'Firefox/140').map(([label]) => label)).toEqual(['stage', 'error', 'browser']);
    expect(pdfFailureContext(failure, 'Firefox/140')).toBe('stage=fetch | error=Error: PDF unavailable (HTTP 502) | browser=Firefox/140');
  });
});

// The frame and its worker run the same polyfills before anything else: a browser
// without Promise.try / Map.getOrInsertComputed / ... otherwise ends in "Could not render this PDF".
describe('PDF.js polyfills for older browsers', () => {
  // The module is plain JavaScript beside the vendored files; a query string loads a fresh copy each time.
  const loadPolyfills = async (query: string): Promise<{ installed: string[] }> => (await import(/* @vite-ignore */ `../../public/pdfjs/polyfills.mjs?${query}`)) as { installed: string[] };
  type Upsert = Map<string, number> & { getOrInsert(key: string, value: number): number; getOrInsertComputed(key: string, callback: (key: string) => number): number };
  const math = Math as unknown as { sumPrecise(values: number[]): number };
  const promise_ = Promise as unknown as { try<T>(callback: (...args: never[]) => T, ...args: unknown[]): Promise<T> };
  const regexp = RegExp as unknown as { escape(value: string): string };
  const url_ = URL as unknown as { parse(url: string, base?: string): URL | null };
  const bytes = Uint8Array as unknown as { fromHex(text: string): Uint8Array; fromBase64(text: string, options?: { alphabet: string }): Uint8Array };
  const NAMES: [object, string][] = [[Map.prototype, 'getOrInsert'], [Map.prototype, 'getOrInsertComputed'], [WeakMap.prototype, 'getOrInsert'], [WeakMap.prototype, 'getOrInsertComputed'], [Math, 'sumPrecise'], [Promise, 'try'], [RegExp, 'escape'], [URL, 'parse'], [Uint8Array.prototype, 'toHex'], [Uint8Array.prototype, 'toBase64'], [Uint8Array, 'fromBase64'], [Uint8Array, 'fromHex'], [Promise, 'withResolvers']];
  it('supplies every missing built-in and leaves present ones alone', async () => {
    const saved = NAMES.map(([target, name]) => [target, name, Object.getOwnPropertyDescriptor(target, name)] as const);
    try {
      for (const [target, name] of NAMES) delete (target as Record<string, unknown>)[name];
      const { installed } = await loadPolyfills('missing');
      expect(installed).toHaveLength(NAMES.length);
      const map = new Map<string, number>() as Upsert;
      expect(map.getOrInsert('a', 1)).toBe(1);
      expect(map.getOrInsert('a', 2)).toBe(1);
      expect(map.getOrInsertComputed('b', key => key.length + 4)).toBe(5);
      expect(map.getOrInsertComputed('b', () => { throw new Error('computed twice'); })).toBe(5);
      expect(math.sumPrecise([0.1, 0.2, 0.3, 1e100, -1e100])).toBeCloseTo(0.6, 12);
      expect(Object.is(math.sumPrecise([]), -0)).toBe(true);
      expect(math.sumPrecise([1, Infinity])).toBe(Infinity);
      expect(math.sumPrecise([Infinity, -Infinity])).toBeNaN();
      await expect(promise_.try((a: number, b: number) => a + b, 2, 3)).resolves.toBe(5);
      await expect(promise_.try(() => { throw new Error('boom'); })).rejects.toThrow('boom');
      expect(regexp.escape('a.b*c(1)')).toBe('\\x61\\.b\\*c\\(1\\)');
      expect(new RegExp(regexp.escape('1+1=2 [x]')).test('1+1=2 [x]')).toBe(true);
      expect(url_.parse('/x?y=1', 'https://example.test')?.href).toBe('https://example.test/x?y=1');
      expect(url_.parse('not a url')).toBeNull();
      expect((new Uint8Array([0, 15, 255]) as unknown as { toHex(): string }).toHex()).toBe('000fff');
      expect(bytes.fromHex('000fff')).toEqual(new Uint8Array([0, 15, 255]));
      expect((new Uint8Array([251, 255, 190]) as unknown as { toBase64(): string }).toBase64()).toBe('+/++');
      expect((new Uint8Array([251, 255]) as unknown as { toBase64(o: object): string }).toBase64({ alphabet: 'base64url', omitPadding: true })).toBe('-_8');
      expect(bytes.fromBase64('+/++')).toEqual(new Uint8Array([251, 255, 190]));
      expect(() => bytes.fromBase64('@@@')).toThrow(SyntaxError);
      const { promise, resolve } = (Promise as unknown as {withResolvers<T>():{promise:Promise<T>;resolve:(value:T)=>void}}).withResolvers<number>();
      resolve(7);
      await expect(promise).resolves.toBe(7);
      const again = await loadPolyfills('present');
      expect(again.installed).toEqual([]);
    } finally {
      for (const [target, name, descriptor] of saved) { if (descriptor) Object.defineProperty(target, name, descriptor); else delete (target as Record<string, unknown>)[name]; }
    }
  });
  it('is loaded by the frame before anything else and by the worker before the PDF.js worker', () => {
    const boot = readFileSync(resolve(process.cwd(), 'public/pdfjs/bootstrap.mjs'), 'utf8');
    expect(boot).toContain("import { installed as polyfilled } from './polyfills.mjs'");
    expect(boot).toMatch(/await import\(\$\{JSON\.stringify\(polyfills\)\}\);await import\(\$\{JSON\.stringify\(worker\)\}\)/);
  });
});
