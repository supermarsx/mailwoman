import {onLinkedMailboxChange} from '../../api/linkedMailboxSession.ts';
// The in-place Export dialog (reader toolbar, message menus, bulk bar).
//
// The context decides the scopes: this message, its conversation, the selected
// messages, the whole folder. Everything the administrator limits is applied
// here and again on the server: the format list, the message and byte caps, the
// download switch. A refusal is shown in the dialog with its reason; the step-up
// the server asks for is the shared StepUpDialog, raised by apiFetch. Nothing
// here leaves the page except through apiFetch.
import {For,Show,createEffect,createMemo,createSignal,createUniqueId,on,onCleanup,onMount,type JSX} from 'solid-js';
import {SecurityPopup} from '../SecurityPanel.tsx';
import {Icon} from '../Icon.tsx';
import {Spinner} from '../spinner/Spinner.tsx';
import {toast} from '../toast/toast.ts';
import {formatBytes} from '../properties/folderStats.ts';
import {fetchExportEstimate,ExportRefused,saveBlob,type ExportEstimate} from '../properties/folderExport.ts';
import * as shared from '../properties/properties.css.ts';
import {loadCatalog,t,isolate} from '../../i18n';
import * as css from './exportDialog.css.ts';
import {DEFAULT_MAX_NAME,DEFAULT_PATTERN,estimateFor,exportPolicy,fileNames,MAX_NAME,MIN_NAME,offeredScopes,packagings,scopeFormats,startScope,EXT,type ExportFormatId,type ExportScope,type Packaging,type ScopeSource} from './exportPlan.ts';
import {ExportError,runExport,type Progress,type RunResult} from './exportRun.ts';

type Phase='idle'|'running'|'done'|'error';
const KNOWN_PROBLEMS=['exportDisabled','exportFormatNotAllowed','exportTooManyMessages','exportTooLarge','exportPolicyUnavailable','exportRequiresEngine','legalHold','processingRestricted','attachmentContainerRestricted','folderNotFound','rateLimited'] as const;
/** The text for a refusal code from the server (or a status), falling back to a generic failure. */
export function problemKey(error:unknown):string {
  if(error instanceof ExportError){
    if((KNOWN_PROBLEMS as readonly string[]).includes(error.code))return `export-problem-${error.code}`;
    if(error.status===501)return 'export-problem-exportRequiresEngine';
    if(error.status===429)return 'export-problem-rateLimited';
    if(error.status===403)return 'export-problem-forbidden';
    if(error.status===404)return 'export-problem-gone';
  }
  return 'export-problem-failed';
}

export function ExportDialog(props:{source:ScopeSource;scope?:ExportScope;onOpenSettings?:()=>void;onClose:()=>void}):JSX.Element {
  const titleId=`export-${createUniqueId()}`,policy=exportPolicy();
  onMount(()=>{void loadCatalog('export-dialog');});
  const scopes=offeredScopes(props.source,policy);
  const [scope,setScope]=createSignal<ExportScope>(startScope(scopes,props.scope??policy.defaultScope));
  const formats=createMemo(()=>scopeFormats(scope(),policy));
  const [format,setFormat]=createSignal<ExportFormatId>(policy.defaultFormat);
  const [packaging,setPackaging]=createSignal<Packaging>('single');
  const [pattern,setPattern]=createSignal(DEFAULT_PATTERN);
  const [maxName,setMaxName]=createSignal(DEFAULT_MAX_NAME);
  const [stripReceived,setStripReceived]=createSignal(false);
  const [redactBcc,setRedactBcc]=createSignal(false);
  const [subfolders,setSubfolders]=createSignal(props.source.folder?.hasSubfolders===true);
  const [phase,setPhase]=createSignal<Phase>('idle');
  const [progress,setProgress]=createSignal<Progress>({done:0,total:0,bytes:0});
  const [problem,setProblem]=createSignal<string|null>(null);
  const [result,setResult]=createSignal<RunResult|null>(null);
  const [folderEstimate,setFolderEstimate]=createSignal<ExportEstimate|null>(null);
  const [folderProblem,setFolderProblem]=createSignal<string|null>(null);
  let controller:AbortController|undefined;
  onCleanup(()=>controller?.abort());
  onCleanup(onLinkedMailboxChange(()=>{controller?.abort();props.onClose();}));

  const emails=createMemo(()=>scope()==='conversation'?props.source.conversation??[]:scope()==='selection'?props.source.selection??[]:scope()==='message'?props.source.message:[]);
  // The format falls back when a scope switch makes the chosen one unavailable.
  createEffect(()=>{const first=formats()[0];if(first&&!formats().includes(format()))setFormat(first);});
  const allowedPackaging=createMemo(()=>packagings(scope(),format(),scope()==='folder'?0:emails().length));
  createEffect(()=>{const list=allowedPackaging();if(list.length>0&&!list.includes(packaging()))setPackaging(list[0]!);});

  // A folder is sized by the server; the other scopes by the loaded rows.
  createEffect(on([scope,format,subfolders],()=>{
    setFolderEstimate(null);setFolderProblem(null);
    const folder=props.source.folder;
    if(scope()!=='folder'||!folder||(format()!=='eml'&&format()!=='mbox'))return;
    const lookup=new AbortController();onCleanup(()=>lookup.abort());
    fetchExportEstimate(folder.id,{format:format() as 'eml'|'mbox',subfolders:subfolders()},lookup.signal)
      .then(setFolderEstimate)
      .catch((error:unknown)=>{if((error as {name?:string}).name!=='AbortError')setFolderProblem(problemKey(error instanceof ExportRefused?new ExportError(error.code,error.status):error));});
  }));

  const estimate=createMemo(()=>{
    if(scope()==='folder'){
      const e=folderEstimate();
      return e?{messages:e.messages,bytes:e.bytes,allowed:e.allowed,...(e.allowed?{}:{reason:e.messages>e.maxMessages?'tooManyMessages' as const:'tooLarge' as const})}:null;
    }
    return estimateFor(emails(),policy);
  });
  const refusal=():string|null=>{
    if(!policy.downloadEnabled)return 'export-problem-exportDisabled';
    if(formats().length===0)return 'export-problem-exportFormatNotAllowed';
    const verdict=estimate();
    if(verdict&&!verdict.allowed)return verdict.reason==='tooManyMessages'?'export-problem-exportTooManyMessages':verdict.reason==='tooLarge'?'export-problem-exportTooLarge':'export-problem-nothing';
    return folderProblem();
  };
  const example=createMemo(()=>{const first=emails()[0];return first&&format()!=='pdf'?fileNames([first],pattern(),EXT[format()],maxName())[0]:undefined;});
  const canStart=()=>phase()!=='running'&&format()!=='pdf'&&refusal()===null&&(scope()!=='folder'||folderEstimate()!==null);
  const label=createMemo(()=>scope()==='folder'?props.source.folder?.name??'folder':(emails()[0]?.subject?.trim()||t('export-default-label')));

  async function start():Promise<void> {
    if(!canStart())return;
    controller?.abort();controller=new AbortController();
    setPhase('running');setProblem(null);setProgress({done:0,total:emails().length,bytes:0});
    try{
      const out=await runExport({
        scope:scope(),format:format(),packaging:allowedPackaging().includes(packaging())?packaging():allowedPackaging()[0]??'zip',emails:emails(),
        ...(props.source.folder?{folder:props.source.folder}:{}),subfolders:subfolders(),pattern:pattern(),maxName:maxName(),
        privacy:{stripReceived:stripReceived(),redactBcc:redactBcc()},label:label(),
      },setProgress,controller.signal);
      setResult(out);saveBlob(out.blob,out.name);setPhase('done');
      toast.success(t('export-done-toast',{name:isolate(out.name)}));
    }catch(error){
      if((error as {name?:string}).name==='AbortError'){setPhase('idle');return;}
      setProblem(problemKey(error));setPhase('error');
    }
  }
  const cancel=()=>{controller?.abort();};
  const close=()=>{controller?.abort();props.onClose();};
  const scopeLabel=(value:ExportScope)=>value==='message'?t('export-scope-message'):value==='conversation'?t('export-scope-conversation',{count:props.source.conversation?.length??0}):value==='selection'?t('export-scope-selection',{count:props.source.selection?.length??0}):t('export-scope-folder',{name:isolate(props.source.folder?.name??'')});
  const exporting=()=>phase()==='running';
  const percent=()=>progress().total>0?Math.round(progress().done/progress().total*100):0;

  return <SecurityPopup title={t('export-title')} labelledBy={titleId} onClose={close}>
    <div class={`${shared.body} ${shared.phone}`} data-testid="export-dialog">
      <div class={`${css.body} ${shared.panel}`}>
        <Show when={scopes.length>1}>
          <fieldset class={css.section} disabled={exporting()}>
            <legend class={css.legend}>{t('export-scope')}</legend>
            <div class={css.scopes} role="radiogroup" aria-label={t('export-scope')}>
              <For each={scopes}>{value=><label class={css.scope}><input type="radio" name={`${titleId}-scope`} value={value} checked={scope()===value} data-testid={`export-scope-${value}`} onChange={()=>{setScope(value);setPhase('idle');}}/>{scopeLabel(value)}</label>}</For>
            </div>
          </fieldset>
        </Show>
        <Show when={scopes.length===1}><p class={shared.note} data-testid="export-scope-only">{scopeLabel(scopes[0]!)}</p></Show>

        <fieldset class={css.section} disabled={exporting()}>
          <legend class={css.legend}>{t('export-format')}</legend>
          <Show when={formats().length>0} fallback={<p class={shared.danger} role="alert" data-testid="export-no-formats">{t('export-problem-exportFormatNotAllowed')}</p>}>
            <label class={css.field}>{t('export-format')}
              <select class={css.input} value={format()} data-testid="export-format" onChange={event=>{setFormat(event.currentTarget.value as ExportFormatId);setPhase('idle');}}>
                <For each={formats()}>{value=><option value={value} selected={value===format()}>{t(`export-format-${value}`)}</option>}</For>
              </select>
            </label>
            <Show when={format()==='pdf'}><p class={shared.note} data-testid="export-pdf-hint">{t('export-pdf-hint')}</p></Show>
            <Show when={scope()!=='folder'&&format()==='csv'}><p class={shared.note}>{t('export-csv-note')}</p></Show>
          </Show>
        </fieldset>

        <Show when={format()!=='pdf'&&formats().length>0}>
          <fieldset class={css.section} disabled={exporting()}>
            <legend class={css.legend}>{t('export-packaging')}</legend>
            <div class={css.scopes} role="radiogroup" aria-label={t('export-packaging')}>
              <For each={allowedPackaging()}>{value=><label class={css.scope}><input type="radio" name={`${titleId}-pack`} value={value} checked={packaging()===value} data-testid={`export-packaging-${value}`} onChange={()=>setPackaging(value)}/>{t(`export-packaging-${value}`)}</label>}</For>
            </div>
            <Show when={scope()==='folder'}>
              <Show when={props.source.folder?.hasSubfolders}><label><input type="checkbox" checked={subfolders()} data-testid="export-subfolders" onChange={event=>setSubfolders(event.currentTarget.checked)}/> {t('export-subfolders')}</label></Show>
            </Show>
            <Show when={scope()!=='folder'&&(packaging()==='zip'||(format()!=='mbox'&&format()!=='csv'&&format()!=='json'))}>
              <label class={css.field}>{t('export-pattern')}
                <input class={css.input} type="text" value={pattern()} spellcheck={false} data-testid="export-pattern" aria-describedby={`${titleId}-pattern-help`} onInput={event=>setPattern(event.currentTarget.value)}/>
              </label>
              <label class={css.field}>{t('export-max-name')}
                <input class={css.input} type="number" min={MIN_NAME} max={MAX_NAME} value={maxName()} data-testid="export-max-name" onInput={event=>{const n=Number(event.currentTarget.value);setMaxName(Number.isFinite(n)?Math.max(MIN_NAME,Math.min(MAX_NAME,Math.floor(n))):DEFAULT_MAX_NAME);}}/>
              </label>
              <p id={`${titleId}-pattern-help`} class={shared.note}>{t('export-pattern-help')}</p>
              <Show when={example()}><p class={shared.note} data-testid="export-example">{t('export-example',{name:isolate(example()!)})}</p></Show>
            </Show>
          </fieldset>

          <Show when={scope()!=='folder'&&(format()==='eml'||format()==='mbox'||format()==='json')}>
            <fieldset class={css.section} disabled={exporting()}>
              <legend class={css.legend}>{t('export-privacy')}</legend>
              <Show when={format()==='eml'||format()==='mbox'}><label><input type="checkbox" checked={stripReceived()} data-testid="export-strip-received" onChange={event=>setStripReceived(event.currentTarget.checked)}/> {t('export-strip-received')}</label></Show>
              <label><input type="checkbox" checked={redactBcc()} data-testid="export-redact-bcc" onChange={event=>setRedactBcc(event.currentTarget.checked)}/> {t('export-redact-bcc')}</label>
              <Show when={stripReceived()||redactBcc()}><p class={shared.note}>{t('export-privacy-note')}</p></Show>
            </fieldset>
          </Show>
        </Show>

        <div role="status" aria-live="polite" data-testid="export-estimate">
          <Show when={estimate()} fallback={<Show when={scope()==='folder'&&refusal()===null}><span class={shared.note}><Spinner label={t('export-estimating')}/> {t('export-estimating')}</span></Show>}>
            {value=><p class={shared.note}>{t('export-estimate',{count:value().messages,size:formatBytes(value().bytes)})}</p>}
          </Show>
        </div>
        <Show when={refusal()}>{key=><p class={`${css.verdict} ${shared.danger}`} role="alert" data-testid="export-refusal">{t(key())}</p>}</Show>

        <Show when={exporting()}>
          <div data-testid="export-progress">
            <Show when={scope()!=='folder'} fallback={<p role="status"><Spinner label={t('export-running')}/> {t('export-progress-bytes',{size:formatBytes(progress().bytes)})}</p>}>
              <progress class={css.progress} max={Math.max(1,progress().total)} value={progress().done} aria-label={t('export-running')}/>
              <p role="status" class={shared.note}>{t('export-progress',{done:progress().done,total:progress().total,percent:percent()})}</p>
            </Show>
          </div>
        </Show>
        <Show when={phase()==='error'}>
          <div class={css.verdict} role="alert" data-testid="export-error">
            <p class={shared.danger}>{t(problem()??'export-problem-failed')}</p>
            <button type="button" class="btn btn--ghost" data-testid="export-retry" onClick={()=>void start()}>{t('export-retry')}</button>
          </div>
        </Show>
        <Show when={phase()==='done'&&result()}>{out=>
          <p role="status" data-testid="export-result">{t('export-done',{name:isolate(out().name)})} <button type="button" class="btn btn--ghost" data-testid="export-again" onClick={()=>saveBlob(out().blob,out().name)}>{t('export-again')}</button></p>
        }</Show>
      </div>
      <div class={css.footer}>
        <Show when={props.onOpenSettings}><a href="#" data-testid="export-more" onClick={event=>{event.preventDefault();props.onClose();props.onOpenSettings?.();}}>{t('export-more')}</a></Show>
        <div class={shared.actions}>
          <Show when={exporting()} fallback={<button type="button" class="btn btn--ghost" data-testid="export-close" onClick={close}>{t(phase()==='done'?'export-close':'export-cancel')}</button>}>
            <button type="button" class="btn btn--ghost" data-testid="export-cancel" onClick={cancel}>{t('export-cancel-run')}</button>
          </Show>
          <button type="button" class="btn btn--primary" data-testid="export-start" disabled={!canStart()} onClick={()=>void start()}><Icon name="download"/>{t('export-start')}</button>
        </div>
      </div>
    </div>
  </SecurityPopup>;
}
