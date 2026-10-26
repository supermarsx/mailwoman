import type {Client} from '../api/client.ts';
import {emptyTrash,type TrashResult,type TrashSnapshot} from '../api/emptyTrash.ts';
import {observePreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {toast} from './toast/toast.ts';
import {t} from '../i18n';

interface TrashJobHooks {current:()=>boolean;refresh:()=>Promise<unknown>;clearSelection:()=>void}
let active:{current:()=>boolean}|null=null;
let sequence=0;
const counts=(value:TrashResult)=>({deleted:value.deleted,skipped:value.skipped,remaining:value.remaining,uncertain:value.uncertain});
/** The job outlives its confirmation dialog, never its authenticated account. */
export function startTrashJob(client:Pick<Client,'jmap'>,snapshot:TrashSnapshot,hooks:TrashJobHooks):boolean {
 if(!hooks.current())return false;
 if(active?.current()){toast.info(t('mail-trash-already-running'));return false;}
 const job={current:hooks.current};active=job;
 const controller=new AbortController(),id=`empty-trash:${++sequence}`;
 let stopping=false,finished=false,hidden=false;
 let latest:TrashResult={deleted:0,skipped:0,remaining:snapshot.ids.length,uncertain:0,stopped:false};
 const progress=(value:TrashResult)=>{
  latest=value;if(!hooks.current()||hidden)return;
  toast.info(t(stopping?'mail-trash-cancelling':'mail-trash-progress',counts(value)),{
   id,persistent:true,duration:Infinity,testId:'empty-trash-progress',
   ...(stopping?{}:{action:{label:t('mail-trash-stop'),run:()=>{stopping=true;progress(latest);}}}),
   onClose:reason=>{if(!finished&&reason!=='action'){stopping=true;hidden=true;}},
  });
 };
 const stopObserving=observePreferenceAccount(()=>{if(!hooks.current()){controller.abort();toast.dismiss(id);}});
 progress(latest);
 void (async()=>{
  try {
   const result=await emptyTrash(client,snapshot,{current:hooks.current,signal:controller.signal,stopRequested:()=>stopping},progress);
   if(!hooks.current())return;
   hooks.clearSelection();
   await hooks.refresh();
   if(!hooks.current())return;
   finished=true;
   toast.show(result.stopped?'warning':'success',t(result.stopped?'mail-trash-stopped':'mail-trash-progress',counts(result)),{id,testId:'empty-trash-result',duration:result.stopped?Infinity:8000});
  } catch {
   if(hooks.current()){finished=true;toast.error(t('mail-trash-stopped',counts(latest)),{id,duration:Infinity,testId:'empty-trash-result'});}
  } finally {
   stopObserving();if(active===job)active=null;
   if(!hooks.current())toast.dismiss(id);
  }
 })();
 return true;
}
