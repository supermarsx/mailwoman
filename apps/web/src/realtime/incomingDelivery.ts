import type {IncomingMail} from './incomingMail.ts';
/** Policy lookup failures must not consume the only incoming-mail alert. The
 * queue is memory-only, bounded, authority-checked and cleared on sign-out. */
export function createIncomingDelivery(options:{deliver:(batch:IncomingMail)=>Promise<void>;current:(batch:IncomingMail)=>boolean}):{send:(batch:IncomingMail)=>void;clear:()=>void}{
 type Entry={batch:IncomingMail;attempt:number;timer?:ReturnType<typeof setTimeout>};
 const pending=new Map<string,Entry>();
 const keyOf=(batch:IncomingMail):string=>`${batch.target?.storageAuthority??batch.owner.storageAuthority??batch.owner.id}:${batch.messages[0]?.id??''}`;
 function discard(key:string,entry:Entry):void{if(entry.timer)clearTimeout(entry.timer);if(pending.get(key)===entry)pending.delete(key);}
 async function run(key:string,entry:Entry):Promise<void>{
  if(pending.get(key)!==entry)return;if(!options.current(entry.batch)){discard(key,entry);return;}
  try{await options.deliver(entry.batch);discard(key,entry);}catch(error){
   const value=error as {status?:number;name?:string;retryAfterSecs?:number};
   if(pending.get(key)!==entry||!options.current(entry.batch)||value?.name==='AbortError'||[400,401,403,404,409,422].includes(value?.status??0)){discard(key,entry);return;}
   const delay=Math.max(Math.min(30000,1000*2**Math.min(entry.attempt++,5)),Math.min(3600000,Math.max(0,value?.retryAfterSecs??0)*1000));
   entry.timer=setTimeout(()=>{delete entry.timer;void run(key,entry);},delay);
  }
 }
 return {
  send(batch){if(!options.current(batch))return;const key=keyOf(batch);if(pending.has(key))return;while(pending.size>=100){const [oldKey,entry]=pending.entries().next().value!;discard(oldKey,entry);}const entry:Entry={batch,attempt:0};pending.set(key,entry);void run(key,entry);},
  clear(){for(const [key,entry] of pending)discard(key,entry);},
 };
}
