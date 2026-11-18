import {createSignal} from 'solid-js';

/** Three losses of an established connection within a minute indicate flapping.
 * Startup failures and repeated reports of the same outage count only once. */
export function createConnectionStability(changed:(unstable:boolean)=>void){
 let healthy=false,unstable=false,losses:number[]=[];
 let stableTimer:ReturnType<typeof setTimeout>|undefined;
 const cancel=()=>{if(stableTimer!==undefined)clearTimeout(stableTimer);stableTimer=undefined;};
 const set=(value:boolean)=>{if(unstable!==value){unstable=value;changed(value);}};
 return {
  observe(connected:boolean):void{
   if(connected===healthy)return;
   healthy=connected;cancel();
   if(connected){
    stableTimer=setTimeout(()=>{stableTimer=undefined;losses=[];set(false);},60_000);
   }else{
    const now=Date.now();losses=losses.filter(at=>now-at<60_000).slice(-2);losses.push(now);
    if(losses.length>=3)set(true);
   }
  },
  reset():void{cancel();healthy=false;losses=[];set(false);},
 };
}

const [actionConnectionUnstable,setActionConnectionUnstable]=createSignal(false);
export {actionConnectionUnstable};
export const actionConnectionStability=createConnectionStability(setActionConnectionUnstable);
