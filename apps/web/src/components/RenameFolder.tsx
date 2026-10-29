// "Rename…" for a user folder (t65): inline in the folder list, a small modal on
// phones, in the compact folder rail and in a folder tab.
//
// Inline: the folder's name becomes a text field with the name selected. Enter
// saves, Escape cancels, leaving the field saves. A name the server would refuse
// (empty, too long, the hierarchy delimiter, a sibling's name) is said under the
// field before any request, and a refusal from the server is said the same way;
// the field stays open until the name is fixed or the edit is cancelled. The
// folder keeps its id, so its subfolders, counts, tabs and pin follow it.
import {createSignal,createUniqueId,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {Portal} from 'solid-js/web';
import {useApp} from '../state/context.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {canRename,folderDelimiter,nameProblem,renameAbility,renameFolder,type NameProblem,type RenameAbility,type RenameOutcome} from '../api/renameFolder.ts';
import type {Mailbox} from '../api/jmap-types.ts';
import {isolate,loadCatalog,t} from '../i18n';
import {createFocusTrap} from './a11y';
import {NARROW_QUERY} from './narrowViewport.ts';
import * as shell from '../screens/Admin/shell/shell.css.ts';
import * as css from './renameFolder.css.ts';

/** The account's rename ability plus its hierarchy delimiter. */
export interface RenameContext extends RenameAbility {delimiter:string|null}

const [contexts,setContexts]=createSignal<Readonly<Record<string,RenameContext>>>({});
const loading=new Set<string>();

/** This account's rename context (null until known). Reading it starts the fetch. */
export function useRenameContext():()=>RenameContext|null {
 const app=useApp();
 return ()=>{
  const owner=preferenceAccount(),accountId=app.accountId();if(!owner||!accountId||owner.id!==accountId)return null;
  const key=`${accountId}:${owner.generation}`,known=contexts()[key];
  if(known)return known;
  if(!loading.has(key)){
   loading.add(key);
   const client=createConfiguredClient();
   void Promise.resolve().then(async()=>{
    const ability=renameAbility(await client.session(),accountId);
    const delimiter=ability.manages?await folderDelimiter(client,accountId).catch(()=>null):null;
    if(preferenceAccountIsCurrent(owner))setContexts(all=>({...all,[key]:{...ability,delimiter}}));
   }).catch(()=>{}).finally(()=>loading.delete(key));
  }
  return null;
 };
}

/** Whether to offer "Rename…" for `box` in this account now. */
export function useCanRename():(box:Mailbox)=>boolean {
 const context=useRenameContext();
 return box=>canRename(box,context());
}

/** Inline editing does not fit a phone or the icon-only folder rail. */
export function prefersRenameDialog(row:Element|null|undefined):boolean {
 if(typeof matchMedia==='function'&&matchMedia(NARROW_QUERY).matches)return true;
 return !!row?.closest('[data-folder-mode="compact"]');
}

export function problemText(problem:NameProblem|'forbidden'|'notFound'|'failed',name:string,context:RenameContext):string {
 switch(problem){
  case 'empty':return t('rename-folder-error-empty');
  case 'long':return t('rename-folder-error-long',{max:context.maxLength});
  case 'separator':return t('rename-folder-error-separator',{separator:context.delimiter??''});
  case 'reserved':return t('rename-folder-error-reserved');
  case 'duplicate':return t('rename-folder-error-duplicate',{name:isolate(name)});
  case 'forbidden':return t('rename-folder-error-forbidden');
  case 'notFound':return t('rename-folder-error-not-found');
  default:return t('rename-folder-error-failed');
 }
}

/** Validate and send one rename. Resolves to the error to show, or null once
 *  the folder has the new name (or the name did not change). */
function useSubmit(box:()=>Mailbox,context:()=>RenameContext):(name:string)=>Promise<string|null> {
 const app=useApp(),owner=preferenceAccount(),accountId=app.accountId(),client=createConfiguredClient(),controller=new AbortController();
 onCleanup(()=>controller.abort());
 const current=()=>!!owner&&preferenceAccountIsCurrent(owner)&&app.accountId()===accountId;
 return async name=>{
  const target=app.mailboxes().find(item=>item.id===box().id)??box();
  if(name===target.name)return null;
  const problem=nameProblem(name,target,app.mailboxes(),context().delimiter,context().maxLength);
  if(problem)return problemText(problem,name,context());
  if(!accountId||!current())return t('rename-folder-error-failed');
  let outcome:RenameOutcome;
  try{outcome=await renameFolder(client,accountId,target.id,name,{current,signal:controller.signal});}
  catch{return controller.signal.aborted?null:t('rename-folder-error-failed');}
  if(!outcome.ok)return problemText(outcome.problem,name,context());
  if(current()){await app.refreshFolders().catch(()=>{});app.showToast('success',t('rename-folder-done',{name:isolate(name)}));}
  return null;
 };
}

/** The folder's name as a text field, in place of the folder row. */
export function FolderRenameField(props:{box:Mailbox;context:RenameContext;onDone:(saved:boolean)=>void}):JSX.Element {
 const id=createUniqueId();
 const submit=useSubmit(()=>props.box,()=>props.context);
 const [error,setError]=createSignal('');const [busy,setBusy]=createSignal(false);
 // Read once: a count update replaces the folder object and must not reset what is typed.
 const initial=props.box.name;
 let input:HTMLInputElement|undefined,settled=false;
 onMount(()=>{void loadCatalog('rename-folder');queueMicrotask(()=>{input?.focus();input?.select();});});
 function finish(saved:boolean):void {if(settled)return;settled=true;props.onDone(saved);}
 async function save():Promise<void>{
  if(settled||busy()||!input)return;
  const value=input.value;
  setBusy(true);const failure=await submit(value);setBusy(false);
  if(failure===null){finish(true);return;}
  setError(failure);queueMicrotask(()=>input?.focus());
 }
 return <div class={css.field} data-folder-rename>
  <input ref={input} id={id} class={css.input} type="text" value={initial} autocomplete="off" spellcheck={false} disabled={busy()} aria-busy={busy()?'true':undefined}
   aria-label={t('rename-folder-input',{name:isolate(props.box.name)})} aria-invalid={error()?'true':undefined} aria-describedby={error()?`${id}-error`:undefined}
   onInput={()=>{if(error())setError('');}}
   onKeyDown={event=>{event.stopPropagation();
    if(event.key==='Enter'){event.preventDefault();void save();}
    else if(event.key==='Escape'){event.preventDefault();finish(false);}
   }}
   onBlur={()=>{if(!error()&&!busy())void save();}}/>
  <Show when={error()}><span id={`${id}-error`} role="alert" class={css.error}>{error()}</span></Show>
 </div>;
}

/** The same edit as a small modal: phones, the compact rail, a folder tab. */
export function RenameFolderDialog(props:{box:Mailbox;context:RenameContext;onClose:(saved:boolean)=>void;mount?:Node}):JSX.Element {
 const id=createUniqueId();
 const submit=useSubmit(()=>props.box,()=>props.context);
 const [value,setValue]=createSignal(props.box.name);const [error,setError]=createSignal('');const [busy,setBusy]=createSignal(false);
 const [el,setEl]=createSignal<HTMLElement>();let input:HTMLInputElement|undefined;
 onMount(()=>{void loadCatalog('rename-folder');queueMicrotask(()=>input?.select());});
 const cancel=()=>{if(!busy())props.onClose(false);};
 createFocusTrap(el,{onEscape:cancel,initialFocus:()=>input});
 async function save(event:Event):Promise<void>{
  event.preventDefault();if(busy())return;
  setBusy(true);const failure=await submit(value());setBusy(false);
  if(failure===null){props.onClose(true);return;}
  setError(failure);queueMicrotask(()=>input?.focus());
 }
 return <Portal {...(props.mount?{mount:props.mount}:{})}>
  <div class={shell.dialogScrim} onClick={cancel}/>
  <form ref={setEl} class={shell.dialog} role="dialog" aria-modal="true" aria-labelledby={`${id}-title`} tabindex="-1" data-testid="rename-folder-dialog" onSubmit={event=>void save(event)}>
   <h2 id={`${id}-title`} class={shell.dialogTitle}>{t('rename-folder-title',{name:isolate(props.box.name)})}</h2>
   <label class={css.dialogField} for={`${id}-name`}>
    <span>{t('rename-folder-name')}</span>
    <input ref={input} id={`${id}-name`} class={css.input} type="text" value={value()} autocomplete="off" spellcheck={false} disabled={busy()}
     aria-invalid={error()?'true':undefined} aria-describedby={error()?`${id}-error`:undefined}
     onInput={event=>{setValue(event.currentTarget.value);if(error())setError('');}}/>
   </label>
   <Show when={error()}><p id={`${id}-error`} role="alert" class={css.error}>{error()}</p></Show>
   <div class={shell.dialogActions}>
    <button type="button" class="btn btn--ghost" disabled={busy()} onClick={cancel}>{t('rename-folder-cancel')}</button>
    <button type="submit" class="btn btn--primary" disabled={busy()}>{t(busy()?'rename-folder-saving':'rename-folder-save')}</button>
   </div>
  </form>
 </Portal>;
}
