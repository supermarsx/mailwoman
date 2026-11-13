import {readSettingsSource} from './settingsTransferSource.ts';
import {workspaceMailboxes} from '../../api/workspaceMail.ts';
import {createResource,createMemo,createSignal,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {useApp} from '../../state/context.ts';
import {loadCatalog,t} from '../../i18n';
import {loadSettings,settings,settingsLoadStatus,writeSetting} from '../../api/settings.ts';
import {accountUiPreferences,accountUiPreferenceStatus,importAccountUiPreferences,type AccountUiPreferences} from '../../api/accountPreferences.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from '../../api/accountPreferenceLifecycle.ts';
import {appearanceSync,createAppearancePrefsApi} from '../../api/prefs.ts';
import {parseAppearancePrefs} from '../../theme/appearance.ts';
import {settingLabel} from '../Admin/Settings/labels.ts';
import {SettingsCard} from './layout/SettingsCard.tsx';
import type {CardSpec} from './layout/rows.tsx';
import {settingErrorMessage} from './useSetting.ts';
import {applyReviewed,buildBackup,parseBackup,reviewBackup,TRANSFER_GROUPS,TRANSFER_LIMIT,type SettingsBackup,type TransferGroup,type TransferReview} from './settingsTransferModel.ts';
import './settingsTransfer.css';

void loadCatalog('settings-transfer');
class TransferFeedback extends Error {}
export const SETTINGS_TRANSFER_CARD:CardSpec={id:'settings-transfer',section:'appearance',title:'st-title',help:'st-help',rows:[{id:'settings-transfer-export',label:'st-start-export',keywords:'backup download preferences settings'},{id:'settings-transfer-import',label:'st-start-import',keywords:'restore upload preferences settings'}]};
export function SettingsTransfer():JSX.Element {
 const app=useApp();
 const [owner,setOwner]=createSignal(preferenceAccount());
 const [flow,setFlow]=createSignal<'export'|'import'>();const [step,setStep]=createSignal(1);
 const [groups,setGroups]=createSignal<TransferGroup[]>([...TRANSFER_GROUPS]);const [folders,setFolders]=createSignal(false);
 const [backup,setBackup]=createSignal<SettingsBackup>();const [selected,setSelected]=createSignal<Set<string>>(new Set<string>());
 const [error,setError]=createSignal('');const [busy,setBusy]=createSignal(false);const [fileName,setFileName]=createSignal('');
 const [results,setResults]=createSignal<Record<string,string>>({});const [query,setQuery]=createSignal('');
 let stopped=false,read=0,operation=0;let input:HTMLInputElement|undefined;
 const source=()=>({accountId:owner()?.id??'',...(owner()?.storageAuthority?{storageAuthority:owner()!.storageAuthority!}:{}),origin:location.origin});
 const [sourceAccount,setSourceAccount]=createSignal('');
 const [sources]=createResource(()=>flow()==='import'?owner():null,async()=>workspaceMailboxes(app.linkedMailboxId()));
 const availableSources=()=>sources.state==='ready'?sources().filter(target=>target.storageAuthority!==owner()?.storageAuthority):[];
 let sourceRequest:AbortController|undefined;onCleanup(()=>sourceRequest?.abort());
 const appearance=()=>app.appearancePrefs();
 const rows=createMemo(()=>backup()?reviewBackup(backup()!,settings(),accountUiPreferences(),appearance(),source()):[]);
 const ready=()=>rows().filter(row=>row.status==='ready');
 const label=(row:TransferReview)=>row.kind==='setting'?settingLabel(row.key):row.kind==='appearance'?t('st-appearance'):t(`st-ui-${row.key}`);
 const visible=()=>rows().filter(row=>`${label(row)} ${row.key}`.toLocaleLowerCase().includes(query().trim().toLocaleLowerCase()));
 const summary=(value:unknown)=>value===undefined?t('st-unset'):typeof value==='string'?value:JSON.stringify(value);
 const total=()=>Object.keys(results()).length;
 const failed=()=>Object.values(results()).filter(value=>value!=='').length;
 function begin(mode:'export'|'import'):void {operation++;read++;stopped=false;setFlow(mode);setStep(1);setBackup(undefined);setError('');setFileName('');setResults({});setQuery('');setSelected(new Set<string>());}
 onMount(()=>{
  void loadCatalog('settings-transfer');void loadCatalog('admin-settings');void loadSettings();
  const stop=observePreferenceAccount(account=>{operation++;read++;stopped=true;sourceRequest?.abort();setOwner(account);setFlow(undefined);setBackup(undefined);setBusy(false);setError('');});onCleanup(stop);
 });
 onCleanup(()=>{operation++;read++;stopped=true;});
 function exportReview():void {
  if(!owner()||settingsLoadStatus()!=='ready')return;
  const value=buildBackup(settings(),accountUiPreferences(),appearance(),source(),groups(),folders());setBackup(value);setStep(2);
 }
 async function copySource():Promise<void>{
  const target=availableSources().find(target=>(target.linkId??'primary')===sourceAccount()),account=owner();if(!target||!account)return;
  sourceRequest?.abort();const request=new AbortController();sourceRequest=request;const sequence=++read;setBusy(true);setError('');
  try{const value=await readSettingsSource(target,request.signal);if(request.signal.aborted||sequence!==read||!preferenceAccountIsCurrent(account))return;
   setBackup(value);setFileName(target.label);setSelected(new Set(reviewBackup(value,settings(),accountUiPreferences(),appearance(),source()).filter(row=>row.status==='ready').map(row=>row.id)));setStep(2);
  }catch{if(!request.signal.aborted&&sequence===read&&preferenceAccountIsCurrent(account))setError(t('st-source-error'));}
  finally{if(sequence===read&&preferenceAccountIsCurrent(account))setBusy(false);}
 }
 async function choose(file:File|undefined):Promise<void> {
  if(!file)return;const sequence=++read;const account=owner();setError('');
  try {
   if(file.size>TRANSFER_LIMIT)throw new Error('st-file-large');
   const value=parseBackup(await file.text());
   if(sequence!==read||!account||!preferenceAccountIsCurrent(account))return;
   setBackup(value);setFileName(file.name);setSelected(new Set(reviewBackup(value,settings(),accountUiPreferences(),appearance(),source()).filter(row=>row.status==='ready').map(row=>row.id)));setStep(2);
  }catch(reason){if(sequence===read)setError(reason instanceof Error&&reason.message.startsWith('st-file-')?t(reason.message):t('st-file-invalid'));}
  finally{if(input)input.value='';}
 }
 function download():void {
  const account=owner(),value=backup();if(!account||!preferenceAccountIsCurrent(account)||!value)return;
  const blob=new Blob([JSON.stringify(value,null,2)+'\n'],{type:'application/json'});
  if(blob.size>TRANSFER_LIMIT){setError(t('st-file-large'));return;}
  const url=URL.createObjectURL(blob);try{const link=document.createElement('a');link.href=url;link.download=`mailwoman-settings-${new Date().toISOString().slice(0,10)}.json`;document.body.append(link);link.click();link.remove();setStep(3);}finally{setTimeout(()=>URL.revokeObjectURL(url),1000);}
 }
 async function apply():Promise<void> {
  const account=owner();if(!account||!preferenceAccountIsCurrent(account))return;
  const run=++operation;stopped=false;setBusy(true);setError('');setResults({});setStep(3);
  const current=()=>run===operation&&!stopped&&preferenceAccountIsCurrent(account);
  const targets=rows().map(row=>({...row})),choices=new Set(selected());
  await applyReviewed(targets,choices,current,async row=>{
   // Recheck policy and value immediately before each write, not only in the preview.
   const latest=reviewBackup({...backup()!,items:[row]},settings(),accountUiPreferences(),appearance(),source())[0]!;
   if(latest.status==='unchanged')return;
   if(latest.status!=='ready')throw new TransferFeedback(t(`st-status-${latest.status}`));
   if(row.kind==='setting')await writeSetting(row.key,row.value);
   else if(row.kind==='appearance') {
    await appearanceSync()?.flush();if(!current())return;
    const api=createAppearancePrefsApi();await api.save(parseAppearancePrefs(row.value));if(!current())return;
    const saved=await api.load();if(current()&&saved.appearance)app.setAppearancePrefs(saved.appearance);
   }else {
    if(!['synced','saving'].includes(accountUiPreferenceStatus()))throw new TransferFeedback(t('st-preferences-unavailable'));
    await importAccountUiPreferences({[row.key]:row.value} as AccountUiPreferences,account);
   }
  },(id,reason)=>{setResults(previous=>({...previous,[id]:reason===undefined?'':reason instanceof TransferFeedback?reason.message:settingErrorMessage(reason)}));if(reason===undefined)setSelected(previous=>new Set([...previous].filter(value=>value!==id)));});
  if(run===operation&&preferenceAccountIsCurrent(account)){setBusy(false);if(!stopped)void loadSettings({refresh:true});}
 }
 return <SettingsCard id="settings-transfer" title={t('st-title')} description={t('st-help')} testId="settings-transfer">
  <div class="st-buttons"><button id="settings-transfer-export" type="button" class="btn" disabled={!owner()||busy()} onClick={()=>begin('export')}>{t('st-start-export')}</button><button id="settings-transfer-import" type="button" class="btn" disabled={!owner()||busy()} onClick={()=>begin('import')}>{t('st-start-import')}</button></div>
  <Show when={flow()}><section class="st-wizard" aria-label={flow()==='export'?t('st-start-export'):t('st-start-import')} aria-busy={busy()}>
   <ol class="st-steps" aria-label={t('st-steps')}><For each={['choose','review','done']}>{(id,index)=><li aria-current={step()===index()+1?'step':undefined}>{t(`st-step-${id}`)}</li>}</For></ol>
   <p>{t('st-scope')}</p>
   <Show when={error()}><p role="alert">{error()}</p></Show>
   <Show when={step()===1}>
    <Show when={flow()==='export'} fallback={<><label class="st-file">{t('st-file-label')}<input ref={input} type="file" accept=".json,application/json" onChange={event=>void choose(event.currentTarget.files?.[0])}/></label><p>{t('st-import-help')}</p><Show when={availableSources().length}><label class="st-file">{t('st-source-account')}<select value={sourceAccount()} disabled={busy()} onChange={event=>setSourceAccount(event.currentTarget.value)}><option value="">{t('st-source-choose')}</option><For each={availableSources()}>{target=><option value={target.linkId??'primary'}>{target.label}</option>}</For></select></label><p>{t('st-source-help')}</p><button type="button" class="btn" disabled={busy()||!sourceAccount()} onClick={()=>void copySource()}>{t('st-source-review')}</button></Show><Show when={sources.error}><p role="status">{t('st-source-error')}</p></Show></>}>
     <fieldset class="st-categories"><legend>{t('st-categories')}</legend><For each={TRANSFER_GROUPS}>{group=><label><input type="checkbox" checked={groups().includes(group)} onChange={event=>setGroups(values=>event.currentTarget.checked?[...values,group]:values.filter(value=>value!==group))}/>{t(`st-group-${group}`)}</label>}</For></fieldset>
     <label class="st-choice"><input type="checkbox" checked={folders()} onChange={event=>setFolders(event.currentTarget.checked)}/>{t('st-folders')}</label><p>{t('st-folders-help')}</p>
     <button type="button" class="btn" disabled={!groups().length||settingsLoadStatus()!=='ready'} onClick={exportReview}>{t('st-review-export')}</button>
    </Show>
   </Show>
   <Show when={step()===2}>
    <Show when={flow()==='import'}><p>{fileName()}</p><p>{t('st-import-summary',{count:rows().length,ready:ready().length})}</p><label class="st-choice"><input type="checkbox" checked={ready().length>0&&ready().every(row=>selected().has(row.id))} onChange={event=>setSelected(event.currentTarget.checked?new Set(ready().map(row=>row.id)):new Set<string>())}/>{t('st-select-ready')}</label></Show>
    <label>{t('st-search')}<input type="search" class="st-search" value={query()} onInput={event=>setQuery(event.currentTarget.value)}/></label>
    <div class="st-review"><For each={visible()}>{row=><article class="st-review-row">
     <Show when={flow()==='import'}><input type="checkbox" aria-label={label(row)} checked={selected().has(row.id)} disabled={row.status!=='ready'} onChange={event=>setSelected(previous=>{const next=new Set(previous);if(event.currentTarget.checked)next.add(row.id);else next.delete(row.id);return next;})}/></Show>
     <div><strong>{label(row)}</strong><small>{t(`st-group-${row.group}`)}</small><Show when={flow()==='import'}><span class="st-status">{t(`st-status-${row.status}`)}</span></Show><details><summary>{t('st-values')}</summary><Show when={flow()==='import'}><p>{t('st-current')}</p><pre>{summary(row.current)}</pre></Show><p>{t('st-incoming')}</p><pre>{summary(row.value)}</pre></details></div>
    </article>}</For></div>
    <Show when={rows().length===0}><p>{t('st-empty')}</p></Show>
    <div class="st-buttons"><button type="button" class="btn" onClick={()=>setStep(1)}>{t('st-back')}</button><Show when={flow()==='export'} fallback={<button type="button" class="btn" disabled={!ready().some(row=>selected().has(row.id))} onClick={()=>void apply()}>{t('st-apply',{count:ready().filter(row=>selected().has(row.id)).length})}</button>}><button type="button" class="btn" onClick={download}>{t('st-download',{count:rows().length})}</button></Show></div>
   </Show>
   <Show when={step()===3}><Show when={flow()==='export'} fallback={<>
    <p role="status" aria-live="polite">{t(busy()?'st-applying':'st-results',{count:total()-failed(),failed:failed()})}</p>
    <Show when={busy()} fallback={<p>{t('st-kept')}</p>}><button type="button" class="btn" onClick={()=>{operation++;stopped=true;setBusy(false);}}>{t('st-stop')}</button></Show>
    <For each={Object.entries(results()).filter(([,message])=>message)}>{([id,message])=><p role="alert">{label(rows().find(row=>row.id===id)!)}: {message}</p>}</For>
    <Show when={!busy()&&failed()>0}><button type="button" class="btn" onClick={()=>setStep(2)}>{t('st-review-again')}</button></Show>
   </>}><p role="status">{t('st-export-done')}</p></Show></Show>
  </section></Show>
 </SettingsCard>;
}
