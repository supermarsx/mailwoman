import { describe, it, expect } from 'vitest';
import { quoteForForward, quoteForReply } from './quote.ts';
import { docFromHtml, htmlFromDoc, inertBody } from './richtext.ts';

/** A message body holding everything a quote must not carry out. The quote is
 *  built from sanitised HTML in production; this is deliberately NOT sanitised,
 *  so the test shows what the quote step removes by itself. */
const HOSTILE = [
  '<html><head><title>T-TITLE</title><link rel="stylesheet" href="https://evil.example/a.css">',
  '<style>p { background: url(https://evil.example/bg.png) } S-STYLE</style></head><body>',
  '<p style="background-image:url(https://evil.example/p.png)" onclick="steal()" class="c" id="i">',
  'Hello <b onmouseover="steal()">bold</b> <a href="javascript:alert(1)">js-link</a> ',
  '<a href="https://ok.example/page" onclick="steal()" target="_blank">ok-link</a> ',
  '<a href="data:text/html,<script>alert(1)</script>">data-link</a></p>',
  '<img src="http://tracker.example/pixel.gif" width="1" height="1" onerror="steal()">',
  '<img src="cid:inline-part"><picture><source srcset="https://evil.example/s.png"></picture>',
  '<script>S-SCRIPT()</script><script src="https://evil.example/x.js"></script>',
  '<iframe src="https://evil.example/frame">F-FALLBACK</iframe>',
  '<object data="https://evil.example/o.swf">O-FALLBACK</object><embed src="https://evil.example/e">',
  '<svg onload="steal()"><text>S-SVG</text></svg>',
  '<form action="https://evil.example/post"><input name="q" value="I-VALUE"><button>B-LABEL</button></form>',
  '<video src="https://evil.example/v.mp4" poster="https://evil.example/poster.png">V-FALLBACK</video>',
  '<table background="https://evil.example/t.png"><tr><td>cell text</td></tr></table>',
  '<blockquote>earlier quote</blockquote>',
  '</body></html>',
].join('');

describe('the quote of a hostile original', () => {
  it('is exactly the inert markup below: text, bold, one http link, a blockquote', async () => {
    const quote = await quoteForReply({ html: HOSTILE }, 'On 1 Jan 2026, Mallory wrote:');
    expect(quote.html).toBe(
      '<p></p><p>On 1 Jan 2026, Mallory wrote:</p><blockquote>' +
        '<p>Hello <strong>bold</strong> js-link <a href="https://ok.example/page">ok-link</a> data-link</p>' +
        '<p>cell text</p>' +
        '<blockquote><p>earlier quote</p></blockquote>' +
        '</blockquote>',
    );
  });

  it('holds no element, attribute or URL that loads or runs anything', async () => {
    const { html } = await quoteForReply({ html: HOSTILE }, 'A wrote:');
    const body = inertBody(html);
    const tags = new Set(Array.from(body.querySelectorAll('*')).map((el) => el.tagName.toLowerCase()));
    expect([...tags].sort()).toEqual(['a', 'blockquote', 'p', 'strong']);
    const attrs = Array.from(body.querySelectorAll('*')).flatMap((el) =>
      Array.from(el.attributes).map((a) => `${el.tagName.toLowerCase()}[${a.name}=${a.value}]`),
    );
    expect(attrs).toEqual(['a[href=https://ok.example/page]']);
    expect(html).not.toMatch(/evil\.example|tracker\.example|javascript:|data:|cid:|url\(|<img|<script|<style|<link|<iframe|\son\w+=/i);
    // Content of the removed elements is gone with them, not left behind as text.
    expect(html).not.toMatch(/S-SCRIPT|S-STYLE|T-TITLE|F-FALLBACK|O-FALLBACK|S-SVG|I-VALUE|B-LABEL|V-FALLBACK/);
  });

  it('the plain-text form holds the same text behind quote markers', async () => {
    const { text } = await quoteForReply({ html: HOSTILE }, 'A wrote:');
    expect(text).toBe(
      '\n\nA wrote:\n> Hello bold js-link ok-link data-link\n>\n> cell text\n>\n> earlier quote',
    );
  });

  it('survives the composer: parsing the quote through the schema again changes nothing', async () => {
    const { html } = await quoteForReply({ html: HOSTILE }, 'A wrote:');
    expect(htmlFromDoc(docFromHtml(html))).toBe(html);
  });

  it('a forward carries the same reduced body under an escaped header block', async () => {
    const quote = await quoteForForward({ html: HOSTILE }, [
      '---------- Forwarded message ----------',
      'From: Mallory <img src=x onerror=steal()> <m@evil.example>',
      'Subject: <script>alert(1)</script>',
    ]);
    expect(quote.html).toBe(
      '<p></p><p>---------- Forwarded message ----------<br>' +
        'From: Mallory &lt;img src=x onerror=steal()&gt; &lt;m@evil.example&gt;<br>' +
        'Subject: &lt;script&gt;alert(1)&lt;/script&gt;</p>' +
        '<p>Hello <strong>bold</strong> js-link <a href="https://ok.example/page">ok-link</a> data-link</p>' +
        '<p>cell text</p>' +
        '<blockquote><p>earlier quote</p></blockquote>',
    );
    expect(inertBody(quote.html).querySelectorAll('img, script').length).toBe(0);
  });
});

describe('quote sources', () => {
  it('escapes the attribution line', async () => {
    const { html } = await quoteForReply({ html: '<p>x</p>' }, 'On <b>today</b>, "A" & B wrote:');
    expect(html).toBe('<p></p><p>On &lt;b&gt;today&lt;/b&gt;, "A" &amp; B wrote:</p><blockquote><p>x</p></blockquote>');
  });

  it('quotes plain text as text: markup in it is not parsed', async () => {
    const quote = await quoteForReply({ text: 'line one\n<img src=http://t.example/p.gif>\n\nline four' }, 'A wrote:');
    expect(quote.html).toBe(
      '<p></p><p>A wrote:</p><blockquote><p>line one<br>&lt;img src=http://t.example/p.gif&gt;<br><br>line four</p></blockquote>',
    );
    expect(quote.text).toBe('\n\nA wrote:\n> line one\n> <img src=http://t.example/p.gif>\n>\n> line four');
  });

  it('quotes nothing when nothing was displayed', async () => {
    expect(await quoteForReply({}, 'A wrote:')).toEqual({ html: '', text: '' });
    expect(await quoteForReply({ html: '<img src="http://t.example/p.gif"><p> </p>' }, 'A wrote:')).toEqual({
      html: '',
      text: '',
    });
  });
});

describe('inert parsing', () => {
  it('parses into a document that has no browsing context', () => {
    const body = inertBody('<img src="http://tracker.example/pixel.gif"><p>x</p>');
    expect(body.ownerDocument).not.toBe(document);
    expect(body.ownerDocument.defaultView).toBeNull();
    expect(body.querySelectorAll('img').length).toBe(1);
  });
});
