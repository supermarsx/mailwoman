import {ReaderAction} from '../../components/ReaderAction.tsx';
import {createContextMenu} from '../../components/ContextMenu.tsx';
import {SearchInput} from '../../components/SearchInput.tsx';
import './settingsTransfer.css';
import {createSignal,For,onMount,Show,type JSX} from 'solid-js';
import {settings} from '../../api/settings.ts';
import {Icon} from '../../components/Icon.tsx';
import {readerActionPlacement,READER_ACTION_ICONS} from '../../components/readerActionPlacement.ts';
import {loadCatalog,t} from '../../i18n';
import {READER_ACTIONS,READER_ACTION_PLACEMENTS,readerActionKey,isReaderActionPlacement,type ReaderActionPlacement} from '../../components/readerActionPlacement.ts';
import {useSetting} from './useSetting.ts';
import {SettingsCard} from './layout/SettingsCard.tsx';
import {type CardSpec} from './layout/rows.tsx';
import {LockNote} from './LockNote.tsx';
import * as compact from './readerActionPreferences.css.ts';

// This module also supplies the Settings search index before its card is opened.
void loadCatalog('reader-actions');
void loadCatalog('settings-transfer');
export const READER_ACTIONS_CARD:CardSpec={
 id:'reader-card-action-placement',section:'reader',title:'reader-actions-title',help:'reader-actions-help',
 rows:READER_ACTIONS.map(action=>({id:`reader-action-${action.id}`,label:action.labelKey})),
};
function ActionChoice(props:{action:typeof READER_ACTIONS[number]}):JSX.Element {
 const handle=useSetting<ReaderActionPlacement>(readerActionKey(props.action.id),props.action.defaultPlacement,isReaderActionPlacement);
 // A refused request leaves the shared value unchanged. Re-read it when the
 // error changes so the browser's pending selection cannot imply a saved pick.
 const displayed={...handle,value:()=>{handle.error();return handle.value();}};
 // Keep native option nodes stable across settings/presentation updates.
 // Replacing them after the select value changes resets browsers to option 0.
 const options=READER_ACTION_PLACEMENTS.map(value=>({value,get label(){return t(`reader-actions-option-${value}`);}}));
 const id=`reader-action-${props.action.id}`,disabled=()=>!handle.entry()||handle.locked()||handle.saving();
 return <div class={compact.row} data-setting-row>
  <Icon name={READER_ACTION_ICONS[props.action.id]}/><label class={compact.label} for={id}>{t(props.action.labelKey)}</label>
  <select id={id} class={compact.select} value={displayed.value()} disabled={disabled()} aria-describedby={handle.locked()?`${id}-lock`:undefined} onChange={event=>{const value=event.currentTarget.value;if(!disabled()&&isReaderActionPlacement(value))void handle.set(value);}}><For each={options}>{option=><option value={option.value}>{option.label}</option>}</For></select>
  <button type="button" class={`btn btn--ghost ${compact.reset}`} disabled={disabled()} aria-label={t('reader-inherit')} title={t('reader-inherit')} onClick={()=>void handle.reset()}><Icon name="restore"/></button>
  <Show when={handle.locked()||handle.error()}><div class={compact.note}><LockNote id={`${id}-lock`} locked={handle.locked()}/><Show when={handle.error()}><p role="alert">{handle.error()}</p></Show></div></Show>
 </div>;
}
/** Placement changes presentation only; the reader still applies feature and
 * permission checks before offering any action. Settings keeps all choices so
 * an unavailable message-specific action can be configured for future use. */
export function ReaderActionPreferences():JSX.Element {
 const [query,setQuery]=createSignal('');
 const previewMenu=createContextMenu({label:()=>t('reader-actions-option-menu'),scope:()=> 'reader-preview',actions:()=>READER_ACTIONS.filter(action=>readerActionPlacement(action.id,settings())==='menu').map(action=>({id:action.id,label:t(action.labelKey),icon:READER_ACTION_ICONS[action.id],run:()=>{}}))});
 onMount(()=>{void loadCatalog('reader-actions');void loadCatalog('settings-transfer');});
 return <SettingsCard id={READER_ACTIONS_CARD.id} title={t(READER_ACTIONS_CARD.title)} description={t('reader-actions-help')} testId="reader-action-preferences">
  <label for="reader-actions-search">{t('st-ribbon-search')}</label><SearchInput id="reader-actions-search"  value={query()} onInput={event=>setQuery(event.currentTarget.value)} style={{'max-inline-size':'28rem','inline-size':'100%','box-sizing':'border-box'}}/>
  <details><summary>{t('st-ribbon-preview')}</summary><div class={compact.preview} role="toolbar" aria-label={t('st-ribbon-preview')}><For each={READER_ACTIONS.filter(action=>readerActionPlacement(action.id,settings())==='toolbar')}>{action=><ReaderAction icon={READER_ACTION_ICONS[action.id]} label={t(action.labelKey)} help={t(action.labelKey)} onClick={()=>{}}/>}</For><Show when={READER_ACTIONS.some(action=>readerActionPlacement(action.id,settings())==='menu')}><ReaderAction icon="more" label={t('reader-actions-option-menu')} help={t('st-ribbon-preview-help')} popup="menu" expanded={previewMenu.isOpen()} onClick={event=>previewMenu.openBelow(event.currentTarget as HTMLButtonElement)}/></Show></div><p>{t('st-ribbon-preview-help')}</p></details>{previewMenu.menu()}
  <div class={compact.list}><For each={READER_ACTIONS}>{action=><Show when={t(action.labelKey).toLocaleLowerCase().includes(query().trim().toLocaleLowerCase())}><ActionChoice action={action}/></Show>}</For></div>
 </SettingsCard>;
}
