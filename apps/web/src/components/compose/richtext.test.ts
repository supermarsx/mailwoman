import { describe, it, expect } from 'vitest';
import {
  docFromHtml,
  docFromText,
  htmlFromDoc,
  htmlToText,
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
