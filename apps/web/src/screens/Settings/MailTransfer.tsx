import {createEffect,createSignal,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {mailTransferApi,MailTransferError,type MailTransferApi,type MailImportFormat,type MailExportFormat,type MailImportFormats,type MailImportResult} from '../../api/mailTransfer.ts';
import {loadCatalog,t} from '../../i18n';
import {AsyncPending} from '../../components/AsyncState.tsx';
import {useEditorGuard,useEntityNavigation} from './entityEditor.tsx';
import * as css from './styles.css.ts';
import * as layout from './MailTransfer.css.ts';

export interface TransferMailbox {id:string;name:string}
export function MailTransfer(props:{accountId:string;mailboxes:readonly TransferMailbox[];api?:MailTransferApi;onImported?:()=>void}):JSX.Element {
  const nav=useEntityNavigation('mail-transfer');
  onMount(()=>{void loadCatalog('mail-transfer');});
  return <section class={layout.section}>
    <Show when={nav.route().editor?.kind==='edit'&&nav.route().editor?.id} fallback={<Show when={nav.route().editor?.kind==='new'} fallback={<>
      <h2 class={css.heading}>{t('mail-transfer-title')}</h2><p>{t('mail-transfer-summary')}</p>
      <button class={css.button} onClick={()=>void nav.navigate({section:'mail-transfer',editor:{kind:'new'}})}>{t('mail-transfer-import-open')}</button>
      <p class={css.meta}>{t('mail-transfer-export-help')}</p>
    </>}>
      <MailImportEditor accountId={props.accountId} mailboxes={props.mailboxes} api={props.api??mailTransferApi}
        onImported={()=>props.onImported?.()} onBack={()=>nav.navigate({section:'mail-transfer'}).then(()=>undefined)}/>
    </Show>}>{id=><MailExportEditor accountId={props.accountId} emailId={id()} api={props.api??mailTransferApi} onBack={()=>nav.navigate({section:'mail-transfer'}).then(()=>undefined)}/>}</Show>
  </section>;
}

export function MailExportEditor(props:{accountId:string;emailId:string;api:MailTransferApi;onBack:()=>Promise<void>}):JSX.Element {
  onMount(()=>{void loadCatalog('mail-transfer');});
  const [format,setFormat]=createSignal<MailExportFormat>('eml');const [busy,setBusy]=createSignal(false);const [error,setError]=createSignal('');const [saved,setSaved]=createSignal(false);
  let controller:AbortController|undefined;let generation=0;
  createEffect(()=>{void props.accountId;void props.emailId;generation++;controller?.abort();setBusy(false);setError('');setSaved(false);});
  onCleanup(()=>{generation++;controller?.abort();});
  async function download():Promise<void>{
    if(busy())return;const epoch=generation;const selected=format();controller=new AbortController();setBusy(true);setError('');setSaved(false);
    try{
      const blob=await props.api.export(props.emailId,selected,controller.signal);if(epoch!==generation)return;
      const url=URL.createObjectURL(blob);const link=document.createElement('a');link.href=url;link.download=`message.${selected}`;link.click();setTimeout(()=>URL.revokeObjectURL(url),0);setSaved(true);
    }catch(reason){if(epoch===generation)setError(reason instanceof MailTransferError&&reason.status===403?'mail-transfer-export-forbidden':reason instanceof MailTransferError&&reason.status===501?'mail-transfer-export-unavailable':'mail-transfer-export-error');}
    finally{if(epoch===generation)setBusy(false);}
  }
  return <><h2 class={css.heading}>{t('mail-transfer-export-title')}</h2><p>{t('mail-transfer-export-detail')}</p>
    <label class={layout.label}>{t('mail-transfer-format')}<select disabled={busy()} class={css.input} value={format()} onChange={event=>setFormat(event.currentTarget.value as MailExportFormat)}>
      <option value="eml">EML (.eml)</option><option value="mbox">mbox</option><option value="txt">Text (.txt)</option><option value="md">Markdown (.md)</option><option value="msg">Outlook (.msg)</option><option value="docx">Word (.docx)</option>
    </select></label>
    <button class={css.button} disabled={busy()} onClick={()=>void download()}>{t(busy()?'mail-transfer-exporting':'mail-transfer-export-submit')}</button>
    <Show when={saved()}><p role="status">{t('mail-transfer-export-saved')}</p></Show><Show when={error()}><p role="alert">{t(error())}</p></Show>
    <button class={css.ghost} disabled={busy()} onClick={()=>void props.onBack()}>{t('mail-transfer-back')}</button>
  </>;
}

export function MailImportEditor(props:{accountId:string;mailboxes:readonly TransferMailbox[];api:MailTransferApi;onImported:()=>void;onBack:()=>Promise<void>}):JSX.Element {
  const [formats,setFormats]=createSignal<MailImportFormats>();
  const [format,setFormat]=createSignal<MailImportFormat>('eml');
  const [mailbox,setMailbox]=createSignal('');
  const [file,setFile]=createSignal<File>();
  const [busy,setBusy]=createSignal(false);
  const [error,setError]=createSignal('');
  const [result,setResult]=createSignal<MailImportResult>();
  const guard=useEditorGuard(()=>!!file(),busy);
  let controller:AbortController|undefined;let generation=0;let input:HTMLInputElement|undefined;
  createEffect(()=>{
    const account=props.accountId;const epoch=++generation;
    controller?.abort();controller=new AbortController();setFormats(undefined);setFile(undefined);setResult(undefined);setMailbox('');setError('');setBusy(false);
    if(input)input.value='';
    if(account)void props.api.formats(controller.signal).then(value=>{if(epoch===generation)setFormats(value);}).catch(reason=>{
      if(epoch===generation)setError(reason instanceof MailTransferError&&reason.status===403?'mail-transfer-forbidden':'mail-transfer-load-error');
    });
  });
  onCleanup(()=>{generation++;controller?.abort();});
  async function submit(event:SubmitEvent):Promise<void>{
    event.preventDefault();const selected=file();const limits=formats();const destination=mailbox();
    if(busy()||!selected||!limits?.available||!props.mailboxes.some(m=>m.id===destination))return;
    if(selected.size===0||selected.size>limits.maxBytes){setError('mail-transfer-size-error');return;}
    const epoch=generation;controller?.abort();controller=new AbortController();setBusy(true);setError('');setResult(undefined);
    try{
      const outcome=await props.api.import(format(),selected,destination,controller.signal);
      if(epoch!==generation)return;
      setResult(outcome);setFile(undefined);if(input)input.value='';props.onImported();
    }catch(reason){
      if(epoch===generation)setError(reason instanceof MailTransferError&&reason.status===403?'mail-transfer-forbidden':'mail-transfer-import-uncertain');
    }finally{if(epoch===generation)setBusy(false);}
  }
  return <><h2 class={css.heading}>{t('mail-transfer-import-title')}</h2><p>{t('mail-transfer-import-detail')}</p>
    <Show when={!formats()&&!error()}><AsyncPending/></Show>
    <Show when={formats()}>{limits=><>
      <Show when={limits().available} fallback={<p role="status">{t('mail-transfer-unavailable')}</p>}>
        <form onSubmit={event=>void submit(event)}><fieldset disabled={busy()} class={layout.fields}>
          <label class={layout.label}>{t('mail-transfer-format')}<select class={css.input} value={format()} onChange={event=>setFormat(event.currentTarget.value as MailImportFormat)}>
            <option value="eml">EML (.eml)</option><option value="mbox">mbox</option><option value="maildir">Maildir (.tar)</option>
          </select></label>
          <label class={layout.label}>{t('mail-transfer-folder')}<select class={css.input} required value={mailbox()} onChange={event=>setMailbox(event.currentTarget.value)}>
            <option value="">{t('mail-transfer-folder-placeholder')}</option><For each={props.mailboxes}>{item=><option value={item.id}>{item.name}</option>}</For>
          </select></label>
          <label class={layout.label}>{t('mail-transfer-file')}<input ref={input} class={css.input} type="file" required onChange={event=>{setFile(event.currentTarget.files?.[0]);setResult(undefined);setError('');}}/></label>
          <p class={css.meta}>{t('mail-transfer-limits',{bytes:limits().maxBytes,messages:limits().maxMessages})}</p>
          <button class={css.button} type="submit" disabled={!file()||!mailbox()}>{t(busy()?'mail-transfer-importing':'mail-transfer-import-submit')}</button>
        </fieldset></form>
      </Show>
    </>}</Show>
    <Show when={result()}>{outcome=><p role="status">{t('mail-transfer-result',{imported:outcome().imported,failed:outcome().failed,total:outcome().total})}</p>}</Show>
    <Show when={error()}><p role="alert">{t(error())}</p></Show>
    <button class={css.ghost} disabled={busy()} onClick={()=>void guard.leave(props.onBack)}>{t('mail-transfer-back')}</button>{guard.dialog()}
  </>;
}
