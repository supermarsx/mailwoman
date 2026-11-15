import {createEffect,createMemo,createSignal,untrack,onCleanup,getOwner,type Accessor} from 'solid-js';
import {preferenceAccount,preferenceAccountIsCurrent,observePreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {applyClick,extendByKey,pruneSelection,selectAll,EMPTY_SELECTION,type SelectionState} from './folderSelectionModel.ts';
import {toast} from './toast/toast.ts';
import {t} from '../i18n/index.ts';
export interface SelectionModifiers {shiftKey:boolean;ctrlKey:boolean;metaKey:boolean}
export function createRangeSelection(options:{ids:Accessor<readonly string[]>;scope:Accessor<unknown>;selected?:Accessor<ReadonlySet<string>>;setSelected?:(ids:ReadonlySet<string>)=>void;onError?:(error:unknown)=>void}){
 const [state,setState]=createSignal<SelectionState>({...EMPTY_SELECTION}),[mode,setMode]=createSignal(false),[busy,setBusy]=createSignal(false);
 const [identity,setIdentity]=createSignal(preferenceAccount());const stop=observePreferenceAccount(setIdentity);if(getOwner())onCleanup(stop);
 // Initial reads must not subscribe the caller's render computation. Some
 // embedded views are invoked directly; tracking their filter here would
 // recreate the whole view (and discard focus) whenever that filter changes.
 let scope=untrack(options.scope),owner=untrack(identity);
 const selected=createMemo(()=>options.selected?.()??state().selected);
 const current=():SelectionState=>({...state(),selected:selected()});
 function write(next:SelectionState):void{setState(next);options.setSelected?.(next.selected);}
 createEffect(()=>{
  const nextScope=options.scope(),nextOwner=identity(),visible=new Set(options.ids());
  untrack(()=>{
   if(nextScope!==scope||nextOwner!==owner){scope=nextScope;owner=nextOwner;write({...EMPTY_SELECTION});setMode(false);return;}
   const old=current(),next=pruneSelection(old,visible);if(next!==old)write(next);
  });
 });
 function pick(id:string,event:SelectionModifiers,checkbox=false):boolean{
  if(busy())return true;
  if(!options.ids().includes(id))return false;
  if(checkbox||mode()||event.shiftKey||event.ctrlKey||event.metaKey){
   setMode(true);write(applyClick(current(),options.ids(),id,{shift:event.shiftKey,toggle:!event.shiftKey}));return true;
  }
  write({...current(),anchor:id,rangeAdded:[]});return false;
 }
 function click(id:string,event:MouseEvent,open?:()=>void):void{if(pick(id,event)){event.preventDefault();event.stopPropagation();}else open?.();}
 function checkbox(id:string,event:MouseEvent):void{event.stopPropagation();pick(id,event,true);const input=event.currentTarget;if(input instanceof HTMLInputElement)input.checked=selected().has(id);}
 function clear():void{write({...current(),selected:new Set(),rangeAdded:[]});}
 function dismiss():void{clear();setMode(false);}
 function all():void{setMode(true);write(selectAll(options.ids()));}
 function keydown(id:string,event:KeyboardEvent,focus?:(id:string)=>void):boolean{
  if(event.target instanceof Element&&event.target.closest('input,textarea,select,[contenteditable="true"]'))return false;
  const modifier=event.ctrlKey||event.metaKey;
  if(modifier&&event.key.toLowerCase()==='a'){event.preventDefault();event.stopPropagation();all();return true;}
  if(event.key==='Escape'&&(mode()||selected().size)){event.preventDefault();event.stopPropagation();if(selected().size)clear();else dismiss();return true;}
  if(event.shiftKey&&(event.key==='ArrowDown'||event.key==='ArrowUp')){const value=extendByKey(current(),options.ids(),id,event.key==='ArrowDown'?1:-1);if(value){event.preventDefault();event.stopPropagation();setMode(true);write(value.state);if(focus)focus(value.focus);else{const root=event.currentTarget instanceof Element?event.currentTarget.closest('[data-range-selection-root]')??event.currentTarget.closest('table,ul'):null;[...root?.querySelectorAll<HTMLElement>('[data-selection-id]')??[]].find(row=>row.dataset.selectionId===value.focus)?.focus();}}return true;}
  if(event.key===' '){event.preventDefault();event.stopPropagation();pick(id,event,true);return true;}
  return false;
 }
 const reportError=(error:unknown)=>options.onError?options.onError(error):toast.error(t('common-error'));
 async function runAll(action:(ids:readonly string[],isCurrent:()=>boolean)=>void|Promise<unknown>):Promise<void>{
  if(busy())return;const owner=preferenceAccount(),captured=options.scope(),ids=options.ids().filter(id=>selected().has(id));setBusy(true);
  const isCurrent=()=>options.scope()===captured&&(owner?preferenceAccountIsCurrent(owner):preferenceAccount()===null);
  try{if(isCurrent())await action(ids,isCurrent);}
  catch(error){if(isCurrent())reportError(error);}finally{setBusy(false);}
 }
 const run=(action:(id:string)=>void|Promise<unknown>)=>runAll(async(ids,isCurrent)=>{for(const id of ids){if(!isCurrent())break;await action(id);}});
 return {selected,mode,setMode,busy,scope:()=>String(options.scope()),click,pick,checkbox,keydown,clear,dismiss,all,run,runAll,reportError,toggleMode:()=>mode()?dismiss():setMode(true),set:(ids:Iterable<string>)=>{write({...current(),selected:new Set(ids),rangeAdded:[]});}};
}
export type RangeSelection=ReturnType<typeof createRangeSelection>;
