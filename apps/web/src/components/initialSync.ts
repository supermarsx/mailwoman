// First-sync progress for one mailbox view.
//
// After sign-in the server has the folder list but may still be indexing
// messages in the background (dcd88d3). An empty list during that window is
// not "No messages"; it is "not indexed yet". The engine reports both halves on
// `Mailbox/get`: the account's `mailwomanInitialSync` (ready | syncing |
// retrying | failed) and, when asked for it, each mailbox's `mailwomanSynced`
// (its messages have been indexed at least once). This hook polls those for
// the mailbox on screen until the account is ready, so a list can show
// skeleton rows while the folder is still being indexed, a retry when indexing
// failed, and its empty state only once the folder is known to be empty.
//
// There is no timer standing in for the answer: the hook keeps saying
// "pending" until the server says otherwise. Polling is scoped to the signed-in
// account and its authentication generation, so a response that arrives after
// an account switch or sign-out is dropped.

import {createEffect,createSignal,onCleanup,type Accessor} from 'solid-js';
import {useApp} from '../state/context.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {CAP_CORE,CAP_MAIL} from '../api/jmap-types.ts';
import {responseFor} from '../api/jmap.ts';
import {preferenceAccount,preferenceAccountIsCurrent,observePreferenceAccount} from '../api/accountPreferenceLifecycle.ts';

/** `unknown` until the first answer for the current account and mailbox. */
export type InitialSyncStatus='unknown'|'ready'|'syncing'|'retrying'|'failed';

export interface InitialSync {
  /** The account's indexing state. */
  status:Accessor<InitialSyncStatus>;
  /** The mailbox on screen has not been indexed yet and indexing is still running. */
  pending:Accessor<boolean>;
  /** Indexing gave up before this mailbox was indexed. */
  failed:Accessor<boolean>;
  /** Indexing is still running for the account (any folder). */
  running:Accessor<boolean>;
  /** Ask the server to try again, then resume polling. */
  retry():void;
}

const STATES=new Set(['ready','syncing','retrying','failed']);
const POLL_MS=2000,ERROR_POLL_MS=3000,MAX_FAILURES=3;

/**
 * Watch first-sync progress for `mailbox` (or for the whole account when it is
 * `null`, as for search results). `onIndexed` runs once when the mailbox, or
 * the account, finishes indexing after it was seen unfinished, so the caller
 * can reload what it shows.
 */
export function createInitialSync(mailbox:Accessor<string|null>,onIndexed:()=>void|Promise<void>):InitialSync {
  const app=useApp(),client=createConfiguredClient();
  const [generation,setGeneration]=createSignal(0),[attempt,setAttempt]=createSignal(0);
  const [status,setStatus]=createSignal<InitialSyncStatus>('unknown');
  const [indexed,setIndexed]=createSignal(false);
  let retryRequested=false;
  // Once an account has finished indexing it stays finished for this sign-in,
  // so later folder switches need no probe.
  let readyAccount:ReturnType<typeof preferenceAccount>=null;
  onCleanup(observePreferenceAccount(()=>setGeneration(value=>value+1)));
  createEffect(()=>{
    void generation();void attempt();
    const id=app.accountId(),account=preferenceAccount(),box=mailbox();
    setIndexed(false);
    // No signed-in mail account to ask about (or not this one): nothing pends.
    if(!id||!account||account.id!==id){setStatus('ready');return;}
    if(readyAccount===account){setStatus('ready');setIndexed(true);return;}
    setStatus('unknown');
    const abort=new AbortController();let timer:ReturnType<typeof setTimeout>|undefined;let failures=0,sawUnfinished=false;
    onCleanup(()=>{abort.abort();if(timer)clearTimeout(timer);});
    const current=()=>!abort.signal.aborted&&preferenceAccountIsCurrent(account)&&app.accountId()===id&&mailbox()===box;
    async function poll():Promise<void>{
      const retry=retryRequested;retryRequested=false;
      try{
        const args:Record<string,unknown>={accountId:id,ids:box?[box]:[],properties:['id','mailwomanSynced']};
        if(retry)args.mailwomanRetryInitialSync=true;
        const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Mailbox/get',args,'sync']]},{signal:abort.signal});
        if(!current())return;
        const result=responseFor<{accountId:string;mailwomanInitialSync?:string;list?:{id:string;mailwomanSynced?:boolean}[]}>(response,'sync');
        if(result.accountId!==id)throw new Error('Unexpected synchronization account');
        const state=result.mailwomanInitialSync??'ready';
        if(!STATES.has(state))throw new Error('Unknown synchronization state');
        // A server that predates per-mailbox progress omits the flag; then the
        // account state alone decides.
        const flag=box?result.list?.find(entry=>entry.id===box)?.mailwomanSynced:undefined;
        const done=state==='ready'||flag===true;
        if(state==='ready')readyAccount=account;
        failures=0;setStatus(state as InitialSyncStatus);setIndexed(done);
        if(done&&sawUnfinished){sawUnfinished=false;await onIndexed();}
        if(!done)sawUnfinished=true;
        if(state==='ready'||state==='failed')return;
        timer=setTimeout(()=>void poll(),POLL_MS);
      }catch{
        if(!current())return;
        // Unanswerable progress is not "indexed": after a few tries this reads
        // as a failure the user can retry, never as an empty folder.
        if(++failures>=MAX_FAILURES){setStatus('failed');return;}
        timer=setTimeout(()=>void poll(),ERROR_POLL_MS);
      }
    }
    void poll();
  });
  const pending=():boolean=>!indexed()&&(status()==='unknown'||status()==='syncing'||status()==='retrying');
  return {
    status,
    pending,
    failed:()=>!indexed()&&status()==='failed',
    running:()=>status()==='syncing'||status()==='retrying',
    retry(){retryRequested=true;setAttempt(value=>value+1);},
  };
}
