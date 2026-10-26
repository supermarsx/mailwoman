import type {Client} from '../api/client.ts';
import {CAP_CORE,CAP_MAIL,type EmailGetResponse} from '../api/jmap-types.ts';
import {responseFor} from '../api/jmap.ts';
import {parseAttachments} from '../viewers/attachments.ts';
/** Resolve an opaque saved reference through current account authorization. */
export async function resolveWorkspaceAttachment(client:Pick<Client,'jmap'>,accountId:string,messageId:string,blobId:string){
 const abort=new AbortController(),timer=setTimeout(()=>abort.abort(),10000);
 try{
  const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/get',{accountId,ids:[messageId],properties:['id','attachments','subject','from','receivedAt']},'workspace-attachment']]},{signal:abort.signal});
  const got=responseFor<EmailGetResponse>(response,'workspace-attachment');
  if(abort.signal.aborted||got.accountId!==accountId||got.list.length!==1||got.list[0]?.id!==messageId)return null;
  return parseAttachments(got.list).find(item=>item.blobId===blobId)??null;
 }finally{clearTimeout(timer);}
}
