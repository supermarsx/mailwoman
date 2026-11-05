import { describe, expect, it } from 'vitest';
import { BLOCKED_IMAGE_STYLE, bodyFrameDoc, withBlockedImagePlaceholders } from './sandbox.ts';

const GIF = 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';
const body = (html: string, mode: 'full-sanitized' | 'sanitized-no-media' = 'full-sanitized') => {
  const doc = new DOMParser().parseFromString(bodyFrameDoc(mode, { html }), 'text/html');
  return doc.body.querySelector('img');
};
const kind = (html: string, mode?: 'full-sanitized' | 'sanitized-no-media') => body(html, mode)?.getAttribute('data-mw-ph');

describe('blocked image placeholders', () => {
  it('marks every real blocked image in both frame modes, and keeps the element', () => {
    for (const mode of ['full-sanitized', 'sanitized-no-media'] as const) {
      const img = body(`<img src="${GIF}">`, mode);
      expect(img, mode).not.toBeNull();
      expect(img?.getAttribute('data-mw-ph')).toBe('box');
      expect(img?.hasAttribute('src')).toBe(false);
      expect(img?.getAttribute('data-mw-blocked-src')).toBe('');
      expect(bodyFrameDoc(mode, { html: `<img src="${GIF}">` })).toContain(BLOCKED_IMAGE_STYLE);
    }
  });
  it('classifies sizing', () => {
    expect(kind(`<img src="${GIF}" width="600" height="200">`)).toBe('sized');
    expect(body(`<img src="${GIF}" width="600" height="200">`)?.getAttribute('style')).toContain('aspect-ratio:600/200');
    expect(kind(`<img src="${GIF}" width="50%">`)).toBe('partial');
    expect(kind(`<img src="${GIF}" style="width:320px">`)).toBe('partial');
    expect(kind(`<img src="${GIF}" style="width:100%;height:120px">`)).toBe('sized');
    expect(kind(`<img src="${GIF}" width="12" height="12" alt="icon">`)).toBe('min');
    expect(kind(`<img src="${GIF}" alt="">`)).toBe('deco');
    expect(kind(`<img>`)).toBe('box');
    expect(kind(`<img srcset="mw-rc:1 2x" alt="x">`)).toBe('box');
  });
  it('keeps trackers invisible rather than drawing a placeholder', () => {
    expect(kind(`<img src="${GIF}" width="1" height="1">`)).toBe('hidden');
    expect(kind(`<img src="${GIF}" style="display:none">`)).toBe('hidden');
    expect(kind(`<img src="${GIF}" height="0" alt="t">`)).toBe('hidden');
    expect(BLOCKED_IMAGE_STYLE).toContain('visibility:hidden');
  });
  it('gives accessible names from alt without parsing it as markup', () => {
    const img = body(`<img src="${GIF}" alt="&lt;b&gt;Sale&lt;/b&gt; שלום">`);
    expect(img?.getAttribute('role')).toBe('img');
    expect(img?.getAttribute('aria-label')).toBe('<b>Sale</b> שלום');
    expect(img?.getAttribute('dir')).toBe('auto');
    expect(body(`<img src="${GIF}" alt="">`)?.getAttribute('aria-hidden')).toBe('true');
  });
  it('leaves loaded and signed images alone', () => {
    const html = '<img src="http://localhost/api/rc/i/abc" width="10" height="10">';
    expect(withBlockedImagePlaceholders(html)).toBe(html);
    expect(withBlockedImagePlaceholders('<p>no images</p>')).toBe('<p>no images</p>');
  });
  it('only matches while the image has no real source, so a hot-patched src takes over', () => {
    expect(BLOCKED_IMAGE_STYLE).toContain('img[data-mw-ph]:not([src])');
    expect(BLOCKED_IMAGE_STYLE).not.toMatch(/url\(/);
  });
  it('keeps picture/source structure out of the no-media frame without losing the image', () => {
    const doc = new DOMParser().parseFromString(bodyFrameDoc('sanitized-no-media', { html: '<picture><source srcset="mw-rc:1"><img src="mw-rc:1" alt="Hero" width="300" height="100"></picture>' }), 'text/html');
    expect(doc.querySelector('source')).toBeNull();
    expect(doc.querySelector('picture > img[data-mw-ph="sized"]')).not.toBeNull();
  });
});
