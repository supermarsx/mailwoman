import {expect,it} from 'vitest';
import {inspectMessageLink,reviewMessageLinks} from './messageLinks.ts';
it('neutralizes navigation while preserving CSP, body text and local fragment navigation',()=>{
 const source='<!doctype html><html><head><meta http-equiv="Content-Security-Policy" content="default-src \'none\'"></head><body><a href="https://evil.example/path" target="_self" ping="https://tracker.example" download>https://bank.example</a><a href="#foot">Footnote</a></body></html>';
 const result=reviewMessageLinks(source,'Review above');const doc=new DOMParser().parseFromString(result.html,'text/html');const anchors=doc.querySelectorAll('a');
 expect(anchors[0]?.hasAttribute('href')).toBe(false);expect(anchors[0]?.hasAttribute('target')).toBe(false);expect(anchors[0]?.hasAttribute('ping')).toBe(false);expect(anchors[0]?.hasAttribute('download')).toBe(false);expect(anchors[0]?.getAttribute('title')).toBe('Review above');expect(anchors[1]?.getAttribute('href')).toBe('#foot');expect(doc.querySelector('meta')?.getAttribute('content')).toBe("default-src 'none'");expect(result.links[0]?.warnings).toEqual(['mismatch']);expect(source).toContain('href="https://evil.example/path"');
});
it('rejects executable, relative, credentialed and control-character addresses',()=>{
 for(const url of ['javascript:alert(1)','data:text/html,x','/login','https://me:password@evil.example','https://evil.example/\nother','https://evil.example/\u202ehidden','mailto:user@example.org?bcc=hidden@example.org'])expect(inspectMessageLink(url,'label'),url).toBeNull();
 expect(inspectMessageLink('http://127.0.0.1/path','Go')?.warnings).toEqual(['http','address']);expect(inspectMessageLink('https://bücher.example','Go')?.warnings).toContain('international');expect(inspectMessageLink('mailto:user@example.org?subject=Hello','Write')?.kind).toBe('mail');
});
it('preserves spaces in URL parameters and paths, outer spacing and scheme-relative web links',()=>{
 const link=inspectMessageLink('  https://shop.example/order status?name=Jane Doe&note=a b&encoded=a%2Bb&plus=a+b  ','Order');
 expect(link?.href).toBe('https://shop.example/order%20status?name=Jane%20Doe&note=a%20b&encoded=a%2Bb&plus=a+b');
 const url=new URL(link!.href);expect(url.searchParams.get('name')).toBe('Jane Doe');expect(url.searchParams.get('encoded')).toBe('a+b');expect(url.searchParams.get('plus')).toBe('a b');
 expect(inspectMessageLink('//shop.example/order?name=Jane Doe','Order')?.href).toBe('https://shop.example/order?name=Jane%20Doe');
 for(const value of [' https://bad host.example/x ',' javascript:alert(1) ','//me:secret@shop.example/x','https://shop.example/?value=a\tb'])expect(inspectMessageLink(value,'Order')).toBeNull();
 const reviewed=reviewMessageLinks('<a href="https://shop.example/?name=Jane Doe&amp;query=a b"><b>Order status</b></a>','Review',true);
 expect(reviewed.links[0]?.label).toBe('Order status');
 expect(new URL(reviewed.links[0]!.href).searchParams.get('query')).toBe('a b');
 const anchor=new DOMParser().parseFromString(reviewed.html,'text/html').querySelector('a')!;
 expect(anchor.href).toBe('https://shop.example/?name=Jane%20Doe&query=a%20b');expect(anchor.getAttribute('rel')).toBe('noopener noreferrer');
});
it('bounds the review list and still disables every additional destination',()=>{
 const result=reviewMessageLinks(Array.from({length:110},(_,i)=>`<a href="https://example.org/${i}">Link</a>`).join(''),'Review');expect(result.links).toHaveLength(100);expect(result.truncated).toBe(true);expect(new DOMParser().parseFromString(result.html,'text/html').querySelectorAll('a[href]')).toHaveLength(0);
});

it('enables only validated destinations with isolated navigation after explicit consent',()=>{
 const source='<a href="https://example.org/path" target="_self" ping="https://tracker.example" download>Visit</a><a href="mailto:friend@example.org">Write</a><a href="javascript:alert(1)">Bad</a><a href="data:text/html,evil">Data</a><a href="custom:thing">Custom</a><a href="#foot">Foot</a>';
 const doc=new DOMParser().parseFromString(reviewMessageLinks(source,'Review',true).html,'text/html');
 const links=doc.querySelectorAll('a');
 for(const i of [0,1]){expect(links[i]?.getAttribute('target')).toBe('_blank');expect(links[i]?.getAttribute('rel')).toBe('noopener noreferrer');expect(links[i]?.getAttribute('referrerpolicy')).toBe('no-referrer');expect(links[i]?.hasAttribute('aria-disabled')).toBe(false);}
 for(const i of [2,3,4])expect(links[i]?.hasAttribute('href')).toBe(false);
 expect(links[0]?.hasAttribute('ping')).toBe(false);expect(links[0]?.hasAttribute('download')).toBe(false);expect(links[5]?.getAttribute('href')).toBe('#foot');expect(links[5]?.hasAttribute('target')).toBe(false);
 expect(new DOMParser().parseFromString(reviewMessageLinks(source,'Review',false).html,'text/html').querySelectorAll('a[href]')).toHaveLength(1);
});

it('keeps only canonical inert mailto contact metadata while body navigation remains disabled',()=>{
 const doc=new DOMParser().parseFromString(reviewMessageLinks('<a href="mailto:Case@EXAMPLE.ORG?bcc=hidden@example.org">Write</a><a href="https://example.org" data-mw-contact-address="spoof@example.org">Web</a><a data-mw-contact-address="spoof@example.org">Plain</a>','Review').html,'text/html');
 const anchors=doc.querySelectorAll('a');expect(anchors[0]?.getAttribute('data-mw-contact-address')).toBe('Case@example.org');expect(doc.querySelector('a[href]')).toBeNull();
 expect(anchors[1]?.hasAttribute('data-mw-contact-address')).toBe(false);expect(anchors[2]?.hasAttribute('data-mw-contact-address')).toBe(false);
});
