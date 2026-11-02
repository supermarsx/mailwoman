import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import { initSync, sanitizeSignatureHtml } from '../../wasm/mw-sanitize/mw_sanitize.js';
import { normalizeHtmlSource } from './htmlSource.ts';

beforeAll(() => {
  initSync({ module: readFileSync(resolve(process.cwd(), 'src/wasm/mw-sanitize/mw_sanitize_bg.wasm')) });
});

describe('native HTML source with the shipped Rust WASM policy', () => {
  it('preserves document presentation, tables, styles and classes without schema reduction', async () => {
    const source = '<html lang="en"><head><style>td{padding:12px}.brand{color:#123456}</style></head><body dir="rtl" style="font-family:Georgia;font-size:18px"><table width="600" cellpadding="8" cellspacing="0" class="brand"><tr><td valign="middle"><font face="Arial" size="4">Native signature</font></td></tr></table></body></html>';
    const result = await normalizeHtmlSource(source);
    const doc = new DOMParser().parseFromString(result.html, 'text/html');
    expect(doc.querySelector('.mw-email-body')?.getAttribute('dir')).toBe('rtl');
    expect(doc.querySelector('.mw-email-body')?.getAttribute('style')).toContain('font-family: Georgia');
    expect(doc.querySelector('table')?.getAttribute('width')).toBe('600');
    expect(doc.querySelector('table')?.getAttribute('cellpadding')).toBe('8');
    expect(doc.querySelector('table')?.className).toBe('brand');
    expect(doc.querySelector('td')?.getAttribute('valign')).toBe('middle');
    expect(doc.querySelector('font')?.getAttribute('face')).toBe('Arial');
    expect(doc.querySelector('style')?.textContent).toContain('.mw-email-body td{padding:12px}');
    expect(result.text).toBe('Native signature');
  });

  it('removes active markup, external images, unsafe links and unowned CID references', async () => {
    const source = '<style>p{background:url(cid:foreign)}</style><p onclick="evil()" style="color:red;background:url(cid:owned)">Safe</p><script>evil()</script><iframe srcdoc="evil()"></iframe><a href="java&#x09;script:evil()">Link</a><img src="https://evil.test/track.gif"><img src="cid:foreign"><img src="cid:owned"><table background="cid:foreign"><tr><td>Cell</td></tr></table>';
    const result = await normalizeHtmlSource(source, new Set(['owned']));
    const doc = new DOMParser().parseFromString(result.html, 'text/html');
    expect(doc.querySelector('script,iframe,[onclick]')).toBeNull();
    expect(doc.querySelector('a')?.getAttribute('href')).toBeNull();
    expect(result.html).not.toContain('foreign');
    expect(doc.querySelector('img[src]')?.getAttribute('src')).toMatch(/^cid:owned$/i);
    expect(result.html).not.toContain('evil.test');
    expect(doc.querySelector('p')?.getAttribute('style')).toContain('cid:owned');
    expect(result.text).toBe('Safe\nLinkCell');
  });

  it('admits validated embedded raster images only for signatures', async () => {
    const png = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAusB9Y9ZlS8AAAAASUVORK5CYII=';
    const source = `<img src="${png}"><img src="data:image/svg+xml;base64,PHN2Zy8+"><img src="data:text/html;base64,PHNjcmlwdD4=">`;
    expect(sanitizeSignatureHtml(source)).toContain(png);
    expect((await normalizeHtmlSource(source)).html).not.toContain('data:');
    const signature = await normalizeHtmlSource(source, undefined, true);
    expect(signature.html).toContain(png);
    expect(signature.html).not.toMatch(/svg\+xml|text\/html/);
  });

  it('normalizes emitted layout repeatedly without wrapper growth or CSS re-scoping', async () => {
    const first = await normalizeHtmlSource('<body style="font-family:Georgia"><style>body{color:red}td{padding:12px}</style><table><tr><td>Signature</td></tr></table></body>');
    const second = await normalizeHtmlSource(first.html);
    const third = await normalizeHtmlSource(second.html);
    expect(second).toEqual(first);
    expect(third).toEqual(first);
    expect(first.html.match(/class="mw-email-body"/g)).toHaveLength(1);
    expect(first.html).not.toContain('.mw-email-body .mw-email-body');
  });

  it('narrows ownership again on every normalization and rejects oversized source', async () => {
    const first = await normalizeHtmlSource('<p style="background:url(cid:mine)">Hello<br>World</p><img src="cid:mine">', new Set(['mine']));
    expect(first.text).toBe('Hello\nWorld');
    const second = await normalizeHtmlSource(first.html);
    expect(second.html).not.toContain('cid:');
    await expect(normalizeHtmlSource('x'.repeat(8 * 1024 * 1024 + 1))).rejects.toThrow('too large');
  });

});
