type Change='queue'|'drafts'|'logout';
const listeners=new Set<(kind:Change)=>void>();
let channel:BroadcastChannel|undefined;
function receive(value:unknown):void{
 if(value==='queue'||value==='drafts'||value==='logout')for(const listener of listeners)listener(value);
}
function open():void{
 if(channel||typeof window==='undefined'||typeof BroadcastChannel==='undefined')return;
 try{channel=new BroadcastChannel('mw-encrypted-storage');channel.onmessage=event=>receive(event.data);}catch{/* Isolated browsing contexts still work locally. */}
}
/** Only an invalidation kind crosses windows, never message or draft content. */
export function publishOfflineChange(kind:Change):void{open();channel?.postMessage(kind);receive(kind);}
export function observeOfflineChanges(listener:(kind:Change)=>void):()=>void{open();listeners.add(listener);return()=>listeners.delete(listener);}
