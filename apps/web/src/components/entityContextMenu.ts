import {createSignal,onCleanup,type Accessor} from 'solid-js';
import {observePreferenceAccount,preferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {t} from '../i18n';
import {toast} from './toast/index.ts';
import {createContextMenu,type ContextAction} from './ContextMenu.tsx';

/** Resolve by ID at execution so a refreshed or deleted row cannot retain stale actions. */
export function createEntityContextMenu<T>(options:{items:Accessor<readonly T[]>;id:(item:T)=>string;name:(item:T)=>string;actions:(item:T)=>ContextAction[];scope?:Accessor<unknown>;onError?:()=>void}){
 const [selected,setSelected]=createSignal('');
 const [owner,setOwner]=createSignal(preferenceAccount());
 onCleanup(observePreferenceAccount(setOwner));
 const item=()=>options.items().find(value=>options.id(value)===selected());
 const context=createContextMenu({
  label:()=>t('common-context-actions',{name:item()?options.name(item()!):''}),
  scope:()=>JSON.stringify([owner(),options.scope?.(),selected(),!!item()]),
  actions:()=>{const value=item();return value?options.actions(value):[];},
  onError:options.onError??(()=>toast.error(t('mail-context-failed'))),
 });
 return {menu:context.menu,bind:(id:string)=>({
  onContextMenu:(event:MouseEvent)=>{setSelected(id);context.open(event);},
  onKeyDown:(event:KeyboardEvent)=>{if(event.key==='ContextMenu'||event.shiftKey&&event.key==='F10'){setSelected(id);context.keydown(event);}},
 })};
}
