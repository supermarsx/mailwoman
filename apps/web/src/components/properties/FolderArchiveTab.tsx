import {createResource,createSignal,createEffect,onCleanup,For,Show,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {useApp} from '../../state/context.ts';
import {loadRetention,saveSettings,type FolderDefault} from '../../api/retention.ts';
import {toast} from '../toast/index.ts';
import {preferenceAccount,preferenceAccountIsCurrent,observePreferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {AsyncError,AsyncPending} from '../AsyncState.tsx';
import * as css from './folderDialog.css.ts';
void loadCatalog('retention');
export function isArchiveDefault(rule:FolderDefault,folder:string):boolean{return rule.body.scope.folders.length===1&&rule.body.scope.folders[0]===folder&&['archive','move'].includes(rule.body.action.kind);}
export function FolderArchiveTab(props:{id:string;name:string}):JSX.Element {
 const app=useApp();const [owner,setOwner]=createSignal(preferenceAccount());
 onCleanup(observePreferenceAccount(next=>{if(!owner())setOwner(next);}));
 const [overview,{refetch}]=createResource(owner,async()=>loadRetention());
 const current=()=>{const account=owner();return !!account&&preferenceAccountIsCurrent(account);};
 const defaultTarget=()=>app.mailboxes().find(box=>box.role==='archive')?.id===props.id?(app.mailboxes().find(box=>box.id!==props.id&&box.mailwomanSearchQuery===undefined)?.id??''):'role:archive';
 const [runRules,setRunRules]=createSignal(false);
 const [enabled,setEnabled]=createSignal(false),[days,setDays]=createSignal(60),[target,setTarget]=createSignal('role:archive'),[flagged,setFlagged]=createSignal(true),[unread,setUnread]=createSignal(true),[busy,setBusy]=createSignal(false);
 createEffect(()=>{setRunRules(overview()?.settings.enabled??false);const rule=overview()?.folderDefaults.find(rule=>isArchiveDefault(rule,props.id));setEnabled(rule?.enabled??false);setDays(rule?.body.action.days??60);setTarget(rule?.body.action.target??defaultTarget());setFlagged(rule?.body.exclude.flagged??true);setUnread(rule?.body.exclude.unread??true);});
 async function save():Promise<void>{
  if(busy()||!current()||!overview()?.policy.enabled)return;
  setBusy(true);try{
   const latest=await loadRetention();if(!current()||(enabled()&&!latest.settings.enabled&&!runRules()))return;
   const old=latest.folderDefaults.find(rule=>isArchiveDefault(rule,props.id));
   const defaults=latest.folderDefaults.filter(rule=>enabled()?!(rule.body.scope.folders.length===1&&rule.body.scope.folders[0]===props.id):!isArchiveDefault(rule,props.id)).map(rule=>({id:rule.id,name:rule.name,enabled:rule.enabled,folder:rule.body.scope.folders[0]!,action:rule.body.action,exclude:rule.body.exclude}));
   if(enabled())defaults.push({id:old?.id??crypto.randomUUID(),name:t('props-archive-rule',{folder:props.name}),enabled:true,folder:props.id,action:{kind:target()==='role:archive'?'archive':'move',days:days(),target:target()==='role:archive'?null:target()},exclude:{flagged:flagged(),unread:unread(),tags:['keep']}});
   await saveSettings({folderDefaults:defaults,...(enabled()&&runRules()?{enabled:true}:{})});if(!current())return;await refetch();toast.success(t('props-archive-saved'));
  }catch{if(current())toast.error(t('props-archive-error'));}finally{setBusy(false);}
 }
 return <section class={css.card}><h3 class={css.cardTitle}>{t('props-tab-archive')}</h3><p class={css.cardHelp}>{t('props-archive-help')}</p>
  <Show when={!overview.loading} fallback={<AsyncPending message={t('mail-loading')}/>}><Show when={!overview.error} fallback={<AsyncError message={t('props-archive-error')} onRetry={()=>void refetch()}/>}>
   <Show when={!overview()?.policy.enabled}><p class={css.cardHelp}>{t('props-archive-policy')}</p></Show>
   <form class={css.group} onSubmit={e=>{e.preventDefault();void save();}}><fieldset class={css.group} disabled={busy()||!current()||!overview()?.policy.enabled}>
    <Show when={!overview()?.settings.enabled}><label class={css.row}><input type="checkbox" required={enabled()} checked={runRules()} onChange={e=>setRunRules(e.currentTarget.checked)}/>{t('retention-enabled')}</label></Show>
    <label class={css.row}><input type="checkbox" checked={enabled()} onChange={e=>setEnabled(e.currentTarget.checked)}/>{t('props-archive-enabled')}</label>
    <label class={css.group}>{t('props-archive-days')}<input class={css.textField} type="number" required min="1" max="3650" value={days()} onInput={e=>setDays(e.currentTarget.valueAsNumber)}/></label>
    <label class={css.group}>{t('props-archive-target')}<select required class={css.textField} value={target()} onChange={e=>setTarget(e.currentTarget.value)}><option value="role:archive" disabled={app.mailboxes().find(box=>box.role==='archive')?.id===props.id}>{t('retention-action-archive')}</option><For each={app.mailboxes().filter(box=>box.id!==props.id&&box.mailwomanSearchQuery===undefined)}>{box=><option value={box.id}>{box.name}</option>}</For></select></label>
    <label class={css.row}><input type="checkbox" checked={flagged()} onChange={e=>setFlagged(e.currentTarget.checked)}/>{t('props-archive-keep-flagged')}</label>
    <label class={css.row}><input type="checkbox" checked={unread()} onChange={e=>setUnread(e.currentTarget.checked)}/>{t('props-archive-keep-unread')}</label>
    <button type="submit" class={`btn btn--primary ${css.startAction}`}>{t('props-archive-save')}</button>
   </fieldset></form>
  </Show></Show>
 </section>;
}
