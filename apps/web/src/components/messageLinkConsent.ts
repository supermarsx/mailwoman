import {createEffect,createMemo,createSignal,on,type Accessor} from 'solid-js';
/** Ephemeral consent, scoped to the current account/message/security policy. */
export function createMessageLinkConsent(scope:Accessor<string>,ready:Accessor<boolean>=()=>true,trusted:Accessor<boolean>=()=>false):{enabled:Accessor<boolean>;toggle:()=>void;enable:()=>void;disable:()=>void}{
 const current=createMemo(scope);
 const [granted,setGranted]=createSignal<string|null>(null);
 const [denied,setDenied]=createSignal<string|null>(null);
 createEffect(on(current,()=>{setGranted(null);setDenied(null);}));
 const enabled=()=>ready()&&denied()!==current()&&(granted()===current()||trusted());
 const enable=():void=>{if(!ready())return;setDenied(null);setGranted(current());};
 const disable=():void=>{setGranted(null);setDenied(current());};
 return {enabled,enable,disable,toggle:()=>enabled()?disable():enable()};
}
