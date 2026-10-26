import {createEffect,createSignal,onCleanup,Show,type JSX} from 'solid-js';
import {useApp} from '../state/context.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {CAP_CORE,CAP_MAIL} from '../api/jmap-types.ts';
import {responseFor} from '../api/jmap.ts';
import {preferenceAccount,preferenceAccountIsCurrent,observePreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {t} from '../i18n';
import './initialSyncNotice.css.ts';

export function InitialSyncNotice():JSX.Element{
 const app=useApp(),client=createConfiguredClient();const [generation,setGeneration]=createSignal(0),[state,setState]=createSignal('ready');
 onCleanup(observePreferenceAccount(()=>setGeneration(value=>value+1)));
 createEffect(()=>{
  void generation();const id=app.accountId(),account=preferenceAccount();setState('ready');if(!id||!account||account.id!==id)return;
  const abort=new AbortController();let timer:ReturnType<typeof setTimeout>|undefined;let failures=0;let wasSyncing=false;
  onCleanup(()=>{abort.abort();if(timer)clearTimeout(timer);});
  const current=()=>!abort.signal.aborted&&preferenceAccountIsCurrent(account)&&app.accountId()===id;
  async function poll():Promise<void>{
   try{const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Mailbox/get',{accountId:id,ids:[],properties:['id']},'sync']]},{signal:abort.signal});if(!current())return;
    const result=responseFor<{accountId:string;mailwomanInitialSync?:string}>(response,'sync');if(result.accountId!==id)throw new Error('Unexpected synchronization account');const status=result.mailwomanInitialSync??'ready';if(!['ready','syncing','retrying','failed'].includes(status))throw new Error('Unknown synchronization state');setState(status);failures=0;
    if(status==='ready'){if(wasSyncing)await app.refreshCurrentMailbox();return;}if(status==='failed')return;wasSyncing=true;timer=setTimeout(()=>void poll(),2000);
   }catch{if(!current())return;if(++failures>=3){if(wasSyncing)setState('failed');return;}timer=setTimeout(()=>void poll(),3000);}
  }
  void poll();
 });
 return <Show when={state()!=='ready'}><p class="mail-sync-notice" role="status" aria-live="polite">{t(state()==='failed'?'mail-initial-sync-failed':state()==='retrying'?'mail-initial-sync-retrying':'mail-initial-sync-running')}</p></Show>;
}
