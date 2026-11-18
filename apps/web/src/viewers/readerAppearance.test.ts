import { describe, it, expect, vi } from 'vitest';
import { bodyFrameDoc } from './sandbox.ts';
import { adaptReaderColors, readerAppearanceCss, type ReaderAppearance } from './readerAppearance.ts';
import { themeCssVars } from '../theme/themeCssVars.ts';

const appearance: ReaderAppearance = { colorMode: 'adaptive', fontSize: 18, lineHeight: 1.8, contentWidth: 'comfortable' };
describe('isolated reader appearance', () => {
  it('uses the available width without adding body padding or centering the message', () => {
    const doc = bodyFrameDoc('full-sanitized', { html: '<p>Original alignment</p>' }, { appearance: { ...appearance, contentWidth: 'full' } });
    expect(doc).toContain('body{padding:0;');
    expect(doc).toContain('max-width:none;margin-inline:0');
    expect(doc).not.toContain('margin-inline:auto');
  });
  it('removes sender paint overrides while retaining layout and image destinations', () => {
    const doc = new DOMParser().parseFromString(bodyFrameDoc('full-sanitized', { html: '<table bgcolor="white"><tr><td style="color:black!important;background:#fff;width:200px">Readable</td></tr></table><img src="data:image/png;base64,AA==" width="32">' }, { themeVars: themeCssVars('dark'), appearance }), 'text/html');
    expect(doc.querySelector('td')?.style.color).toBe('');
    expect(doc.querySelector('td')?.style.width).toBe('200px');
    expect(doc.querySelector('table')?.hasAttribute('bgcolor')).toBe(false);
    expect(doc.querySelector('img')?.getAttribute('src')).toBe('data:image/png;base64,AA==');
    expect(doc.querySelector('meta[http-equiv]')?.getAttribute('content')).toContain("default-src 'none'");
    expect(doc.querySelector('script')).toBeNull();
  });
  it('removes sender paint without constructing an active DOMParser document', () => {
    const parser=vi.spyOn(DOMParser.prototype,'parseFromString').mockImplementation(()=>{throw new Error('must remain inert');});
    let html:string;
    try {html=adaptReaderColors('<p style="COLOR: red; background-image:url(data:image/gif;base64,R0lG);margin:1px;font-family:format">x</p><table><tr><td style="background:#fff">y</td></tr></table>');}
    finally {parser.mockRestore();}
    const doc=new DOMParser().parseFromString(html!,'text/html');
    expect(doc.querySelector('p')?.style.color).toBe('');expect(doc.querySelector('p')?.style.margin).toBe('1px');
    expect(doc.querySelector('td')?.hasAttribute('style')).toBe(false);
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
    expect(Array.from(parsed.querySelectorAll('style')).map(style => style.textContent).join('')).toContain('background:var(--mw-color-bg)!important');
    expect(parsed.querySelector('meta[http-equiv]')?.getAttribute('content')).toContain("default-src 'none'");
    expect(parsed.querySelector('script')).toBeNull();
  });
});
