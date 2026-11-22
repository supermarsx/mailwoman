import {getOwner,onCleanup} from 'solid-js';
import {activeLinkedMailbox,linkedMailboxEpoch,onLinkedMailboxChange} from '../../api/linkedMailboxSession.ts';
import {observePreferenceAccount,preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';

/** A rules API belongs to one verified mailbox generation. In particular an
 * upload that finishes after a switch must not issue its follow-up set call. */
export function ruleAccountContext(accountId:string):(()=>void)&{readonly signal:AbortSignal} {
  const epoch=linkedMailboxEpoch(),owner=preferenceAccount();
  const controller=new AbortController(),stops:(()=>void)[]=[];
  const matches=()=>!!accountId&&epoch===linkedMailboxEpoch()&&owner===preferenceAccount()&&(owner===null||owner.id===accountId)
    && (activeLinkedMailbox()===null||activeLinkedMailbox()?.accountId===accountId);
  const stop=()=>{controller.abort();for(const dispose of stops.splice(0))dispose();};
  const changed=()=>{if(!matches())stop();};
  stops.push(onLinkedMailboxChange(changed));stops.push(observePreferenceAccount(changed));
  if(controller.signal.aborted)stop();
  if(getOwner())onCleanup(stop);
  return Object.assign(()=>{
    if(controller.signal.aborted||!matches()){
      throw new DOMException('Mailbox changed','AbortError');
    }
  },{signal:controller.signal});
}
