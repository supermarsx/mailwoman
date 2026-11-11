import {createEffect,createSignal,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {loadReceiptDecision,saveReceiptSchedule,type ReceiptDecision,type ReceiptScheduleScope} from '../api/receipts.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {CAP_CORE,CAP_MAIL} from '../api/jmap-types.ts';
import {formatDate,loadCatalog,t} from '../i18n';

export interface ReadReceiptApi {
 schedule?(email:string,scope:ReceiptScheduleScope,at:number|null,signal:AbortSignal):Promise<void>;
 load(account:string,email:string,signal:AbortSignal):Promise<ReceiptDecision|null>;
 send(account:string,email:string,automatic:boolean,signal:AbortSignal):Promise<void>;
}
const client=createConfiguredClient();
const api:ReadReceiptApi={load:loadReceiptDecision,schedule:saveReceiptSchedule,async send(account,email,automatic,signal){
 const result=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['MDN/send',{accountId:account,emailId:email,automatic},'receipt']]},{signal});
 const response=result.methodResponses.find(row=>row[2]==='receipt');
 if(response?.[0]!=='MDN/send'||response[1]['sent']!==true)throw new Error('Receipt outcome not confirmed');
}};

/** Preview/background reads never send automatically. A displayed active full view may. */
export function ReadReceipt(props:{accountId:string;emailId:string;active:boolean;displayed:boolean;fullView:boolean;api?:ReadReceiptApi}):JSX.Element {
 const [decision,setDecision]=createSignal<ReceiptDecision|null>(null);const [busy,setBusy]=createSignal(false);
 const [attempted,setAttempted]=createSignal(false);const [error,setError]=createSignal(false);const [dismissed,setDismissed]=createSignal(false);
 const [scheduleOpen,setScheduleOpen]=createSignal(false);
 const [scheduleTime,setScheduleTime]=createSignal('');
 const [scheduleScope,setScheduleScope]=createSignal<'message'|'sender'>('message');
 const [scheduleError,setScheduleError]=createSignal(false);
 const [enrolAttempted,setEnrolAttempted]=createSignal(false);
 const scheduled=()=>decision()?.scheduledAt??null;
 const senderAt=()=>decision()?.senderScheduleAt??null;
 const canSchedule=()=>decision()?.scheduleAllowed===true&&Boolean((props.api??api).schedule);
 const canRespond=()=>props.active&&props.displayed&&visible();
 const [visible,setVisible]=createSignal(document.visibilityState==='visible');
 let version=0;let controller:AbortController|undefined;let identity='';
  onMount(()=>{void loadCatalog('receipts');const update=():void=>{setVisible(document.visibilityState==='visible');};document.addEventListener('visibilitychange',update);onCleanup(()=>document.removeEventListener('visibilitychange',update));});
 createEffect(()=>{
  const account=props.accountId,email=props.emailId,displayed=props.displayed;const epoch=++version;
  controller?.abort();controller=new AbortController();setDecision(null);setBusy(false);
  const key=JSON.stringify([account,email]);if(key!==identity){identity=key;setAttempted(false);setError(false);setDismissed(false);setScheduleOpen(false);setScheduleTime('');setScheduleScope('message');setScheduleError(false);setEnrolAttempted(false);}
  if(!displayed)return;
  void (props.api??api).load(account,email,controller.signal).then(value=>{if(epoch===version)setDecision(value);}).catch(()=>{/* No automatic fallback when eligibility is unavailable. */});
 });
 onCleanup(()=>{version++;controller?.abort();});
 async function send(automatic:boolean):Promise<void>{
  if(busy()||attempted()||scheduled()!==null||!canRespond())return;
  const current=decision();if(!current||current.handled||current.action==='none')return;
  if(automatic&&(!props.fullView||current.action!=='automatic'))return;
  const epoch=version,account=props.accountId,email=props.emailId;setAttempted(true);setBusy(true);
  // Its own signal, never aborted: the decision effect aborts `controller` whenever it
  // reloads, and cutting a sent receipt off mid-request loses its outcome (t93).
  try{await (props.api??api).send(account,email,automatic,new AbortController().signal);if(epoch===version)setDecision({...current,handled:true,action:'none'});}
  catch{if(epoch===version)setError(true);}
  finally{if(epoch===version)setBusy(false);}
 }
 async function schedule(scope:ReceiptScheduleScope,at:number|null):Promise<void>{
  const save=(props.api??api).schedule;
  if(!save||busy()||attempted()||!canRespond())return;
  const epoch=version,account=props.accountId,email=props.emailId,signal=controller!.signal;
  setBusy(true);setScheduleError(false);
  try{
   await save(email,scope,at,signal);
   const updated=await (props.api??api).load(account,email,signal);
   if(epoch===version){setDecision(updated);setScheduleOpen(false);}
  }catch{if(epoch===version)setScheduleError(true);}
  finally{if(epoch===version)setBusy(false);}
 }
 createEffect(()=>{
  if(!canRespond()||!canSchedule()||!props.fullView||dismissed()||busy()||scheduled()!==null||senderAt()===null||enrolAttempted())return;
  setEnrolAttempted(true);
  void schedule('enrol',null);
 });
 createEffect(()=>{
  if(!props.active||!props.displayed||!props.fullView||!visible()||attempted()||dismissed()||enrolAttempted()||scheduleOpen()||scheduled()!==null||senderAt()!==null||decision()?.action!=='automatic')return;
  const timer=setTimeout(()=>void send(true),1500);onCleanup(()=>clearTimeout(timer));
 });
 return <Show when={props.active&&props.displayed&&!dismissed()&&decision()&&(!decision()!.handled&&decision()!.action!=='none'||scheduled()!==null)}>
  <section aria-label={t('receipts-title')} class="reader__receipt"><p>{t('receipts-request',{sender:decision()!.requestedBy})}</p>
   <Show when={scheduled()!==null} fallback={<>
   <Show when={error()} fallback={<><button type="button" class="btn" disabled={busy()||attempted()} onClick={()=>void send(false)}>{t('receipts-send')}</button><button type="button" class="btn btn--ghost" disabled={busy()} onClick={()=>setDismissed(true)}>{t('receipts-not-now')}</button></>}><p role="status">{t('receipts-uncertain')}</p></Show>
   </>}>
    <p role="status">{t('receipts-scheduled',{time:formatDate(new Date(scheduled()!*1000),{dateStyle:'medium',timeStyle:'short'})})}</p>
    <button type="button" class="btn btn--ghost" disabled={busy()} onClick={()=>{setEnrolAttempted(true);void schedule('message',null);}}>{t('receipts-cancel-schedule')}</button>
   </Show>
   <Show when={canSchedule()&&scheduled()===null&&!error()}>
    <button type="button" class="btn btn--ghost" disabled={busy()} aria-expanded={scheduleOpen()} onClick={()=>setScheduleOpen(value=>!value)}>{t('receipts-schedule')}</button>
    <Show when={scheduleOpen()}>
     <form onSubmit={event=>{event.preventDefault();const at=Math.floor(new Date(scheduleTime()).getTime()/1000);if(!Number.isFinite(at)||at<=Date.now()/1000){setScheduleError(true);return;}void schedule(scheduleScope(),at);}}>
      <label>{t('receipts-schedule-time')} <input required type="datetime-local" value={scheduleTime()} disabled={busy()} onInput={event=>setScheduleTime(event.currentTarget.value)}/></label>
      <Show when={decision()?.scheduleSender}><label>{t('receipts-schedule-scope')} <select disabled={busy()} value={scheduleScope()} onChange={event=>setScheduleScope(event.currentTarget.value==='sender'?'sender':'message')}><option value="message">{t('receipts-schedule-message')}</option><option value="sender">{t('receipts-schedule-sender')}</option></select></label></Show>
      <Show when={scheduleScope()==='sender'}><p>{t('receipts-schedule-sender-help')}</p></Show>
      <button type="submit" class="btn" disabled={busy()}>{t('receipts-save-schedule')}</button>
     </form>
    </Show>
   </Show>
   <Show when={senderAt()!==null}><button type="button" class="btn btn--ghost" disabled={busy()} onClick={()=>void schedule('sender',null)}>{t('receipts-stop-sender-schedule')}</button></Show>
   <Show when={scheduleError()}><p role="alert">{t('receipts-schedule-error')}</p></Show>
  </section>
 </Show>;
}
