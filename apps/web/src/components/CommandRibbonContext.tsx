import {createContext,createEffect,createSignal,onCleanup,untrack,useContext,type JSX} from 'solid-js';
import type {ContextAction} from './ContextMenu.tsx';
import type {RibbonSlotTarget} from './compose/ribbonParts.tsx';
import type {IconName} from './Icon.tsx';
export interface RibbonReaderActions {toolbar:ContextAction[];menu:ContextAction[]}
export interface RibbonPreview {token:symbol;name:string;icon:IconName}
export function createCommandRibbon(enabled:()=>boolean){
 const [reader,setReader]=createSignal<{token:symbol;actions:RibbonReaderActions}>();
 const [preview,setPreview]=createSignal<RibbonPreview>();
 const [fileTarget,setFileTarget]=createSignal<RibbonSlotTarget>();
 return {enabled,active:()=>true,reader,preview,fileTarget,setFileTarget,
  publishReader(token:symbol,actions:RibbonReaderActions|undefined){if(actions)setReader({token,actions});else if(untrack(reader)?.token===token)setReader(undefined);},
  publishPreview(token:symbol,value:Omit<RibbonPreview,'token'>|undefined){if(value)setPreview({token,...value});else if(untrack(preview)?.token===token)setPreview(undefined);},
 };
}
export type CommandRibbonBridge=ReturnType<typeof createCommandRibbon>;
export const CommandRibbonContext=createContext<CommandRibbonBridge>();
export const useCommandRibbon=()=>useContext(CommandRibbonContext);
export function CommandRibbonScope(props:{active:boolean;children:JSX.Element}):JSX.Element {
 const parent=useCommandRibbon();
 return parent?<CommandRibbonContext.Provider value={{...parent,active:()=>parent.active()&&props.active}}>{props.children}</CommandRibbonContext.Provider>:props.children;
}
/** A preview owns the File ribbon only while it is visible in the active pane. */
export function useFileRibbon(name:()=>string,icon:()=>IconName=()=>'attachment'):()=>boolean {
 const bridge=useCommandRibbon(),token=Symbol('file-ribbon');
 createEffect(()=>{if(bridge?.enabled()&&bridge.active()&&name()){bridge.publishPreview(token,{name:name(),icon:icon()});onCleanup(()=>bridge.publishPreview(token,undefined));}});
 return ()=>bridge?.enabled()===true&&bridge.active()&&!!name();
}
