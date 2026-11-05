// Sanitiser worker: hosts one instance of the `mw-sanitize` wasm and nothing else (no key
// material), so sanitising never queues behind crypto and the reverse. The pool compiles
// the module once and posts the compiled `WebAssembly.Module` here; instantiation is
// synchronous and cheap.

import {
  initSync,
  sanitizeEmailHtml,
  sanitizeEmailHtmlReader,
  sanitizeEmailHtmlReaderSafe,
} from '../wasm/mw-sanitize/mw_sanitize.js';
import type { FromWorker, SanitizeRequest, ToWorker } from './protocol.ts';

/** Wasm exports are synchronous: run one request. */
export function run(req: SanitizeRequest): string {
  if (typeof req.html !== 'string') throw new Error('invalid sanitize request');
  switch (req.op) {
    case 'strip':
      return sanitizeEmailHtml(req.html);
    case 'reader':
      return sanitizeEmailHtmlReader(req.html, req.blockTrackers !== false, req.cleanLinks !== false);
    case 'readerSafe':
      return sanitizeEmailHtmlReaderSafe(req.html, req.blockTrackers !== false, req.cleanLinks !== false);
    default:
      throw new Error('unknown sanitize op');
  }
}

let ready = false;

self.onmessage = (ev: MessageEvent<ToWorker>): void => {
  const msg = ev.data;
  if (msg.type === 'init') {
    initSync({ module: msg.module });
    ready = true;
    return;
  }
  const reply = (r: FromWorker): void => (self as unknown as Worker).postMessage(r);
  try {
    if (!ready) throw new Error('sanitizer worker not initialised');
    reply({ type: 'result', id: msg.id, ok: true, html: run(msg.req) });
  } catch (err) {
    reply({ type: 'result', id: msg.id, ok: false, error: err instanceof Error ? err.message : String(err) });
  }
};
