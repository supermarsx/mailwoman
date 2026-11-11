import {createEffect,createSignal,createUniqueId,For,onMount,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {useSettingsNavigation} from './navigation.tsx';
import * as tabs from './securityTabs.css.ts';
import * as css from './styles.css.ts';

type PrivacyTab='content'|'consent'|'diagnostics'|'data'|'requests';

function tabForCard(card:string|undefined):PrivacyTab|undefined {
  if(!card)return undefined;
  if(card==='your-data-requests'||card.startsWith('data-requests-'))return 'requests';
  if(card.startsWith('your-data-'))return 'data';
  if(card==='privacy-card-analytics'||card==='privacy-card-cookies')return 'consent';
  if(card==='privacy-card-load-time'||card==='privacy-card-error-reports')return 'diagnostics';
  if(card.startsWith('privacy-card-'))return 'content';
  return undefined;
}

export function PrivacyCenter(props:Record<PrivacyTab,JSX.Element>):JSX.Element {
  const navigation=useSettingsNavigation();
  const [selected,setSelected]=createSignal<PrivacyTab>('content');
  const uid=createUniqueId();
  const sections:readonly PrivacyTab[]=['content','consent','diagnostics','data','requests'];
  onMount(()=>void loadCatalog('privacy'));
  createEffect(()=>{
    const route=navigation?.route();
    if(route?.section!=='privacy')return;
    const target=tabForCard(route.card);
    if(target)setSelected(target);
  });
  function key(event:KeyboardEvent,index:number):void {
    const delta=event.key==='ArrowRight'?1:event.key==='ArrowLeft'?-1:0;
    const target=event.key==='Home'?0:event.key==='End'?sections.length-1:delta?(index+delta+sections.length)%sections.length:-1;
    if(target<0)return;
    event.preventDefault();
    const next=sections[target]!;
    setSelected(next);
    (event.currentTarget as HTMLButtonElement).ownerDocument.getElementById(`${uid}-${next}-tab`)?.focus();
  }
  return <div class={css.stack} data-testid="privacy-center">
    <div class={tabs.list} role="tablist" aria-label={t('privacy-tabs-label')}>
      <For each={sections}>{(name,index)=><button type="button" role="tab" id={`${uid}-${name}-tab`} aria-controls={`${uid}-${name}-panel`}
        aria-selected={selected()===name} tabIndex={selected()===name?0:-1} onClick={()=>setSelected(name)} onKeyDown={event=>key(event,index())}>
        {t(`privacy-tab-${name}`)}
      </button>}</For>
    </div>
    <For each={sections}>{name=><div id={`${uid}-${name}-panel`} role="tabpanel" aria-labelledby={`${uid}-${name}-tab`}
      hidden={selected()!==name} tabIndex={0} class={tabs.panel}>{props[name]}</div>}</For>
  </div>;
}
