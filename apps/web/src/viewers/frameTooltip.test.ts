import { describe, expect, it } from 'vitest';
import { bodyCsp, bodyFrameDoc } from './sandbox.ts';
import { FRAME_TIP_STYLE, frameTipParts, frameTipVars, withoutImageTitles } from './frameTooltip.ts';
import { THEMES } from '../theme/tokens.ts';

const labels = { blocked: 'Image blocked. Choose Show images to load it.', loading: 'Loading image…', error: 'Image unavailable' };
const parse = (html: string): Document => new DOMParser().parseFromString(html, 'text/html');

describe('withoutImageTitles', () => {
  it('moves an image title into data-mw-tip so no native tooltip remains', () => {
    const img = parse(withoutImageTitles('<img src="x" alt="a" title="Sender hint">')).querySelector('img')!;
    expect(img.hasAttribute('title')).toBe(false);
    expect(img.getAttribute('data-mw-tip')).toBe('Sender hint');
    expect(img.getAttribute('alt')).toBe('a');
  });
  it('strips the title of a link that wraps only an image and keeps it as the tip', () => {
    const doc = parse(withoutImageTitles('<a href="https://e.test" title="Go"><img alt="Logo"></a>'));
    expect(doc.querySelector('a')!.hasAttribute('title')).toBe(false);
    expect(doc.querySelector('img')!.getAttribute('data-mw-tip')).toBe('Go');
    expect(doc.querySelector('a')!.getAttribute('href')).toBe('https://e.test');
  });
  it('keeps the image title over the link title and drops empty titles', () => {
    const doc = parse(withoutImageTitles('<a href="#" title="Link"><img title="Img"></a><img title="  ">'));
    const [first, second] = Array.from(doc.querySelectorAll('img'));
    expect(first!.getAttribute('data-mw-tip')).toBe('Img');
    expect(second!.hasAttribute('data-mw-tip')).toBe(false);
    expect(doc.querySelectorAll('[title]').length).toBe(0);
  });
  it('leaves ordinary text links and links with text beside the image alone', () => {
    const doc = parse(withoutImageTitles('<a href="#" title="Text link">read</a><a href="#" title="Mixed">more <img alt="i" title="t"></a>'));
    expect(doc.querySelectorAll('a[title]').length).toBe(2);
    expect(doc.querySelector('img')!.hasAttribute('title')).toBe(false);
  });
  it('returns the input byte-identical when nothing applies', () => {
    expect(withoutImageTitles('<p title="x">hi</p>')).toBe('<p title="x">hi</p>');
    expect(withoutImageTitles('<img src="x">')).toBe('<img src="x">');
  });
});

describe('frame tooltip tokens and style', () => {
  it('takes colours, radius, shadow and face from the app theme tokens (light, dark, high contrast)', () => {
    for (const name of ['light', 'dark', 'hc-dark'] as const) {
      const css = frameTipVars(name), t = THEMES[name];
      expect(css).toContain(`--mw-tip-bg:${t.color.surface}`);
      expect(css).toContain(`--mw-tip-text:${t.color.text}`);
      expect(css).toContain(`--mw-tip-border:${t.color.borderStrong}`);
      expect(css).toContain(`--mw-tip-shadow:${t.elevation[2]}`);
      expect(css).toContain(`--mw-tip-radius:${t.radius.sm}`);
    }
    expect(frameTipVars('light')).not.toBe(frameTipVars('dark'));
  });
  it('is sized in em, honours reduced motion and forced colours, and takes no pointer events', () => {
    expect(FRAME_TIP_STYLE).toContain('font:normal var(--mw-tip-size,13px)');
    expect(frameTipVars('dark')).toContain('--mw-tip-size:13px');
    expect(FRAME_TIP_STYLE).toContain('padding:.615em .923em');
    expect(FRAME_TIP_STYLE).toContain('prefers-reduced-motion:reduce');
    expect(FRAME_TIP_STYLE).toContain('forced-colors:active');
    expect(FRAME_TIP_STYLE).toContain('pointer-events:none');
    expect(FRAME_TIP_STYLE).toContain('unicode-bidi:plaintext');
    expect(FRAME_TIP_STYLE).not.toMatch(/(?:padding|max-width):[^;]*\dpx/);
  });
  it('counter-inverts the bubble under the invert reader mode and escapes label text', () => {
    expect(frameTipParts({ theme: 'dark', labels, invert: true }).css).toContain('.mw-frame-tip{filter:invert(1)');
    expect(frameTipParts({ theme: 'dark', labels }).css).not.toContain('filter:invert');
    expect(frameTipParts({ theme: 'dark', labels: { ...labels, error: 'a"<b' } }).bodyAttrs).toContain('data-mw-tip-error="a&quot;&lt;b"');
    expect(frameTipParts(undefined)).toEqual({ css: '', bodyAttrs: '' });
  });
});

describe('bodyFrameDoc wiring', () => {
  const html = '<img src="mw-rc:3" alt="Hero" title="Native"><a href="https://e.test" title="L"><img src="mw-rc:4"></a>';
  it('emits no title on body images, plus the tooltip css and strings, in every html mode', () => {
    for (const mode of ['full-sanitized', 'sanitized-no-media'] as const) {
      const doc = bodyFrameDoc(mode, { html }, { tips: { theme: 'dark', labels } });
      const parsed = parse(doc);
      expect(parsed.body.querySelectorAll('[title]').length).toBe(0);
      expect(doc).toContain('.mw-frame-tip{');
      expect(doc).toContain(frameTipVars('dark'));
      expect(parsed.body.getAttribute('data-mw-tip-loading')).toBe(labels.loading);
      expect(parsed.querySelector('img')!.getAttribute('data-mw-tip')).toBe('Native');
    }
  });
  it('adds nothing for plain text, and never adds script or widens the CSP', () => {
    expect(bodyFrameDoc('plain-text', { text: 'x' }, { tips: { theme: 'dark', labels } })).not.toContain('mw-frame-tip');
    for (const mode of ['full-sanitized', 'sanitized-no-media', 'plain-text'] as const) {
      const doc = bodyFrameDoc(mode, { html: '<p>x</p>', text: 'x' }, { tips: { theme: 'light', labels } });
      expect(doc).not.toMatch(/<script/i);
      expect(doc).toContain(bodyCsp(mode).replace(/"/g, '&quot;'));
      expect(bodyCsp(mode)).not.toMatch(/script-src/);
    }
  });
});
