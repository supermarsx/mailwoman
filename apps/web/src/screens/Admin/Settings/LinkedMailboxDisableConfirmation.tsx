import {createEffect,createSignal,type Accessor} from 'solid-js';
import {ConfirmDialog} from '../shell/ConfirmDialog.tsx';
import {t} from '../../../i18n';
import type {Draft} from './value.ts';

/** These global switches erase stored renewal credentials when saved as false. */
export function createLinkedMailboxDisableConfirmation(options:{key:Accessor<string>;draft:Accessor<Draft>;stored:Accessor<Draft>;scope:Accessor<string>;setDraft:(value:Draft)=>void}){
 const [pending,setPending]=createSignal<{scope:string;previous:Draft}>();
 createEffect(()=>{const request=pending();if(request&&request.scope!==options.scope())setPending(undefined);});
 function change(value:Draft):void {
  const previous=options.draft();
  options.setDraft(value);
  if(value===false&&previous===true&&options.stored()===true&&['accounts.linkingEnabled','accounts.storeLinkedCredentials'].includes(options.key()))setPending({scope:options.scope(),previous});
  else setPending(undefined);
 }
 function cancel():void {
  const request=pending();setPending(undefined);
  if(request?.scope===options.scope())options.setDraft(request.previous);
 }
 const dialog=()=><ConfirmDialog open={pending()!==undefined} title={t('admin-settings-linked-disable-title')} body={t('admin-settings-linked-disable-body')} confirmLabel={t('admin-settings-linked-disable-confirm')} onConfirm={()=>setPending(undefined)} onCancel={cancel}/>;
 return {change,pending:()=>pending()!==undefined,dialog};
}
