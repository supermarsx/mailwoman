import {preferenceAccount,preferenceAccountIsCurrent,type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';

const MIME='application/x-mailwoman-workspace-tab';
let current:{id:string;token:string;owner:PreferenceAccount}|null=null;
/** Native drags carry an opaque token, never an externally trusted tab ID. */
export function beginWorkspaceTabDrag(event:DragEvent,tabId:string):void {
  current=null;
  const owner=preferenceAccount();
  if(!owner||!event.dataTransfer){event.preventDefault();return;}
  const token=crypto.randomUUID();current={id:tabId,token,owner};
  event.dataTransfer.effectAllowed='move';event.dataTransfer.setData(MIME,token);
}
/** Data is unreadable during dragover; drop must request token verification. */
export function workspaceTabDragId(event:DragEvent,verifyToken=false):string|null {
  const drag=current;
  if(!drag||!preferenceAccountIsCurrent(drag.owner)||!event.dataTransfer?.types.includes(MIME))return null;
  if(verifyToken&&event.dataTransfer.getData(MIME)!==drag.token)return null;
  return drag.id;
}
export function endWorkspaceTabDrag():void {current=null;}
