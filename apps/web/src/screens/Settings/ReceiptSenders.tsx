import {createEffect,createSignal,For,onCleanup,Show,type JSX} from 'solid-js';
import {loadReceiptSenders,saveReceiptSender,ReceiptPolicyError,type ReceiptSenderSnapshot} from '../../api/receipts.ts';
import {t} from '../../i18n';
import {isMailbox} from '../../api/jmap.ts';
import {useEditorGuard,useEntityNavigation} from './entityEditor.tsx';
import * as css from './styles.css.ts';
import * as layout from './MailTransfer.css.ts';

export function ReceiptSenders(props:{accountId:string;locked:boolean}):JSX.Element {
 const nav=useEntityNavigation('receipts');const [snapshot,setSnapshot]=createSignal<ReceiptSenderSnapshot>();const [error,setError]=createSignal('');
 const [reload,setReload]=createSignal(0);let controller:AbortController|undefined,epoch=0;
 createEffect(()=>{void props.accountId;void reload();const current=++epoch;controller?.abort();controller=new AbortController();setSnapshot(undefined);setError('');
  void loadReceiptSenders(controller.signal).then(value=>{if(epoch===current)setSnapshot(value);}).catch(()=>{if(epoch===current)setError('receipts-senders-load-error');});});
 onCleanup(()=>{epoch++;controller?.abort();});
 const editor=()=>nav.route().editor?.id?.startsWith('sender:')===true;
 return <Show when={snapshot()} fallback={<Show when={error()}><p role="alert">{t(error())}</p></Show>}>{value=><Show when={editor()} fallback={<section class={css.section}>
  <h3 class={css.heading}>{t('receipts-senders-title')}</h3><p class={css.prose}>{t('receipts-senders-help')}</p>
  <Show when={value().rules.length>0}><ul class={css.list}><For each={value().rules}>{rule=><li class={css.item}><span><button class={css.ghost} onClick={()=>void nav.navigate({section:'receipts',editor:{kind:'edit',id:`sender:${rule.address}`}})}>{rule.address}</button> — {t(`receipts-sender-${rule.decision}`)}</span></li>}</For></ul></Show>
  <div class={css.actions}><button class={css.button} disabled={props.locked||value().rules.length>=500} onClick={()=>void nav.navigate({section:'receipts',editor:{kind:'edit',id:'sender:'}})}>{t('receipts-senders-add')}</button></div>
 </section>}><SenderEditor accountId={props.accountId} locked={props.locked} address={(nav.route().editor?.id??'').slice(7)} snapshot={value()} onDone={async()=>{await nav.navigate({section:'receipts'});setReload(value=>value+1);}}/></Show>}</Show>;
}

function SenderEditor(props:{accountId:string;locked:boolean;address:string;snapshot:ReceiptSenderSnapshot;onDone:()=>Promise<void>}):JSX.Element {
 const initial=()=>props.snapshot.rules.find(row=>row.address===props.address);
 const [address,setAddress]=createSignal(props.address);const [decision,setDecision]=createSignal<'always'|'never'>(initial()?.decision??'never');
 const [busy,setBusy]=createSignal(false),[saved,setSaved]=createSignal(false),[error,setError]=createSignal('');
 let controller:AbortController|undefined,version=0;
 createEffect(()=>{void props.accountId;setAddress(props.address);setDecision(initial()?.decision??'never');version++;controller?.abort();setBusy(false);setSaved(false);setError('');});
 onCleanup(()=>{version++;controller?.abort();});
 const guard=useEditorGuard(()=>!saved()&&(address()!==props.address||decision()!==(initial()?.decision??'never')),busy);
 async function save(remove:boolean):Promise<void>{
  if(props.locked||busy())return;if(!remove&&!isMailbox(address())){setError('receipts-save-error');return;}const epoch=version;controller=new AbortController();setBusy(true);setError('');
  try{await saveReceiptSender(address(),remove?null:decision(),props.snapshot.revision,controller.signal);if(epoch!==version)return;setSaved(true);await props.onDone();}
  catch(reason){if(epoch===version)setError(reason instanceof ReceiptPolicyError&&reason.status===412?'receipts-senders-conflict':'receipts-save-error');}
  finally{if(epoch===version)setBusy(false);}
 }
 return <section class={css.section}><h3 class={css.heading}>{t('receipts-senders-editor')}</h3><form class={layout.fields} onSubmit={event=>{event.preventDefault();void save(false);}}><fieldset class={layout.fields} disabled={props.locked||busy()}>
  <label class={layout.label}>{t('receipts-sender-address')}<input class={css.input} type="text" inputMode="email" autocapitalize="off" spellcheck={false} required maxlength={254} value={address()} disabled={!!props.address} onInput={event=>setAddress(event.currentTarget.value)}/></label>
  <label class={layout.label}>{t('receipts-sender-decision')}<select class={css.input} value={decision()} onChange={event=>setDecision(event.currentTarget.value as 'always'|'never')}><option value="never">{t('receipts-sender-never')}</option><option value="always">{t('receipts-sender-always')}</option></select></label>
 </fieldset><Show when={error()}><p role="alert">{t(error())}</p></Show>
  <div class={css.actions}><button class={css.button} type="submit" disabled={props.locked||busy()}>{t('receipts-senders-save')}</button><button type="button" class={css.ghost} disabled={busy()} onClick={()=>void guard.leave(props.onDone)}>{t('receipts-back')}</button></div>
  <Show when={props.address}><div class={css.dangerZone}><button class={css.danger} type="button" disabled={props.locked||busy()} onClick={()=>void save(true)}>{t('receipts-senders-remove')}</button></div></Show>
 </form>{guard.dialog()}</section>;
}
