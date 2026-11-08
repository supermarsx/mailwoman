// The Actions tab of the folder dialog: the ZIP download (with the administrator's
// export policy shown inline), the folder actions, and a separate danger zone. Every
// action that leaves the dialog closes it first, as before; confirmations belong to
// the code the action opens.
import {For,Show,createSignal,onCleanup,type JSX} from 'solid-js';
import {useApp} from '../../state/context.ts';
import type {Mailbox} from '../../api/jmap-types.ts';
import {t} from '../../i18n';
import {ACTION_ICONS} from '../../actionIcons.ts';
import {Icon,type IconName} from '../Icon.tsx';
import {toast} from '../toast/toast.ts';
import {formatBytes} from './folderStats.ts';
import {downloadFolderZip,exportUrl,fetchExportEstimate,ExportRefused,IN_PAGE_LIMIT_BYTES,saveBlob,type ExportEstimate} from './folderExport.ts';
import {exportFormats,exportSubfoldersDefault,folderExportEnabled,type ExportFormat} from './policy.ts';
import {copyText} from './folderDialogUtil.ts';
import type {FolderDialogActions} from './FolderProperties.tsx';
import * as css from './folderDialog.css.ts';

function ActionRow(props:{icon:IconName;title:string;description:string;danger?:boolean;disabled?:boolean;onRun:()=>void;testId?:string}):JSX.Element {
 const id=`act-${Math.random().toString(36).slice(2,8)}`;
 return <button type="button" class={`${css.action}${props.danger?` ${css.actionDanger}`:''}`} aria-label={props.title} aria-describedby={id} disabled={props.disabled} data-testid={props.testId} onClick={props.onRun}>
  <Icon name={props.icon}/>
  <span><span class={css.actionTitle} aria-hidden="true">{props.title}</span><span id={id} class={css.actionDesc}>{props.description}</span></span>
 </button>;
}

export function Actions(props:{box:Mailbox;actions:FolderDialogActions|undefined;onClose:()=>void;/** Present when the name can be edited in the header. */onStartRename?:(()=>void)|undefined}):JSX.Element {
 const app=useApp();
 const [format,setFormat]=createSignal<ExportFormat>(exportFormats()[0]??'eml');
 const [subfolders,setSubfolders]=createSignal(exportSubfoldersDefault());
 const [estimate,setEstimate]=createSignal<ExportEstimate|null>(null);
 const [problem,setProblem]=createSignal<string|null>(null);
 const [received,setReceived]=createSignal<number|null>(null);
 let controller:AbortController|undefined;
 onCleanup(()=>controller?.abort());
 const hasSub=()=>app.mailboxes().some(value=>value.parentId===props.box.id);
 const options=()=>({format:format(),subfolders:subfolders()});
 async function check():Promise<void> {
  setProblem(null);setEstimate(null);
  try{setEstimate(await fetchExportEstimate(props.box.id,options()));}
  catch(error){setProblem(error instanceof ExportRefused?error.code:'exportFailed');}
 }
 async function start():Promise<void> {
  const est=estimate();if(!est?.allowed)return;
  if(est.bytes>IN_PAGE_LIMIT_BYTES){window.location.assign(exportUrl(props.box.id,options()));return;}
  controller=new AbortController();setReceived(0);setProblem(null);
  try{
   const blob=await downloadFolderZip(props.box.id,options(),setReceived,controller.signal);
   saveBlob(blob,`${props.box.name.replace(/[\\/:*?"<>|]/g,'_')||'folder'}.zip`);
   toast.success(t('props-act-zip-done'));
  }catch(error){
   if((error as {name?:string}).name!=='AbortError')setProblem(error instanceof ExportRefused?error.code:'exportFailed');
  }finally{setReceived(null);controller=undefined;}
 }
 const run=(fn?:()=>void)=>()=>{props.onClose();fn?.();};
 const a=()=>props.actions;
 const canRenameHere=()=>props.onStartRename!==undefined||a()?.onRename!==undefined;
 const danger=()=>a()?.onEmpty!==undefined||a()?.onDelete!==undefined;
 return <div data-testid="props-actions" style={{display:'contents'}}>
  <Show when={folderExportEnabled()}><section class={css.card} aria-labelledby="props-zip-title">
   <h3 id="props-zip-title" class={css.cardTitle}>{t('props-act-zip')}</h3>
   <p class={css.cardHelp}>{t('props-act-zip-help')}</p>
   <div class={css.row}>
    <Show when={exportFormats().length>1}><label>{t('props-act-zip-format')} <select value={format()} onChange={event=>{setFormat(event.currentTarget.value as ExportFormat);setEstimate(null);}}><For each={exportFormats()}>{f=><option value={f}>{t(`props-act-format-${f}`)}</option>}</For></select></label></Show>
    <Show when={hasSub()}><label><input type="checkbox" checked={subfolders()} onChange={event=>{setSubfolders(event.currentTarget.checked);setEstimate(null);}}/> {t('props-act-zip-sub')}</label></Show>
    <button type="button" class="btn btn--ghost" data-testid="props-zip-check" onClick={()=>void check()}>{t('props-act-zip-estimate')}</button>
   </div>
   <Show when={estimate()}>{est=><p role="status" class={css.note} data-testid="props-zip-estimate">{est().allowed?t('props-act-zip-estimate-ok',{messages:est().messages,bytes:formatBytes(est().bytes)}):t('props-act-zip-estimate-refused',{messages:est().messages,bytes:formatBytes(est().bytes),max:formatBytes(est().maxBytes)})}</p>}</Show>
   <Show when={problem()}><p role="alert" class={css.problem} data-testid="props-zip-problem">{t(`props-act-problem-${problem()}`)}</p></Show>
   <div class={css.row}>
    <button type="button" class="btn btn--primary" data-testid="props-zip-start" disabled={!estimate()?.allowed||received()!==null} onClick={()=>void start()}><Icon name={ACTION_ICONS.downloadAll}/>{t('props-act-zip-start')}</button>
    <Show when={received()!==null}><span role="status">{t('props-act-progress',{bytes:formatBytes(received()!)})}</span><button type="button" class="btn btn--ghost" data-testid="props-zip-cancel" onClick={()=>controller?.abort()}>{t('props-act-cancel')}</button></Show>
   </div>
  </section></Show>
  <section class={css.card} aria-labelledby="props-acts-title">
   <h3 id="props-acts-title" class={css.cardTitle}>{t('props-card-actions')}</h3>
   <div>
    <Show when={a()?.onMarkRead}><ActionRow icon={ACTION_ICONS.markFolderRead} title={t('props-act-mark-read')} description={t('props-act-mark-read-help')} onRun={run(a()!.onMarkRead)}/></Show>
    <Show when={canRenameHere()}><ActionRow icon="edit" title={t('props-act-rename')} description={t('props-act-rename-help')} onRun={()=>{if(props.onStartRename)props.onStartRename();else run(a()!.onRename)();}}/></Show>
    <Show when={a()?.onMove}><ActionRow icon={ACTION_ICONS.moveTo} title={t('props-act-folder-move')} description={t('props-act-move-help')} onRun={run(a()!.onMove)}/></Show>
    <ActionRow icon={ACTION_ICONS.copyPath} title={t('props-act-copy-path')} description={t('props-act-copy-path-help')} onRun={()=>void copyText(props.box.mailwomanPath??props.box.name)}/>
    <Show when={a()?.onRetention}><ActionRow icon="archive" title={t('props-ovr-retention')} description={t('props-act-retention-help')} onRun={run(a()!.onRetention)}/></Show>
   </div>
  </section>
  <Show when={danger()}><section class={`${css.card} ${css.cardDanger}`} aria-labelledby="props-danger-title" data-testid="props-danger">
   <h3 id="props-danger-title" class={`${css.cardTitle} ${css.actionDanger}`}>{t('props-card-danger')}</h3>
   <p class={css.cardHelp}>{t('props-danger-help')}</p>
   <div>
    <Show when={a()?.onEmpty}><ActionRow danger icon={ACTION_ICONS.emptyFolder} title={t('props-act-empty')} description={t('props-act-empty-help')} onRun={run(a()!.onEmpty)}/></Show>
    <Show when={a()?.onDelete}><ActionRow danger icon={ACTION_ICONS.deleteFolder} title={t('props-act-delete')} description={t('props-act-delete-help')} onRun={run(a()!.onDelete)}/></Show>
   </div>
  </section></Show>
 </div>;
}
