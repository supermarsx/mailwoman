import { describe, it, expect } from 'vitest';
import { SANDBOX_TOKENS, bodyCsp, escapeHtml, mediaFrameDoc, textFrameDoc } from './sandbox.ts';

describe('SANDBOX_TOKENS', () => {
  it('grants neither allow-scripts nor allow-same-origin (opaque, script-free)', () => {
    expect(SANDBOX_TOKENS).toBe('');
    expect(SANDBOX_TOKENS).not.toContain('allow-scripts');
    expect(SANDBOX_TOKENS).not.toContain('allow-same-origin');
  });
});

describe('mediaFrameDoc', () => {
  it('pins an image frame to default-src none + img-src data:', () => {
    const doc = mediaFrameDoc('image', 'data:image/png;base64,AAAA');
    expect(doc).toContain('http-equiv="Content-Security-Policy"');
    expect(doc).toContain("default-src 'none'");
    expect(doc).toContain('img-src data:');
    // Ignored in a <meta> policy (and warned about by Firefox), so never sent there.
    expect(doc).not.toContain('frame-ancestors');
    expect(doc).toContain('<img');
    expect(doc).not.toContain('<script');
  });

  it('uses media-src for audio and video with native controls', () => {
    const audio = mediaFrameDoc('audio', 'data:audio/mpeg;base64,AAAA');
    expect(audio).toContain('media-src data:');
    expect(audio).toContain('<audio controls');

    const video = mediaFrameDoc('video', 'data:video/mp4;base64,AAAA');
    expect(video).toContain('media-src data:');
    expect(video).toContain('<video controls');
  });
});

describe('textFrameDoc', () => {
  it('inlines escaped text with a no-external-load CSP', () => {
    const doc = textFrameDoc('<b>hi & bye</b>');
    expect(doc).toContain("default-src 'none'");
    expect(doc).toContain('&lt;b&gt;hi &amp; bye&lt;/b&gt;');
    expect(doc).not.toContain('<b>hi');
    // no resource dir at all — text needs zero external loads
    expect(doc).not.toContain('img-src');
    expect(doc).not.toContain('media-src');
  });
});

describe('escapeHtml', () => {
  it('escapes the HTML-significant characters', () => {
    expect(escapeHtml('a<b>&"')).toBe('a&lt;b&gt;&amp;"');
  });
});

it('admits only an explicit HTTP server origin for opaque signed-image frames', () => {
  expect(bodyCsp('full-sanitized', false, 'https://mail.test/prefix')).toContain("img-src 'self' data: blob: https://mail.test;");
  expect(bodyCsp('full-sanitized', false, 'https://user:pass@mail.test')).not.toContain('mail.test');
  expect(bodyCsp('full-sanitized', false, "data:text/html,evil")).not.toContain('evil');
  expect(bodyCsp('sanitized-no-media', false, 'https://mail.test')).not.toContain('mail.test');
});

it('no meta-delivered frame policy carries frame-ancestors', () => {
  for (const mode of ['full-sanitized', 'sanitized-no-media', 'plain-text'] as const) expect(bodyCsp(mode)).not.toContain('frame-ancestors');
  expect(textFrameDoc('x')).not.toContain('frame-ancestors');
  expect(mediaFrameDoc('video', 'data:video/mp4;base64,AA')).not.toContain('frame-ancestors');
});

describe('broken remote image placeholder (t56)', () => {
  it('ships in every image-capable body frame and only there', async () => {
    const { BROKEN_IMAGE_STYLE, bodyFrameDoc } = await import('./sandbox.ts');
    const doc = bodyFrameDoc('full-sanitized', { html: '<img src="/api/rc/i/t" width="300" height="120" alt="Banner">' });
    expect(doc).toContain(BROKEN_IMAGE_STYLE);
    // Sender dimensions survive, so the placeholder occupies the image's own box.
    expect(doc).toMatch(/<img[^>]*width="300"[^>]*height="120"/);
    expect(doc).toContain('alt="Banner"');
    // plain-text renders no <img>; the style is inert there but harmless.
    expect(bodyFrameDoc('plain-text', { text: 'x' })).not.toContain('<img');
  });

  it('draws only for an image that shows nothing, keeps alt text, and adds no script or remote load', async () => {
    const { BROKEN_IMAGE_STYLE } = await import('./sandbox.ts');
    // Generated content: ::before/::after render for an <img> only while it is broken.
    expect(BROKEN_IMAGE_STYLE).toContain('img::before{content:""');
    expect(BROKEN_IMAGE_STYLE).toContain('img::after{content:attr(alt)');
    // Decorative (alt="") images collapse; no-alt keeps the native icon.
    expect(BROKEN_IMAGE_STYLE).toContain('img[alt=""]::before,img[alt=""]::after,img:not([alt])::before,img:not([alt])::after{content:none}');
    // The label is clipped to one line inside the image box: no reflow from long alt text.
    expect(BROKEN_IMAGE_STYLE).toMatch(/white-space:nowrap;overflow:hidden;text-overflow:ellipsis/);
    // The glyph is an inline data: SVG (allowed by img-src data:), never a fetch.
    const urls = [...BROKEN_IMAGE_STYLE.matchAll(/url\("([^"]*)"\)/g)].map(m => m[1] ?? '');
    expect(urls.length).toBe(1);
    expect(urls[0]?.startsWith('data:image/svg+xml,')).toBe(true);
    // (the SVG's own xmlns is a namespace name, not a load, so the data: URL is set aside)
    expect(BROKEN_IMAGE_STYLE.replace(urls[0] ?? '', '')).not.toMatch(/https?:|@import|expression\(|javascript:/i);
    // Themed through the frame's --mw-* vars with readable fallbacks.
    expect(BROKEN_IMAGE_STYLE).toContain('var(--mw-color-border,');
    expect(BROKEN_IMAGE_STYLE).toContain('var(--mw-color-text-dim,');
  });

  it('the placeholder CSS parses as a stylesheet with the rules intact', async () => {
    const { BROKEN_IMAGE_STYLE } = await import('./sandbox.ts');
    const style = document.createElement('style');
    style.textContent = BROKEN_IMAGE_STYLE;
    document.head.append(style);
    const selectors = [...(style.sheet?.cssRules ?? [])].map(r => (r as CSSStyleRule).selectorText ?? r.cssText.slice(0, 9));
    style.remove();
    expect(selectors).toEqual(expect.arrayContaining(['img', 'img::before', 'img::after']));
  });
});
