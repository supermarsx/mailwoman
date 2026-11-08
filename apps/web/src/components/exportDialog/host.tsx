// One place that owns the open export dialog. The reader, the message menus and
// the bulk bar call `openExportDialog`; `ExportDialogHost` (mounted once in the
// mail screen) draws it. A request is refused (returns false) when the
// administrator has switched the dialog off, so the caller can fall back.
import {Show,Suspense,createSignal,lazy,onCleanup,type JSX} from 'solid-js';
import {preferenceAccount,observePreferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {onLinkedMailboxChange} from '../../api/linkedMailboxSession.ts';
import {loadCatalog} from '../../i18n';
import {exportPolicy,type ExportScope,type ScopeSource} from './exportPlan.ts';

void loadCatalog('export-dialog');
const ExportDialog=lazy(async()=>({default:(await import('./ExportDialog.tsx')).ExportDialog}));

export interface ExportRequest {source:ScopeSource;scope?:ExportScope;onOpenSettings?:()=>void}
type Open=ExportRequest&{owner:string|null};
const [open,setOpen]=createSignal<Open|null>(null);

/** Whether the dialog is available at all (`export.dialog.enabled`). */
export const exportDialogAvailable=():boolean=>exportPolicy().enabled;

export function openExportDialog(request:ExportRequest):boolean {
  if(!exportDialogAvailable())return false;
  setOpen({...request,owner:preferenceAccount()?.id??null});
  return true;
}
export function closeExportDialog():void {setOpen(null);}
export function exportDialogOpen():boolean {return open()!==null;}

export function ExportDialogHost():JSX.Element {
  // The request belongs to this mounted mail screen, including the lazy-load
  // gap before ExportDialog installs its own transport cleanup.
  let owner=preferenceAccount();
  onCleanup(observePreferenceAccount(next=>{if(next!==owner){owner=next;closeExportDialog();}}));
  onCleanup(onLinkedMailboxChange(closeExportDialog));
  onCleanup(closeExportDialog);
  return <Suspense fallback={null}>
    <Show when={open()}>{current=>{
      const value=current();
      // A dialog never outlives the account it was opened for.
      if(value.owner!==(preferenceAccount()?.id??null))return null;
      return <ExportDialog source={value.source} {...(value.scope?{scope:value.scope}:{})} {...(value.onOpenSettings?{onOpenSettings:value.onOpenSettings}:{})} onClose={closeExportDialog}/>;
    }}</Show>
  </Suspense>;
}
