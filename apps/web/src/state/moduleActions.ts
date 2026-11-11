import {createEffect,createSignal,untrack} from 'solid-js';
import {preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';

export type ModuleAction='new'|'import'|'my-day'|'pinned'|'refresh';
const [pending,setPending]=createSignal<{surface:string;action:ModuleAction;owner:ReturnType<typeof preferenceAccount>}>();
/** A sidebar command survives a lazy module mount, but never an account switch. */
export function requestModuleAction(surface:string,action:ModuleAction):void {const owner=preferenceAccount();if(owner)setPending({surface,action,owner});}
export function handleModuleActions(surface:string,handle:(action:ModuleAction)=>void):void {
 createEffect(()=>{
  const command=pending();if(!command||command.surface!==surface)return;
  setPending(undefined);
  if(!command.owner||!preferenceAccountIsCurrent(command.owner))return;
  untrack(()=>handle(command.action));
 });
}
