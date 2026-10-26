import {createSignal,onCleanup,type Accessor} from 'solid-js';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from './accountPreferenceLifecycle.ts';
import {canAdminister,createAclClient} from './acl-types.ts';
import type {Client} from './client.ts';
/** Verify the actual folder's ACL contract; method names alone are not capability evidence. */
export function createSharingAccess(account:Accessor<string|null>,client:Pick<Client,'jmap'>) {
 const [verified,setVerified]=createSignal<Record<string,boolean>>({});
 let pending:AbortController|undefined;
 onCleanup(observePreferenceAccount(()=>{pending?.abort();setVerified({});}));
 onCleanup(()=>pending?.abort());
 async function probe(mailboxId:string,force=false):Promise<boolean>{
  const owner=preferenceAccount();if(!owner||owner.id!==account())return false;
  if(!force&&verified()[mailboxId]!==undefined)return verified()[mailboxId]===true;
  pending?.abort();setVerified(old=>{const next={...old};delete next[mailboxId];return next;});const controller=new AbortController();pending=controller;const timeout=setTimeout(()=>controller.abort(),8000);
  try {const rights=await createAclClient(owner.id,request=>client.jmap(request,{signal:controller.signal})).getMailboxRights(mailboxId);
   if(controller.signal.aborted||!preferenceAccountIsCurrent(owner)||account()!==owner.id)return false;
   const allowed=canAdminister(rights.myRights);setVerified(old=>({...old,[mailboxId]:allowed}));return allowed;
  }catch{if(!controller.signal.aborted&&preferenceAccountIsCurrent(owner))setVerified(old=>({...old,[mailboxId]:false}));return false;}
  finally{clearTimeout(timeout);if(pending===controller)pending=undefined;}
 }
 return {canShare:(id:string)=>verified()[id]===true,probe};
}
