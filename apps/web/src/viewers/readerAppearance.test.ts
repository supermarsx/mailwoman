import { describe, it, expect, vi } from 'vitest';
import { bodyFrameDoc } from './sandbox.ts';
import { adaptReaderColors, readerAppearanceCss, type ReaderAppearance } from './readerAppearance.ts';
import { themeCssVars } from '../theme/themeCssVars.ts';

const appearance: ReaderAppearance = { colorMode: 'adaptive', fontSize: 18, lineHeight: 1.8, contentWidth: 'comfortable' };
describe('isolated reader appearance', () => {
  it('removes sender paint overrides while retaining layout and image destinations', () => {
    const doc = new DOMParser().parseFromString(bodyFrameDoc('full-sanitized', { html: '<table bgcolor="white"><tr><td style="color:black!important;background:#fff;width:200px">Readable</td></tr></table><img src="data:image/png;base64,AA==" width="32">' }, { themeVars: themeCssVars('dark'), appearance }), 'text/html');
    expect(doc.querySelector('td')?.style.color).toBe('');
    expect(doc.querySelector('td')?.style.width).toBe('200px');
    expect(doc.querySelector('table')?.hasAttribute('bgcolor')).toBe(false);
    expect(doc.querySelector('img')?.getAttribute('src')).toBe('data:image/png;base64,AA==');
    expect(doc.querySelector('meta[http-equiv]')?.getAttribute('content')).toContain("default-src 'none'");
    expect(doc.querySelector('script')).toBeNull();
  });
  it('removes sender paint even where the browser leaves inline styles unparsed', () => {
    // Chromium does not parse style attributes in a DOMParser document under the app's
    // style-src CSP, so CSSOM edits there are no-ops; model that with a no-op removeProperty.
    const removeProperty = vi.spyOn(CSSStyleDeclaration.prototype, 'removeProperty').mockReturnValue('');
    try {
      const html = adaptReaderColors('<p style="COLOR: red; background-image:url(data:image/gif;base64,R0lG);margin:1px;font-family:\'a;b\'">x</p><table><tr><td style="background:#fff">y</td></tr></table>');
      const doc = new DOMParser().parseFromString(html, 'text/html');
      expect(doc.querySelector('p')?.getAttribute('style')).toBe("margin:1px;font-family:'a;b'");
      expect(doc.querySelector('td')?.hasAttribute('style')).toBe(false);
    } finally { removeProperty.mockRestore(); }
  });
  it('preserves sender paint in original mode and reverses image inversion', () => {
    const original = bodyFrameDoc('full-sanitized', { html: '<p style="color:red">Original</p>' }, { appearance: { ...appearance, colorMode: 'original' } });
    expect(original).toContain('style="color:red"');
    const invert = readerAppearanceCss({ ...appearance, colorMode: 'invert' });
    expect(invert).toContain('img,video,svg{filter:invert(1) hue-rotate(180deg)}');
    expect(invert).toContain('@media print{html{filter:none!important}');
  });
  it('bounds numeric style values and keeps plain text escaped', () => {
    expect(readerAppearanceCss({ ...appearance, fontSize: Infinity, lineHeight: -5 })).toContain('font-size:14px;line-height:1.2');
    const doc = bodyFrameDoc('plain-text', { text: '<img src=x onerror=alert(1)>' }, { appearance });
    expect(doc).toContain('&lt;img');
    expect(doc).not.toContain('<img src=x');
  });
});

describe('reader body zoom (t59, t79)', () => {
  it('never reaches the frame document, so a zoom change does not reload it', () => {
    // The app scales the frame from outside (ReaderZoomBox); the document the
    // frame loads is the same at every level.
    const doc = bodyFrameDoc('full-sanitized', { html: '<p style="color:black">Hi</p><img src="data:image/png;base64,AA==">' }, { themeVars: themeCssVars('dark'), appearance });
    expect(doc).not.toContain('zoom');
    const parsed = new DOMParser().parseFromString(doc, 'text/html');
    expect(parsed.querySelector('style')?.textContent ?? '').toContain('background:var(--mw-color-bg)!important');
    expect(parsed.querySelector('meta[http-equiv]')?.getAttribute('content')).toContain("default-src 'none'");
    expect(parsed.querySelector('script')).toBeNull();
  });
});
