import { describe, it, expect } from 'vitest';
import { bodyFrameDoc } from './sandbox.ts';
import { readerAppearanceCss, type ReaderAppearance } from './readerAppearance.ts';
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
