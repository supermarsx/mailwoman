import {describe,expect,it} from 'vitest';
import {hasFileAttachments,isInlineResourcePart} from './attachmentMetadata.ts';

describe('MIME-aware attachment metadata',()=>{
 it.each(['inline',undefined])('a PDF with Content-ID is a file with disposition %s',disposition=>{
  const pdf={type:'application/pdf',cid:'invoice@sender',disposition,name:'iban_Habito.pdf'};
  expect(isInlineResourcePart(pdf)).toBe(false);
  expect(hasFileAttachments({hasAttachment:false,attachments:[pdf]})).toBe(true);
 });
 it('does not require a PDF filename to recognise it as a file',()=>{
  expect(isInlineResourcePart({type:'application/pdf',cid:'part',disposition:'inline'})).toBe(false);
 });
 it('retains only image body resources as inline, with explicit attachment winning',()=>{
  const image={type:'image/png',cid:'logo',disposition:'inline',name:'logo.png'};
  expect(isInlineResourcePart(image)).toBe(true);
  expect(hasFileAttachments({hasAttachment:false,attachments:[image]})).toBe(false);
  expect(isInlineResourcePart({...image,disposition:'ATTACHMENT'})).toBe(false);
  expect(isInlineResourcePart({type:'image/jpeg',disposition:'inline'})).toBe(true);
  expect(isInlineResourcePart({type:'image/jpeg',disposition:'inline',name:'photo.jpg'})).toBe(false);
 });
 it('does not silently classify unknown or other document types as body images',()=>{
  for(const type of [undefined,'application/octet-stream','application/msword','text/calendar'])expect(isInlineResourcePart({type,cid:'x',disposition:'inline'})).toBe(false);
 });
 it('tolerates malformed type/disposition metadata without crashing the list',()=>{
  expect(isInlineResourcePart({type:42,disposition:13,cid:'part'} as unknown as Parameters<typeof isInlineResourcePart>[0])).toBe(false);
 });
 it('preserves positive upstream flags but cannot invent attachments from no metadata',()=>{
  expect(hasFileAttachments({hasAttachment:true,attachments:[]})).toBe(true);
  expect(hasFileAttachments({hasAttachment:false,attachments:[]})).toBe(false);
  expect(hasFileAttachments({hasAttachment:false})).toBe(false);
 });
});
