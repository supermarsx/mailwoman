import type {Client} from '../api/client.ts';
import {runEmpty,type EmptyPreview,type EmptyResult} from '../api/emptyFolder.ts';
import {observePreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {isolate,t} from '../i18n';
import {toast} from './toast/toast.ts';

interface EmptyJobHooks {current:()=>boolean;refresh:()=>Promise<unknown>;clearSelection:()=>void}
let active:{current:()=>boolean}|null=null;
let sequence=0;

/** Whether a folder is being emptied right now (one at a time per window). */
export function emptyJobRunning():boolean {return active?.current()===true;}

/** Empty `preview`'s folder in the background. The job outlives its dialog,
 *  never its authenticated account: an account change aborts it. Progress is a
 *  persistent toast with Stop (honoured between batches); the result replaces it. */
export function startEmptyJob(client:Pick<Client,'jmap'>,preview:EmptyPreview,hooks:EmptyJobHooks):boolean {
 if(!hooks.current())return false;
 if(active?.current()){toast.info(t('empty-folder-running'));return false;}
 const job={current:hooks.current};active=job;
 const controller=new AbortController(),id=`empty-folder:${++sequence}`;
 const name=isolate(preview.name),total=preview.total;
 let stopping=false,finished=false,hidden=false;
 let latest:EmptyResult={processed:0,remaining:total,refused:0,uncertain:false,stopped:false};
 const progress=(value:EmptyResult):void=>{
  latest=value;if(!hooks.current()||hidden||finished)return;
  const message=stopping?t('empty-folder-stopping',{done:value.processed,total}):t(preview.outcome==='trash'?'empty-folder-progress-trash':'empty-folder-progress-destroy',{name,done:value.processed,total});
  toast.info(message,{
   id,persistent:true,duration:Infinity,testId:'empty-folder-progress',
   ...(stopping?{}:{action:{label:t('empty-folder-stop'),run:()=>{stopping=true;progress(latest);}}}),
   onClose:reason=>{if(!finished&&reason!=='action'){stopping=true;hidden=true;}},
  });
 };
 const stopObserving=observePreferenceAccount(()=>{if(!hooks.current()){controller.abort();toast.dismiss(id);}});
 progress(latest);
 void (async()=>{
  try{
   const result=await runEmpty(client,preview,{current:hooks.current,signal:controller.signal,stopRequested:()=>stopping},progress);
   if(!hooks.current())return;
   hooks.clearSelection();
   await hooks.refresh();
   if(!hooks.current())return;
   finished=true;
   const counts={name,done:result.processed,total,remaining:result.remaining,refused:result.refused};
   if(result.uncertain)toast.error(t('empty-folder-failed',counts),{id,duration:Infinity,testId:'empty-folder-result'});
   else if(result.refused>0)toast.warning(t('empty-folder-refused',counts),{id,duration:Infinity,testId:'empty-folder-result'});
   else if(result.stopped)toast.warning(t('empty-folder-stopped',counts),{id,duration:Infinity,testId:'empty-folder-result'});
   else toast.success(t(preview.outcome==='trash'?'empty-folder-done-trash':'empty-folder-done-destroy',counts),{id,testId:'empty-folder-result',duration:8000});
  }catch{
   if(hooks.current()){finished=true;toast.error(t('empty-folder-failed',{name,done:latest.processed,total}),{id,duration:Infinity,testId:'empty-folder-result'});}
  }finally{
   stopObserving();if(active===job)active=null;
   if(!hooks.current())toast.dismiss(id);
  }
 })();
 return true;
}
