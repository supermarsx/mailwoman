import { describe, it, expect } from 'vitest';
import { sanitizeNoteHtml, isSafeHref, htmlToText } from './sanitize.ts';

describe('sanitizeNoteHtml', () => {
  it('keeps allowlisted formatting elements', () => {
    const out = sanitizeNoteHtml('<p>Hello <strong>bold</strong> and <em>em</em></p><ul><li>x</li></ul>');
    expect(out).toContain('<strong>bold</strong>');
    expect(out).toContain('<em>em</em>');
    expect(out).toContain('<li>x</li>');
  });

  it('strips <script> entirely — no script survives', () => {
    const out = sanitizeNoteHtml('<p>ok</p><script>alert(1)</script>');
    expect(out).toContain('<p>ok</p>');
    expect(out.toLowerCase()).not.toContain('<script');
    expect(out).not.toContain('alert(1)');
  });

  it('drops <style> content and <iframe>', () => {
    const out = sanitizeNoteHtml('<style>body{}</style><iframe src="x"></iframe><p>keep</p>');
    expect(out.toLowerCase()).not.toContain('<style');
    expect(out.toLowerCase()).not.toContain('<iframe');
    expect(out).toContain('<p>keep</p>');
  });

  it('removes event-handler and style attributes', () => {
    const out = sanitizeNoteHtml('<p onclick="steal()" style="color:red">hi</p>');
    expect(out).toContain('hi');
    expect(out).not.toContain('onclick');
    expect(out).not.toContain('steal');
    expect(out).not.toContain('style');
  });

  it('keeps safe anchors but rejects javascript: URLs', () => {
    const safe = sanitizeNoteHtml('<a href="https://example.org">link</a>');
    expect(safe).toContain('href="https://example.org"');
    expect(safe).toContain('rel="noopener noreferrer nofollow"');

    const evil = sanitizeNoteHtml('<a href="javascript:alert(1)">x</a>');
    expect(evil).not.toContain('javascript:');
    expect(evil).toContain('x'); // text kept, href dropped
  });

  it('preserves a mailwoman: cross-link href', () => {
    const out = sanitizeNoteHtml('<a href="mailwoman:event/e1">ev</a>');
    expect(out).toContain('href="mailwoman:event/e1"');
  });

  it('unwraps unknown elements but keeps their text', () => {
    const out = sanitizeNoteHtml('<marquee>scroll</marquee>');
    expect(out).toBe('scroll');
  });

  it('is idempotent', () => {
    const once = sanitizeNoteHtml('<p>hi<script>x</script></p>');
    expect(sanitizeNoteHtml(once)).toBe(once);
  });
});

describe('isSafeHref', () => {
  it('accepts http/https/mailto/mailwoman/anchor/relative', () => {
    for (const href of ['https://a', 'http://a', 'mailto:a@b', 'mailwoman:email/1', '#x', '/rel']) {
      expect(isSafeHref(href)).toBe(true);
    }
  });

  it('rejects javascript: and data: — including whitespace obfuscation', () => {
    expect(isSafeHref('javascript:alert(1)')).toBe(false);
    expect(isSafeHref('  javascript:alert(1)')).toBe(false);
    expect(isSafeHref('java\tscript:alert(1)')).toBe(false);
    expect(isSafeHref('data:text/html,<script>')).toBe(false);
  });
});

describe('htmlToText', () => {
  it('projects HTML to collapsed plain text', () => {
    expect(htmlToText('<p>one</p><p>two  three</p>')).toBe('onetwo three');
  });
});
