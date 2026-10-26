import type { Email } from '../api/jmap-types.ts';
import { escapeHtml } from './sandbox.ts';

/** A separate print document: never changes the reader's opaque sandbox. */
export function printMessageDocument(email: Email, content: { html?: string; text?: string }): string {
  const addresses = (items: Email['from']): string => (items ?? []).map(item => item.name ? `${item.name} <${item.email}>` : item.email).join(', ');
  const headers = [addresses(email.from), addresses(email.to), email.receivedAt ?? ''];
  return '<!doctype html><html><head><meta charset="utf-8">' +
    '<meta http-equiv="Content-Security-Policy" content="default-src \'none\'; script-src \'none\'; img-src data:; style-src \'unsafe-inline\'; base-uri \'none\'; form-action \'none\'">' +
    `<title>${escapeHtml(email.subject ?? '')}</title>` +
    '<style>@page{margin:18mm}body{font:12pt/1.5 sans-serif;color:#000;background:#fff;overflow-wrap:anywhere}header{border-bottom:1px solid #777;margin-bottom:1em;padding-bottom:1em}h1{font-size:18pt}pre{white-space:pre-wrap}img,table{max-width:100%}a{color:inherit}article{color-scheme:light}</style></head><body>' +
    `<header><h1>${escapeHtml(email.subject ?? '')}</h1>${headers.map(value => `<div>${escapeHtml(value)}</div>`).join('')}</header>` +
    `<article>${content.html ?? `<pre>${escapeHtml(content.text ?? '')}</pre>`}</article></body></html>`;
}

/** Uses the browser's printer/PDF destination selection; no remote print service. */
export function printMessage(email: Email, content: { html?: string; text?: string }): void {
  const frame = document.createElement('iframe');
  frame.title = email.subject ?? 'Message';
  // Same-origin permits this parent to invoke print. Scripts remain prohibited.
  frame.setAttribute('sandbox', 'allow-same-origin allow-modals');
  frame.setAttribute('aria-hidden', 'true');
  frame.style.cssText = 'position:fixed;inset:0;width:1px;height:1px;border:0;opacity:0;pointer-events:none';
  const cleanup = (): void => { frame.remove(); window.clearTimeout(timeout); };
  const timeout = window.setTimeout(cleanup, 120_000);
  frame.onload = () => {
    const target = frame.contentWindow;
    if (!target) { cleanup(); return; }
    target.addEventListener('afterprint', cleanup, { once: true });
    try { target.focus(); target.print(); } catch { cleanup(); }
  };
  frame.srcdoc = printMessageDocument(email, content);
  document.body.append(frame);
}
