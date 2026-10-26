import {createEffect,createSignal,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {receiptPolicyApi,ReceiptPolicyError,type ReceiptMode,type ReceiptPolicy,type ReceiptPolicyApi} from '../../api/receipts.ts';
import {loadCatalog,t} from '../../i18n';
import {AsyncPending} from '../../components/AsyncState.tsx';
import {useEditorGuard,useEntityNavigation} from './entityEditor.tsx';
import {LockNote} from './LockNote.tsx';
import {ReceiptSenders} from './ReceiptSenders.tsx';
import * as css from './styles.css.ts';
import * as layout from './MailTransfer.css.ts';
export function Receipts(props:{accountId:string;api?:ReceiptPolicyApi}):JSX.Element {
 const nav=useEntityNavigation('receipts');const [policy,setPolicy]=createSignal<ReceiptPolicy>();const [error,setError]=createSignal('');
 let controller:AbortController|undefined;let version=0;
 onMount(()=>{void loadCatalog('receipts');});
 createEffect(()=>{void props.accountId;const epoch=++version;controller?.abort();controller=new AbortController();setPolicy(undefined);setError('');
  void (props.api??receiptPolicyApi).load(controller.signal).then(value=>{if(epoch===version)setPolicy(value);}).catch(reason=>{if(epoch===version)setError(reason instanceof ReceiptPolicyError&&(reason.status===404||reason.status===501)?'receipts-unavailable':'receipts-load-error');});});
 onCleanup(()=>{version++;controller?.abort();});
 return <section class={layout.section} aria-label={t('receipts-title')}>
 <Show when={error()}><p role="alert">{t(error())}</p></Show><Show when={!policy()&&!error()}><AsyncPending/></Show>
 <Show when={policy()}>{current=><><Show when={!nav.route().editor?.id?.startsWith('sender:')}><Show when={nav.route().editor} fallback={<><h2 class={css.heading}>{t('receipts-title')}</h2><p>{t('receipts-description')}</p><p>{t(`receipts-mode-${current().policy}`)}</p><LockNote locked={current().locked}/><button class={css.button} onClick={()=>void nav.navigate({section:'receipts',editor:{kind:'edit',id:'response'}})}>{t('receipts-edit')}</button></>}>
  <ReceiptPolicyEditor accountId={props.accountId} value={current()} api={props.api??receiptPolicyApi} onSaved={setPolicy} onBack={()=>nav.navigate({section:'receipts'}).then(()=>undefined)}/>
 </Show></Show><Show when={!nav.route().editor||nav.route().editor?.id?.startsWith('sender:')}><ReceiptSenders accountId={props.accountId} locked={current().locked}/></Show></>}</Show></section>;
}
export function ReceiptPolicyEditor(props:{accountId:string;value:ReceiptPolicy;api:ReceiptPolicyApi;onSaved:(value:ReceiptPolicy)=>void;onBack:()=>Promise<void>}):JSX.Element {
 onMount(()=>{void loadCatalog('receipts');});
 const [draft,setDraft]=createSignal(props.value.policy);const [busy,setBusy]=createSignal(false);const [error,setError]=createSignal('');const [saved,setSaved]=createSignal(false);
 let controller:AbortController|undefined;let version=0;
 createEffect(()=>{void props.accountId;setDraft(props.value.policy);version++;controller?.abort();setBusy(false);setError('');setSaved(false);});
 onCleanup(()=>{version++;controller?.abort();});
 const dirty=()=>!saved()&&draft()!==props.value.policy;const guard=useEditorGuard(dirty,busy);
 async function save(event:SubmitEvent):Promise<void>{event.preventDefault();if(busy()||props.value.locked||!props.value.allowedModes.includes(draft()))return;
  const epoch=version;const selected=draft();controller=new AbortController();setBusy(true);setError('');
  try{await props.api.save(selected,controller.signal);if(epoch!==version)return;setSaved(true);props.onSaved({...props.value,policy:selected});await props.onBack();}
  catch(reason){if(epoch===version)setError(reason instanceof ReceiptPolicyError&&reason.status===403?'receipts-save-forbidden':'receipts-save-error');}
  finally{if(epoch===version)setBusy(false);}
 }
 return <><h2 class={css.heading}>{t('receipts-edit')}</h2><p>{t('receipts-safety')}</p><form onSubmit={event=>void save(event)}><fieldset class={layout.fields} disabled={busy()||props.value.locked}>
 <label class={layout.label}>{t('receipts-response')}<select class={css.input} value={draft()} onChange={event=>{setDraft(event.currentTarget.value as ReceiptMode);setSaved(false);}}><For each={props.value.allowedModes}>{item=><option value={item}>{t(`receipts-mode-${item}`)}</option>}</For></select></label>
 <button class={css.button} type="submit" disabled={!dirty()}>{t('receipts-save')}</button></fieldset></form><LockNote locked={props.value.locked}/><Show when={error()}><p role="alert">{t(error())}</p></Show><button class={css.ghost} disabled={busy()} onClick={()=>void guard.leave(props.onBack)}>{t('receipts-back')}</button>{guard.dialog()}</>;
}
