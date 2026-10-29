import { describe, it, expect } from 'vitest';
import {
  docFromElement,
  docFromHtml,
  hydrateInlineStyles,
  styleDeclarations,
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
  it('round-trips approved fonts and sizes while discarding unsafe style properties', () => {
    const html=htmlFromDoc(docFromHtml('<p><span style="font-family:Georgia;font-size:18pt;position:fixed;background:url(https://tracker.test/x)">Styled</span></p>'));
    expect(html).toContain('font-family: Georgia');expect(html).toContain('font-size: 18pt');
    expect(html).not.toContain('tracker.test');expect(html).not.toContain('position');
    expect(htmlFromDoc(docFromHtml(html))).toBe(html);
    expect(htmlFromDoc(docFromHtml('<p><span style="font-size:9999pt;font-family:Unapproved">Text</span></p>'))).toBe('<p>Text</p>');
  });
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

describe('embedded signature images (t49)', () => {
  const PNG = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';
  it('keeps a raster data: image only when asked, and serialises it unchanged', () => {
    const html = `<p>Hi <img src="${PNG}" alt="logo"></p>`;
    expect(htmlFromDoc(docFromHtml(html))).toBe('<p>Hi</p>');
    expect(htmlFromDoc(docFromHtml(html, undefined, true))).toBe(`<p>Hi <img src="${PNG}" alt="logo"></p>`);
  });
  it('drops SVG, non-base64 and remote images even when embedded images are kept', () => {
    for (const src of ['data:image/svg+xml;base64,PHN2Zy8+', 'data:image/png,raw', 'https://x.example/a.png', 'data:text/html;base64,PGI+']) {
      expect(htmlFromDoc(docFromHtml(`<p>a<img src="${src}"></p>`, undefined, true))).toBe('<p>a</p>');
    }
  });
});

describe('inline styles under a strict CSP (t49)', () => {
  it('splits declarations outside quotes and brackets', () => {
    expect(styleDeclarations('color:rgb(1, 2, 3);font-family:Georgia, "A;B", serif ; margin: 0 !important;bad')).toEqual([
      ['color', 'rgb(1, 2, 3)', ''],
      ['font-family', 'Georgia, "A;B", serif', ''],
      ['margin', '0', 'important'],
    ]);
  });
  it('applies a style attribute the browser left unapplied, so formatting survives the parse', () => {
    const root = inertBody('<p><span>x</span></p>');
    const span = root.querySelector('span')!;
    // What Chromium does under style-src 'self': the attribute is kept, the style is not applied.
    span.setAttribute('data-style', 'color:rgb(31, 78, 121);font-family:Georgia, "Times New Roman", serif');
    Object.defineProperty(span, 'getAttribute', { value: (n: string) => (n === 'style' ? span.dataset['style'] : null) });
    span.setAttribute('style', '');
    expect(span.style.length).toBe(0);
    hydrateInlineStyles(root);
    expect(span.style.color).toBe('rgb(31, 78, 121)');
    expect(htmlFromDoc(docFromElement(root))).toContain('font-family');
  });
});

describe('style attributes never reach the HTML parser (t67)', () => {
  it('renames style attribute names inside start tags only', async () => {
    const { renameStyleAttributes } = await import('./richtext.ts');
    expect(renameStyleAttributes('<p style="color:red" class=a>style="x" <b STYLE=\'a:b\'>t</b></p>'))
      .toBe('<p data-mw-parsed-style="color:red" class=a>style="x" <b data-mw-parsed-style=\'a:b\'>t</b></p>');
    // Attribute values, comments, raw text and end tags stay as they are.
    expect(renameStyleAttributes('<a title="style=1" href=x>l</a><!-- <p style="c"> --><style>p[style]{}</style><textarea><i style="z"></textarea></p >'))
      .toBe('<a title="style=1" href=x>l</a><!-- <p style="c"> --><style>p[style]{}</style><textarea><i style="z"></textarea></p >');
    expect(renameStyleAttributes('a < b style=c, <span\nstyle = "x">')).toBe('a < b style=c, <span\ndata-mw-parsed-style = "x">');
    expect(renameStyleAttributes('<img src=x.png style=width:3px/>')).toBe('<img src=x.png data-mw-parsed-style=width:3px/>');
  });
  it('parses stored markup with its formatting and no style attribute in the parse', async () => {
    const { inertBody, docFromHtml, htmlFromDoc } = await import('./richtext.ts');
    const body = inertBody('<p style="font-family: Georgia; color: #1f4e79">Kind regards</p>');
    const p = body.querySelector('p')!;
    expect(p.hasAttribute('data-mw-parsed-style')).toBe(false);
    expect(p.style.fontFamily).toBe('Georgia');
    const html = htmlFromDoc(docFromHtml('<p><span style="font-family: Georgia">Kind</span> <span style="color: #1f4e79">regards</span></p>'));
    expect(html).toContain('Georgia');
    expect(html).toMatch(/color/);
    expect(html).not.toContain('data-mw-parsed-style');
  });
});
