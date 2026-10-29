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

describe('reader body zoom (t59)', () => {
  it('adds no zoom rule at 100% or when absent', () => {
    expect(readerAppearanceCss(appearance)).not.toContain('zoom');
    expect(readerAppearanceCss({ ...appearance, zoom: 100 })).not.toContain('zoom');
  });
  it('scales the frame root, bounds the level and prints at 100%', () => {
    expect(readerAppearanceCss({ ...appearance, zoom: 125 })).toContain('html{zoom:1.25}');
    expect(readerAppearanceCss({ ...appearance, zoom: 500 })).toContain('html{zoom:2}');
    expect(readerAppearanceCss({ ...appearance, zoom: 10 })).toContain('html{zoom:0.5}');
    expect(readerAppearanceCss({ ...appearance, zoom: Number.NaN })).not.toContain('zoom');
    expect(readerAppearanceCss({ ...appearance, zoom: 150 })).toContain('@media print{html{zoom:1!important}}');
  });
  it('reaches the body through the frame stylesheet, keeping the CSP and dark adaptation', () => {
    const doc = new DOMParser().parseFromString(bodyFrameDoc('full-sanitized', { html: '<p style="color:black">Hi</p><img src="data:image/png;base64,AA==">' }, { themeVars: themeCssVars('dark'), appearance: { ...appearance, zoom: 175 } }), 'text/html');
    const css = doc.querySelector('style')?.textContent ?? '';
    expect(css).toContain('html{zoom:1.75}');
    expect(css).toContain('background:var(--mw-color-bg)!important');
    expect(doc.querySelector('p')?.hasAttribute('style')).toBe(false);
    expect(doc.querySelector('meta[http-equiv]')?.getAttribute('content')).toContain("default-src 'none'");
    expect(doc.querySelector('script')).toBeNull();
    expect(doc.body.querySelector('[style*="zoom"]')).toBeNull();
  });
});
