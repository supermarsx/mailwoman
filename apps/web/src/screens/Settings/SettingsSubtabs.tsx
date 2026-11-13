import {createEffect,createSignal,createUniqueId,For,type JSX} from 'solid-js';
import {t} from '../../i18n';
import {useSettingsNavigation,type SettingsNavigation} from './navigation.tsx';
import * as tabs from './securityTabs.css.ts';
import * as css from './styles.css.ts';

export interface SettingsSubtab {
  id:string;
  label:string;
  /** Existing search card IDs stay stable when a section is divided into tabs. */
  cards:readonly string[];
  content:JSX.Element;
}
/** Keep panels mounted so switching tabs preserves unfinished field edits. */
export function SettingsSubtabs(props:{section:string;label:string;tabs:readonly SettingsSubtab[];navigation?:SettingsNavigation}):JSX.Element {
  const navigation=props.navigation??useSettingsNavigation(),uid=createUniqueId();
  // The section schema is static; construct its panels once, outside reactive tab changes.
  const sections=props.tabs;
  const [selected,setSelected]=createSignal(sections[0]?.id??'');
  createEffect(()=>{
    const route=navigation?.route();
    if(route?.section!==props.section)return;
    const target=sections.find(tab=>!!route.card&&tab.cards.includes(route.card));
    if(target)setSelected(target.id);
  });
  function key(event:KeyboardEvent,index:number):void {
    const delta=event.key==='ArrowRight'?1:event.key==='ArrowLeft'?-1:0;
    const target=event.key==='Home'?0:event.key==='End'?sections.length-1:delta?(index+delta+sections.length)%sections.length:-1;
    if(target<0)return;
    event.preventDefault();const next=sections[target]!;setSelected(next.id);
    (event.currentTarget as HTMLButtonElement).ownerDocument.getElementById(`${uid}-${next.id}-tab`)?.focus();
  }
  return <div class={css.stack} data-settings-subtabs={props.section}>
    <div class={tabs.list} role="tablist" aria-label={t(props.label)}><For each={sections}>{(tab,index)=><button type="button" role="tab" id={`${uid}-${tab.id}-tab`} aria-controls={`${uid}-${tab.id}-panel`} aria-selected={selected()===tab.id} tabIndex={selected()===tab.id?0:-1} onClick={()=>setSelected(tab.id)} onKeyDown={event=>key(event,index())}>{t(tab.label)}</button>}</For></div>
    <For each={sections}>{tab=><div id={`${uid}-${tab.id}-panel`} role="tabpanel" aria-labelledby={`${uid}-${tab.id}-tab`} hidden={selected()!==tab.id} tabIndex={0} class={tabs.panel}>{tab.content}</div>}</For>
  </div>;
}
