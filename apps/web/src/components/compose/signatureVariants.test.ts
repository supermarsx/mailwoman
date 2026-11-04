import {expect,it} from 'vitest';
import {assignedSignature,signatureVariants} from './signatureVariants.ts';
import type {Identity} from '../../api/jmap-types.ts';
const identity=(over:Partial<Identity>={}):Identity=>({id:'work',name:'Work',email:'work@example.org',replyTo:null,sentMailboxId:null,signatureHtml:'<p>New brand</p>',signatureText:'New brand',signatureFormat:'html',...over});
const labels={new:'New messages',replies:'Replies'};
it('uses resolved distinct variants while keeping their owning identity for duplicate prevention',()=>{
 const id=identity({replySignatureHtml:'<table><tr><td>Reply brand</td></tr></table>',replySignatureText:'Reply brand',replySignatureFormat:'html'});
 expect(assignedSignature(id,false)).toMatchObject({id:'work',identityId:'work',text:'New brand'});expect(assignedSignature(id,true)).toMatchObject({id:'work:reply',identityId:'work',text:'Reply brand'});
 expect(signatureVariants([id],labels).map(sig=>sig.name)).toEqual(['Work — New messages','Work — Replies']);
});
it('inherits on old servers and deduplicates identical resolved new/reply assignments',()=>{
 const old=identity(),same=identity({replySignatureHtml:old.signatureHtml,replySignatureText:old.signatureText,replySignatureFormat:'html'});
 expect(assignedSignature(old,true)).toEqual(assignedSignature(old,false));expect(signatureVariants([same],labels)).toHaveLength(1);expect(signatureVariants([same],labels)[0]?.id).toBe('work');
});
it('keeps explicit none empty instead of falling back to the new signature',()=>{
 const id=identity({replySignatureHtml:null,replySignatureText:null,replySignatureFormat:'plain'});expect(assignedSignature(id,true)).toBeNull();expect(assignedSignature(id,false)?.text).toBe('New brand');
});
it('offers a reply-only signature and preserves literal plaintext spacing',()=>{
 const id=identity({signatureHtml:null,signatureText:null,replySignatureText:'  Reply\nRegards  ',replySignatureFormat:'plain'});expect(signatureVariants([id],labels)).toEqual([expect.objectContaining({id:'work:reply',plainText:'  Reply\nRegards  ',html:null})]);
});
