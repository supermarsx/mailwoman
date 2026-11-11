// The message actions no other menu offered: Not spam, View source, Create filter
// from this message and Block sender. One list feeds the message context menu,
// the reader's More menu and the Properties dialog, so they always agree.
import type {useApp} from '../state/context.ts';
import type {Email} from '../api/jmap-types.ts';
import {CAP_CORE} from '../api/jmap-types.ts';
import {CAP_CRYPTO,CAP_SECURITY} from '../api/crypto-types.ts';
import {quickPrint} from './messageQuickActions.ts';
import {responseFor} from '../api/jmap.ts';
import {apiFetch} from '../api/http.ts';
import {withBase} from '../api/basePath.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {t} from '../i18n';
import type {ContextAction} from './ContextMenu.tsx';
import {queueRuleDraft,senderRuleDraft} from '../state/ruleDraft.ts';
import {requestSettings} from '../state/settingsRequest.ts';
import type {MessageDialogActions} from './properties/MessageProperties.tsx';
import {featureOn} from '../state/features.ts';
import {createMessageTask} from './messageTask.ts';
import {moveActions} from './ReaderMoveMenu.tsx';
import { loadCatalog } from '../i18n/catalog.ts';
// i18n: the ids this file shows live in these catalogs, which load with it.
void loadCatalog('properties');

type App=ReturnType<typeof useApp>;

const senderOf=(email:Email)=>(email.from?.[0]?.email??'').trim();
const inRole=(email:Email,app:App,role:string)=>app.mailboxes().some(box=>box.role===role&&email.mailboxIds?.[box.id]===true);

/** Open the raw message in a new tab as plain text. */
export async function viewSource(app:Pick<App,'showToast'>,id:string):Promise<void> {
 const tab=window.open('about:blank','_blank','noopener');
 try{
  const response=await apiFetch(withBase(`/api/export/${encodeURIComponent(id)}?format=eml`));
  if(!response.ok)throw new Error(String(response.status));
  const url=URL.createObjectURL(new Blob([await response.text()],{type:'text/plain;charset=utf-8'}));
  if(tab)tab.location.href=url;else window.open(url,'_blank','noopener');
  setTimeout(()=>URL.revokeObjectURL(url),60_000);
 }catch{tab?.close();app.showToast('error',t('mail-context-failed'));}
}

/** `SenderControl/set` block, the same call the reader's sender controls make. */
export async function blockSender(app:Pick<App,'accountId'|'showToast'>,email:Email):Promise<boolean> {
 const account=app.accountId(),address=senderOf(email);
 if(!account||!address)return false;
 try{
  const response=await createConfiguredClient().jmap({using:[CAP_CORE,CAP_CRYPTO,CAP_SECURITY],methodCalls:[['SenderControl/set',{accountId:account,emailId:email.id,action:'block',address},'sc']]});
  const result=responseFor<{updated?:boolean}>(response,'sc');
  if(result.updated)return true;
 }catch{/* falls through to the toast */}
 app.showToast('error',t('mail-context-failed'));
 return false;
}

/** Start a rule from this message: the Rules editor opens with the sender filled in. */
export function createFilterFrom(email:Email):void {
 const address=senderOf(email);
 if(!address)return;
 queueRuleDraft(senderRuleDraft(address));
 requestSettings('rules');
}

/** Not spam: back to the inbox from the junk folder. */
export function notSpam(app:App,email:Email):Promise<void>|void {
 const inbox=app.mailboxes().find(box=>box.role==='inbox');
 if(inbox)return app.moveMessage(email.id,inbox.id);
}

/** The rows for one message; `skip` leaves out ids a menu already has. */
export function messageActionRows(app:App,email:Email,skip:ReadonlySet<string>=new Set()):ContextAction[] {
 const rows:ContextAction[]=[];
 const add=(row:ContextAction)=>{if(!skip.has(row.id))rows.push(row);};
 add({id:'move-to',searchLabel:t('mail-move-search'),label:t('mail-reader-move-to'),icon:'moveTo',children:moveActions(email,app),run:()=>{}});
 if(inRole(email,app,'junk')&&app.mailboxes().some(box=>box.role==='inbox'))add({id:'not-spam',label:t('props-act-not-spam'),icon:'spam',run:()=>notSpam(app,email)});
 if(featureOn('tasks') && app.accountId?.())add({id:'create-task',label:t('message-task-action'),icon:'check',run:()=>createMessageTask(app,email)});
 add({id:'view-source',label:t('props-act-source'),icon:'file',run:()=>viewSource(app,email.id)});
 if(senderOf(email)){
  add({id:'create-filter',label:t('props-act-filter'),icon:'filter',run:()=>createFilterFrom(email)});
  add({id:'block-sender',label:t('props-act-block'),icon:'block',run:async()=>{if(await blockSender(app,email))app.showToast('success',t('props-act-block'));}});
 }
 return rows;
}

/** What the Properties dialog may ask the menus to do for this message. */
export function dialogActionsFor(app:App,email:Email):MessageDialogActions {
 const actions:MessageDialogActions={onViewSource:()=>void viewSource(app,email.id),onPrint:()=>void quickPrint(app,email.id)};
 const junk=app.mailboxes().find(box=>box.role==='junk'),inbox=app.mailboxes().find(box=>box.role==='inbox');
 if(inRole(email,app,'junk')&&inbox)actions.onNotSpam=()=>void notSpam(app,email);
 else if(junk&&!inRole(email,app,'junk'))actions.onSpam=()=>void app.markSpam(email.id);
 if(senderOf(email)){actions.onCreateFilter=()=>createFilterFrom(email);actions.onBlockSender=()=>void blockSender(app,email);}
 return actions;
}
