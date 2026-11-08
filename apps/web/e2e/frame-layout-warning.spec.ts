import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {createServer,type Server} from 'node:http';
import type {AddressInfo} from 'node:net';
import {expect,test,type ConsoleMessage} from '@playwright/test';
import {resolve} from 'node:path';
import {traceLayoutReads} from './layoutTrace.ts';
import {buildPdf} from './pdf-fixtures.ts';

/**
 * Firefox logs "Layout was forced before the page was fully loaded. If stylesheets
 * are not yet loaded this may cause a flash of unstyled content." whenever script
 * reads layout (getBoundingClientRect, scrollTo, ...) on a document that has not
 * finished loading. This drives the REAL reader bridge (crates/mw-server/src/
 * remote-frame.js + reader-address-handler.js, the same files the server embeds)
 * through many message loads, including the awkward ones (unsized images that are
 * still loading, a same-document swap that restores the scroll position), and fails
 * on any such warning. It needs no mw-server and no mail backend: a throw-away HTTP
 * server plays the app (parent) origin and `/api/rc/frame`.
 *
 * Firefox only logs the message, and only Firefox is affected, so Chromium is skipped.
 */
const read = (name: string): string => readFileSync(new URL(`../../../crates/mw-server/src/${name}`, import.meta.url), 'utf8').replace(/\r\n?/g, '\n');
const WARNING = /layout was forced before the page was fully loaded/i;
const LOADS = Number(process.env['MW_FRAME_LOADS'] ?? 40);

let server: Server; let origin = '';
test.beforeAll(async () => {
  const handler = read('reader-address-handler.js');
  const handlerHash = createHash('sha256').update(handler).digest('base64');
  const frame = read('remote-frame.js');
  server = createServer((request, response) => {
    const url = new URL(request.url ?? '/', origin);
    if (url.pathname === '/api/rc/frame') {
      const script = frame.replace('__MW_PARENT_ORIGIN__', JSON.stringify(origin)).replace('__MW_ADDRESS_HANDLER_SOURCE__', () => JSON.stringify(handler)).replace('__MW_ADDRESS_HANDLER_HASH__', JSON.stringify(handlerHash));
      const hash = createHash('sha256').update(script).digest('base64');
      response.writeHead(200, {'content-type': 'text/html; charset=utf-8', 'content-security-policy': `default-src 'none'; script-src 'sha256-${hash}' 'sha256-${handlerHash}'; style-src 'unsafe-inline'; frame-src 'self'; img-src * data: blob:; base-uri 'none'; form-action 'none'`});
      response.end(`<!doctype html><html><head><meta charset=utf-8><style>html,body{margin:0;width:100%;height:100%;overflow:hidden}</style></head><body><script>${script}</script></body></html>`);
    } else if (url.pathname.startsWith('/pdfjs/') || url.pathname === '/pdf.worker.mjs') {
      // The embedded PDF viewer's files, served as mw-server serves them (CORS + CORP for the opaque-origin frame).
      const root = resolve(process.cwd(), 'public');
      const file = resolve(root, '.' + decodeURIComponent(url.pathname));
      const types: Record<string, string> = {mjs: 'text/javascript', js: 'text/javascript', css: 'text/css', html: 'text/html; charset=utf-8', json: 'application/json', wasm: 'application/wasm', svg: 'image/svg+xml', ftl: 'text/plain'};
      try {
        if (!file.startsWith(root)) throw new Error('outside');
        response.writeHead(200, {'content-type': types[file.split('.').pop() ?? ''] ?? 'application/octet-stream', 'access-control-allow-origin': '*', 'cross-origin-resource-policy': 'cross-origin', 'cache-control': 'no-store'});
        response.end(readFileSync(file));
      } catch { response.writeHead(404); response.end(); }
    } else if (url.pathname === '/slow.css') {
      setTimeout(() => { response.writeHead(200, {'content-type': 'text/css', 'cross-origin-resource-policy': 'cross-origin'}); response.end('p{color:#333}'); }, 500);
    } else if (url.pathname.startsWith('/img/')) {
      // A slow, unsized-friendly picture: the document stays "loading" meanwhile.
      setTimeout(() => { response.writeHead(200, {'content-type': 'image/gif', 'cross-origin-resource-policy': 'cross-origin', 'cache-control': 'no-store'}); response.end(Buffer.from('R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7', 'base64')); }, Number(url.searchParams.get('delay') ?? 300));
    } else {
      response.writeHead(200, {'content-type': 'text/html; charset=utf-8'});
      response.end('<!doctype html><html><head><meta charset=utf-8><title>harness</title></head><body><div id=host style="height:600px"></div></body></html>');
    }
  });
  await new Promise<void>(resolve => server.listen(0, '127.0.0.1', resolve));
  origin = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
});
test.afterAll(async () => { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); });

const STYLES = '<style>html,body{margin:0;background:transparent}body{font:14px/1.6 ui-sans-serif,system-ui,sans-serif}img,video{max-width:100%;height:auto}</style><style>:root{--mw-color-text:#1c1e21}</style>';
const documentOf = (inner: string): string => `<!doctype html><html><head><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src * data:; style-src 'unsafe-inline'">${STYLES}</head><body>${inner}</body></html>`;
const variants: Record<string, (n: number) => string> = {
  small: n => documentOf(`<p>Message ${n}</p>`),
  large: n => documentOf(`<h1>Message ${n}</h1>` + Array.from({length: 1500}, (_, i) => `<p>Paragraph ${i} <a href="https://example.com/${i}">link</a> lorem ipsum dolor sit amet</p>`).join('') + '<table>' + Array.from({length: 300}, (_, i) => `<tr><td>${i}</td><td>cell</td></tr>`).join('') + '</table>'),
  // No width/height: the handler measures them while they are still loading.
  unsizedImages: n => documentOf(`<p>Message ${n}</p>` + Array.from({length: 12}, (_, i) => `<img alt="i${i}" loading="eager" src="${origin}/img/${n}-${i}?delay=${150 + i * 40}">`).join('')),
  sizedImages: n => documentOf(Array.from({length: 12}, (_, i) => `<img alt="i${i}" width="200" height="100" src="${origin}/img/s${n}-${i}?delay=200">`).join('')),
  placeholders: n => documentOf(Array.from({length: 12}, (_, i) => `<img data-mw-ph="box" alt="blocked ${n}-${i}">`).join('')),
  // Positive control (the sanitizer strips <link>): a pending render-blocking sheet plus a layout read must warn.
  controlPendingSheet: n => documentOf(`<p>Message ${n}</p><img alt=x src="${origin}/img/c${n}?delay=300">`).replace('</head>', `<link rel=stylesheet href="${origin}/slow.css?n=${n}"></head>`).replace("style-src 'unsafe-inline'", "style-src 'unsafe-inline' *"),
  inlineSheets: n => documentOf(`<style>p{color:#333}</style><p>Message ${n}</p><style>@media print{p{display:none}}</style>`),
};

test.describe('reader frame', () => {
  test.skip(({browserName}) => browserName !== 'firefox', 'the warning is a Firefox console message');

  for (const [name, make] of Object.entries(variants)) test(`${name}: ${LOADS} loads log no forced-layout warning`, async ({page}) => {
    const warnings: string[] = []; const traces = new Set<string>();
    const collect = (message: ConsoleMessage): void => { if (WARNING.test(message.text())) warnings.push(`${message.location().url.slice(0, 60)}:${message.location().lineNumber}`); };
    page.on('console', collect);
    {
      page.on('console', message => { if (message.text().startsWith('LAYOUTREAD')) traces.add(message.text()); });
      await traceLayoutReads(page);
    }
    await page.goto(`${origin}/`);
    await page.evaluate(async ({origin, count}) => {
      const token = Array.from(crypto.getRandomValues(new Uint8Array(24)), b => b.toString(16).padStart(2, '0')).join('');
      const frame = document.createElement('iframe');
      frame.setAttribute('sandbox', 'allow-scripts');
      frame.style.cssText = 'width:600px;height:600px;border:0';
      frame.src = `${origin}/api/rc/frame#${new URLSearchParams({token, origin})}`;
      document.getElementById('host')!.append(frame);
      let revision = 0;
      const waiters = new Map<number, () => void>();
      addEventListener('message', event => {
        if (event.data?.type === 'mw-reader-ready') { void next(); }
        if (event.data?.type === 'mw-reader-displayed') { (window as unknown as {shown: number}).shown = ((window as unknown as {shown?: number}).shown ?? 0) + 1; waiters.get(event.data.revision)?.(); }
      });
      (window as unknown as {sendNext: () => Promise<void>}).sendNext = next;
      let queue: string[] = [];
      (window as unknown as {queue: string[]}).queue = queue;
      async function next(): Promise<void> {
        const html = (window as unknown as {queue: string[]}).queue.shift();
        if (html === undefined) return;
        revision += 1; const current = revision;
        await new Promise<void>(resolve => { waiters.set(current, resolve); setTimeout(resolve, 4000); frame.contentWindow!.postMessage({type: 'mw-reader-frame', token, revision: current, html, swap: current % 3 === 0}, '*'); });
        await new Promise(resolve => setTimeout(resolve, 30));
        await next();
      }
      void count; void queue;
    }, {origin, count: LOADS});
    const docs = Array.from({length: LOADS}, (_, i) => make(i));
    await page.evaluate(async list => { (window as unknown as {queue: string[]}).queue.push(...list); await (window as unknown as {sendNext: () => Promise<void>}).sendNext(); }, docs);
    await page.waitForTimeout(1500);
    const shown = await page.evaluate(() => (window as unknown as {shown?: number}).shown ?? 0);
    console.log(name, 'displayed', shown, 'of', LOADS, 'warnings', warnings.length);
    if (traces.size) console.log(name, 'TRACE ' + [...traces].join(' ## '));
    expect(shown, 'the bridge displayed the messages').toBeGreaterThanOrEqual(Math.floor(LOADS * 0.8));
    // Firefox logs the warning only in some timings, so also assert the cause: no frame script reads layout before load.
    const early = [...traces].filter(line => !/ready=complete/.test(line));
    expect(early, 'layout reads before the document finished loading').toEqual([]);
    expect(warnings, `forced-layout warnings (url:line)`).toEqual([]);
  });

  // The PDF viewer frame (apps/web/public/pdfjs/web/mailwoman.html). Its own document and bootstrap
  // must read no layout before load; the vendored PDF.js files (build/pdf.mjs, web/viewer.mjs) are upstream
  // code and are excepted, but are reported so a change in their behaviour is visible.
  test('PDF viewer: the document and its bootstrap log no forced-layout warning', async ({page}) => {
    const mine: string[] = []; const upstream: string[] = [];
    page.on('console', message => {
      if (!WARNING.test(message.text())) return;
      const where = `${message.location().url.replace(origin, '')}:${message.location().lineNumber}`;
      (/\/pdfjs\/(build\/|web\/viewer\.mjs)/.test(where) ? upstream : mine).push(where);
    });
    await page.goto(`${origin}/`);
    const pdf = buildPdf(2);
    await page.evaluate(async ({origin, pdf}) => {
      const frame = document.createElement('iframe');
      frame.setAttribute('sandbox', 'allow-scripts allow-downloads allow-modals');
      frame.style.cssText = 'width:900px;height:600px;border:0';
      frame.src = `${origin}/pdfjs/web/mailwoman.html`;
      document.getElementById('host')!.append(frame);
      const state = window as unknown as {pdfLoaded?: boolean};
      addEventListener('message', event => {
        const data = event.data as {channel?: string; type?: string; instance?: string};
        if (event.source !== frame.contentWindow || data?.channel !== 'mailwoman-pdf-v1') return;
        if (data.type === 'ready' && !('sent' in state)) {
          Object.assign(state, {sent: true});
          const bytes = Uint8Array.from(pdf, c => c.charCodeAt(0)).buffer;
          frame.contentWindow!.postMessage({channel: 'mailwoman-pdf-v1', type: 'open', instance: data.instance, nonce: 'n', bytes, name: 'a.pdf', locale: 'en', theme: {}}, '*', [bytes]);
        }
        if (data.type === 'loaded') state.pdfLoaded = true;
      });
    }, {origin, pdf});
    await expect.poll(() => page.evaluate(() => (window as unknown as {pdfLoaded?: boolean}).pdfLoaded === true), {timeout: 30_000}).toBe(true);
    await page.waitForTimeout(1000);
    console.log('pdf viewer forced-layout warnings: own', mine.length, 'upstream', upstream.length, upstream.join(' '));
    expect(mine, 'forced-layout warnings from the viewer document / bootstrap (url:line)').toEqual([]);
  });
});
