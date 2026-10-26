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
/** Called only after effective Me and discovery have verified this context. */
export function installLinkedMailbox(context:LinkedAccountContext|null):void {
  epoch++;
  setActive(context?Object.freeze({...context}):null);
}
export class LinkedContextUnsupportedError extends Error {
  constructor(){super('This operation is unavailable in a linked mailbox. Switch to your primary account.');this.name='LinkedContextUnsupportedError';}
}
