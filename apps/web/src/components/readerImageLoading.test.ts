import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {expect,it,vi} from 'vitest';

const source=readFileSync(resolve(__dirname,'../../../../crates/mw-server/src/reader-address-handler.js'),'utf8');

function frame(html:string){
 const doc=document.implementation.createHTMLDocument('Message');doc.body.innerHTML=html;
 Object.defineProperty(doc,'readyState',{value:'complete'});
 const parent={postMessage:vi.fn()},port={start:vi.fn(),postMessage:vi.fn(),close:vi.fn()};
 const listeners=new Map<string,(event:unknown)=>void>();
 const start=()=>{
  new Function('document','parent','addEventListener','removeEventListener',source)(doc,parent,(type:string,listener:(event:unknown)=>void)=>listeners.set(type,listener),(type:string)=>listeners.delete(type));
  listeners.get('message')!({source:parent,origin:'null',data:{type:'mw-reader-address-init',nonce:'a'.repeat(48),revision:1},ports:[port]});
 };
 return {doc,start,close:()=>listeners.get('pagehide')!({}),port};
}
function resource(image:HTMLImageElement,complete:boolean,width:number){
 Object.defineProperty(image,'complete',{configurable:true,value:complete});
 Object.defineProperty(image,'naturalWidth',{configurable:true,value:width});
}
const mutations=async()=>{await Promise.resolve();await Promise.resolve();};

it('keeps sender sizing and styles while slow, then settles load/error without waiting to display text',()=>{
 const f=frame('<p>Readable now</p><img src="/slow" width="640" height="320" style="width:50%;aspect-ratio:2;background:red"><img src="/failure" alt="Banner">');
 const [slow,failed]=Array.from(f.doc.images);resource(slow!,false,0);resource(failed!,false,0);
 const authored=slow!.getAttribute('style');f.start();
 expect(f.port.postMessage).toHaveBeenCalledWith(expect.objectContaining({type:'displayed'}));
 expect(slow!.dataset.mwImageState).toBe('loading');expect(slow!.hasAttribute('data-mw-image-unsized')).toBe(false);
 expect(slow!.getAttribute('width')).toBe('640');expect(slow!.getAttribute('height')).toBe('320');expect(slow!.getAttribute('style')).toBe(authored);
 expect(failed!.hasAttribute('data-mw-image-unsized')).toBe(true);
 resource(slow!,true,640);slow!.dispatchEvent(new Event('load'));
 resource(failed!,true,0);failed!.dispatchEvent(new Event('error'));
 expect(slow!.dataset.mwImageState).toBe('loaded');expect(failed!.dataset.mwImageState).toBe('error');
 expect(failed!.hasAttribute('data-mw-image-unsized')).toBe(false);expect(slow!.getAttribute('style')).toBe(authored);f.close();
});

it('recognizes cached success/failure and empty images without animating them',()=>{
 const f=frame('<img src="/cached"><img src="/broken"><img><img src="/tiny" width="1" height="1">');
 const [cached,broken,empty,tiny]=Array.from(f.doc.images);resource(cached!,true,20);resource(broken!,true,0);resource(tiny!,false,0);f.start();
 expect(cached!.dataset.mwImageState).toBe('loaded');expect(broken!.dataset.mwImageState).toBe('error');expect(empty!.dataset.mwImageState).toBeUndefined();
 expect(tiny!.dataset.mwImageState).toBe('loading');expect(tiny!.hasAttribute('data-mw-image-unsized')).toBe(false);f.close();
});

it('handles in-place retry, cached source changes and dynamically inserted images',async()=>{
 const f=frame('<img src="/failed" width="120" height="60">');const image=f.doc.images[0]!;resource(image,true,0);f.start();
 resource(image,false,0);image.src='/retry';await mutations();expect(image.dataset.mwImageState).toBe('loading');
 resource(image,true,120);image.dispatchEvent(new Event('load'));expect(image.dataset.mwImageState).toBe('loaded');
 image.src='/cached';await mutations();expect(image.dataset.mwImageState).toBe('loaded');
 const inserted=f.doc.createElement('img');resource(inserted,false,0);inserted.src='/new';f.doc.body.append(inserted);await mutations();expect(inserted.dataset.mwImageState).toBe('loading');
 image.removeAttribute('src');await mutations();expect(image.dataset.mwImageState).toBeUndefined();f.close();
});

it('disconnects image events and queued retry mutations when a message document is replaced',async()=>{
 const old=frame('<img src="/old">');const image=old.doc.images[0]!;resource(image,false,0);old.start();image.src='/old-retry';old.close();
 const next=frame('<img src="/new">');const current=next.doc.images[0]!;resource(current,false,0);next.start();
 resource(image,true,100);image.dispatchEvent(new Event('load'));await mutations();
 expect(image.dataset.mwImageState).toBe('loading');expect(current.dataset.mwImageState).toBe('loading');expect(old.port.close).toHaveBeenCalledOnce();next.close();
});
