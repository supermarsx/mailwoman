import {expect,it} from 'vitest';
import {inspectMessageLink,reviewMessageLinks} from './messageLinks.ts';
it('neutralizes navigation while preserving CSP, body text and local fragment navigation',()=>{
 const source='<!doctype html><html><head><meta http-equiv="Content-Security-Policy" content="default-src \'none\'"></head><body><a href="https://evil.example/path" target="_self" ping="https://tracker.example" download>https://bank.example</a><a href="#foot">Footnote</a></body></html>';
 const result=reviewMessageLinks(source,'Review above');const doc=new DOMParser().parseFromString(result.html,'text/html');const anchors=doc.querySelectorAll('a');
 expect(anchors[0]?.hasAttribute('href')).toBe(false);expect(anchors[0]?.hasAttribute('target')).toBe(false);expect(anchors[0]?.hasAttribute('ping')).toBe(false);expect(anchors[0]?.hasAttribute('download')).toBe(false);expect(anchors[0]?.getAttribute('title')).toBe('Review above');expect(anchors[1]?.getAttribute('href')).toBe('#foot');expect(doc.querySelector('meta')?.getAttribute('content')).toBe("default-src 'none'");expect(result.links[0]?.warnings).toEqual(['mismatch']);expect(source).toContain('href="https://evil.example/path"');
});
it('rejects executable, relative, credentialed and control-character addresses',()=>{
 for(const url of ['javascript:alert(1)','data:text/html,x','/login','//evil.example','https://me:password@evil.example','https://evil.example/\nother','https://evil.example/\u202ehidden','mailto:user@example.org?bcc=hidden@example.org'])expect(inspectMessageLink(url,'label'),url).toBeNull();
 expect(inspectMessageLink('http://127.0.0.1/path','Go')?.warnings).toEqual(['http','address']);expect(inspectMessageLink('https://bücher.example','Go')?.warnings).toContain('international');expect(inspectMessageLink('mailto:user@example.org?subject=Hello','Write')?.kind).toBe('mail');
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
