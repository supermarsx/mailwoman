import {observeToastEvents,toast,allToasts,type ToastTone,type ToastCloseReason} from './toast.ts';
import {publishSharedView,readSharedView,observeSharedViews,sharedWindowId} from '../../worker/networkProxy.ts';
import {observePreferenceAccount,preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
interface Descriptor {respectPreferences?:boolean;id:string;message:string;tone:ToastTone;duration:number;expires:number;owner:string;action?:{label:string;token:string};secondaryAction?:{label:string;token:string};persistent:boolean;countdown:boolean;pausable:boolean;busy:boolean}
interface Snapshot {items:Descriptor[];closed?:{id:string;reason:ToastCloseReason}}
/** UI callbacks stay in their originating window. Only scoped presentation
 * data and one-use action handles cross the coordinator. */
export function startSharedToasts():()=>void{
 let applying=false,sequence=0;const descriptors=new Map<string,Descriptor>(),actions=new Map<string,()=>void|Promise<void>>();
 function apply(value:unknown):void{
  const snapshot=value as Snapshot|null;if(!snapshot||!Array.isArray(snapshot.items)||snapshot.items.length>209)return;
  if(snapshot.items.some(row=>!row||typeof row.id!=='string'||row.id.length>512||typeof row.message!=='string'||row.message.length>8192||!['success','info','warning','error'].includes(row.tone)||typeof row.expires!=='number'||typeof row.duration!=='number'||typeof row.owner!=='string'||[row.action,row.secondaryAction].some(action=>action&&(typeof action.label!=='string'||typeof action.token!=='string'))))return;
  applying=true;
  try{
   for(const row of allToasts())if(!snapshot.items.some(item=>item.id===row.id))toast.dismiss(row.id,snapshot.closed?.id===row.id?snapshot.closed.reason:'dismiss');
   for(const row of snapshot.items){
    if(row.expires>0&&row.expires<=Date.now())continue;
    if(JSON.stringify(descriptors.get(row.id))===JSON.stringify(row))continue;
    descriptors.set(row.id,row);
    const remote=(action:NonNullable<Descriptor['action']>)=>({label:action.label,run:()=>publishSharedView('toast-action',{owner:row.owner,token:action.token})});
    toast.show(row.tone,row.message,{id:row.id,duration:row.expires>0?Math.max(1,row.expires-Date.now()):row.duration||Infinity,persistent:row.persistent,countdown:row.countdown,pausable:row.pausable,busy:row.busy,respectPreferences:row.respectPreferences===true,...(row.action?{action:remote(row.action)}:{}),...(row.secondaryAction?{secondaryAction:remote(row.secondaryAction)}:{})});
   }
  }finally{applying=false;}
 }
 const stopEvents=observeToastEvents(event=>{
  if(applying||!preferenceAccount()?.storageAuthority)return;
  if(event.kind==='show'){
   const row=event.item,options=event.options;
   const action=(value:typeof options.action)=>{if(!value)return;const token=`${sharedWindowId}:${++sequence}`;actions.set(token,value.run);while(actions.size>418)actions.delete(actions.keys().next().value!);return {label:value.label,token};};
   const first=action(options.action),second=action(options.secondaryAction);
   descriptors.set(row.id,{...(options.respectPreferences?{respectPreferences:true}:{}),id:row.id,message:row.message,tone:row.tone,duration:Number.isFinite(row.duration)?row.duration:0,expires:!row.pausable&&Number.isFinite(row.duration)?Date.now()+row.duration:0,owner:sharedWindowId,persistent:row.persistent,countdown:row.countdown,pausable:row.pausable,busy:row.busy,...(first?{action:first}:{}),...(second?{secondaryAction:second}:{})});
  }
  if(event.kind==='clear')descriptors.clear();
  const liveIds=new Set(allToasts().map(row=>row.id));
  for(const id of descriptors.keys())if(!liveIds.has(id))descriptors.delete(id);
  const tokens=new Set([...descriptors.values()].flatMap(row=>[row.action?.token,row.secondaryAction?.token].filter((token):token is string=>!!token)));
  for(const token of actions.keys())if(!tokens.has(token))actions.delete(token);
  const items=allToasts().map(row=>descriptors.get(row.id)).filter((row):row is Descriptor=>!!row);
  void publishSharedView('toasts',{items,...(event.kind==='dismiss'?{closed:{id:event.id,reason:event.reason}}:{})} satisfies Snapshot);
 });
 const stopViews=observeSharedViews(view=>{
  if(view.topic==='toasts')apply(view.value);
  else if(view.topic==='toast-action'){
   const value=view.value as {owner?:string;token?:string};if(value?.owner!==sharedWindowId||typeof value.token!=='string')return;
   const run=actions.get(value.token);actions.delete(value.token);if(run)void Promise.resolve().then(run).catch(()=>undefined);
  }
 });
 const stopAccount=observePreferenceAccount(owner=>{actions.clear();descriptors.clear();if(owner?.storageAuthority)void readSharedView('toasts').then(value=>{if(preferenceAccount()===owner)apply(value);});});
 return ()=>{stopEvents();stopViews();stopAccount();actions.clear();descriptors.clear();};
}
