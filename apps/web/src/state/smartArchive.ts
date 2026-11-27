import {settings,settingsLoadStatus} from '../api/settings.ts';
import {CAP_CORE,CAP_MAIL,type Email,type Mailbox,type JmapRequest,type JmapResponse} from '../api/jmap-types.ts';
import {responseFor} from '../api/jmapWire.ts';

export const SMART_ARCHIVE_STRATEGIES=['sender','domain','conversation','sender-frequency'] as const;
export type SmartArchiveStrategy=typeof SMART_ARCHIVE_STRATEGIES[number];
export const SMART_ARCHIVE_DEFAULTS={enabled:false,replaceArchive:false,strategy:'sender' as SmartArchiveStrategy,multipleFolders:'ask' as 'ask'|'latest'|'most-used',confirmation:'when-needed' as 'always'|'when-needed'|'never',fallback:'archive' as 'archive'|'choose'|'cancel',historyLimit:100,includeSubdomains:false,excludedFolders:['role:inbox','role:sent','role:drafts','role:junk','role:trash']};
export type SmartArchivePolicy=typeof SMART_ARCHIVE_DEFAULTS;
export function smartArchivePolicy(map:Record<string,{value:unknown}|undefined>=settings(),ready=settingsLoadStatus()==='ready'):SmartArchivePolicy{
 const result={...SMART_ARCHIVE_DEFAULTS,excludedFolders:[...SMART_ARCHIVE_DEFAULTS.excludedFolders]};
 for(const key of ['enabled','replaceArchive','includeSubdomains'] as const){const value=map[`mail.smartArchive.${key}`]?.value;if(typeof value==='boolean')result[key]=value;}
 const choices={strategy:SMART_ARCHIVE_STRATEGIES,multipleFolders:['ask','latest','most-used'],confirmation:['always','when-needed','never'],fallback:['archive','choose','cancel']} as const;
 for(const key of Object.keys(choices) as (keyof typeof choices)[]){const value=map[`mail.smartArchive.${key}`]?.value;if(typeof value==='string'&&(choices[key] as readonly string[]).includes(value))Object.assign(result,{[key]:value});}
 const limit=map['mail.smartArchive.historyLimit']?.value;if(typeof limit==='number'&&Number.isInteger(limit)&&limit>=10&&limit<=500)result.historyLimit=limit;
 const excluded=map['mail.smartArchive.excludedFolders']?.value;if(Array.isArray(excluded)&&excluded.length<=100&&excluded.every(v=>typeof v==='string'&&/^(role|id):[^\r\n]+$/.test(v)))result.excludedFolders=[...excluded];
 if(!ready)result.enabled=false;
 return result;
}
export const replacesArchive=():boolean=>{const policy=smartArchivePolicy();return policy.enabled&&policy.replaceArchive;};
export const senderAddress=(email:Email):string=>(email.from?.[0]?.email??'').trim().toLowerCase();
const senderDomain=(email:Email):string=>{const address=senderAddress(email),at=address.lastIndexOf('@');return at>0?address.slice(at+1):'';};
export interface SmartArchiveCandidate {id:string;name:string;count:number;latest:string}
export interface SmartArchiveGroup {key:string;label:string;ids:string[];candidates:SmartArchiveCandidate[];target:string;fallback:boolean}
function folderPath(box:Mailbox,boxes:readonly Mailbox[]):string{
 const names=[box.name],seen=new Set([box.id]);let parent=box.parentId;
 while(parent&&names.length<50){const ancestor=boxes.find(value=>value.id===parent);if(!ancestor||seen.has(ancestor.id))break;seen.add(ancestor.id);names.unshift(ancestor.name);parent=ancestor.parentId;}
 return names.join(' / ');
}
/** Special-use folders are never inferred destinations. Archive is admitted
 * only for the separately configured ordinary-Archive fallback. */
export function eligibleSmartArchiveFolders(boxes:readonly Mailbox[],policy:SmartArchivePolicy,allowArchiveFallback=false):Mailbox[]{
 return boxes.filter(box=>(!box.role||allowArchiveFallback&&box.role==='archive')&&box.myRights?.mayAddItems!==false&&box.mailwomanSearchQuery===undefined&&!policy.excludedFolders.includes(`id:${box.id}`)&&!(box.role&&policy.excludedFolders.includes(`role:${box.role}`)));
}
export function smartArchiveCandidates(message:Email,history:readonly Email[],boxes:readonly Mailbox[],policy:SmartArchivePolicy,strategy:SmartArchiveStrategy,excludedIds:ReadonlySet<string>):SmartArchiveCandidate[]{
 const allowed=new Map(eligibleSmartArchiveFolders(boxes,policy).filter(box=>!message.mailboxIds[box.id]).map(box=>[box.id,box]));
 const candidates=new Map<string,SmartArchiveCandidate>(),seen=new Set<string>();
 for(const email of history){
  if(excludedIds.has(email.id)||seen.has(email.id))continue;seen.add(email.id);
  const domain=senderDomain(message),other=senderDomain(email);
  const matches=strategy==='conversation'?!!message.threadId&&email.threadId===message.threadId:strategy==='domain'?!!domain&&(other===domain||(policy.includeSubdomains&&other.endsWith(`.${domain}`))):!!senderAddress(message)&&senderAddress(email)===senderAddress(message);
  if(!matches)continue;
  for(const id of Object.keys(email.mailboxIds??{})){
   const box=allowed.get(id);if(!box||!email.mailboxIds[id])continue;
   const candidate=candidates.get(id)??{id,name:folderPath(box,boxes),count:0,latest:''};candidate.count++;
   if(!Number.isFinite(Date.parse(candidate.latest))||Date.parse(email.receivedAt)>Date.parse(candidate.latest))candidate.latest=email.receivedAt;
   candidates.set(id,candidate);
  }
 }
 const frequency=strategy==='sender-frequency'||policy.multipleFolders==='most-used';
 return [...candidates.values()].sort((a,b)=>(frequency?b.count-a.count:0)||(Date.parse(b.latest)||0)-(Date.parse(a.latest)||0)||a.id.localeCompare(b.id));
}
/** Searches metadata only and verifies exact sender/domain matches locally. */
export async function planSmartArchive(messages:readonly Email[],boxes:readonly Mailbox[],policy:SmartArchivePolicy,strategy:SmartArchiveStrategy,account:string,query:(request:JmapRequest)=>Promise<JmapResponse>):Promise<SmartArchiveGroup[]>{
 const requests=new Map<string,Promise<Email[]>>(),groups=new Map<string,SmartArchiveGroup>(),excluded=new Set(messages.map(message=>message.id));
 const eligible=new Set(eligibleSmartArchiveFolders(boxes,policy).map(box=>box.id));
 const excludedMailboxes=boxes.filter(box=>!eligible.has(box.id)).map(box=>box.id);
 for(const message of messages){
  const match=strategy==='conversation'?message.threadId??'':strategy==='domain'?senderDomain(message):senderAddress(message);
  const currentFolders=Object.keys(message.mailboxIds).filter(id=>message.mailboxIds[id]).sort();
  const excludedForMessage=[...new Set([...excludedMailboxes,...currentFolders])];
  const requestKey=JSON.stringify([strategy,match,currentFolders]);
  if(match&&!requests.has(requestKey)){
   const metadata=(ids?:string[]):JmapRequest=>({using:[CAP_CORE,CAP_MAIL],methodCalls:[
    ...(!ids?[['Email/query',{accountId:account,filter:{from:match,...(excludedForMessage.length?{inMailboxOtherThan:excludedForMessage}:{})},sort:[{property:'receivedAt',isAscending:false}],limit:policy.historyLimit},'sq'] as JmapRequest['methodCalls'][number]]:[]),
    ['Email/get',{accountId:account,...(ids?{ids}:{'#ids':{resultOf:'sq',name:'Email/query',path:'/ids'}}),properties:['id','from','threadId','receivedAt','mailboxIds'],fetchHTMLBodyValues:false,fetchTextBodyValues:false,mailwomanBodyUse:false},'sg'],
   ]});
   requests.set(requestKey,(async()=>{
    let ids:string[]|undefined;
    if(strategy==='conversation'){
     const response=await query({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Thread/get',{accountId:account,ids:[match]},'st']]});
     const thread=responseFor<{accountId:string;list:{id:string;emailIds:string[]}[]}>(response,'st');
     if(thread.accountId!==account)throw new Error('Smart archive account changed');
     ids=(thread.list.find(value=>value.id===match)?.emailIds??[]).slice(-policy.historyLimit);
     if(!ids.length)return [];
    }
    const result=responseFor<{accountId:string;list:Email[]}>(await query(metadata(ids)),'sg');
    if(result.accountId!==account)throw new Error('Smart archive account changed');
    return result.list;
   })());
  }
  const history=match?await requests.get(requestKey)!:[];
  let candidates=smartArchiveCandidates(message,history,boxes,policy,strategy,excluded),fallback=false,target='';
  if(candidates.length===0){
   fallback=true;
   if(policy.fallback==='cancel')continue;
   candidates=eligibleSmartArchiveFolders(boxes,policy,policy.fallback==='archive').filter(box=>!message.mailboxIds[box.id]&&(policy.fallback==='choose'||box.role==='archive')).map(box=>({id:box.id,name:folderPath(box,boxes),count:0,latest:''}));
  }
  if(candidates.length===0)continue;
  if((!fallback||policy.fallback!=='choose')&&(candidates.length===1||policy.multipleFolders!=='ask'))target=candidates[0]!.id;
  const key=JSON.stringify([match,candidates.map(candidate=>candidate.id),fallback]);
  const existing=groups.get(key);if(existing){existing.ids.push(message.id);continue;}
  groups.set(key,{key,label:(strategy==='domain'?match:strategy==='conversation'?message.subject:senderAddress(message))||message.subject||message.id,ids:[message.id],candidates,target,fallback});
 }
 return [...groups.values()];
}
export const needsSmartArchiveReview=(groups:readonly SmartArchiveGroup[],policy:SmartArchivePolicy):boolean=>groups.some(group=>!group.target)||policy.confirmation==='always'||policy.confirmation==='when-needed'&&(groups.length>1||groups.some(group=>group.fallback));
