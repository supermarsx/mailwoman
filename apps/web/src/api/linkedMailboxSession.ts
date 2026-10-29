import {createSignal} from 'solid-js';
import type {LinkedAccountContext} from './linkedAccountContext.ts';

const [active,setActive]=createSignal<Readonly<LinkedAccountContext>|null>(null);
const [catalogRevision,setCatalogRevision]=createSignal(0);
export const linkedAccountsRevision=catalogRevision;
export function invalidateLinkedAccounts():void{setCatalogRevision(value=>value+1);}
let epoch=0;
let primaryAuthority:string|undefined;
export function setPrimaryMailboxAuthority(value:string|undefined):void{primaryAuthority=value;}
export function primaryMailboxAuthority():string|undefined{return primaryAuthority;}
export const activeLinkedMailbox=active;
export function linkedMailboxEpoch():number{return epoch;}

/** What a store sees when the mailbox the app acts for changes. `context` is
 * the newly selected linked mailbox, or null for the sign-in account. `epoch`
 * is the value `linkedMailboxEpoch()` now returns: a load that captured an
 * older epoch belongs to the previous mailbox and must be dropped. Fired on
 * every install, including sign-in, sign-out and a switch back to the primary,
 * because each one ends the previous mailbox's data. */
export interface LinkedMailboxChange {
  readonly context:Readonly<LinkedAccountContext>|null;
  readonly previous:Readonly<LinkedAccountContext>|null;
  readonly epoch:number;
}
export type LinkedMailboxChangeListener=(change:LinkedMailboxChange)=>void;
const listeners=new Set<LinkedMailboxChangeListener>();

/** Subscribe a per-mailbox store (calendar, tasks, notes, contacts, keys, ...)
 * to mailbox changes. Listeners run synchronously inside `installLinkedMailbox`,
 * after the new context is active and before any request for it is sent, so a
 * store can drop cached account ids, data and unlocked secrets in time. One
 * listener throwing does not stop the others. Returns the unsubscribe. */
export function onLinkedMailboxChange(listener:LinkedMailboxChangeListener):()=>void {
  listeners.add(listener);
  return ()=>{listeners.delete(listener);};
}

/** Called only after effective Me and discovery have verified this context. */
export function installLinkedMailbox(context:LinkedAccountContext|null):void {
  epoch++;
  const previous=active();
  const next=context?Object.freeze({...context}):null;
  setActive(next);
  const change:LinkedMailboxChange=Object.freeze({context:next,previous,epoch});
  for(const listener of [...listeners]){
    try{listener(change);}catch(error){console.error('linked mailbox change listener failed',error);}
  }
}
export class LinkedContextUnsupportedError extends Error {
  constructor(){super('This operation is unavailable in a linked mailbox. Switch to your primary account.');this.name='LinkedContextUnsupportedError';}
}
