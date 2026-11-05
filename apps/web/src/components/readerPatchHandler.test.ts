import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {beforeEach,expect,it,vi} from 'vitest';
const source=readFileSync(resolve(__dirname,'../../../../crates/mw-server/src/reader-address-handler.js'),'utf8');
const base='https://mail.example.test/api/rc/';
const gif='data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';
function boot(body:string,scroll?:{x:number;y:number}){
 document.head.innerHTML='<style>.h{color:red}</style>';document.body.innerHTML=body;
 const parent={postMessage:vi.fn()};const port={start:vi.fn(),postMessage:vi.fn(),close:vi.fn(),onmessage:null as null|((e:{data:unknown})=>void)};
 const listeners=new Map<string,(event:Record<string,unknown>)=>void>();
 new Function('document','parent','addEventListener','removeEventListener',source)(document,parent,(t:string,fn:(e:Record<string,unknown>)=>void)=>listeners.set(t,fn),(t:string)=>listeners.delete(t));
 const nonce='b'.repeat(48);
 listeners.get('message')!({source:parent,origin:'null',data:{type:'mw-reader-address-init',nonce,revision:3,prefixes:[base+'i/',base+'p/'],...(scroll?{scroll}:{})},ports:[port]});
 const send=(ops:unknown[],over:Record<string,unknown>={})=>{port.postMessage.mockClear();port.onmessage!({data:{type:'patch',nonce,revision:3,id:7,ops,...over}});return port.postMessage.mock.calls.at(-1)?.[0];};
 return {port,send,listeners};
}
beforeEach(()=>{vi.stubGlobal('scrollTo',vi.fn());});
it('applies validated image patches in place, keeping the same elements',()=>{
 const h=boot(`<p>Hi</p><img src="${gif}"><div style="background:none"></div><style>.a{}</style>`);
 const img=document.querySelector('img')!;
 const result=h.send([{s:'b',i:1,a:'src',v:base+'i/tok.en'},{s:'b',i:2,a:'style',v:`background:url("${base}i/x")`},{s:'b',i:3,a:'text',v:`.a{background:url("${base}p/y")}`},{s:'h',i:0,a:'text',v:'.h{color:blue}'}]);
 expect(result).toEqual({type:'patch-result',nonce:'b'.repeat(48),revision:3,id:7,ok:true});
 expect(document.querySelector('img')).toBe(img);expect(img.getAttribute('src')).toBe(base+'i/tok.en');
 expect(document.head.querySelector('style')!.textContent).toBe('.h{color:blue}');
});
it('refuses the whole batch when any operation is not a signed handle or inline image',()=>{
 const h=boot(`<img src="${gif}"><img src="${gif}">`);
 for(const bad of ['https://evil.test/a.png',base+'f/tok',base+'i/a/b','javascript:1','data:text/html;base64,AA=='])
  expect(h.send([{s:'b',i:0,a:'src',v:base+'i/ok'},{s:'b',i:1,a:'src',v:bad}])).toMatchObject({ok:false});
 expect(document.querySelectorAll('img')[0]!.getAttribute('src')).toBe(gif);
 expect(h.send([{s:'b',i:0,a:'href',v:'https://evil.test'}])).toMatchObject({ok:false});
 expect(h.send([{s:'b',i:9,a:'src',v:gif}])).toMatchObject({ok:false});
 expect(h.send([{s:'b',i:0,a:'style',v:'background:url(https://evil.test/x)'}])).toMatchObject({ok:false});
 expect(h.send([{s:'b',i:0,a:'style',v:`background:image-set(url(${base}i/a) 1x)`}])).toMatchObject({ok:false});
});
it('ignores patches with a wrong nonce or revision and restores scroll after load',()=>{
 const h=boot('<p>x</p>',{x:0,y:480});
 h.port.postMessage.mockClear();
 h.port.onmessage!({data:{type:'patch',nonce:'c'.repeat(48),revision:3,id:1,ops:[]}});
 h.port.onmessage!({data:{type:'patch',nonce:'b'.repeat(48),revision:2,id:1,ops:[]}});
 expect(h.port.postMessage.mock.calls.filter(c=>c[0].type==='patch-result')).toHaveLength(0);
 expect(globalThis.scrollTo).toHaveBeenCalledWith(0,480);
});
