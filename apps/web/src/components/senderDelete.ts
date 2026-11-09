import type {Client} from '../api/client.ts';
import {request,responseFor} from '../api/jmap.ts';
import {CAP_CORE,CAP_MAIL,type Email,type EmailGetResponse,type EmailQueryResponse,type Mailbox} from '../api/jmap-types.ts';

const MAX_MESSAGES=5000,PAGE_SIZE=200;
export interface SenderDeletePlan {sender:string;emails:Email[];trashId:string}
/** Query the whole account, verifying exact addresses after the server's text
 * candidate search. Missing/truncated/changing pages never authorize a partial sweep. */
export async function prepareSenderDelete(client:Pick<Client,'jmap'>,accountId:string,sender:string,mailboxes:readonly Mailbox[],current:()=>boolean):Promise<SenderDeletePlan>{
  const address=sender.trim().toLowerCase(),trash=mailboxes.find(box=>box.role==='trash');
  if(!address||!address.includes('@')||!trash||trash.myRights?.mayAddItems===false)throw new Error('senderDeleteUnavailable');
  const seen=new Set<string>(),emails:Email[]=[];let position=0,total:number|undefined,state:string|undefined;
  for(;;){
    if(!current())throw new Error('accountChanged');
    const result=await client.jmap(request([CAP_CORE,CAP_MAIL],[
      ['Email/query',{accountId,filter:{from:address,inMailboxOtherThan:[trash.id]},sort:[{property:'receivedAt',isAscending:false}],position,limit:PAGE_SIZE,calculateTotal:true},'q'],
      ['Email/get',{accountId,'#ids':{resultOf:'q',name:'Email/query',path:'/ids'},properties:['id','from','mailboxIds','keywords','subject','receivedAt']},'g'],
    ]));
    if(!current())throw new Error('accountChanged');
    const query=responseFor<EmailQueryResponse>(result,'q'),page=responseFor<EmailGetResponse>(result,'g');
    if(query.accountId!==accountId||page.accountId!==accountId||query.position!==position||typeof query.total!=='number'||!Number.isSafeInteger(query.total)||query.total<0||query.total>MAX_MESSAGES||page.notFound.length)throw new Error('senderDeleteIncomplete');
    total??=query.total;state??=query.queryState;
    if(total!==query.total||state!==query.queryState||query.ids.some(id=>seen.has(id))||page.list.length!==query.ids.length)throw new Error('senderDeleteIncomplete');
    const returned=new Map(page.list.map(email=>[email.id,email]));
    for(const id of query.ids){
      const email=returned.get(id);if(!email)throw new Error('senderDeleteIncomplete');seen.add(id);
      if(email.mailboxIds[trash.id]||!email.from?.some(from=>from.email.trim().toLowerCase()===address))continue;
      emails.push(email);
    }
    position+=query.ids.length;
    if(position===total)break;
    if(query.ids.length===0||position>total)throw new Error('senderDeleteIncomplete');
  }
  return{sender:address,emails,trashId:trash.id};
}

export function senderDeleteRights(plan:SenderDeletePlan,mailboxes:readonly Mailbox[]):boolean{
  const boxes=new Map(mailboxes.map(box=>[box.id,box]));
  if(!boxes.has(plan.trashId)||boxes.get(plan.trashId)?.myRights?.mayAddItems===false)return false;
  return plan.emails.every(email=>Object.entries(email.mailboxIds).filter(([,present])=>present).every(([id])=>boxes.has(id)&&boxes.get(id)?.myRights?.mayRemoveItems!==false));
}
