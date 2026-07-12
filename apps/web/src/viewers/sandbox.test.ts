import { describe, it, expect } from 'vitest';
import { SANDBOX_TOKENS, escapeHtml, mediaFrameDoc, textFrameDoc } from './sandbox.ts';

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
    expect(doc).toContain("frame-ancestors 'none'");
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
