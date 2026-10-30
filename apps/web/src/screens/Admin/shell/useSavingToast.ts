import {createEffect, createUniqueId, on, onCleanup, type Accessor} from 'solid-js';
import {toast} from '../../../components/toast/index.ts';
/** A busy action is announced in the shared region, without changing its button label. */
export function useSavingToast(busy:Accessor<boolean>, message:Accessor<string>):void {
  const id=`admin-saving-${createUniqueId()}`;
  createEffect(on(()=>[busy(),message()] as const,([active,text])=>{if(active)toast.info(text,{id,busy:true,persistent:true,duration:Infinity});else toast.dismiss(id);}));
  onCleanup(()=>toast.dismiss(id));
}
