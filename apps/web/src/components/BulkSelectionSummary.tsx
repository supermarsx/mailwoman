import {createSignal,For,Show,onCleanup} from 'solid-js';
import {useApp} from '../state/context.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {t} from '../i18n/index.ts';
import {Icon} from './Icon.tsx';
import {openBulkProperties} from './properties/host.tsx';
import {messageDialogEnabled} from './properties/policy.ts';
export function BulkSelectionSummary(props:{onCancel:()=>void}) {
 const app=useApp();const ids=()=>[...app.selectedIds()];
 const [busy,setBusy]=createSignal(false);let cancelled=false;onCleanup(()=>{cancelled=true;});
 async function selectFolder():Promise<void>{
  if(busy())return;
  const owner=preferenceAccount(),account=app.accountId(),folder=app.selectedMailboxId(),search=app.searchActive(),query=app.search?.()??'';
  const current=()=>!cancelled&&account===app.accountId()&&folder===app.selectedMailboxId()&&search===app.searchActive()&&query===(app.search?.()??'')&&(!owner||preferenceAccountIsCurrent(owner));
  setBusy(true);
  try{
   while(current()&&app.hasMore()){
    const before=app.messages().length;
    await app.loadMore();
    if(!current())return;
    if(app.messages().length===before&&app.hasMore()){app.showToast('error',t('mail-bulk-selection-load-failed'));return;}
   }
   if(current())app.selectAllInView();
  }catch{if(current())app.showToast('error',t('mail-bulk-selection-load-failed'));}
  finally{if(!cancelled)setBusy(false);}
 }

 return <div class="bulk-selection-summary" data-testid="bulk-selection-summary">
  <h2>{t('mail-bulk-selected',{count:ids().length})}</h2>
  <button class="btn btn--ghost" disabled={busy()||app.loadingMore()} onClick={()=>void selectFolder()}>{t('mail-bulk-select-folder',{folder:app.mailboxes().find(box=>box.id===app.selectedMailboxId())?.name??''})}</button>
  <fieldset class="bulk-selection-actions" disabled={busy()}>
   <button class="btn" onClick={()=>void app.trashMessage(ids())}><Icon name="delete"/>{t('mail-delete')}</button>
   <button class="btn" onClick={()=>void app.flagMessage(ids(),true)}><Icon name="flag"/>{t('mail-flag-message')}</button>
   <button class="btn" onClick={()=>void app.markRead(ids(),false)}><Icon name="mail"/>{t('mail-mark-unread')}</button>
   <button class="btn" onClick={()=>void app.markRead(ids(),true)}><Icon name="mailRead"/>{t('mail-mark-read')}</button>
   <select aria-label={t('selection-bar-move')} value="" onChange={event=>{if(event.currentTarget.value)void app.moveMessage(ids(),event.currentTarget.value);event.currentTarget.value='';}}>
    <option value="">{t('selection-bar-move')}</option><For each={app.mailboxes().filter(box=>box.id!==app.selectedMailboxId()&&!box.mailwomanSearchQuery)}>{box=><option value={box.id}>{box.name}</option>}</For>
   </select>
  </fieldset>
  <Show when={messageDialogEnabled()}><button class="btn btn--ghost" data-testid="bulk-properties-open" onClick={()=>{const chosen=new Set(ids());openBulkProperties(app.messages().filter(message=>chosen.has(message.id)));}}><Icon name="info"/>{t('props-menu-properties')}</button></Show>
  <button class="btn btn--ghost" onClick={()=>{cancelled=true;app.clearSelection();props.onCancel();}}>{t('mail-cancel')}</button>
 </div>;
}
