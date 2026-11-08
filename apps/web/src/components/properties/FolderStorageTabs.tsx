// The Size and storage and Offline cache tabs of the folder dialog.
import {For,Show,createResource,createSignal,type JSX} from 'solid-js';
import {useApp} from '../../state/context.ts';
import {createConfiguredClient} from '../../api/transport.ts';
import {loadQuotas,storageQuota} from '../../api/quota.ts';
import type {Client} from '../../api/client.ts';
import type {Mailbox} from '../../api/jmap-types.ts';
import {isolate,t} from '../../i18n';
import {Icon} from '../Icon.tsx';
import {ConfirmDialog} from '../../screens/Admin/shell/ConfirmDialog.tsx';
import {toast} from '../toast/toast.ts';
import {resourceValue} from '../resourceValue.ts';
import {formatBytes,forgetFolderStats} from './folderStats.ts';
import {offlineClearEnabled} from './policy.ts';
import {dateTimeText} from './folderDialogUtil.ts';
import type {StatsState} from './useFolderStats.ts';
import * as css from './folderDialog.css.ts';

function Pair(props:{label:string;children:JSX.Element}):JSX.Element {return <><dt>{props.label}</dt><dd>{props.children}</dd></>;}

/** A labelled usage bar: `value` of `total`, drawn as a share (0..1). */
export function UsageBar(props:{label:string;share:number;text:string;warn?:boolean;testId?:string}):JSX.Element {
 const pct=()=>Math.max(0,Math.min(100,Math.round(props.share*100)));
 return <div class={css.meter} data-testid={props.testId}>
  <div class={css.row} style={{'justify-content':'space-between'}}><span>{props.label}</span><span class={css.note}>{props.text}</span></div>
  <div class={css.track} role="meter" aria-label={props.label} aria-valuemin={0} aria-valuemax={100} aria-valuenow={pct()} aria-valuetext={props.text}><div class={`${css.fill}${props.warn?` ${css.fillWarn}`:''}`} style={{'inline-size':`${pct()}%`}}/></div>
 </div>;
}

/** Why the numbers are missing, with the way out: retry, or ask for them when the policy leaves it to the user. */
function Unavailable(props:{stats:StatsState}):JSX.Element {
 const s=()=>props.stats;
 return <>
  <Show when={s().failed()}><p class={css.problem} role="alert">{t('props-unavailable')}<button type="button" class="btn btn--ghost" data-testid="props-size-retry" onClick={()=>s().retry()}>{t('props-tile-retry')}</button></p></Show>
  <Show when={s().loading()}><p class={css.note} role="status"><Icon name="sync"/> {t('props-computing')}</p></Show>
  <Show when={s().idle()}><p class={css.note}>{t('props-size-off')}</p><div class={css.row}><button type="button" class="btn btn--ghost" onClick={()=>s().compute()}>{t('props-size-compute')}</button></div></Show>
 </>;
}

export function Size(props:{box:Mailbox;stats:StatsState;onOpenMessage?:((id:string)=>void)|undefined}):JSX.Element {
 const app=useApp();
 const s=()=>props.stats,st=()=>s().stats();
 const [quota]=createResource(()=>st()?app.accountId():null,async accountId=>{
  if(!accountId)return null;
  try{return storageQuota(await loadQuotas(createConfiguredClient() as unknown as Client,accountId));}catch{return null;}
 });
 const q=()=>resourceValue(quota);
 const biggest=()=>Math.max(1,...(st()?.largest.map(item=>item.size)??[1]));
 return <div data-testid="props-size" aria-busy={s().loading()} style={{display:'contents'}}>
  <section class={css.card} aria-labelledby="props-size-title">
   <h3 id="props-size-title" class={css.cardTitle}>{t('props-card-usage')}</h3>
   <Show when={st()}>{stats=><>
    <Show when={q()} fallback={<p class={css.note}>{stats().approximate?t('props-approx-size',{size:formatBytes(stats().sizeBytes)}):formatBytes(stats().sizeBytes)}</p>}>{quota=>
     <UsageBar testId="props-quota-bar" label={t('props-quota-share')} share={stats().sizeBytes/Math.max(1,quota().hardLimit)} warn={quota().used/quota().hardLimit>=0.9} text={t('props-quota-text',{folder:formatBytes(stats().sizeBytes),used:formatBytes(quota().used),limit:formatBytes(quota().hardLimit)})}/>}</Show>
    <dl class={css.grid}>
     <Pair label={t('props-total')}>{stats().totalEmails}</Pair>
     <Pair label={t('props-unread')}>{stats().unreadEmails}</Pair>
     <Pair label={t('props-flagged')}>{stats().flaggedEmails}</Pair>
     <Pair label={t('props-drafts')}>{stats().draftEmails}</Pair>
     <Pair label={t('props-size-server')}>{stats().approximate?t('props-approx-size',{size:formatBytes(stats().sizeBytes)}):formatBytes(stats().sizeBytes)}</Pair>
     <Pair label={t('props-oldest')}>{dateTimeText(stats().oldest)}</Pair>
     <Pair label={t('props-newest')}>{dateTimeText(stats().newest)}</Pair>
    </dl>
    <Show when={stats().approximate}><p class={css.note} role="note">{t('props-approx-note',{cached:stats().cachedEmails,total:stats().totalEmails})}</p></Show>
   </>}</Show>
   <Show when={!st()}><dl class={css.grid}><Pair label={t('props-total')}>{props.box.totalEmails}</Pair><Pair label={t('props-unread')}>{props.box.unreadEmails}</Pair></dl><Unavailable stats={s()}/></Show>
  </section>
  <Show when={st()?.largest.length}>
   <section class={css.card} aria-labelledby="props-largest-title">
    <h3 id="props-largest-title" class={css.cardTitle}>{t('props-largest')}</h3>
    <ol class={css.largest} data-testid="props-largest">
     <For each={st()!.largest}>{(item,index)=><li class={css.largestRow}>
      <span class={css.note}>{index()+1}</span>
      <div class={css.meter}><div class={css.track} aria-hidden="true"><div class={css.fill} style={{'inline-size':`${Math.round(item.size/biggest()*100)}%`}}/></div><span class={css.note}>{formatBytes(item.size)}</span></div>
      <Show when={props.onOpenMessage}><button type="button" class="btn btn--ghost" aria-label={t('props-largest-open',{size:formatBytes(item.size)})} onClick={()=>props.onOpenMessage!(item.id)}><Icon name="mail"/>{t('props-largest-open-short')}</button></Show>
     </li>}</For>
    </ol>
   </section>
  </Show>
 </div>;
}

export function Offline(props:{box:Mailbox;stats:StatsState;onRefresh?:(()=>void)|undefined}):JSX.Element {
 const app=useApp();
 const [confirming,setConfirming]=createSignal(false);
 const cache=()=>app.mailCacheStats();
 const st=()=>props.stats.stats();
 function clear():void {
  try{app.clearFolderCache(props.box.id);forgetFolderStats(props.box.id);setConfirming(false);toast.success(t('props-offline-cleared'));}
  catch{setConfirming(false);toast.error(t('props-offline-clear-failed'));}
 }
 function prefetch():void {try{props.onRefresh!();toast.success(t('props-offline-prefetching'));}catch{toast.error(t('props-offline-prefetch-failed'));}}
 return <div data-testid="props-offline" style={{display:'contents'}}>
  <section class={css.card} aria-labelledby="props-off-title">
   <h3 id="props-off-title" class={css.cardTitle}>{t('props-card-offline')}</h3>
   <p class={css.cardHelp}>{t('props-offline-help')}</p>
   <Show when={st()} fallback={<Show when={props.stats.failed()}><p class={css.problem} role="alert">{t('props-unavailable')}<button type="button" class={css.linkButton} onClick={()=>props.stats.retry()}>{t('props-tile-retry')}</button></p></Show>}>{stats=>
    <UsageBar testId="props-offline-bar" label={t('props-offline-stored')} share={stats().cachedEmails>0?stats().bodiesCached/stats().cachedEmails:0} text={t('props-offline-stored-value',{count:stats().bodiesCached,total:stats().cachedEmails})}/>}</Show>
   <dl class={css.grid}>
    <Pair label={t('props-offline-session')}>{t('props-offline-session-value',{entries:cache().entries,bytes:formatBytes(cache().bytes)})}</Pair>
   </dl>
   <div class={css.row}>
    <Show when={offlineClearEnabled()}><button type="button" class="btn btn--danger" data-testid="props-clear-cache" onClick={()=>setConfirming(true)}><Icon name="delete"/>{t('props-offline-clear')}</button></Show>
    <Show when={props.onRefresh}><button type="button" class="btn btn--ghost" data-testid="props-prefetch" onClick={prefetch}><Icon name="sync"/>{t('props-offline-prefetch')}</button></Show>
   </div>
  </section>
  <ConfirmDialog open={confirming()} title={t('props-offline-clear-title',{name:isolate(props.box.name)})} body={t('props-offline-clear-body')} confirmLabel={t('props-offline-clear')} onCancel={()=>setConfirming(false)} onConfirm={clear} testId="props-clear-confirm"/>
 </div>;
}
