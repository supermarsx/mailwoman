import './settingsTransfer.css';
import {createSignal,For,onMount,Show,type JSX} from 'solid-js';
import {settings} from '../../api/settings.ts';
import {Icon} from '../../components/Icon.tsx';
import {readerActionPlacement} from '../../components/readerActionPlacement.ts';
import {loadCatalog,t} from '../../i18n';
import {READER_ACTIONS,READER_ACTION_PLACEMENTS,readerActionKey,isReaderActionPlacement,type ReaderActionPlacement} from '../../components/readerActionPlacement.ts';
import {useSetting} from './useSetting.ts';
import {SettingsCard} from './layout/SettingsCard.tsx';
import {SelectRow,type CardSpec} from './layout/rows.tsx';

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
 return <SelectRow id={`reader-action-${props.action.id}`} label={t(props.action.labelKey)} handle={displayed}
  options={options} accept={isReaderActionPlacement}/>;
}
/** Placement changes presentation only; the reader still applies feature and
 * permission checks before offering any action. Settings keeps all choices so
 * an unavailable message-specific action can be configured for future use. */
export function ReaderActionPreferences():JSX.Element {
 const [query,setQuery]=createSignal('');
 onMount(()=>{void loadCatalog('reader-actions');void loadCatalog('settings-transfer');});
 return <SettingsCard id={READER_ACTIONS_CARD.id} title={t(READER_ACTIONS_CARD.title)} description={t('reader-actions-help')} testId="reader-action-preferences">
  <label for="reader-actions-search">{t('st-ribbon-search')}</label><input id="reader-actions-search" type="search" value={query()} onInput={event=>setQuery(event.currentTarget.value)} style={{'max-inline-size':'28rem','inline-size':'100%','box-sizing':'border-box'}}/>
  <details><summary>{t('st-ribbon-preview')}</summary><div class="st-buttons" role="list" aria-label={t('st-ribbon-preview')}><For each={READER_ACTIONS.filter(action=>readerActionPlacement(action.id,settings())==='toolbar')}>{action=><span role="listitem" class="st-status">{t(action.labelKey)}</span>}</For><Show when={READER_ACTIONS.some(action=>readerActionPlacement(action.id,settings())==='menu')}><span role="listitem"><Icon name="more"/>{t('reader-actions-option-menu')}</span></Show></div><p>{t('st-ribbon-preview-help')}</p></details>
  <For each={READER_ACTIONS}>{action=><Show when={t(action.labelKey).toLocaleLowerCase().includes(query().trim().toLocaleLowerCase())}><ActionChoice action={action}/></Show>}</For>
 </SettingsCard>;
}
