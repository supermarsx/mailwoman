import { describe, it, expect } from 'vitest';
import {
  docFromHtml,
  docFromText,
  htmlFromDoc,
  htmlToText,
  quoteDocFromElement,
  quoteSchema,
  inertBody,
  richSchema,
  textFromDoc,
} from './richtext.ts';

describe('richtext serialization (W1)', () => {
  it('round-trips inline formatting through the HTML serializer', () => {
    const html = htmlFromDoc(docFromHtml('<p>Hello <strong>bold</strong> and <em>italic</em></p>'));
    expect(html).toContain('<strong>bold</strong>');
    expect(html).toContain('<em>italic</em>');
  });

  it('supports underline and strikethrough marks', () => {
    const html = htmlFromDoc(docFromHtml('<p><u>under</u> <s>struck</s></p>'));
    expect(html).toContain('<u>under</u>');
    expect(html).toContain('<s>struck</s>');
  });

  it('preserves lists and blockquotes', () => {
    const html = htmlFromDoc(docFromHtml('<ul><li><p>one</p></li><li><p>two</p></li></ul>'));
    expect(html).toContain('<ul>');
    expect(html).toContain('one');
    expect(html).toContain('two');
  });

  it('keeps http(s) links from the pasted HTML', () => {
    const html = htmlFromDoc(docFromHtml('<p><a href="https://example.org">site</a></p>'));
    expect(html).toContain('href="https://example.org"');
  });

  // The schema is the outbound filter for HTML the composer did not author
  // (an identity's stored signature goes through docFromHtml -> htmlFromDoc).
  it('drops every <img> when the parse is given no images of the draft', () => {
    expect(htmlFromDoc(docFromHtml('<p>a<img src="https://t.example/p.gif" width="1" height="1">b</p>'))).toBe(
      '<p>ab</p>',
    );
    expect(htmlFromDoc(docFromHtml('<p>a<img src="cid:c1@mailwoman">b</p>'))).toBe('<p>ab</p>');
  });

  // t29-e12: the image node keeps only `cid:` sources this draft uploaded.
  describe('inline images', () => {
    const owned = new Set(['c1@mailwoman']);

    it('keeps an image whose cid the draft owns, and serialises it as that cid only', () => {
      const html = htmlFromDoc(
        docFromHtml('<p>a<img src="cid:c1@mailwoman" alt="chart.png" width="900" style="x" onerror="y()">b</p>', owned),
      );
      expect(html).toBe('<p>a<img src="cid:c1@mailwoman" alt="chart.png">b</p>');
    });

    it('drops a remote image, a data: image and a cid the draft does not own', () => {
      for (const src of [
        'https://t.example/p.gif',
        '//t.example/p.gif',
        'data:image/png;base64,iVBORw0KGgo=',
        'cid:someone-else@example.org',
        'cid:',
        'blob:https://mail.example/1234',
        'cid:c1@mailwoman.example',
        'cid:c1@mailwoman?x',
      ]) {
        expect(htmlFromDoc(docFromHtml(`<p>a<img src="${src}">b</p>`, owned)), src).toBe('<p>ab</p>');
      }
    });

    it('a quoted original keeps no image, even one naming an owned cid', () => {
      expect(quoteSchema.nodes['image']).toBeUndefined();
      expect(richSchema.nodes['image']).toBeDefined();
      const doc = quoteDocFromElement(inertBody('<p>q<img src="cid:c1@mailwoman">r</p>'));
      expect(htmlFromDoc(doc)).toBe('<p>qr</p>');
    });

    it('an image is not text', () => {
      expect(textFromDoc(docFromHtml('<p>a<img src="cid:c1@mailwoman">b</p>', owned))).toBe('ab');
    });
  });

  it('drops script, style and event-handler attributes', () => {
    expect(
      htmlFromDoc(
        docFromHtml('<p onclick="x()" style="position:fixed">hi</p><script>alert(1)</script><style>p{}</style>'),
      ),
    ).toBe('<p>hi</p>');
  });

  it('keeps http, https and mailto links and unlinks every other scheme', () => {
    expect(htmlFromDoc(docFromHtml('<p><a href="https://example.org">site</a></p>'))).toBe(
      '<p><a href="https://example.org">site</a></p>',
    );
    expect(htmlFromDoc(docFromHtml('<p><a href="mailto:a@example.org">mail</a></p>'))).toBe(
      '<p><a href="mailto:a@example.org">mail</a></p>',
    );
    for (const href of ['javascript:alert(1)', ' JavaScript:alert(1)', 'data:text/html,x', '/relative', 'vbscript:x']) {
      expect(htmlFromDoc(docFromHtml(`<p><a href="${href}">x</a></p>`)), href).toBe('<p>x</p>');
    }
  });

  it('projects a document to plain text with blank lines between blocks', () => {
    expect(textFromDoc(docFromHtml('<p>a</p><p>b</p>'))).toBe('a\n\nb');
  });

  it('htmlToText flattens rich HTML to text', () => {
    expect(htmlToText('<p>Hello <strong>world</strong></p>')).toBe('Hello world');
  });

  it('round-trips plain text exactly through docFromText/textFromDoc', () => {
    for (const text of ['line1\nline2', 'a\n\nb', 'single', '', 'x\ny\n\nz']) {
      expect(textFromDoc(docFromText(text))).toBe(text);
    }
  });
});
