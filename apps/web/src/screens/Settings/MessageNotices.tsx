import {LockNote} from './LockNote.tsx';
import {observePreferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {useSettingsNavigation} from './navigation.tsx';
import {useLeaveConfirm} from '../../components/confirm.tsx';
import {createEffect,createSignal,on,onCleanup,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {useSetting} from './useSetting.ts';
import {validMessageNotices,type MessageNotice} from '../../components/compose/messageNotices.ts';
import {NoticeTemplateEditor} from '../../components/NoticeTemplateEditor.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import {ToggleRow} from './layout/rows.tsx';
import {toast} from '../../components/toast/toast.ts';
import * as css from './styles.css.ts';
import {rowReset} from './layout/layout.css.ts';
export function MessageNotices():JSX.Element {
 void loadCatalog('notices');
 void loadCatalog('reader');
 const enabled=useSetting('mail.notices.enabled',true);
 const templates=useSetting<MessageNotice[]>('mail.notices.templates',[],validMessageNotices);
 const [rows,setRows]=createSignal<MessageNotice[]>([]),[baseline,setBaseline]=createSignal('[]');
 onCleanup(observePreferenceAccount(()=>{setRows([]);setBaseline('[]');}));
 const dirty=()=>JSON.stringify(rows())!==baseline();
 const nav=useSettingsNavigation();const Leave=nav?useLeaveConfirm(nav,dirty,()=>t('notices-discard')):()=>null;
 createEffect(on(()=>JSON.stringify(templates.value()),value=>{if(!dirty())setRows(JSON.parse(value) as MessageNotice[]);setBaseline(value);}));
 const disabled=()=>!templates.entry()||templates.locked()||templates.saving();
 async function save(){if(disabled()||!validMessageNotices(rows()))return;const ok=await templates.set(structuredClone(rows()));if(ok)toast.success(t('notices-saved'));else toast.error(templates.error());}
 async function inherit(){if(disabled()||dirty())return;const ok=await templates.reset();if(ok)toast.success(t('notices-saved'));else toast.error(templates.error());}
 return <SettingsCard id="compose-card-notices" title={t('notices-title')} wide>
  <ToggleRow id="notices-enabled" label={t('notices-enabled')} handle={enabled}/>
  <LockNote locked={templates.locked()}/>
  <NoticeTemplateEditor value={rows()} onChange={setRows} disabled={disabled()}/>
  <div class={css.actions}><button type="button" class={css.button} disabled={disabled()||!dirty()||!validMessageNotices(rows())} onClick={()=>void save()}>{t('common-save')}</button><button type="button" class={css.ghost} disabled={!dirty()||templates.saving()} onClick={()=>setRows(structuredClone(templates.value()))}>{t('common-cancel')}</button><button type="button" class={rowReset} disabled={disabled()||dirty()} onClick={()=>void inherit()}>{t('reader-inherit')}</button></div>
 <Leave/></SettingsCard>;
}
