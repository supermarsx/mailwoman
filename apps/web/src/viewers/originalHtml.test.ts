import {expect,it,vi} from 'vitest';
import {readFileSync} from 'node:fs';
import {TRACKER_HOSTS,BEACON_RULES,TRACKING_PARAMETERS,TRACKER_LABELS,TRACKER_SUFFIXES} from './originalHtmlPolicy.ts';
import {prepareOriginalHtml,renderOriginalHtml,reviewOriginalLinks} from './originalHtml.ts';
import {EMPTY_SIGNED,type RemotePolicy} from '../api/remote-content.ts';
const policy:RemotePolicy={mode:'proxy',trackers:true,cleanLinks:true,allowDirect:false,locked:{mode:false,trackers:false,cleanLinks:false}};
const base='https://mail.test/mail';
const rendered=(source:string)=>{const prepared=prepareOriginalHtml(source,policy);return renderOriginalHtml(prepared,{...EMPTY_SIGNED,images:prepared.refs.urls.map(url=>({url,src:'/mail/api/rc/i/image.signature'})),parts:prepared.refs.cids.map(blobId=>({blobId,src:'/mail/api/rc/p/part.signature'}))},policy,Object.fromEntries(prepared.refs.cids.map(value=>[value,value])),base);};
it('keeps original-view tracking protections identical to the reader sanitizer datasets',()=>{for(const [file,values] of [['trackers',TRACKER_HOSTS],['beacons',BEACON_RULES],['tracking_params',TRACKING_PARAMETERS]] as const){const expected=readFileSync(`../../crates/mw-sanitize/data/${file}.txt`,'utf8').split(/\r?\n/).map(value=>value.trim()).filter(value=>value&&!value.startsWith('#'));expect(values).toEqual(expected);}});
it('also keeps the sanitizer conventional-host reporting heuristics for original resources',()=>{const source=readFileSync('../../crates/mw-sanitize/src/lib.rs','utf8');for(const [name,values]of [['TRACKER_LABELS',TRACKER_LABELS],['TRACKER_SUFFIXES',TRACKER_SUFFIXES]]as const){const block=new RegExp(`const ${name}:[^=]+= &\\[([\\s\\S]*?)\\];`).exec(source)?.[1]??'';expect([...block.matchAll(/"([^"]+)"/g)].map(match=>match[1])).toEqual(values);}});
it('preserves full document root, table and benign CSS without invoking a browser parser',()=>{
 const parse=vi.spyOn(DOMParser.prototype,'parseFromString').mockImplementation(()=>{throw Error('must stay inert');});
 try{const result=rendered('<!doctype html><html lang="pt" style="font-family:Georgia"><head><style>@media(max-width:600px){.content{padding:17px;display:grid;grid-template-columns:1fr 2fr}}.content{font:21px Georgia;letter-spacing:2px}</style></head><body bgcolor="#abcdef" style="margin:23px"><table class="content" cellpadding="13"><tr><td colspan="2">Original</td></tr></table></body></html>');expect(result).toContain('lang="pt"');expect(result).toContain('font-family:Georgia');expect(result).toContain('margin:23px');expect(result).toContain('grid-template-columns:1fr 2fr');expect(result).toContain('cellpadding="13"');expect(result).toContain('colspan="2"');expect(parse).not.toHaveBeenCalled();}finally{parse.mockRestore();}
});
it('removes active/navigation/resource constructs before any browser DOM can see them',()=>{
 const result=rendered('<meta http-equiv="refresh" content="0;url=https://evil.test"><base href="https://evil.test"><link rel="stylesheet" href="https://evil.test/x.css"><script>evil()</script><iframe src="https://evil.test" srcdoc="evil"></iframe><form action="https://evil.test"><input type="image" src="https://evil.test"></form><object data="https://evil.test"></object><video poster="https://evil.test"><source src="https://evil.test"></video><svg><image href="https://evil.test/x.svg"/><use href="https://evil.test/x.svg#id"/><set attributeName="href" to="https://evil.test"/></svg><img onerror="evil()" srcset="https://evil.test/a 1x" src="https://images.test/a.png"><a href="javascript:evil()">Bad</a>');
 expect(result).not.toMatch(/evil\(|evil\.test|onerror|srcset|<iframe|<form|<object|<video|<script|<link|<base/);expect(result).toContain('https://mail.test/mail/api/rc/i/image.signature');expect(result).toContain("script-src 'none'");expect(result).toContain("form-action 'none'");expect(result).toContain("frame-src 'none'");
});
it('rewrites CSS escapes and URL functions, blocks imports/fonts/unsupported image functions and retains layout',()=>{
 const prepared=prepareOriginalHtml(String.raw`<style>@\69mport "https://evil.test/import.css";@font-face{font-family:X;src:local("X"),url(https://evil.test/font.woff2)}.a{background:\75rl(\68ttps://images.test/a.png);padding:9px;cursor:image-set("https://evil.test/a" 1x)}.b{background:url("https://images.test/b.png");content:attr(data-url url);filter:expression(evil())}</style><div style="background-image:uRl('https://images.test/c.png');margin:11px">Body</div>`,policy);
 expect(prepared.refs.urls).toEqual(['https://images.test/a.png','https://images.test/b.png','https://images.test/c.png']);
 const result=renderOriginalHtml(prepared,{...EMPTY_SIGNED,images:prepared.refs.urls.map(url=>({url,src:'/mail/api/rc/i/safe.signature'}))},policy,{},base);
 expect(result).not.toMatch(/evil\.test|font-face|@\\69mport|image-set|expression|attr\(/);expect(result).toContain('padding:9px');expect(result).toContain('margin:11px');expect(result).toContain('url("https://mail.test/mail/api/rc/i/safe.signature")');
});
it('never accepts arbitrary server URLs, direct-origin responses, relative resources or SVG data',()=>{
 const prepared=prepareOriginalHtml('<img src="/api/me"><img src="//images.test/a.png"><img src="data:image/svg+xml;base64,PHN2Zz48L3N2Zz4="><div background="https://images.test/b.png"></div>',policy);
 const result=renderOriginalHtml(prepared,{...EMPTY_SIGNED,images:[{url:'https://images.test/a.png',src:'https://images.test/a.png'},{url:'https://images.test/b.png',src:'/mail/api/logout'}]}, {...policy,mode:'direct',allowDirect:true},{},base);
 expect(result).not.toContain('src="/api/me"');expect(result).not.toContain('data:image/svg');expect(result).not.toContain('background="https://images.test');expect(result).not.toContain('src="https://images.test');
 const csp=new DOMParser().parseFromString(result,'text/html').querySelector('meta[http-equiv]')?.getAttribute('content');expect(csp).toContain('img-src data: https://mail.test/mail/api/rc/i/ https://mail.test/mail/api/rc/p/');expect(csp).not.toContain(' https:;');
});
it('uses shared tracker lists and rejects declared tiny images without even requesting a signing capability',()=>{
 const prepared=prepareOriginalHtml('<img src="https://images.test/declared.png" width="1" height="2"><img src="https://x.list-manage.com/track/open.php?u=1"><img src="https://images.test/open.gif"><img src="https://images.test/visible.png" width="20" height="20">',policy);
 expect(prepared.refs.urls).toEqual(['https://images.test/visible.png']);expect(prepared.refs.trackers).toBe(3);
 const result=renderOriginalHtml(prepared,{...EMPTY_SIGNED,images:prepared.resources.map(value=>({url:value.value,src:'/mail/api/rc/i/unexpected.signature'}))},policy,{},base);
 const images=new DOMParser().parseFromString(result,'text/html').querySelectorAll('img');for(const image of [...images].slice(0,3))expect(image.src).toMatch(/^data:image\/gif/);expect(images[3]?.src).toContain('/api/rc/i/unexpected.signature');
});
it('blocks CSS-declared and escaped tiny pixels and conventional tracker labels before signing',()=>{
 const prepared=prepareOriginalHtml(String.raw`<img src="https://images.test/random-a.png" style="width:1px;height:2px"><img src="https://images.test/random-b.png" width="100" height="100" style="\77idth:1p\78;\68 eight:1px"><img src="https://track.example.test/random.png"><img src="https://email.example.test/random.png"><img src="https://images.test/logo.png" style="width:100px!important;width:1px;height:1px"><img src="https://images.test/empty.png" width="" height="">`,policy);
 expect(prepared.refs.urls).toEqual(['https://images.test/logo.png','https://images.test/empty.png']);expect(prepared.refs.trackers).toBe(4);
});
it('preserves inline CID and raster data through controlled resources and honors no-media mode',()=>{
 const prepared=prepareOriginalHtml('<img src="cid:logo@test"><img src="data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7">',policy);
 const signed={...EMPTY_SIGNED,parts:[{blobId:'part',src:'/mail/api/rc/p/part.signature'}]};expect(prepared.refs.cids).toEqual(['logo@test']);
 const result=renderOriginalHtml(prepared,signed,policy,{'logo@test':'part'},base);expect(result).toContain('src="https://mail.test/mail/api/rc/p/part.signature"');
 const blocked=renderOriginalHtml(prepared,signed,policy,{'logo@test':'part'},base,false);expect(blocked).not.toContain('src="https://mail.test');expect(blocked).not.toContain('mw-original:');
});
it('cleans tracking query parameters and retains only explicitly consented validated links',()=>{
 const prepared=prepareOriginalHtml('<a href="https://site.test/page?id=2&amp;utm_source=mail&amp;fbclid=3">Go</a><a href="mailto:Case@EXAMPLE.ORG">Email</a>',policy);
 const source=renderOriginalHtml(prepared,EMPTY_SIGNED,policy,{},base);const inert=reviewOriginalLinks(source,'Review',false);
 expect(inert.links[0]?.href).toBe('https://site.test/page?id=2');expect(new DOMParser().parseFromString(inert.html,'text/html').querySelector('a[href]')).toBeNull();expect(inert.html).toContain('data-mw-contact-address="Case@example.org"');
 const active=reviewOriginalLinks(source,'Review',true);expect(active.html).toContain('target="_blank"');expect(active.html).toContain('rel="noopener noreferrer"');expect(active.html).not.toContain('utm_source');
});
it('keeps authored links with query spaces and nested labels available for original-view review',()=>{
 const prepared=prepareOriginalHtml('<a href="  //shop.example/order status?name=Jane Doe&amp;query=a b&amp;literal=a+b  "><strong>Invoice</strong> details</a>',policy);
 expect(prepared.refs.links).toEqual(['https://shop.example/order%20status?name=Jane%20Doe&query=a%20b&literal=a+b']);
 const source=renderOriginalHtml(prepared,EMPTY_SIGNED,policy,{},base);
 const inert=reviewOriginalLinks(source,'Review',false);
 expect(inert.links[0]?.label).toBe('Invoice details');
 expect(new URL(inert.links[0]!.href).searchParams.get('name')).toBe('Jane Doe');
 expect(new DOMParser().parseFromString(inert.html,'text/html').querySelector('a[href]')).toBeNull();
 const anchor=new DOMParser().parseFromString(reviewOriginalLinks(source,'Review',true).html,'text/html').querySelector('a')!;
 expect(anchor.getAttribute('href')).toBe(prepared.refs.links[0]);expect(anchor.getAttribute('rel')).toBe('noopener noreferrer');
});
