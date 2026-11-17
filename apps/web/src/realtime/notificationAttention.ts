import {createSignal} from 'solid-js';
import {observePreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {alertValue,type AlertValues} from '../api/notificationControls.ts';
const [rows,setRows]=createSignal<ReadonlyMap<string,number>>(new Map());
let expiry:ReturnType<typeof setTimeout>|undefined,flash:ReturnType<typeof setInterval>|undefined,finish:ReturnType<typeof setTimeout>|undefined;
let original='',painted='';
export const incomingHighlighted=(id:string):boolean=>(rows().get(id)??0)>Date.now();
function clearTab():void{clearInterval(flash);clearTimeout(finish);flash=undefined;finish=undefined;if(painted&&document.title===painted)document.title=original;painted='';}
export function clearNotificationAttention():void{clearTab();clearTimeout(expiry);setRows(new Map());}
observePreferenceAccount(clearNotificationAttention);
export function newMailAttention(ids:readonly string[],title:string,values?:AlertValues):void{
 const seconds=Math.max(2,Math.min(120,alertValue('notifications.incoming.durationSeconds',15,values)));
 if(alertValue('notifications.incoming.highlight',true,values)){
  const next=new Map(rows());for(const id of ids)next.set(id,Date.now()+seconds*1000);while(next.size>1000)next.delete(next.keys().next().value!);setRows(next);
  clearTimeout(expiry);expiry=setTimeout(()=>setRows(new Map()),seconds*1000);
 }
 if(!alertValue('notifications.incoming.tab',false,values)||document.hasFocus())return;
 clearTab();original=document.title;painted=`● ${title}`;document.title=painted;
 const reduced=document.documentElement.hasAttribute('data-reduced-motion')||globalThis.matchMedia?.('(prefers-reduced-motion: reduce)').matches;
 if(!reduced)flash=setInterval(()=>{if(document.title!==painted&&document.title!==original)original=document.title;document.title=document.title===painted?original:painted;},1000);
 finish=setTimeout(clearTab,seconds*1000);window.addEventListener('focus',clearTab,{once:true});
}
