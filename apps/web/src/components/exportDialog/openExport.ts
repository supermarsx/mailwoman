// Builds what the export dialog offers from the mail state and opens it. The
// message menus and the reader call `exportFromMenu`; when the administrator has
// switched the dialog off it falls back to the plain .eml download of the one
// message (`app.exportMessage`), never to the Settings screen.
import type {Email} from '../../api/jmap-types.ts';
import {requestSettings} from '../../state/settingsRequest.ts';
import {exportDialogAvailable,openExportDialog} from './host.tsx';
import type {ScopeSource} from './exportPlan.ts';

/** The slice of the app state the sources are built from. */
export interface ExportApp {
  listMessages:()=>readonly Email[];selectedIds:()=>ReadonlySet<string>;selectedMailboxId:()=>string|null;
  mailboxes:()=>readonly {id:string;name:string;parentId:string|null}[];
  exportMessage:(id?:string)=>Promise<void>;
}

/** Scopes for the message `email`: it alone, its thread, the selection that contains it, and the open folder. All from the rows already loaded. */
export function sourceFor(app:Pick<ExportApp,'listMessages'|'selectedIds'|'selectedMailboxId'|'mailboxes'>,email:Email):ScopeSource {
  const rows=app.listMessages(),selected=app.selectedIds();
  const thread=email.threadId?rows.filter(row=>row.threadId===email.threadId):[];
  const selection=selected.has(email.id)?rows.filter(row=>selected.has(row.id)):[];
  const boxId=app.selectedMailboxId(),box=boxId?app.mailboxes().find(value=>value.id===boxId):undefined;
  return {
    message:[email],
    ...(thread.length>1?{conversation:thread}:{}),
    ...(selection.length>1?{selection}:{}),
    ...(box?{folder:{id:box.id,name:box.name,hasSubfolders:app.mailboxes().some(value=>value.parentId===box.id)}}:{}),
  };
}

/** Open the export dialog for `emailId`; returns whether it opened. */
export function exportFromMenu(app:ExportApp,emailId:string,email?:Email):boolean {
  const target=email??app.listMessages().find(row=>row.id===emailId);
  if(!target||!exportDialogAvailable())return false;
  return openExportDialog({source:sourceFor(app,target),onOpenSettings:()=>requestSettings('mail-transfer')});
}
