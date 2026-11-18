import {SearchInput} from '../../components/SearchInput.tsx';
import {createMemo, createSignal, For, Show, type JSX} from 'solid-js';
import {Icon,type IconName} from '../../components/Icon.tsx';
import {t} from '../../i18n';
import type {FolderIcon} from '../../api/accountPreferences.ts';
import {FOLDER_ICON_GROUPS} from './folderPreferenceModel.ts';
import * as css from './styles.css.ts';
import * as folderCss from './FolderPreferences.css.ts';
/** Grouped, searchable icon choice with a roving-focus radio grid. `automatic` renders the folder's own default glyph. */
export function FolderIconPicker(props:{label:string;value:FolderIcon|'';disabled?:boolean;automaticIcon:IconName;onChange:(id:FolderIcon|'')=>void}):JSX.Element {
 const [query,setQuery]=createSignal('');
 const label=(id:string)=>t(`settings-folder-icon-${id}`);
 const normalize=(value:string)=>value.normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLocaleLowerCase();
 const matches=(id:string,group:string)=>{const words=normalize(query()).trim().split(/\s+/).filter(Boolean);const text=normalize(`${id.replace(/([a-z])([A-Z])/g,'$1 $2')} ${label(id)} ${t(`settings-folder-icon-group-${group}`)}`);return words.every(word=>text.includes(word));};
 const groups=createMemo(()=>FOLDER_ICON_GROUPS.map(group=>({id:group.id,icons:group.icons.filter(id=>matches(id,group.id))})).filter(group=>group.icons.length>0));
 const showAutomatic=()=>!query().trim()||t('settings-folders-automatic').toLowerCase().includes(query().trim().toLowerCase());
 const visible=createMemo((): (FolderIcon|'')[]=>[...(showAutomatic()?['' as const]:[]),...groups().flatMap(group=>group.icons)]);
 const stop=()=>visible().includes(props.value)?props.value:visible()[0];
 const key=(event:KeyboardEvent&{currentTarget:HTMLButtonElement}):void=>{
  const buttons=Array.from(event.currentTarget.closest('[role=radiogroup]')!.querySelectorAll<HTMLButtonElement>('[role=radio]'));const at=buttons.indexOf(event.currentTarget);
  const next=event.key==='ArrowRight'||event.key==='ArrowDown'?(at+1)%buttons.length:event.key==='ArrowLeft'||event.key==='ArrowUp'?(at+buttons.length-1)%buttons.length:event.key==='Home'?0:event.key==='End'?buttons.length-1:null;
  if(next!==null){event.preventDefault();buttons[next]?.click();buttons[next]?.focus({preventScroll:true});buttons[next]?.scrollIntoView?.({block:'nearest'});}
 };
 const button=(id:FolderIcon|''):JSX.Element=><button type="button" role="radio" aria-checked={props.value===id} disabled={props.disabled} class={css.ghost} title={id?label(id):t('settings-folders-automatic')} aria-label={id?label(id):t('settings-folders-automatic')} data-icon-id={id} tabIndex={stop()===id?0:-1} onKeyDown={key} onClick={()=>props.onChange(id)}><Icon name={id||props.automaticIcon}/></button>;
 return <div class={folderCss.iconPicker}>
  <SearchInput  class={css.input} value={query()} aria-label={t('settings-folder-icon-search-label')} placeholder={t('settings-folder-icon-search-label')} onInput={event=>setQuery(event.currentTarget.value)}/>
  <div class={folderCss.iconCollection} role="radiogroup" aria-label={props.label}>
   <Show when={showAutomatic()}><div class={folderCss.iconGrid}>{button('')}</div></Show>
   <For each={groups()}>{group=><div role="group" aria-label={t(`settings-folder-icon-group-${group.id}`)}>
    <h4 class={css.meta}>{t(`settings-folder-icon-group-${group.id}`)}</h4>
    <div class={folderCss.iconGrid}><For each={group.icons}>{id=>button(id)}</For></div>
   </div>}</For>
   <Show when={visible().length===0}><p class={css.meta} role="status">{t('settings-folder-icon-no-match')}</p></Show>
  </div>
 </div>;
}
