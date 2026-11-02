import { describe, expect, it } from 'vitest';
import { authorizedPdfBlob, safePdfLink, validPdfMessage, PDF_CHANNEL } from './pdfBridge.ts';
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
    const event = new MessageEvent('message', { source: window, origin: location.origin, data: { channel: PDF_CHANNEL } });
    expect(validPdfMessage(event, window, location.origin)).toBe(true);
    expect(validPdfMessage(event, null, location.origin)).toBe(false);
    expect(validPdfMessage(event, window, 'https://other.test')).toBe(false);
    expect(validPdfMessage(new MessageEvent('message', { source: window, origin: location.origin, data: { channel: 'other' } }), window, location.origin)).toBe(false);
  });
});
