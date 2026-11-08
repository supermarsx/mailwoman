/** Narrow protocol shared with the self-hosted generic PDF viewer bootstrap. */
export const PDF_CHANNEL = 'mailwoman-pdf-v1';
/** The viewer frame is sandboxed without `allow-same-origin`, so its origin is opaque and its messages carry this origin. */
export const PDF_FRAME_ORIGIN = 'null';
/** Scripts only: the frame never shares the app's origin, storage or cookies. */
export const PDF_FRAME_SANDBOX = 'allow-scripts allow-downloads allow-modals';
export function authorizedPdfBlob(value: string, origin = location.origin): boolean {
  try { const url = new URL(value); return url.protocol === 'blob:' && url.origin === origin; } catch { return false; }
}
export function validPdfMessage(event: MessageEvent, source: Window|null, origin: string): boolean {
  return source !== null && event.source === source && event.origin === origin && event.data !== null && typeof event.data === 'object' && event.data.channel === PDF_CHANNEL;
}
export function safePdfLink(value: unknown): string|null {
  if (typeof value !== 'string' || value.length > 16_384 || [...value].some(char => char.charCodeAt(0) < 32 || char.charCodeAt(0) === 127)) return null;
  try {
    const url = new URL(value);
    return ['https:', 'http:', 'mailto:', 'tel:'].includes(url.protocol) && !url.username && !url.password ? url.href : null;
  } catch { return null; }
}
/** Where a PDF open stopped. The viewer frame names its own stages; the host names the first three. */
export type PdfStage = 'authorize' | 'fetch' | 'handshake' | 'timeout' | 'script' | 'worker' | 'stylesheets' | 'library' | 'viewer' | 'initialize' | 'document' | 'loaded' | 'unknown';
const STAGES: readonly string[] = ['authorize', 'fetch', 'handshake', 'timeout', 'script', 'worker', 'stylesheets', 'library', 'viewer', 'initialize', 'document', 'loaded'];
/** What a failed open can say about itself: no document content, only the failing step and the browser's gaps. */
export interface PdfFailure { stage: PdfStage; name: string; message: string; pdfjs: string; polyfilled: string; worker: string }
export interface PdfFailureError extends Error { pdf: PdfFailure }
const text = (v: unknown, max: number): string => (typeof v === 'string' ? v.slice(0, max) : '');
/** The failure of a host-side step (fetching the bytes, the handshake): `reason` is whatever was thrown. */
export function pdfHostFailure(stage: PdfStage, reason: unknown): PdfFailure {
  const e = reason instanceof Error ? reason : null;
  return { stage, name: text(e?.name, 80).replace(/[^\w.$-]/g, '') || 'Error', message: text(e?.message ?? (typeof reason === 'string' ? reason : ''), 300) || 'PDF viewer failed', pdfjs: '', polyfilled: '', worker: '' };
}
/** The failure a PDF frame reports through its `error` message (bounded; the host scrubs it again). */
export function pdfFrameFailure(data: unknown): PdfFailure {
  const d = (data !== null && typeof data === 'object' ? data : {}) as Record<string, unknown>;
  const stage = text(d['stage'], 32);
  return {
    stage: (STAGES.includes(stage) ? stage : 'unknown') as PdfStage,
    name: text(d['name'], 80).replace(/[^\w.$-]/g, '') || 'PdfViewerError',
    message: text(d['message'], 300) || 'PDF viewer failed',
    pdfjs: text(d['pdfjs'], 40).replace(/[^\w.+-]/g, ''),
    polyfilled: text(d['polyfilled'], 400).replace(/[^\w.,]/g, ''),
    // Worker error messages are diagnostic text, with control characters removed.
    // eslint-disable-next-line no-control-regex
    worker: text(d['worker'], 300).replace(/[\u0000-\u001f\u007f]/g, ' '),
  };
}
/** The error a sandboxed PDF frame reports through its `error` message (bounded; the host scrubs it again). */
export function pdfFrameError(data: unknown): PdfFailureError {
  const d = (data !== null && typeof data === 'object' ? data : {}) as Record<string, unknown>;
  const pdf = pdfFrameFailure(data);
  const error = new Error(pdf.message) as PdfFailureError;
  error.name = pdf.name;
  const stack = text(d['stack'], 2000);
  error.stack = stack !== '' ? stack : `${error.name}: ${error.message}`;
  error.pdf = pdf;
  return error;
}
/** The labelled lines of the "Technical details" section (also what the user can copy for a report). */
export function pdfFailureLines(failure: PdfFailure, browser: string): [label: string, value: string][] {
  const lines: [string, string][] = [['stage', failure.stage], ['error', `${failure.name}: ${failure.message}`]];
  if (failure.pdfjs !== '') lines.push(['pdf.js', failure.pdfjs]);
  if (failure.worker !== '') lines.push(['worker', failure.worker]);
  if (failure.polyfilled !== '') lines.push(['polyfilled', failure.polyfilled.split(',').join(', ')]);
  lines.push(['browser', browser.slice(0, 300)]);
  return lines;
}
/** The one-line context filed with the client error reporter next to the error itself. */
export function pdfFailureContext(failure: PdfFailure, browser: string): string {
  return pdfFailureLines(failure, browser).map(([label, value]) => `${label}=${value}`).join(' | ');
}
/** The boundary name the PDF bridge reports under. */
export const PDF_ERROR_BOUNDARY = 'Reader/AttachmentViewer/pdf';
