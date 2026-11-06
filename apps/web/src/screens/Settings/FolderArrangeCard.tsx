import {createUniqueId,Show,type JSX} from 'solid-js';
import type {AccountUiPreferences} from '../../api/accountPreferences.ts';
import type {FolderLayout} from '../../components/folderLayout.ts';
import {toast} from '../../components/toast/toast.ts';
import {t} from '../../i18n';
import {LockNote} from './LockNote.tsx';
import {useSetting} from './useSetting.ts';
import * as css from './styles.css.ts';
/**
 * "Arrange folders": the user's own switches for dragging folders. The administrator's keys decide what is possible
 * (and may lock a value: the lock note and a disabled control say so); these preferences choose within that.
 * Arrangement itself (order, visual parents, pins, stars) is stored in the account's UI preferences.
 */
export function FolderArrangeCard(props:{layout:FolderLayout;prefs:()=>AccountUiPreferences;update:(patch:AccountUiPreferences)=>void;disabled?:boolean}):JSX.Element {
 const enabled=useSetting<boolean>('folders.reorder.enabled',true),placement=useSetting<string>('folders.newFolderPlacement','end',(value):value is string=>value==='end'||value==='alphabetical');
 const id=createUniqueId();
 const adminLocked=()=>props.layout.layout().locked;
 const nestOff=()=>!props.layout.layout().dnd.allowNesting;
 const serverOff=()=>!props.layout.layout().allowServerMove;
 return <section class={css.section} aria-labelledby={`${id}-title`} data-testid="folder-arrange-card">
  <h3 class={css.heading} id={`${id}-title`}>{t('folder-dnd-arrange-title')}</h3>
  <p class={css.meta}>{t('folder-dnd-arrange-help')}</p>
  <Show when={adminLocked()}><p class={css.meta} role="note" data-testid="folder-order-locked"><span aria-hidden="true">🔒 </span>{t('folder-dnd-locked-hint')}</p></Show>
  <label class={css.check}><input class={css.checkbox} type="checkbox" checked={enabled.value()} disabled={props.disabled||!enabled.entry()||enabled.locked()||adminLocked()||enabled.saving()} aria-describedby={`${id}-enabled-lock`} onChange={event=>void enabled.set(event.currentTarget.checked)}/>{t('folder-dnd-reorder-enable')}</label>
  <LockNote locked={enabled.locked()} id={`${id}-enabled-lock`}/>
  <label class={css.field}>{t('folder-dnd-nest-mode')}
   <select class={css.select} value={props.prefs().folderNestMode??'ask'} disabled={props.disabled||nestOff()} onChange={event=>props.update({folderNestMode:event.currentTarget.value as 'ask'|'server'|'local'})}>
    <option value="ask">{t('folder-dnd-nest-mode-ask')}</option>
    <option value="server" disabled={serverOff()}>{t('folder-dnd-nest-mode-server')}</option>
    <option value="local">{t('folder-dnd-nest-mode-local')}</option>
   </select>
  </label>
  <label class={css.field}>{t('folder-dnd-new-placement')}
   <select class={css.select} value={placement.value()} disabled={props.disabled||!placement.entry()||placement.locked()||placement.saving()} aria-describedby={`${id}-placement-lock`} onChange={event=>void placement.set(event.currentTarget.value)}>
    <option value="end">{t('folder-dnd-new-placement-end')}</option>
    <option value="alphabetical">{t('folder-dnd-new-placement-alphabetical')}</option>
   </select>
  </label>
  <LockNote locked={placement.locked()} id={`${id}-placement-lock`}/>
  <label class={css.field}>{t('folder-dnd-handles')}
   <select class={css.select} value={props.prefs().folderGrabHandles??'hover'} disabled={props.disabled} onChange={event=>props.update({folderGrabHandles:event.currentTarget.value as 'hover'|'always'})}>
    <option value="hover">{t('folder-dnd-handles-hover')}</option>
    <option value="always">{t('folder-dnd-handles-always')}</option>
   </select>
  </label>
  <div class={css.actions}><button type="button" class={css.ghost} disabled={props.disabled||adminLocked()} onClick={()=>{props.layout.arranger.reset();toast.success(t('folder-dnd-reset-done'));}}>{t('folder-dnd-reset')}</button></div>
 </section>;
}
