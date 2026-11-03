import {describe,expect,it} from 'vitest';
import {htmlMessageDocument} from './htmlMessage.ts';

describe('outgoing HTML document',()=>{
  it('includes document metadata and content typography without editor theme or chrome',()=>{
    const html=htmlMessageDocument('<p>Hello <strong>world</strong></p>');
    const doc=new DOMParser().parseFromString(html,'text/html');
    expect(doc.doctype?.name).toBe('html');
    expect(doc.head.querySelector('meta[charset]')?.getAttribute('charset')).toBe('utf-8');
    expect(doc.head.querySelector('meta[name="viewport"]')?.getAttribute('content')).toContain('width=device-width');
    expect(doc.body.style.fontSize).toBe('14px');
    expect(doc.body.style.lineHeight).toBe('1.5');
    expect(doc.body.style.fontFamily).toBe('system-ui, sans-serif');
    expect(doc.body.innerHTML).toBe('<p>Hello <strong>world</strong></p>');
    expect(doc.body.style.background).toBe('');
    expect(doc.body.style.color).toBe('');
    expect(doc.querySelector('[contenteditable],script,meta[http-equiv]')).toBeNull();
  });

  it('keeps authored styles, layout, Unicode and inline attachment references in the document',()=>{
    const fragment='<div class="mw-email-body" dir="rtl"><style>.mw-email-body td{padding:12px}</style><table><tbody><tr><td style="font-family:Georgia;color:#123456">Olá <a href="https://example.test/?a=1&amp;b=2">link</a><img src="cid:logo"></td></tr></tbody></table></div>';
    const html=htmlMessageDocument(fragment);
    expect(html).toContain(fragment);
    const doc=new DOMParser().parseFromString(html,'text/html');
    expect(doc.querySelector('td')?.style.fontFamily).toBe('Georgia');
    expect(doc.querySelector('img')?.getAttribute('src')).toBe('cid:logo');
    expect(doc.querySelector('a')?.getAttribute('href')).toBe('https://example.test/?a=1&b=2');
    expect(doc.querySelector('[dir]')?.getAttribute('dir')).toBe('rtl');
  });
});
