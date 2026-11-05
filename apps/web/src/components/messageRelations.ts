import type {Email} from '../api/jmap-types.ts';
import {CAP_CORE,CAP_MAIL,type EmailGetResponse} from '../api/jmap-types.ts';
import {responseFor} from '../api/jmap.ts';
import type {Client} from '../api/client.ts';
import type {AttachmentPart} from '../viewers/attachments.ts';
/** What the list row does not carry: the attachment parts and, for answered or forwarded mail, its thread members. */
export interface MessageExtras {attachments:AttachmentPart[];replies:Email[];forwards:Email[]}
const RELATED=['id','threadId','messageId','inReplyTo','references','subject','to','receivedAt','sentAt','mailboxIds','keywords'];
const FORWARD_PREFIX=/^\s*(fwd?|enc|tr|wg|rv|vs|doorst|vl|i)\s*:/i;
const norm=(ids:string[]|null|undefined):string[]=>(ids??[]).map(id=>id.replace(/^<|>$/g,'').trim().toLowerCase()).filter(Boolean);
/** Heuristic: a thread member answers/forwards `original` when it is a different, non-draft message that cites the original's Message-ID
 * in In-Reply-To or References. A subject starting Fwd:/Fw:/Enc:/TR:/WG: (or a mailbox of role "sent" with such a subject and no citation)
 * makes it a forward; anything else citing the original is a reply. Replies must live in a Sent folder (a reply from someone else is not ours). */
export function classifyRelated(original:Pick<Email,'id'|'messageId'>,members:Email[],sentIds:ReadonlySet<string>):{replies:Email[];forwards:Email[]} {
 const own=new Set(norm(original.messageId)),replies:Email[]=[],forwards:Email[]=[];
 for(const member of members){
  if(member.id===original.id||member.keywords?.['$draft'])continue;
  const cites=[...norm(member.inReplyTo),...norm(member.references)].some(id=>own.has(id));
  const forward=FORWARD_PREFIX.test(member.subject??'');
  const inSent=Object.keys(member.mailboxIds??{}).some(id=>member.mailboxIds[id]&&sentIds.has(id));
  if(forward&&(cites||inSent))forwards.push(member);
  else if(!forward&&cites&&inSent)replies.push(member);
 }
 const byDate=(a:Email,b:Email)=>(b.sentAt??b.receivedAt).localeCompare(a.sentAt??a.receivedAt);
 return {replies:replies.sort(byDate),forwards:forwards.sort(byDate)};
}
/** One round trip: the message's attachments, its thread and the thread members' reference headers. */
export async function fetchMessageExtras(client:Pick<Client,'jmap'>,accountId:string,email:Pick<Email,'id'|'hasAttachment'|'keywords'>,sentIds:ReadonlySet<string>):Promise<MessageExtras> {
 const related=email.keywords?.['$answered']===true||email.keywords?.['$forwarded']===true;
 const calls:[string,Record<string,unknown>,string][]=[['Email/get',{accountId,ids:[email.id],properties:['id','messageId','attachments','threadId']},'self']];
 if(related){
  calls.push(['Thread/get',{accountId,'#ids':{resultOf:'self',name:'Email/get',path:'/list/*/threadId'}},'thread']);
  calls.push(['Email/get',{accountId,'#ids':{resultOf:'thread',name:'Thread/get',path:'/list/*/emailIds'},properties:RELATED},'members']);
 }
 const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:calls});
 const self=responseFor<EmailGetResponse>(response,'self').list[0] as (Email&{attachments?:AttachmentPart[]})|undefined;
 const found=related?classifyRelated({id:email.id,messageId:self?.messageId??null},responseFor<EmailGetResponse>(response,'members').list,sentIds):{replies:[],forwards:[]};
 const attachments=(self?.attachments??[]).filter(part=>part.blobId&&part.disposition!=='inline'||(part.blobId&&!part.cid));
 return {attachments,...found};
}
