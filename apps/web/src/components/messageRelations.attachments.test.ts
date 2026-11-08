import {expect,it,vi} from 'vitest';
import type {JmapResponse} from '../api/jmap-types.ts';
import {fetchMessageExtras} from './messageRelations.ts';

it('keeps inline-disposition CID PDFs for context downloads while omitting actual image resources',async()=>{
 const parts=[
  {partId:'1',blobId:'m.1',name:'iban_Habito.pdf',type:'application/pdf',size:10,cid:'invoice',disposition:'inline'},
  {partId:'2',blobId:'m.2',name:'logo.png',type:'image/png',size:10,cid:'logo',disposition:'inline'},
  {partId:'3',blobId:'m.3',name:'attached.png',type:'image/png',size:10,cid:'photo',disposition:'attachment'},
  {partId:'4',blobId:null,name:'unavailable.pdf',type:'application/pdf',size:10,disposition:'attachment'},
 ];
 const client={jmap:vi.fn(async()=>({sessionState:'s',methodResponses:[['Email/get',{accountId:'a',state:'s',list:[{id:'m',attachments:parts}],notFound:[]},'self']]} as JmapResponse))};
 const found=await fetchMessageExtras(client,'a',{id:'m',hasAttachment:false,keywords:{}},new Set());
 expect(found.attachments.map(part=>part.name)).toEqual(['iban_Habito.pdf','attached.png']);
 expect(found.attachments[0]?.blobId).toBe('m.1');
 expect(client.jmap).toHaveBeenCalledTimes(1);
});
