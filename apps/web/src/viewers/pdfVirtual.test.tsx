// The official viewer now owns page virtualization; browser QA exercises its
// 500-page render buffer. Here we exercise the real owning component's byte,
// origin, document lifecycle and cancellation boundaries (PDF.js needs canvas).
import { afterEach, describe, expect, it, vi } from 'vitest';
import { render, waitFor, cleanup } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import PdfViewer from './PdfViewer.tsx';
import { PDF_CHANNEL } from './pdfBridge.ts';
import { Reporter, setReporterForTests } from '../observability/clientErrors.ts';

vi.mock('../screens/Admin/shell/ConfirmDialog.tsx', () => ({ ConfirmDialog: () => null }));
const part = { partId: '1', blobId: 'b', size: 1, type: 'application/pdf' };
function message(frame: HTMLIFrameElement, data: Record<string, unknown>, source: Window|null = frame.contentWindow, origin = 'null'): void {
  window.dispatchEvent(new MessageEvent('message', { source, origin, data: { channel: PDF_CHANNEL, instance: 'frame-one', ...data } }));
}
afterEach(() => { setReporterForTests(null); cleanup(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });

describe('full PDF viewer authorized byte bridge', () => {
  it('transfers bytes once to its own ready frame, checks nonce, and closes on unmount', async () => {
    const fetch = vi.fn(async (_url: string, _options: RequestInit) => new Response(new Uint8Array([37, 80, 68, 70])));
    vi.stubGlobal('fetch', fetch);
    const view = render(() => <PdfViewer part={part} blobUrl={`blob:${location.origin}/authorized`} mime="application/pdf" name="test.pdf"/>);
    const frame = view.container.querySelector('iframe')!;
    const post = vi.spyOn(frame.contentWindow!, 'postMessage');
    expect(frame.getAttribute('src')).toBe('/pdfjs/web/mailwoman.html');
    expect(frame.sandbox?.value ?? frame.getAttribute('sandbox')).not.toContain('allow-popups');
    message(frame, { type: 'ready' }, window);
    message(frame, { type: 'ready' }, frame.contentWindow, 'https://evil.test');
    expect(post).not.toHaveBeenCalled();
    message(frame, { type: 'ready' });
    await waitFor(() => expect(post).toHaveBeenCalledTimes(1));
    const payload = post.mock.calls[0]![0] as { bytes: ArrayBuffer; nonce: string };
    expect(new Uint8Array(payload.bytes)).toEqual(new Uint8Array([37, 80, 68, 70]));
    // An opaque-origin frame cannot be targeted by origin; the window itself is the address.
    expect(post.mock.calls[0]![1]).toBe('*');
    expect(frame.getAttribute('sandbox')).not.toContain('allow-same-origin');
    message(frame, { type: 'ready' });
    expect(post).toHaveBeenCalledTimes(1);
    message(frame, { type: 'loaded', nonce: 'wrong' });
    expect(view.container.firstElementChild?.getAttribute('data-pdf-state')).toBe('loading');
    message(frame, { type: 'loaded', nonce: payload.nonce });
    expect(view.container.firstElementChild?.getAttribute('data-pdf-state')).toBe('ready');
    const signal = fetch.mock.calls[0]![1] as RequestInit;
    view.unmount();
    expect((signal.signal as AbortSignal).aborted).toBe(true);
    expect(post).toHaveBeenLastCalledWith(expect.objectContaining({ type: 'destroy', nonce: payload.nonce }), '*', []);
  });
  it('aborts the old attachment and rejects its frame after a document switch', async () => {
    const requests: AbortSignal[] = [];
    vi.stubGlobal('fetch', vi.fn((_url: string, options: RequestInit) => { requests.push(options.signal as AbortSignal); return new Promise(() => {}); }));
    const [url, setUrl] = createSignal(`blob:${location.origin}/first`);
    const view = render(() => <PdfViewer part={part} blobUrl={url()} mime="application/pdf" name="test.pdf"/>);
    const previous = view.container.querySelector('iframe')!;
    setUrl(`blob:${location.origin}/second`);
    expect(requests[0]?.aborted).toBe(true);
    expect(view.container.querySelector('iframe')).not.toBe(previous);
    expect(requests).toHaveLength(2);
  });
  it('recovers a reloaded iframe with fresh authorized bytes and a new nonce', async () => {
    const fetch = vi.fn(async () => new Response(new Uint8Array([37, 80, 68, 70])));
    vi.stubGlobal('fetch', fetch);
    const view = render(() => <PdfViewer part={part} blobUrl={`blob:${location.origin}/reload`} mime="application/pdf" name="test.pdf"/>);
    const frame = view.container.querySelector('iframe')!;
    const post = vi.spyOn(frame.contentWindow!, 'postMessage');
    message(frame, { type: 'ready' });
    await waitFor(() => expect(post).toHaveBeenCalledTimes(1));
    const first = post.mock.calls[0]![0] as { nonce: string; instance: string };
    expect(first.instance).toBe('frame-one');
    message(frame, { type: 'loaded', nonce: first.nonce });
    message(frame, { type: 'ready', instance: 'frame-two' });
    await waitFor(() => expect(post).toHaveBeenCalledTimes(2));
    const second = post.mock.calls[1]![0] as { nonce: string; instance: string };
    expect(second.instance).toBe('frame-two');
    expect(second.nonce).not.toBe(first.nonce);
    expect(fetch).toHaveBeenCalledTimes(2);
    message(frame, { type: 'loaded', nonce: first.nonce });
    expect(view.container.firstElementChild?.getAttribute('data-pdf-state')).toBe('loading');
    message(frame, { type: 'loaded', nonce: second.nonce, instance: 'frame-two' });
    expect(view.container.firstElementChild?.getAttribute('data-pdf-state')).toBe('ready');
  });
  it('files a frame error and a failed byte fetch with the client error reporter under the pdf boundary', async () => {
    const report = vi.fn(() => null);
    setReporterForTests({ report } as unknown as Reporter);
    const fetch = vi.fn(async (_url: string, _options: RequestInit) => new Response(new Uint8Array([37, 80, 68, 70])));
    vi.stubGlobal('fetch', fetch);
    const view = render(() => <PdfViewer part={part} blobUrl={`blob:${location.origin}/frame-error`} mime="application/pdf" name="test.pdf"/>);
    const frame = view.container.querySelector('iframe')!;
    const post = vi.spyOn(frame.contentWindow!, 'postMessage');
    message(frame, { type: 'ready' });
    await waitFor(() => expect(post).toHaveBeenCalledTimes(1));
    const nonce = (post.mock.calls[0]![0] as { nonce: string }).nonce;
    message(frame, { type: 'error', nonce, name: 'InvalidPDFException', message: 'Invalid PDF structure', stack: 'InvalidPDFException: Invalid PDF structure\n    at a (pdf.mjs:1:1)' });
    expect(view.container.firstElementChild?.getAttribute('data-pdf-state')).toBe('error');
    expect(report).toHaveBeenCalledTimes(1);
    const [error, kind, boundary] = report.mock.calls[0] as unknown as [Error, string, string];
    expect(error.name).toBe('InvalidPDFException');
    expect(error.message).toBe('Invalid PDF structure');
    expect(kind).toBe('boundary');
    expect(boundary).toBe('Reader/AttachmentViewer/pdf');
    view.unmount();
    vi.stubGlobal('fetch', vi.fn(async () => new Response('', { status: 403 })));
    const second = render(() => <PdfViewer part={part} blobUrl={`blob:${location.origin}/refused-report`} mime="application/pdf" name="test.pdf"/>);
    await waitFor(() => expect(second.container.firstElementChild?.getAttribute('data-pdf-state')).toBe('error'));
    expect(report).toHaveBeenCalledTimes(2);
    expect((report.mock.calls[1] as unknown as [Error])[0].message).toBe('PDF unavailable (HTTP 403)');
  });
  it('shows the real reason in a technical-details section and files it with the stage, pdf.js version and browser', async () => {
    const report = vi.fn(() => null);
    setReporterForTests({ report } as unknown as Reporter);
    vi.stubGlobal('fetch', vi.fn(async () => new Response(new Uint8Array([37, 80, 68, 70]))));
    const view = render(() => <PdfViewer part={part} blobUrl={`blob:${location.origin}/details`} mime="application/pdf" name="test.pdf"/>);
    const frame = view.container.querySelector('iframe')!;
    const post = vi.spyOn(frame.contentWindow!, 'postMessage');
    message(frame, { type: 'ready' });
    await waitFor(() => expect(post).toHaveBeenCalledTimes(1));
    const nonce = (post.mock.calls[0]![0] as { nonce: string }).nonce;
    message(frame, { type: 'error', nonce, stage: 'library', name: 'TypeError', message: 'Promise.try is not a function', pdfjs: '6.1.200', polyfilled: 'Promise.try,Math.sumPrecise', worker: 'main-thread: no ready message within 8 s' });
    const details = view.container.querySelector('[data-testid="pdf-failure-details"]')!;
    expect(details.tagName).toBe('DETAILS');
    expect(details.hasAttribute('open')).toBe(false);
    const text = details.textContent ?? '';
    for (const expected of ['library', 'TypeError: Promise.try is not a function', '6.1.200', 'main-thread: no ready message within 8 s', 'Promise.try, Math.sumPrecise', navigator.userAgent]) expect(text).toContain(expected);
    expect(view.container.querySelector('[role="alert"]')!.contains(details)).toBe(true);
    const context = (report.mock.calls[0] as unknown as [Error, string, string, { componentStack: string }])[3].componentStack;
    expect(context).toContain('stage=library');
    expect(context).toContain('pdf.js=6.1.200');
    expect(context).toContain('browser=');
  });
  it('names the host-side stage: a refused download is a fetch failure, a silent frame a handshake failure', async () => {
    const report = vi.fn(() => null);
    setReporterForTests({ report } as unknown as Reporter);
    vi.stubGlobal('fetch', vi.fn(async () => new Response('', { status: 502 })));
    const refused = render(() => <PdfViewer part={part} blobUrl={`blob:${location.origin}/stage-fetch`} mime="application/pdf" name="test.pdf"/>);
    await waitFor(() => expect(refused.container.querySelector('[data-testid="pdf-failure-details"]')).not.toBeNull());
    expect(refused.container.querySelector('[data-testid="pdf-failure-details"]')!.textContent).toContain('fetch');
    expect(refused.container.querySelector('[data-testid="pdf-failure-details"]')!.textContent).toContain('HTTP 502');
    refused.unmount();
    vi.useFakeTimers();
    try {
      vi.stubGlobal('fetch', vi.fn(() => new Promise(() => {})));
      const silent = render(() => <PdfViewer part={part} blobUrl={`blob:${location.origin}/stage-handshake`} mime="application/pdf" name="test.pdf"/>);
      await vi.advanceTimersByTimeAsync(45_000);
      expect(silent.container.querySelector('[data-testid="pdf-failure-details"]')!.textContent).toContain('handshake');
      expect(silent.container.querySelector('[data-testid="pdf-failure-details"]')!.textContent).toContain('PDF viewer frame did not answer');
    } finally { vi.useRealTimers(); }
  });
  it('shows fetch failures without handing an attachment URL to PDF.js', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('', { status: 403 })));
    const view = render(() => <PdfViewer part={part} blobUrl={`blob:${location.origin}/refused`} mime="application/pdf" name="test.pdf"/>);
    await waitFor(() => expect(view.container.firstElementChild?.getAttribute('data-pdf-state')).toBe('error'));
    expect(view.container.querySelector('iframe')!.src).not.toContain('blob:refused');
  });
  it('refuses arbitrary network URLs without fetching them', async () => {
    const fetch = vi.fn();
    vi.stubGlobal('fetch', fetch);
    const view = render(() => <PdfViewer part={part} blobUrl="https://outside.test/file.pdf" mime="application/pdf" name="test.pdf"/>);
    await waitFor(() => expect(view.container.firstElementChild?.getAttribute('data-pdf-state')).toBe('error'));
    expect(fetch).not.toHaveBeenCalled();
  });
  it('ends a stuck bootstrap load and rejects late messages after its deadline', async () => {
    vi.useFakeTimers();
    try {
      vi.stubGlobal('fetch', vi.fn(() => new Promise(() => {})));
      const view = render(() => <PdfViewer part={part} blobUrl={`blob:${location.origin}/stuck`} mime="application/pdf" name="test.pdf"/>);
      const frame = view.container.querySelector('iframe')!;
      const post = vi.spyOn(frame.contentWindow!, 'postMessage');
      await vi.advanceTimersByTimeAsync(45_000);
      expect(view.container.firstElementChild?.getAttribute('data-pdf-state')).toBe('error');
      message(frame, { type: 'ready' });
      expect(post).not.toHaveBeenCalled();
    } finally { vi.useRealTimers(); }
  });
});
