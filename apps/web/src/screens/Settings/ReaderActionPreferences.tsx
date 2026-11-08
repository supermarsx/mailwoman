import {For,onMount,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {READER_ACTIONS,READER_ACTION_PLACEMENTS,readerActionKey,isReaderActionPlacement,type ReaderActionPlacement} from '../../components/readerActionPlacement.ts';
import {useSetting} from './useSetting.ts';
import {SettingsCard} from './layout/SettingsCard.tsx';
import {SelectRow,type CardSpec} from './layout/rows.tsx';

// This module also supplies the Settings search index before its card is opened.
void loadCatalog('reader-actions');
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
 onMount(()=>void loadCatalog('reader-actions'));
 return <SettingsCard id={READER_ACTIONS_CARD.id} title={t(READER_ACTIONS_CARD.title)} description={t('reader-actions-help')} testId="reader-action-preferences">
  <For each={READER_ACTIONS}>{action=><ActionChoice action={action}/>}</For>
 </SettingsCard>;
}
