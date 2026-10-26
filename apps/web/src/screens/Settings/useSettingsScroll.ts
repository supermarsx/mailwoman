import {createEffect,onCleanup,onMount,type Accessor} from 'solid-js';
import {observePreferenceAccount,preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import type {SettingsRoute} from './navigation.tsx';

/** In-memory positions only. Route guards decide navigation; this hook never does. */
export function useSettingsScroll(route:Accessor<SettingsRoute>,element:Accessor<HTMLElement|undefined>):void {
 const positions=new Map<string,number>();let current='',frame:number|undefined,owner=preferenceAccount();
 const cancel=():void=>{if(frame!==undefined){cancelAnimationFrame(frame);frame=undefined;}};
 const remember=():void=>{const node=element();if(!node||frame!==undefined||!current)return;positions.delete(current);positions.set(current,node.scrollTop);if(positions.size>64)positions.delete(positions.keys().next().value!);};
 const intent=():void=>{cancel();remember();};
 createEffect(()=>{
  const value=route(),key=JSON.stringify([value.section,value.editor?.kind??'',value.editor?.id??'']);if(key===current)return;
  cancel();current=key;const target=positions.get(key)??0;
  frame=requestAnimationFrame(()=>{frame=undefined;if(current===key){const node=element();if(node)node.scrollTop=target;}});
 });
 onMount(()=>{
  const node=element();if(!node)return;
  node.addEventListener('scroll',remember,{passive:true});
  for(const name of ['wheel','touchstart','pointerdown','keydown','focusin'])node.addEventListener(name,intent,{passive:true});
  onCleanup(()=>{node.removeEventListener('scroll',remember);for(const name of ['wheel','touchstart','pointerdown','keydown','focusin'])node.removeEventListener(name,intent);});
 });
 onCleanup(observePreferenceAccount(account=>{if(account===owner)return;owner=account;cancel();positions.clear();const node=element();if(node)node.scrollTop=0;}));
 onCleanup(cancel);
}
