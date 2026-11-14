import {settings} from '../api/settings.ts';
export const AFTER_MOVE_KEY='mail.afterMove';
export const AFTER_MOVE_MODES=['keep','next','previous','close'] as const;
export type AfterMoveMode=typeof AFTER_MOVE_MODES[number];
export function afterMoveMode():AfterMoveMode{const value=settings()[AFTER_MOVE_KEY]?.value;return AFTER_MOVE_MODES.includes(value as AfterMoveMode)?value as AfterMoveMode:'keep';}
export interface MovedMessages{accountId:string;mailboxId:string|null;sort:unknown;ids:readonly string[];order:readonly string[];mode:AfterMoveMode}
const listeners=new Set<(event:MovedMessages)=>void>();
export function onMessagesMoved(listener:(event:MovedMessages)=>void):()=>void{listeners.add(listener);return ()=>listeners.delete(listener);}
export function messagesMoved(event:MovedMessages):void{for(const listener of listeners)listener(event);}
/** Use the pre-move display order and skip every successfully moved or unavailable row. */
export function moveNeighbour(id:string,event:MovedMessages,available:ReadonlySet<string>):string|undefined{
 if(event.mode!=='next'&&event.mode!=='previous')return;
 const index=event.order.indexOf(id);if(index<0)return;
 const moved=new Set(event.ids),remaining=event.order.filter(id=>!moved.has(id)&&available.has(id));
 const matches=(id:string)=>!moved.has(id)&&available.has(id);
 return event.mode==='previous'?[...event.order.slice(0,index)].reverse().find(matches)??remaining[0]:event.order.slice(index+1).find(matches)??remaining.at(-1);
}
