import {describe,expect,it} from 'vitest';
import {diffReaderDocument,patchOpAllowed,patchPrefixes,patchUrlAllowed} from './readerPatch.ts';
const base='https://mail.example.test';const prefixes=patchPrefixes(base);
const gif='data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';
const doc=(body:string,head='')=>`<!doctype html><html><head>${head}</head><body>${body}</body></html>`;
describe('reader hot patch diff',()=>{
 it('turns placeholder to signed image sources without touching the rest',()=>{
  const before=doc(`<p>Hello</p><img data-mw-rc="1" src="${gif}" width="10"><div style="background:url(&quot;${gif}&quot;)"></div>`);
  const after=doc(`<p>Hello</p><img data-mw-rc="1" src="${base}/api/rc/i/abc.def" width="10"><div style="background:url(&quot;${base}/api/rc/i/xyz&quot;)"></div>`);
  expect(diffReaderDocument(before,after,prefixes)).toEqual([
   {s:'b',i:1,a:'src',v:`${base}/api/rc/i/abc.def`},
   {s:'b',i:2,a:'style',v:`background:url("${base}/api/rc/i/xyz")`},
  ]);
  expect(diffReaderDocument(before,before,prefixes)).toEqual([]);
 });
 it('patches <style> text in head and body',()=>{
  const before=doc('<style>.a{background:url("'+gif+'")}</style><p>x</p>','<style>.b{background:none}</style>');
  const after=doc(`<style>.a{background:url("${base}/api/rc/i/t1")}</style><p>x</p>`,'<style>.b{background:url("'+base+'/api/rc/i/t2")}</style>');
  expect(diffReaderDocument(before,after,prefixes)).toEqual([
   {s:'h',i:0,a:'text',v:`.b{background:url("${base}/api/rc/i/t2")}`},
   {s:'b',i:0,a:'text',v:`.a{background:url("${base}/api/rc/i/t1")}`},
  ]);
 });
 it('falls back (null) when anything but image references differs',()=>{
  const before=doc(`<p>One</p><img src="${gif}">`);
  expect(diffReaderDocument(before,doc(`<p>Two</p><img src="${gif}">`),prefixes)).toBeNull();
  expect(diffReaderDocument(before,doc(`<p>One</p><img src="${gif}"><p>more</p>`),prefixes)).toBeNull();
  expect(diffReaderDocument(doc('<a>x</a>'),doc('<a href="https://x.test/">x</a>'),prefixes)).toBeNull();
 });
 it('refuses to introduce anything but signed handles or inline images',()=>{
  const before=doc(`<img src="${gif}">`);
  for(const bad of ['https://evil.test/p.png',`${base}/api/rc/f/token`,`${base}/api/rc/i/a/b`,`${base}/api/rc/i/a?x=1`,'javascript:alert(1)','data:text/html;base64,AAAA','//evil.test/x.png'])
   expect(diffReaderDocument(before,doc(`<img src="${bad}">`),prefixes),bad).toBeNull();
  expect(diffReaderDocument(doc('<div></div>'),doc(`<div style="background:url(https://evil.test/x)"></div>`),prefixes)).toBeNull();
  expect(diffReaderDocument(doc('<div></div>'),doc(`<div style="background:image-set(url(${base}/api/rc/i/a) 1x)"></div>`),prefixes)).toBeNull();
  expect(diffReaderDocument(doc('<div></div>'),doc(`<div style="background:u${'\\'}72l(https://evil.test/x)"></div>`),prefixes)).toBeNull();
  expect(diffReaderDocument(doc(`<img srcset="${gif} 1x">`),doc(`<img srcset="${base}/api/rc/i/a 1x, https://evil.test/b 2x">`),prefixes)).toBeNull();
  expect(diffReaderDocument(doc(`<img srcset="${gif} 1x">`),doc(`<img srcset="${base}/api/rc/i/a 1x, ${base}/api/rc/i/b 2x">`),prefixes)).not.toBeNull();
 });
 it('validates operations structurally',()=>{
  expect(patchUrlAllowed(`${base}/api/rc/p/abc`,prefixes)).toBe(true);
  for(const op of [null,{},{s:'h',i:0,a:'src',v:gif},{s:'b',i:-1,a:'src',v:gif},{s:'b',i:0,a:'href',v:gif},{s:'b',i:0,a:'src',v:5},{s:'b',i:0,a:'text',v:null}])expect(patchOpAllowed(op,prefixes)).toBe(false);
  expect(patchOpAllowed({s:'b',i:0,a:'src',v:null},prefixes)).toBe(true);
 });
});
