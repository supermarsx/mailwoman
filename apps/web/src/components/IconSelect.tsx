import {onCleanup} from 'solid-js';
import {observePreferenceAccount,preferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {Icon,type IconName} from './Icon.tsx';
import {createContextMenu} from './ContextMenu.tsx';
import * as css from './iconSelect.css.ts';

export interface IconSelectOption {value:string;label:string;icon:IconName}
/** Icon-only trigger and icon-labelled choices share the app's themed menu. */
export function IconSelect(props:{label:string;value:string;options:readonly IconSelectOption[];onChange:(value:string)=>void}) {
 const selected=()=>props.options.find(option=>option.value===props.value)??props.options[0];
 const menu=createContextMenu({label:()=>props.label,scope:()=>`${props.label}:${preferenceAccount()?.generation??'signed-out'}`,focusCurrent:true,actions:()=>props.options.map(option=>({id:option.value,label:option.label,icon:option.icon,radio:true,checked:option.value===props.value,current:option.value===props.value,run:()=>props.onChange(option.value)}))});
 onCleanup(observePreferenceAccount(()=>menu.close(false)));
 return <><button type="button" class={`btn ${css.root}`} aria-label={props.label} title={`${props.label}: ${selected()?.label??''}`} aria-haspopup="menu" aria-expanded={menu.isOpen()}
  onClick={event=>{if(menu.isOpen())menu.close();else menu.openBelow(event.currentTarget);}}
  onKeyDown={event=>{if(event.key==='ArrowDown'||event.key==='ArrowUp'){event.preventDefault();menu.openBelow(event.currentTarget);}else if(event.key==='Escape'&&menu.isOpen()){event.preventDefault();menu.close();}}}>
  <Icon name={selected()?.icon??'preferences'}/><span class={css.chevron}><Icon name="chevronDown"/></span>
 </button>{menu.menu()}</>;
}
