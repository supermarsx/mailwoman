import {onCleanup} from 'solid-js';
import type {PrefetchTrigger} from '../state/mailReadCache.ts';
/** Dwell ownership follows the actual pointer/focus gesture, not a render. */
export function mailPrefetchEvents(start:(trigger:PrefetchTrigger)=>()=>void){
 let hover:(()=>void)|undefined,focus:(()=>void)|undefined;
 const stopHover=()=>{hover?.();hover=undefined;},stopFocus=()=>{focus?.();focus=undefined;};
 onCleanup(()=>{stopHover();stopFocus();});
 return {onPointerEnter:()=>{stopHover();hover=start('hover');},onPointerLeave:stopHover,onFocus:()=>{stopFocus();focus=start('focus');},onBlur:stopFocus};
}
