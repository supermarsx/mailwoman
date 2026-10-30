import {createEffect,createMemo,createSignal,on,type Accessor} from 'solid-js';
/** Ephemeral consent, scoped to the current account/message/security policy. */
export function createMessageLinkConsent(scope:Accessor<string>,ready:Accessor<boolean>=()=>true):{enabled:Accessor<boolean>;toggle:()=>void}{
 const current=createMemo(scope);
 const [granted,setGranted]=createSignal<string|null>(null);
 createEffect(on(current,()=>setGranted(null)));
 const enabled=()=>granted()===current();
 return {enabled,toggle:()=>setGranted(enabled()?null:ready()?current():null)};
}
