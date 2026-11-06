import {Show,type JSX} from 'solid-js';
import type {AccountUiPreferences} from '../../api/accountPreferences.ts';
import type {FolderLayout} from '../../components/folderLayout.ts';
import {toast} from '../../components/toast/toast.ts';
import {t} from '../../i18n';
import {LockNote} from './LockNote.tsx';
import {useSetting} from './useSetting.ts';
import {ControlRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import * as css from './styles.css.ts';
/**
 * "Arrange folders": the user's own switches for dragging folders. The administrator's keys decide what is possible
 * (and may lock a value: the lock note and a disabled control say so); these preferences choose within that.
 * Arrangement itself (order, visual parents, pins, stars) is stored in the account's UI preferences.
 */
export function FolderArrangeCard(props:{layout:FolderLayout;prefs:()=>AccountUiPreferences;update:(patch:AccountUiPreferences)=>void;disabled?:boolean}):JSX.Element {
 const enabled=useSetting<boolean>('folders.reorder.enabled',true),placement=useSetting<string>('folders.newFolderPlacement','end',(value):value is string=>value==='end'||value==='alphabetical');
 const id='folders-arrange';
 const adminLocked=()=>props.layout.layout().locked;
 const nestOff=()=>!props.layout.layout().dnd.allowNesting;
 const serverOff=()=>!props.layout.layout().allowServerMove;
 return <SettingsCard id={`${id}-card`} title={t('folder-dnd-arrange-title')} description={t('folder-dnd-arrange-help')} testId="folder-arrange-card">
  <Show when={adminLocked()}><p class={css.meta} role="note" data-testid="folder-order-locked"><span aria-hidden="true">🔒 </span>{t('folder-dnd-locked-hint')}</p></Show>
  <ControlRow controlId={`${id}-enabled`} label={t('folder-dnd-reorder-enable')} note={<LockNote locked={enabled.locked()} id={`${id}-enabled-lock`}/>}>{()=>
   <input id={`${id}-enabled`} type="checkbox" checked={enabled.value()} disabled={props.disabled||!enabled.entry()||enabled.locked()||adminLocked()||enabled.saving()} aria-describedby={`${id}-enabled-lock`} onChange={event=>void enabled.set(event.currentTarget.checked)}/>
  }</ControlRow>
  <ControlRow controlId={`${id}-nest`} label={t('folder-dnd-nest-mode')}>{()=>
   <select id={`${id}-nest`} class={css.select} value={props.prefs().folderNestMode??'ask'} disabled={props.disabled||nestOff()} onChange={event=>props.update({folderNestMode:event.currentTarget.value as 'ask'|'server'|'local'})}>
    <option value="ask">{t('folder-dnd-nest-mode-ask')}</option>
    <option value="server" disabled={serverOff()}>{t('folder-dnd-nest-mode-server')}</option>
    <option value="local">{t('folder-dnd-nest-mode-local')}</option>
   </select>
  }</ControlRow>
  <ControlRow controlId={`${id}-placement`} label={t('folder-dnd-new-placement')} note={<LockNote locked={placement.locked()} id={`${id}-placement-lock`}/>}>{()=>
   <select id={`${id}-placement`} class={css.select} value={placement.value()} disabled={props.disabled||!placement.entry()||placement.locked()||placement.saving()} aria-describedby={`${id}-placement-lock`} onChange={event=>void placement.set(event.currentTarget.value)}>
    <option value="end">{t('folder-dnd-new-placement-end')}</option>
    <option value="alphabetical">{t('folder-dnd-new-placement-alphabetical')}</option>
   </select>
  }</ControlRow>
  <ControlRow controlId={`${id}-handles`} label={t('folder-dnd-handles')}>{()=>
   <select id={`${id}-handles`} class={css.select} value={props.prefs().folderGrabHandles??'hover'} disabled={props.disabled} onChange={event=>props.update({folderGrabHandles:event.currentTarget.value as 'hover'|'always'})}>
    <option value="hover">{t('folder-dnd-handles-hover')}</option>
    <option value="always">{t('folder-dnd-handles-always')}</option>
   </select>
  }</ControlRow>
  <div class={css.actions}><button type="button" class={css.ghost} disabled={props.disabled||adminLocked()} onClick={()=>{props.layout.arranger.reset();toast.success(t('folder-dnd-reset-done'));}}>{t('folder-dnd-reset')}</button></div>
 </SettingsCard>;
}
