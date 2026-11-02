/** Narrow protocol shared with the self-hosted generic PDF viewer bootstrap. */
export const PDF_CHANNEL = 'mailwoman-pdf-v1';
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
