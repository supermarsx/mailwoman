import {createEffect,onCleanup} from 'solid-js';
import {observePreferenceAccount,preferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {Icon,type IconName} from './Icon.tsx';
import {createContextMenu} from './ContextMenu.tsx';
import {useCollapsibleBar} from './bars/CollapsibleBar.tsx';
import * as css from './iconSelect.css.ts';

export interface IconSelectOption {value:string;label:string;icon:IconName}
/** Icon-only trigger and icon-labelled choices share the app's themed menu. */
export function IconSelect(props:{label:string;value:string;options:readonly IconSelectOption[];onChange:(value:string)=>void;showLabel?:boolean;testId?:string}) {
 let trigger:HTMLButtonElement|undefined;
 const bar=useCollapsibleBar();
 const selected=()=>props.options.find(option=>option.value===props.value)??props.options[0];
 const menu=createContextMenu({ownerDocument:()=>trigger?.ownerDocument??document,label:()=>props.label,scope:()=>`${props.label}:${preferenceAccount()?.generation??'signed-out'}`,focusCurrent:true,actions:()=>props.options.map(option=>({id:option.value,label:option.label,icon:option.icon,radio:true,checked:option.value===props.value,current:option.value===props.value,run:()=>props.onChange(option.value)}))});
 createEffect(()=>{if(menu.isOpen()){const release=bar?.hold();onCleanup(()=>release?.());}});
 onCleanup(observePreferenceAccount(()=>menu.close(false)));
 return <><button ref={trigger} type="button" value={props.value} data-testid={props.testId} class={`btn btn--ghost ${css.root}`} aria-label={props.label} title={`${props.label}: ${selected()?.label??''}`} aria-haspopup="menu" aria-expanded={menu.isOpen()}
  on:click={event=>{if(menu.isOpen())menu.close();else menu.openBelow(event.currentTarget);}}
  on:keydown={event=>{if(event.key==='ArrowDown'||event.key==='ArrowUp'){event.preventDefault();menu.openBelow(event.currentTarget);}else if(event.key==='Escape'&&menu.isOpen()){event.preventDefault();menu.close();}}}>
  <Icon name={selected()?.icon??'preferences'}/>{props.showLabel&&<span>{selected()?.label??''}</span>}<span class={css.chevron}><Icon name="chevronDown"/></span>
 </button>{menu.menu()}</>;
}
