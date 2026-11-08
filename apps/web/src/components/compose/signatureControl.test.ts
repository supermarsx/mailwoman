import {createRoot,createSignal} from 'solid-js';
import {describe,it,expect,vi} from 'vitest';
import {createSignatureController,type SignatureHost} from './signatureControl.ts';
import {stripSignatureMarker} from './signatureRegion.ts';
import type {ComposeSignature} from './ComposerExtras.tsx';
const first:ComposeSignature={id:'a',identityId:'a',name:'Personal',text:'Alice',html:null};
const second:ComposeSignature={id:'b',identityId:'b',name:'Work',text:'Alice at Work',html:null};
function setup(rich=false,overrides:Partial<SignatureHost>={}){return createRoot(dispose=>{
 const [body,setBody]=createSignal('Answer'),[html,setHtml]=createSignal('<p>Answer</p>');
 const markInserted=vi.fn(),markRemoved=vi.fn();
 const controller=createSignatureController({richMode:()=>rich,body,bodyHtml:html,setPlain:setBody,setHtml,insertAtCursor:vi.fn(),renderHtml:async s=>`<p>${s.text}</p>`,renderPlain:async s=>s.text,variants:()=>[first,second],defaultFor:id=>id==='a'?first:second,identityId:()=> 'a',quoteAnchor:()=> 'Original message',quoteKind:()=> 'reply',defaultPosition:()=> 'above-quote',markInserted,markRemoved,...overrides});
 return {dispose,body,setBody,html,setHtml,controller,markInserted,markRemoved};
});}
describe('managed signature control',()=>{
 it('replaces rather than duplicates the selected plain signature and removes it without the answer',async()=>{
  const h=setup();await h.controller.apply(first);await h.controller.apply(second);
  expect(h.body()).toBe('Answer\n\n-- \nAlice at Work');expect(h.controller.applied()?.sig.id).toBe('b');
  h.controller.remove();expect(h.body()).toBe('Answer');expect(h.markRemoved).toHaveBeenCalledWith('b');h.dispose();
 });
 it('does not remove the original sender signature while inserting above the quoted message',async()=>{
  const h=setup();h.setBody('Answer\n\nOriginal message\nSender text\n-- \nSender');
  await h.controller.apply(first);expect(h.body()).toContain('Alice\n\nOriginal message\nSender text\n-- \nSender');h.dispose();
 });
 it('None suppresses automatic append even when no managed region is in the body',()=>{
  const h=setup();h.controller.remove();expect(h.body()).toBe('Answer');expect(h.markRemoved).toHaveBeenCalledWith('a');h.dispose();
 });
 it('keeps HTML selection to one managed region and strips its marker on send',async()=>{
  const h=setup(true);await h.controller.apply(first);await h.controller.apply(second);
  expect(h.html()).toContain('Answer');expect(h.html()).not.toMatch(/>Alice</);expect(h.html()).toContain('Alice at Work');
  expect(h.html().match(/data-mw-signature/g)).toHaveLength(1);expect(stripSignatureMarker(h.html())).not.toContain('data-mw-signature');h.dispose();
 });
 it('places HTML above the quoted header, and can move it to the end',async()=>{
  const h=setup(true);h.setHtml('<p>Answer</p><p>Original message</p><blockquote>Sender</blockquote>');
  await h.controller.apply(first);expect(h.html().indexOf('Alice')).toBeLessThan(h.html().indexOf('Original message'));
  await h.controller.reposition('end');expect(h.html().indexOf('Alice')).toBeGreaterThan(h.html().indexOf('Sender'));h.dispose();
 });
 it('replaces an unedited previous identity default on From change',async()=>{
  const h=setup();await h.controller.apply(first);await h.controller.identityChanged('a','b');expect(h.body()).toContain('Alice at Work');h.dispose();
 });
 it('offers replacement instead of overwriting an edited signature',async()=>{
  const h=setup();await h.controller.apply(first);h.setBody(h.body()+' edited');await h.controller.identityChanged('a','b');
  expect(h.body()).toContain('Alice edited');expect(h.controller.offer()?.id).toBe('b');await h.controller.acceptOffer();expect(h.body()).toContain('Alice at Work');h.dispose();
 });
 it('exposes pending rendering so a quick send waits for insertion',async()=>{
  const h=setup();const work=h.controller.apply(first);expect(h.controller.busy()).toBe(true);await h.controller.whenReady();await work;
  expect(h.body()).toContain('Alice');expect(h.controller.busy()).toBe(false);h.dispose();
 });
 it.each(['apply','insertCopy'] as const)('refuses stale asynchronous %s work after a draft/mailbox/identity switch',async operation=>{
  let valid=true,release:((value:string)=>void)|undefined;
  const inserted=vi.fn();
  const h=setup(true,{captureAuthority:()=>()=>valid,renderHtml:()=>new Promise<string>(resolve=>{release=resolve;}),insertAtCursor:inserted});
  const work=h.controller[operation](first);const rejection=expect(work).rejects.toThrow('authority changed');
  valid=false;release!('<p>Alice</p>');await rejection;
  expect(h.html()).toBe('<p>Answer</p>');expect(h.markInserted).not.toHaveBeenCalled();expect(inserted).not.toHaveBeenCalled();h.dispose();
 });

});
