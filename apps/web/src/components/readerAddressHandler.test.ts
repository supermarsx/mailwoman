import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {expect,it,vi} from 'vitest';
import {normalizeContactAddress} from '../modules/contacts/contactAddress.ts';
const source=readFileSync(resolve(__dirname,'../../../../crates/mw-server/src/reader-address-handler.js'),'utf8');
function handler(){
 const parent={postMessage:vi.fn()};const port={start:vi.fn(),postMessage:vi.fn(),close:vi.fn()};
 const listeners=new Map<string,(event:Record<string,unknown>)=>void>();
 let context:((event:Record<string,unknown>)=>void)|undefined;
 const documentListeners=new Map<string,()=>void>();
 const doc={readyState:'loading',removeEventListener:(type:string)=>documentListeners.delete(type),addEventListener:(type:string,fn:typeof context)=>{if(type==='contextmenu')context=fn;else if(fn)documentListeners.set(type,fn as ()=>void);},caretPositionFromPoint:()=>null as {offsetNode:Text;offset:number}|null,createRange:()=>({setStart:vi.fn(),setEnd:vi.fn(),getClientRects:()=>[{left:0,right:999,top:0,bottom:999}]})};
 new Function('document','parent','addEventListener','removeEventListener',source)(doc,parent,(type:string,fn:(event:Record<string,unknown>)=>void)=>listeners.set(type,fn),(type:string)=>listeners.delete(type));
 const init={source:parent,origin:'null',data:{type:'mw-reader-address-init',nonce:'a'.repeat(48),revision:1},ports:[port]};
 const anchor=document.createElement('a');
 const event={isTrusted:true,defaultPrevented:false,target:anchor,clientX:12,clientY:20,preventDefault:vi.fn()};
 return {parent,port,listeners,init,anchor,event,doc,documentListeners,context:()=>context!(event)};
}
it('requires the opaque parent authenticated one-shot channel and a trusted real context gesture',()=>{
 const h=handler();h.anchor.href='mailto:Case@EXAMPLE.ORG';
 h.context();expect(h.event.preventDefault).not.toHaveBeenCalled();
 const initialize=h.listeners.get('message')!;
 initialize({...h.init,source:{}});initialize({...h.init,origin:'https://mail.example'});initialize({...h.init,data:{...h.init.data,nonce:'bad'}});expect(h.port.start).not.toHaveBeenCalled();
 initialize(h.init);expect(h.port.start).toHaveBeenCalledOnce();expect(h.listeners.has('message')).toBe(false);
 h.event.isTrusted=false;h.context();expect(h.event.preventDefault).not.toHaveBeenCalled();
 h.event.isTrusted=true;h.context();expect(h.event.preventDefault).toHaveBeenCalledOnce();expect(h.port.postMessage).toHaveBeenCalledWith({type:'address-context',nonce:'a'.repeat(48),revision:1,address:'Case@example.org',x:12,y:20});
 h.listeners.get('pagehide')!({});expect(h.port.close).toHaveBeenCalledOnce();
});
it('matches shared canonical mailbox validation for aliases, quoted/EAI/literals and refuses action injection',()=>{
 const h=handler();h.listeners.get('message')!(h.init);
 for(const candidate of ['Case+tag@EXAMPLE.ORG','"First Last"@EXAMPLE.ORG','José@EXAMPLE.ORG','a@[127.0.0.1]','a@[IPv6:::1]','not an address','a@example.org,b@example.org','a@example.org\r\nbcc:x@y.org','a@exa;mple.org','a@.example.org']){
  h.anchor.setAttribute('data-mw-contact-address',candidate);const previous=h.port.postMessage.mock.calls.length;h.context();const expected=normalizeContactAddress(candidate);
  if(expected)expect(h.port.postMessage.mock.calls.at(-1)?.[0]).toMatchObject({address:expected});else expect(h.port.postMessage).toHaveBeenCalledTimes(previous);
 }
});
it('ignores mailto header actions and preserves ordinary non-email context menus',()=>{
 const h=handler();h.listeners.get('message')!(h.init);
 h.anchor.href='mailto:A@EXAMPLE.ORG?bcc=hidden@example.org';h.context();expect(h.port.postMessage.mock.calls[0]?.[0]).toMatchObject({address:'A@example.org'});
 h.port.postMessage.mockClear();h.event.preventDefault.mockClear();
 for(const value of ['https://example.org','javascript:evil()','mailto:a@example.org,b@example.org','mailto:a%0Ab@example.org']){h.anchor.setAttribute('href',value);h.context();}
 expect(h.port.postMessage).not.toHaveBeenCalled();expect(h.event.preventDefault).not.toHaveBeenCalled();
});

it('preserves native web URL context instead of treating URL userinfo as an email local part',()=>{
 const h=handler();h.listeners.get('message')!(h.init);
 const text=document.createTextNode('Visit https://person@example.org today');
 h.doc.caretPositionFromPoint=()=>({offsetNode:text,offset:22});h.context();
 expect(h.port.postMessage).not.toHaveBeenCalled();expect(h.event.preventDefault).not.toHaveBeenCalled();
});

it('announces readable content after DOM parsing without waiting for image load, and removes pending listeners on replacement',()=>{
 const h=handler();h.listeners.get('message')!(h.init);
 expect(h.port.postMessage).not.toHaveBeenCalled();
 h.documentListeners.get('DOMContentLoaded')!();
 expect(h.port.postMessage).toHaveBeenCalledWith({type:'displayed',nonce:'a'.repeat(48),revision:1});
 h.listeners.get('pagehide')!({});expect(h.documentListeners.has('DOMContentLoaded')).toBe(false);
 const loaded=handler();loaded.doc.readyState='complete';loaded.listeners.get('message')!(loaded.init);
 expect(loaded.port.postMessage).toHaveBeenCalledWith({type:'displayed',nonce:'a'.repeat(48),revision:1});
});
