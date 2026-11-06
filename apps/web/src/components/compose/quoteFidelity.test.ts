import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import { initSync } from '../../wasm/mw-sanitize/mw_sanitize.js';
import { localUrl, nativeHtmlBody, stripLocalUrls } from './htmlSource.ts';
import { quoteForForward, quoteForOutlook, quoteForReply } from './quote.ts';

beforeAll(() => {
  initSync({ module: readFileSync(resolve(process.cwd(), 'src/wasm/mw-sanitize/mw_sanitize_bg.wasm')) });
});

const OUTLOOK = '<html xmlns:v="urn:schemas-microsoft-com:vml"><head><style>@import url(file:///C:/x.css);p.MsoNormal{margin:0cm;font-size:11pt;font-family:Calibri,sans-serif;background:url("file:///C:/b.png")}</style></head>'
  + '<body background="file:///C:/bg.jpg" style="background-image:url(file:///C:/a.png)"><!--[if gte vml 1]><v:shape><v:imagedata src="file:///C:/Users/a/image001.png"/></v:shape><![endif]-->'
  + '<p class="MsoNormal" style="color:#1f497d">Hi <a href="file:///C:/Users/a/doc.docx">the doc</a> <a href="FILE:///x">upper</a> <a href="  file:///z">spaced</a> <a href="ms-help://x">h</a> <a href="mk:@MSITStore:x">m</a> <a href="https://ok.example/x">ok</a></p>'
  + '<p><img src="file:///C:/Users/a/logo.png" alt="Company logo" srcset="file:///a.png 1x" width="120" height="40"><img src="file:///C:/Users/a/nope.png"></p>'
  + '<table width="600" style="border-collapse:collapse"><tr><td background="file:///t.png" style="list-style-image:url(file:///l.png);padding:8px">Cell</td></tr></table><video poster="file:///p.png"></video></body></html>';

const NEWSLETTER = '<div style="font-family:Arial,sans-serif;font-size:14px;color:#222"><table width="600" cellpadding="0" style="margin:0 auto"><tr><td style="background-color:#f4f4f4;padding:12px"><table width="100%"><tr><td style="font-size:20px;font-weight:bold">Big news</td></tr></table></td></tr><tr><td><img src="cid:hero" width="560" alt="Hero"><p style="margin:0 0 1em">Body copy</p></td></tr></table></div>';

const GMAIL = '<div dir="ltr">Thanks!<br><div class="gmail_quote"><div dir="ltr" class="gmail_attr">On Mon, Bob wrote:<br></div><blockquote class="gmail_quote" style="margin:0px 0px 0px 0.8ex;border-left:1px solid rgb(204,204,204);padding-left:1ex">First<blockquote class="gmail_quote" style="margin:0px 0px 0px 0.8ex;border-left:1px solid rgb(204,204,204);padding-left:1ex">Second</blockquote></blockquote></div></div>';

describe('local and dangerous URL schemes never reach the quote', () => {
  const URL_ATTR = /(?:src|srcset|href|background|poster|action|formaction|xlink:href)\s*=\s*["']?\s*(?:file|ms-help|ms-its|mk|res|chrome|resource|view-source|javascript|vbscript):/i;
  const forms = [
    ['reply', () => quoteForReply({ html: OUTLOOK }, 'Bob wrote:')],
    ['forward', () => quoteForForward({ html: OUTLOOK }, ['Forwarded'])],
    ['outlook', () => quoteForOutlook({ html: OUTLOOK }, [{ label: 'From:', value: 'Bob' }])],
  ] as const;
  for (const [name, build] of forms) {
    it(`${name}: no file: reference in markup or CSS; link text and alt text stay`, async () => {
      const { html, text } = await build();
      expect(html).not.toMatch(/file:|ms-help:|mk:@|@import|v:imagedata/i);
      expect(html).not.toMatch(URL_ATTR);
      expect(html).toContain('the doc');
      expect(html).toContain('Company logo');
      expect(text).toContain('the doc');
      expect(html).toMatch(/<a [^>]*href="https:\/\/ok\.example\/x"/);
      expect(html).not.toMatch(/<a [^>]*href="(?:file|ms-help|mk)/i);
      expect(html).not.toContain('<img');
      expect(html).not.toContain('<video');
    });
  }

  it('stripLocalUrls is a second line of defence for markup the policy did not see', () => {
    const body = nativeHtmlBody('<a href="file:///a">t</a><img src="javascript:x"><img src="data:text/html,x"><img src="data:image/png;base64,AAAA"><a href="about:blank">b</a><a href="about:config">c</a><div style="background:url(file:///q);color:red">d</div><style>@import url(x.css);p{background:url(file:///z)}</style>');
    stripLocalUrls(body);
    expect(body.querySelector('a')?.hasAttribute('href')).toBe(false);
    expect(body.querySelectorAll('img[src]')).toHaveLength(1);
    expect(body.querySelectorAll('a[href]')).toHaveLength(1);
    expect(body.innerHTML).not.toMatch(/file:|@import/);
    expect(body.querySelector('div')?.getAttribute('style')).toContain('color');
  });

  it('classifies schemes', () => {
    for (const bad of ['file:///x', ' FILE:///x', 'fi\nle:///x', 'ms-its:x', 'res://x', 'chrome://x', 'resource://x', 'view-source:x', 'javascript:1', 'about:config', 'data:text/html,x', 'data:image/svg+xml;base64,AA']) expect(localUrl(bad), bad).toBe(true);
    for (const good of ['https://a.example', 'mailto:a@b.c', 'cid:x', 'about:blank', 'data:image/png;base64,AAAA', '/file:///x', '#a']) expect(localUrl(good), good).toBe(false);
  });
});

describe('the quote keeps the original layout', () => {
  it('newsletter tables, inline styles, and fonts survive; unowned images are dropped; owned ones stay', async () => {
    const plain = await quoteForReply({ html: NEWSLETTER }, 'Bob wrote:');
    const a = new DOMParser().parseFromString(plain.html, 'text/html');
    expect(a.querySelectorAll('table')).toHaveLength(2);
    expect(a.querySelectorAll('img')).toHaveLength(0);
    expect(plain.html).toContain('font-family');
    expect(plain.html).toMatch(/background-color:\s*(?:#f4f4f4|rgb\(244, 244, 244\))/);
    expect(plain.html).toMatch(/margin:\s*0(?:px)?\s+0(?:px)?\s+1em/);
    expect(plain.text).toContain('Big news');

    const withImage = await quoteForReply({ html: NEWSLETTER, ownedCids: new Set(['hero']) }, 'Bob wrote:');
    const b = new DOMParser().parseFromString(withImage.html, 'text/html');
    expect(b.querySelectorAll('img')).toHaveLength(1);
    expect(b.querySelector('img')?.getAttribute('src')).toBe('cid:hero');
    expect(b.querySelector('img')?.getAttribute('style')).toMatch(/max-width:\s*100%/);
    for (const table of Array.from(b.querySelectorAll('table'))) expect(table.getAttribute('style')).toMatch(/max-width:\s*100%/);
  });

  it('an Outlook message keeps MsoNormal rules scoped to the quote, with inline colour', async () => {
    const { html } = await quoteForReply({ html: OUTLOOK }, 'Bob wrote:');
    expect(html).toContain('.mw-quote-body p.MsoNormal');
    expect(html).not.toContain('.mw-email-body');
    expect(html).toMatch(/color:\s*(?:#1f497d|rgb\(31, 73, 125\))/i);
    expect(new DOMParser().parseFromString(html, 'text/html').querySelectorAll('table')).toHaveLength(1);
  });

  it('Gmail nested blockquotes keep their depth and borders', async () => {
    const { html } = await quoteForReply({ html: GMAIL }, 'Bob wrote:');
    const doc = new DOMParser().parseFromString(html, 'text/html');
    expect(doc.querySelectorAll('blockquote')).toHaveLength(3);
    expect(doc.querySelectorAll('blockquote blockquote blockquote')).toHaveLength(1);
    expect(html).toMatch(/border-left:\s*1px solid/);
    expect(html).toContain('Second');
  });

  it('the original is wrapped in its own scoped container after the attribution', async () => {
    const { html } = await quoteForReply({ html: '<style>p{color:red}</style><p>x</p>' }, 'Bob wrote:');
    expect(html.startsWith('<p><br></p><p><br></p><p>Bob wrote:</p><blockquote')).toBe(true);
    expect(html).toContain('.mw-quote-body p{color:red}');
  });
});
