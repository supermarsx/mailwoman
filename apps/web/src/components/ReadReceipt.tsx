import {createEffect,createSignal,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {loadReceiptDecision,type ReceiptDecision} from '../api/receipts.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {CAP_CORE,CAP_MAIL} from '../api/jmap-types.ts';
import {loadCatalog,t} from '../i18n';

export interface ReadReceiptApi {
 load(account:string,email:string,signal:AbortSignal):Promise<ReceiptDecision|null>;
 send(account:string,email:string,automatic:boolean,signal:AbortSignal):Promise<void>;
}
const client=createConfiguredClient();
const api:ReadReceiptApi={load:loadReceiptDecision,async send(account,email,automatic,signal){
 const result=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['MDN/send',{accountId:account,emailId:email,automatic},'receipt']]},{signal});
 const response=result.methodResponses.find(row=>row[2]==='receipt');
 if(response?.[0]!=='MDN/send'||response[1]['sent']!==true)throw new Error('Receipt outcome not confirmed');
}};

/** Preview/background reads never send automatically. A displayed active full view may. */
export function ReadReceipt(props:{accountId:string;emailId:string;active:boolean;displayed:boolean;fullView:boolean;api?:ReadReceiptApi}):JSX.Element {
 const [decision,setDecision]=createSignal<ReceiptDecision|null>(null);const [busy,setBusy]=createSignal(false);
 const [attempted,setAttempted]=createSignal(false);const [error,setError]=createSignal(false);const [dismissed,setDismissed]=createSignal(false);
 const [visible,setVisible]=createSignal(document.visibilityState==='visible');
 let version=0;let controller:AbortController|undefined;let identity='';
  onMount(()=>{void loadCatalog('receipts');const update=():void=>{setVisible(document.visibilityState==='visible');};document.addEventListener('visibilitychange',update);onCleanup(()=>document.removeEventListener('visibilitychange',update));});
 createEffect(()=>{
  const account=props.accountId,email=props.emailId,displayed=props.displayed;const epoch=++version;
  controller?.abort();controller=new AbortController();setDecision(null);setBusy(false);
  const key=JSON.stringify([account,email]);if(key!==identity){identity=key;setAttempted(false);setError(false);setDismissed(false);}
  if(!displayed)return;
  void (props.api??api).load(account,email,controller.signal).then(value=>{if(epoch===version)setDecision(value);}).catch(()=>{/* No automatic fallback when eligibility is unavailable. */});
 });
 onCleanup(()=>{version++;controller?.abort();});
 async function send(automatic:boolean):Promise<void>{
  if(busy()||attempted()||!props.active||!props.displayed||!visible())return;
  const current=decision();if(!current||current.handled||current.action==='none')return;
  if(automatic&&(!props.fullView||current.action!=='automatic'))return;
  const epoch=version,account=props.accountId,email=props.emailId;setAttempted(true);setBusy(true);
  try{await (props.api??api).send(account,email,automatic,controller!.signal);if(epoch===version)setDecision({...current,handled:true,action:'none'});}
  catch{if(epoch===version)setError(true);}
  finally{if(epoch===version)setBusy(false);}
 }
 createEffect(()=>{
  if(!props.active||!props.displayed||!props.fullView||!visible()||attempted()||dismissed()||decision()?.action!=='automatic')return;
  const timer=setTimeout(()=>void send(true),1500);onCleanup(()=>clearTimeout(timer));
 });
 return <Show when={props.active&&props.displayed&&!dismissed()&&decision()&&!decision()!.handled&&decision()!.action!=='none'}>
  <section aria-label={t('receipts-title')} class="reader__receipt"><p>{t('receipts-request',{sender:decision()!.requestedBy})}</p>
   <Show when={error()} fallback={<><button type="button" class="btn" disabled={busy()||attempted()} onClick={()=>void send(false)}>{t('receipts-send')}</button><button type="button" class="btn btn--ghost" disabled={busy()} onClick={()=>setDismissed(true)}>{t('receipts-not-now')}</button></>}><p role="status">{t('receipts-uncertain')}</p></Show>
  </section>
 </Show>;
}
