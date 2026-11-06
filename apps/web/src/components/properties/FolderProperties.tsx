// The folder Properties dialog: general facts, the server location, size and
// counts, the offline cache, per-folder overrides of the user's settings, a note,
// and whole-folder actions (download as ZIP, empty, mark read, rename, delete).
// What shows is bounded by the administrator's `properties.*` and `export.*`
// settings (policy.ts); a locked setting shows its lock instead of a control.
import {For,Show,createEffect,createMemo,createResource,createSignal,createUniqueId,onCleanup,onMount,type JSX} from 'solid-js';
import {useApp} from '../../state/context.ts';
import {accountUiPreferences,setAccountUiPreferences} from '../../api/accountPreferences.ts';
import {createConfiguredClient} from '../../api/transport.ts';
import type {Mailbox} from '../../api/jmap-types.ts';
import {isolate,loadCatalog,t} from '../../i18n';
import {SecurityPopup} from '../SecurityPanel.tsx';
import {ConfirmDialog} from '../../screens/Admin/shell/ConfirmDialog.tsx';
import {Icon} from '../Icon.tsx';
import {toast} from '../toast/toast.ts';
import {fetchFolderStats,formatBytes,forgetFolderStats,type FolderStats} from './folderStats.ts';
import {downloadFolderZip,exportUrl,fetchExportEstimate,ExportRefused,IN_PAGE_LIMIT_BYTES,saveBlob,type ExportEstimate} from './folderExport.ts';
import {OVERRIDE_DEFS,effectiveOverride,withOverride,type FolderProps,type OverrideDef} from './folderOverrides.ts';
import {allowedOverrides,exportFormats,exportSubfoldersDefault,folderExportEnabled,folderSections,notesEnabled,notesMaxLength,offlineClearEnabled,remoteForDisplay,remoteVisibility,sizeCompute,type ExportFormat,type FolderSection} from './policy.ts';
import * as css from './properties.css.ts';

/** What the dialog can ask the folder menu to do; an action not supplied is not offered. */
export interface FolderDialogActions {
 onEmpty?:()=>void;onMarkRead?:()=>void;onRename?:()=>void;onDelete?:()=>void;onRetention?:()=>void;onRefresh?:()=>void;
}

const ROLE_KNOWN=new Set(['inbox','drafts','sent','junk','trash','archive']);
export function folderTypeKey(box:Pick<Mailbox,'role'|'mailwomanSearchQuery'>):string {
 if(box.mailwomanSearchQuery!==undefined)return 'virtual';
 return box.role&&ROLE_KNOWN.has(box.role)?box.role:'user';
}
const yesNo=(value:boolean|undefined)=>value===undefined?'':t(value?'props-yes':'props-no');
const dateText=(iso:string|null|undefined)=>{if(!iso)return t('props-none');const d=new Date(iso);return Number.isNaN(d.getTime())?iso:d.toLocaleString();};

export function FolderPropertiesDialog(props:{box:Mailbox;actions?:FolderDialogActions;onClose:()=>void}):JSX.Element {
 const app=useApp(),titleId=`props-${createUniqueId()}`;
 onMount(()=>{void loadCatalog('properties');void loadCatalog('mail');});
 const sections=createMemo(()=>folderSections());
 const [active,setActive]=createSignal<FolderSection>(sections()[0]??'general');
 createEffect(()=>{if(!sections().includes(active()))setActive(sections()[0]??'general');});
 const box=createMemo(()=>app.mailboxes().find(value=>value.id===props.box.id)??props.box);
 const tabRefs:HTMLButtonElement[]=[];
 function onTabKey(event:KeyboardEvent):void {
  const list=sections(),at=list.indexOf(active());
  const next=event.key==='ArrowRight'?(at+1)%list.length:event.key==='ArrowLeft'?(at-1+list.length)%list.length:event.key==='Home'?0:event.key==='End'?list.length-1:-1;
  if(next<0)return;event.preventDefault();setActive(list[next]!);tabRefs[next]?.focus();
 }
 return <SecurityPopup title={t('props-folder-title',{name:isolate(box().name)})} labelledBy={titleId} onClose={props.onClose}>
  <div class={`${css.body} ${css.phone}`} data-testid="folder-properties">
   <div class={css.tabs} role="tablist" aria-label={t('props-sections')} onKeyDown={onTabKey}>
    <For each={sections()}>{(section,index)=><button type="button" role="tab" id={`${titleId}-tab-${section}`} aria-selected={active()===section} aria-controls={`${titleId}-panel`} tabindex={active()===section?0:-1} class={css.tab} ref={el=>{tabRefs[index()]=el;}} onClick={()=>setActive(section)}>{t(`props-tab-${section}`)}</button>}</For>
   </div>
   <div class={css.panel} role="tabpanel" id={`${titleId}-panel`} aria-labelledby={`${titleId}-tab-${active()}`} tabindex="0">
    <Show when={active()==='general'}><General box={box()} account={app.accountId()??''} parent={app.mailboxes().find(value=>value.id===box().parentId)?.name} depth={depthOf(app.mailboxes(),box())}/></Show>
    <Show when={active()==='remote'}><Remote box={box()}/></Show>
    <Show when={active()==='size'}><Size box={box()}/></Show>
    <Show when={active()==='offline'}><Offline box={box()} onRefresh={props.actions?.onRefresh}/></Show>
    <Show when={active()==='overrides'}><Overrides box={box()} onRetention={props.actions?.onRetention ? ()=>{props.onClose();props.actions!.onRetention!();} : undefined}/></Show>
    <Show when={active()==='notes'}><Notes box={box()}/></Show>
    <Show when={active()==='actions'}><Actions box={box()} actions={props.actions} onClose={props.onClose}/></Show>
   </div>
  </div>
 </SecurityPopup>;
}

function depthOf(boxes:readonly Mailbox[],box:Mailbox):number {
 let depth=0,at:Mailbox|undefined=box;const seen=new Set<string>();
 while(at?.parentId&&!seen.has(at.id)){seen.add(at.id);at=boxes.find(value=>value.id===at!.parentId);if(at)depth++;}
 return depth;
}

function Pair(props:{label:string;children:JSX.Element}):JSX.Element {return <><dt>{props.label}</dt><dd>{props.children}</dd></>;}

function General(props:{box:Mailbox;account:string;parent:string|undefined;depth:number}):JSX.Element {
 const b=()=>props.box;
 return <dl class={css.grid} data-testid="props-general">
  <Pair label={t('props-name')}>{isolate(b().name)}</Pair>
  <Pair label={t('props-path')}>{isolate(b().mailwomanPath??b().name)}</Pair>
  <Pair label={t('props-type')}>{t(`props-role-${folderTypeKey(b())}`)}</Pair>
  <Pair label={t('props-subscribed')}>{b().isSubscribed===undefined?t('props-unknown'):yesNo(b().isSubscribed)}</Pair>
  <Pair label={t('props-parent')}>{props.parent?isolate(props.parent):t('props-top-level')}</Pair>
  <Pair label={t('props-depth')}>{props.depth}</Pair>
  <Pair label={t('props-folder-id')}><span class={css.mono}>{b().id}</span></Pair>
  <Pair label={t('props-account')}><span class={css.mono}>{props.account}</span></Pair>
 </dl>;
}

/** Stats for the open dialog, honouring `properties.folder.sizeCompute`. */
function useStats(box:()=>Mailbox):{stats:()=>FolderStats|undefined;loading:()=>boolean;failed:()=>boolean;compute:()=>void;mode:()=>string} {
 const app=useApp(),client=createConfiguredClient();
 const [wanted,setWanted]=createSignal(sizeCompute()==='on-open'||sizeCompute()==='background');
 const [stats,{refetch}]=createResource(()=>wanted()&&box().mailwomanSearchQuery===undefined?{account:app.accountId(),id:box().id}:null,async key=>{
  if(!key?.account)return undefined;
  return await fetchFolderStats(client,key.account,key.id);
 });
 return {stats:()=>stats.latest,loading:()=>stats.loading,failed:()=>stats.state==='errored',compute:()=>{if(wanted())void refetch();else setWanted(true);},mode:()=>sizeCompute()};
}

function Remote(props:{box:Mailbox}):JSX.Element {
 const s=useStats(()=>props.box);
 const shown=()=>{const stats=s.stats();return stats?remoteForDisplay(stats.remote,remoteVisibility()):null;};
 return <div data-testid="props-remote">
  <Show when={remoteVisibility()!=='hidden'} fallback={<p class={css.note}>{t('props-remote-hidden')}</p>}>
   <Show when={shown()} fallback={<p class={css.note} role="status">{s.loading()?t('props-loading'):s.failed()?t('props-unavailable'):t('props-remote-unavailable')}</p>}>{remote=>
    <dl class={css.grid}>
     <Pair label={t('props-remote-path')}><span class={css.mono}>{isolate(remote().path)}</span></Pair>
     <Show when={remote().host}><Pair label={t('props-remote-host')}><span class={css.mono}>{remote().host}</span></Pair></Show>
     <Show when={remote().delimiter}><Pair label={t('props-remote-delimiter')}><span class={css.mono}>{remote().delimiter}</span></Pair></Show>
     <Show when={remote().uidValidity!==undefined}><Pair label="UIDVALIDITY"><span class={css.mono}>{remote().uidValidity}</span></Pair></Show>
     <Show when={remote().uidNext!==undefined}><Pair label="UIDNEXT"><span class={css.mono}>{remote().uidNext}</span></Pair></Show>
     <Show when={remote().highestModSeq!==undefined}><Pair label="HIGHESTMODSEQ"><span class={css.mono}>{remote().highestModSeq}</span></Pair></Show>
     <Show when={remote().namespace}><Pair label={t('props-remote-namespace')}>{t(`props-ns-${remote().namespace}`)}</Pair></Show>
    </dl>}</Show>
  </Show>
 </div>;
}

function Size(props:{box:Mailbox}):JSX.Element {
 const s=useStats(()=>props.box);
 const st=()=>s.stats();
 return <div data-testid="props-size" aria-busy={s.loading()}>
  <dl class={css.grid}>
   <Pair label={t('props-total')}>{st()?.totalEmails??props.box.totalEmails}</Pair>
   <Pair label={t('props-unread')}>{st()?.unreadEmails??props.box.unreadEmails}</Pair>
   <Show when={st()}>{stats=><>
    <Pair label={t('props-flagged')}>{stats().flaggedEmails}</Pair>
    <Pair label={t('props-drafts')}>{stats().draftEmails}</Pair>
    <Pair label={t('props-size-server')}>{stats().approximate?t('props-approx-size',{size:formatBytes(stats().sizeBytes)}):formatBytes(stats().sizeBytes)}</Pair>
    <Pair label={t('props-oldest')}>{dateText(stats().oldest)}</Pair>
    <Pair label={t('props-newest')}>{dateText(stats().newest)}</Pair>
    <Show when={stats().largest.length}><Pair label={t('props-largest')}>{stats().largest.map(item=>formatBytes(item.size)).join(', ')}</Pair></Show>
   </>}</Show>
  </dl>
  <Show when={st()?.approximate}><p class={css.note} role="note">{t('props-approx-note',{cached:st()!.cachedEmails,total:st()!.totalEmails})}</p></Show>
  <Show when={s.loading()}><p class={css.note} role="status"><Icon name="sync"/> {t('props-computing')}</p></Show>
  <Show when={s.failed()}><p class={css.note} role="alert">{t('props-unavailable')}</p></Show>
  <Show when={!st()&&!s.loading()&&!s.failed()&&s.mode()==='off'}><p class={css.note}>{t('props-size-off')}</p><button type="button" class="btn btn--ghost" onClick={()=>s.compute()}>{t('props-size-compute')}</button></Show>
 </div>;
}

function Offline(props:{box:Mailbox;onRefresh?:(()=>void)|undefined}):JSX.Element {
 const app=useApp(),s=useStats(()=>props.box);
 const [confirming,setConfirming]=createSignal(false);
 const stats=()=>app.mailCacheStats();
 return <div data-testid="props-offline">
  <dl class={css.grid}>
   <Show when={s.stats()}>{st=><Pair label={t('props-offline-stored')}>{t('props-offline-stored-value',{count:st().bodiesCached,total:st().cachedEmails})}</Pair>}</Show>
   <Pair label={t('props-offline-session')}>{t('props-offline-session-value',{entries:stats().entries,bytes:formatBytes(stats().bytes)})}</Pair>
  </dl>
  <div class={css.actions}>
   <Show when={offlineClearEnabled()}><button type="button" class="btn btn--danger" data-testid="props-clear-cache" onClick={()=>setConfirming(true)}><Icon name="delete"/>{t('props-offline-clear')}</button></Show>
   <Show when={props.onRefresh}><button type="button" class="btn btn--ghost" onClick={()=>props.onRefresh!()}><Icon name="sync"/>{t('props-offline-prefetch')}</button></Show>
  </div>
  <ConfirmDialog open={confirming()} title={t('props-offline-clear-title',{name:isolate(props.box.name)})} body={t('props-offline-clear-body')} confirmLabel={t('props-offline-clear')} onCancel={()=>setConfirming(false)} onConfirm={()=>{app.clearFolderCache(props.box.id);forgetFolderStats(props.box.id);setConfirming(false);toast.success(t('props-offline-cleared'));}} testId="props-clear-confirm"/>
 </div>;
}

const OPTION_KEYS:Record<string,string>={none:'props-opt-none',unread:'props-opt-unread',total:'props-opt-total',both:'props-opt-both',on:'props-opt-on',off:'props-opt-off',vips:'props-opt-vips',date:'props-opt-date',from:'props-opt-from',subject:'props-opt-subject',size:'props-opt-size'};

function Overrides(props:{box:Mailbox;onRetention:(()=>void)|undefined}):JSX.Element {
 const prefs=()=>accountUiPreferences().folderProps??{};
 const own=()=>prefs()[props.box.id];
 const defs=createMemo(()=>OVERRIDE_DEFS.filter(def=>allowedOverrides().includes(def.key)));
 function set(patch:{[K in keyof FolderProps]?:FolderProps[K]|undefined}):void {setAccountUiPreferences({folderProps:withOverride(prefs(),props.box.id,patch)});}
 return <div data-testid="props-overrides">
  <p class={css.note}>{t('props-ovr-help')}</p>
  <For each={defs()}>{def=><OverrideRow def={def} props={own()} onSet={set}/>}</For>
  <Show when={props.onRetention}><button type="button" class="btn btn--ghost" onClick={()=>props.onRetention!()}><Icon name="archive"/>{t('props-ovr-retention')}</button></Show>
 </div>;
}

function OverrideRow(props:{def:OverrideDef;props:FolderProps|undefined;onSet:(patch:{[K in keyof FolderProps]?:FolderProps[K]|undefined})=>void}):JSX.Element {
 const id=`ovr-${createUniqueId()}`;
 const eff=()=>effectiveOverride(props.def,props.props,undefined,undefined);
 const prop=props.def.prop,type=props.def.type;
 const lockedTip=()=>t('props-src-admin-locked-tip');
 const choose=(raw:string)=>{
  if(raw==='')return props.onSet({[prop]:undefined} as Partial<FolderProps>);
  const value=type.kind==='bool'?raw==='true':type.kind==='lines'||type.kind==='delay'?Number(raw):raw;
  props.onSet({[prop]:value} as Partial<FolderProps>);
 };
 const shownValue=()=>{const v=eff().value;return v===undefined||v===''?t('props-none'):typeof v==='boolean'?t(v?'props-opt-on':'props-opt-off'):typeof v==='string'&&OPTION_KEYS[v]?t(OPTION_KEYS[v]!):String(v);};
 return <div class={css.overrideRow} data-testid={`ovr-${props.def.key}`} data-source={eff().source}>
  <label for={id}>{t(`props-ovr-${props.def.key}`)}</label>
  <div class={css.row}>
   <Show when={!eff().locked} fallback={<span class={css.locked} title={lockedTip()} data-testid={`ovr-${props.def.key}-locked`}><Icon name="lock"/>{shownValue()}</span>}>
    <Show when={type.kind==='enum'||type.kind==='bool'}>
     <select id={id} value={props.props?.[prop]===undefined?'':String(props.props[prop])} onChange={event=>choose(event.currentTarget.value)}>
      <option value="">{t('props-ovr-inherit')}</option>
      <For each={type.kind==='bool'?['true','false']:(type as {options:readonly string[]}).options}>{option=><option value={option}>{type.kind==='bool'?t(option==='true'?'props-opt-on':'props-opt-off'):t(OPTION_KEYS[option]??option)}</option>}</For>
     </select></Show>
    <Show when={type.kind==='colour'}><input id={id} type="color" value={props.props?.badgeColour??'#888888'} onInput={event=>choose(event.currentTarget.value)}/><Show when={props.props?.badgeColour}><button type="button" class="btn btn--ghost" onClick={()=>choose('')}>{t('props-ovr-inherit')}</button></Show></Show>
    <Show when={type.kind==='lines'}><select id={id} value={props.props?.previewLines===undefined?'':String(props.props.previewLines)} onChange={event=>choose(event.currentTarget.value)}><option value="">{t('props-ovr-inherit')}</option><For each={[0,1,2,3]}>{n=><option value={String(n)}>{String(n)}</option>}</For></select></Show>
    <Show when={type.kind==='delay'}><input id={id} type="number" min="0" max="60000" step="100" placeholder={t('props-ovr-inherit')} value={props.props?.markReadDelayMs??''} onChange={event=>choose(event.currentTarget.value)}/></Show>
    <Show when={type.kind==='identity'}><input id={id} type="text" maxlength="256" placeholder={t('props-ovr-inherit')} value={props.props?.replyIdentity??''} onChange={event=>choose(event.currentTarget.value.trim())}/></Show>
   </Show>
   <span class={css.source} data-testid={`ovr-${props.def.key}-source`}>{t('props-ovr-effective',{value:shownValue(),source:t(`props-src-${eff().source}`)})}</span>
  </div>
 </div>;
}

function Notes(props:{box:Mailbox}):JSX.Element {
 const saved=()=>accountUiPreferences().folderProps?.[props.box.id]?.notes??'';
 const [draft,setDraft]=createSignal(saved());
 const max=()=>notesMaxLength();
 const dirty=()=>draft()!==saved();
 const id=`notes-${createUniqueId()}`;
 function save():void {
  const all=accountUiPreferences().folderProps??{};
  setAccountUiPreferences({folderProps:withOverride(all,props.box.id,{notes:draft().trim()===''?undefined:draft().slice(0,max())})});
  toast.success(t('props-notes-saved'));
 }
 return <Show when={notesEnabled()}><div data-testid="props-notes">
  <label for={id}>{t('props-notes-label')}</label>
  <textarea id={id} class={css.textarea} maxlength={max()} value={draft()} onInput={event=>setDraft(event.currentTarget.value)} aria-describedby={`${id}-count`}/>
  <div class={css.row}><span id={`${id}-count`} class={css.note}>{t('props-notes-count',{count:draft().length,max:max()})}</span>
   <button type="button" class="btn btn--primary" disabled={!dirty()} onClick={save}>{t('props-notes-save')}</button></div>
 </div></Show>;
}

function Actions(props:{box:Mailbox;actions:FolderDialogActions|undefined;onClose:()=>void}):JSX.Element {
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
 async function copyPath():Promise<void> {
  try{await navigator.clipboard.writeText(props.box.mailwomanPath??props.box.name);toast.success(t('props-act-copied'));}catch{toast.error(t('props-act-copy-failed'));}
 }
 const run=(fn?:()=>void)=>()=>{props.onClose();fn?.();};
 return <div data-testid="props-actions">
  <Show when={folderExportEnabled()}><section aria-labelledby="props-zip-title">
   <h4 id="props-zip-title">{t('props-act-zip')}</h4>
   <div class={css.row}>
    <Show when={exportFormats().length>1}><label>{t('props-act-zip-format')} <select value={format()} onChange={event=>{setFormat(event.currentTarget.value as ExportFormat);setEstimate(null);}}><For each={exportFormats()}>{f=><option value={f}>{t(`props-act-format-${f}`)}</option>}</For></select></label></Show>
    <Show when={hasSub()}><label><input type="checkbox" checked={subfolders()} onChange={event=>{setSubfolders(event.currentTarget.checked);setEstimate(null);}}/> {t('props-act-zip-sub')}</label></Show>
    <button type="button" class="btn btn--ghost" data-testid="props-zip-check" onClick={()=>void check()}>{t('props-act-zip-estimate')}</button>
   </div>
   <Show when={estimate()}>{est=><p role="status" class={css.note} data-testid="props-zip-estimate">{est().allowed?t('props-act-zip-estimate-ok',{messages:est().messages,bytes:formatBytes(est().bytes)}):t('props-act-zip-estimate-refused',{messages:est().messages,bytes:formatBytes(est().bytes),max:formatBytes(est().maxBytes)})}</p>}</Show>
   <Show when={problem()}><p role="alert" class={css.danger} data-testid="props-zip-problem">{t(`props-act-problem-${problem()}`)}</p></Show>
   <div class={css.row}>
    <button type="button" class="btn btn--primary" data-testid="props-zip-start" disabled={!estimate()?.allowed||received()!==null} onClick={()=>void start()}><Icon name="download"/>{t('props-act-zip-start')}</button>
    <Show when={received()!==null}><span role="status">{t('props-act-progress',{bytes:formatBytes(received()!)})}</span><button type="button" class="btn btn--ghost" data-testid="props-zip-cancel" onClick={()=>controller?.abort()}>{t('props-act-cancel')}</button></Show>
   </div>
  </section></Show>
  <div class={css.actions}>
   <Show when={props.actions?.onMarkRead}><button type="button" class="btn btn--ghost" onClick={run(props.actions!.onMarkRead)}>{t('props-act-mark-read')}</button></Show>
   <Show when={props.actions?.onEmpty}><button type="button" class="btn btn--ghost" onClick={run(props.actions!.onEmpty)}>{t('props-act-empty')}</button></Show>
   <Show when={props.actions?.onRename}><button type="button" class="btn btn--ghost" onClick={run(props.actions!.onRename)}>{t('props-act-rename')}</button></Show>
   <button type="button" class="btn btn--ghost" onClick={()=>void copyPath()}><Icon name="clone"/>{t('props-act-copy-path')}</button>
   <Show when={props.actions?.onDelete}><button type="button" class="btn btn--danger" onClick={run(props.actions!.onDelete)}><Icon name="delete"/>{t('props-act-delete')}</button></Show>
  </div>
 </div>;
}
