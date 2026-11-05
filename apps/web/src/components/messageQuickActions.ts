import type {Email} from '../api/jmap-types.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {emailGetFull,responseFor} from '../api/jmap.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import type {EmailGetResponse} from '../api/jmap-types.ts';
import {getKeymap} from '../keymap/index.ts';
import {sanitizeBody} from '../sanitize/index.ts';
import {extractHtmlBody} from '../state/slices/mail.ts';
import {printMessage} from '../viewers/printMessage.ts';
import {mailFeature} from '../state/mailFeatures.ts';
import {t} from '../i18n';
/** Actions the message list's context menu runs on a message that is not necessarily open. */
export type ReplyAction='reply'|'replyAll'|'forward';
/** Whether a message action is allowed by the mail feature switches. */
export function replyAllowed(action:ReplyAction):boolean {return mailFeature(action==='forward'?'allowForward':'allowReply');}
interface OpenApp {accountId:()=>string|null|undefined;openMessage:(id:string)=>Promise<void>;openEmail:()=>Email|null;showToast:(tone:'success'|'error'|'info',message:string)=>void}
/** Reply, reply all or forward `id`: the message is opened in the reader, which owns decryption, the quoted body and threading, and then its own reply action runs. */
export async function replyFromList(app:OpenApp,id:string,action:ReplyAction,wait:(ms:number)=>Promise<void>=ms=>new Promise(resolve=>setTimeout(resolve,ms))):Promise<boolean> {
 if(!replyAllowed(action))return false;
 await app.openMessage(id);
 for(let waited=0;waited<4000;waited+=50){
  if(app.openEmail()?.id===id)break;
  await wait(50);
 }
 if(app.openEmail()?.id!==id||!getKeymap().run(action)){app.showToast('error',t('mail-reply-failed'));return false;}
 return true;
}
const plain=(email:Email):string=>{const values=email.bodyValues??{};const text=(email.textBody??[]).map(part=>part.partId!==null?(values[part.partId]?.value??''):'').join('\n');return text.length>0?text:(email.preview??'');};
/** Print `id` without opening it: its body is fetched, made safe with the reader's sanitizer (plain text when there is no HTML), and sent to a hidden print frame. */
export async function quickPrint(app:Pick<OpenApp,'accountId'|'showToast'>,id:string,fetchFull?:(accountId:string,id:string)=>Promise<Email|undefined>):Promise<void> {
 const owner=preferenceAccount(),account=app.accountId();
 if(!owner||!account||owner.id!==account){app.showToast('error',t('mail-context-print-failed'));return;}
 try{
  const email=await (fetchFull?fetchFull(account,id):(async()=>responseFor<EmailGetResponse>(await createConfiguredClient().jmap(emailGetFull(account,id)),'g').list[0])());
  if(!email||!preferenceAccountIsCurrent(owner)||app.accountId()!==account)throw new Error('gone');
  const raw=extractHtmlBody(email);
  const html=raw?await sanitizeBody({op:'reader',html:raw,blockTrackers:true,cleanLinks:true}):undefined;
  printMessage(email,html?{html}:{text:plain(email)});
 }catch{app.showToast('error',t('mail-context-print-failed'));}
}
/** Copy text to the clipboard; the result says whether it worked. */
export async function copyText(text:string):Promise<boolean> {
 try{await navigator.clipboard.writeText(text);return true;}catch{return false;}
}
/** Add `id` to `targetId` (a copy: it stays where it is). Resolves true when the server updated it. */
export async function copyMessageToFolder(app:Pick<OpenApp,'accountId'|'showToast'>,id:string,targetId:string,refresh:()=>Promise<unknown>):Promise<boolean> {
 const owner=preferenceAccount(),account=app.accountId();
 if(!owner||!account||owner.id!==account)return false;
 try{
  const response=await createConfiguredClient().jmap({using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Email/set',{accountId:account,update:{[id]:{[`mailboxIds/${targetId.replaceAll('~','~0').replaceAll('/','~1')}`]:true}}},'copy']]});
  if(!preferenceAccountIsCurrent(owner))return false;
  const reply=response.methodResponses.find(row=>row[2]==='copy')?.[1] as {updated?:Record<string,unknown>|null}|undefined;
  const ok=!!reply?.updated&&id in reply.updated;
  if(ok)void refresh();
  return ok;
 }catch{return false;}
}
