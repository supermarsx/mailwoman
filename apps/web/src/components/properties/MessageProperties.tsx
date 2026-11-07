// The message Properties dialog: summary and server location, scoring and
// authentication, the raw headers with the Received chain, attachments, and
// whole-message actions (download, permanent delete, copy id, go to folder).
// Sections shown, the raw headers, the scoring details, downloads and permanent
// delete follow the administrator's `properties.*` / `export.message.*` settings.
import {resourceValue} from '../resourceValue.ts';
import {For,Show,createEffect,createMemo,createResource,createSignal,createUniqueId,onMount,type JSX} from 'solid-js';
import {useApp} from '../../state/context.ts';
import {createConfiguredClient} from '../../api/transport.ts';
import {CAP_CORE,CAP_MAIL,type Email} from '../../api/jmap-types.ts';
import {CAP_CRYPTO,CAP_SECURITY} from '../../api/crypto-types.ts';
import type {SecurityVerdict} from '../../api/security-types.ts';
import {destroyEmails,responseFor} from '../../api/jmap.ts';
import {fetchMailHeaders} from '../../api/mailHeaders.ts';
import {apiFetch} from '../../api/http.ts';
import {withBase} from '../../api/basePath.ts';
import {isolate,loadCatalog,t} from '../../i18n';
import {SecurityPopup} from '../SecurityPanel.tsx';
import {ConfirmDialog} from '../../screens/Admin/shell/ConfirmDialog.tsx';
import {Icon} from '../Icon.tsx';
import {toast} from '../toast/toast.ts';
import {PriorityMark} from '../PriorityMark.tsx';
import {priorityOf} from '../messagePriority.ts';
import {focusMessageInFolder} from '../messageFocus.ts';
import {buildDownloadUrl} from '../../viewers/attachments.ts';
import {AUTH_RESULT_LABEL,ANOMALY_LABEL} from '../security/model.ts';
import {spamBand} from '../security/spamScore.ts';
import {formatBytes} from './folderStats.ts';
import {trackerCountFor,trainingState} from './scoringExtras.ts';
import {filterHeaders,headerList,headersOnly,hopBars,hopDelayText,spamVerdict} from './messageProps.ts';
import {messageDownloadEnabled,messageSections,permanentDeletePolicy,remoteForDisplay,remoteVisibility,type MessageSection} from './policy.ts';
import * as css from './properties.css.ts';
// i18n: the ids this file shows live in these catalogs, which load with it.
void loadCatalog('security');

/** What the dialog can ask the message menu to do; an action not supplied is not offered. */
export interface MessageDialogActions {
 onMove?:()=>void;onSpam?:()=>void;onNotSpam?:()=>void;onPhishing?:()=>void;onPrint?:()=>void;onViewSource?:()=>void;onCreateFilter?:()=>void;onBlockSender?:()=>void;
}

interface AttachmentPart {name?:string|null;type?:string|null;blobId?:string|null;size?:number}
interface Location {id:string;mailboxId:string;mailboxPath:string|null;uid:number;uidValidity:number}
const DETAIL_PROPERTIES=['id','blobId','from','to','cc','bcc','replyTo','subject','receivedAt','sentAt','messageId','inReplyTo','references','keywords','mailboxIds','threadId','size','hasAttachment','attachments','mailwomanPriority'];
const addresses=(list:ReadonlyArray<{name?:string|null;email:string}>|null|undefined)=>(list??[]).map(a=>a.name?`${a.name} <${a.email}>`:a.email).join(', ');
const when=(iso:string|undefined|null)=>{if(!iso)return t('props-none');const d=new Date(iso);return Number.isNaN(d.getTime())?iso:d.toLocaleString();};

export function MessageDialogs(props:{email:Email;actions?:MessageDialogActions;onClose:()=>void}):JSX.Element {
 const app=useApp(),client=createConfiguredClient(),titleId=`props-${createUniqueId()}`;
 onMount(()=>{void loadCatalog('properties');void loadCatalog('mail');});
 const sections=createMemo(()=>messageSections());
 const [active,setActive]=createSignal<MessageSection>(sections()[0]??'summary');
 createEffect(()=>{if(!sections().includes(active()))setActive(sections()[0]??'summary');});
 const accountId=()=>app.accountId()??'';
 const [detail]=createResource(()=>({account:accountId(),id:props.email.id}),async key=>{
  if(!key.account)return props.email;
  try{
   const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/get',{accountId:key.account,ids:[key.id],properties:DETAIL_PROPERTIES},'d']]});
   const found=responseFor<{list:Email[]}>(response,'d').list[0];
   return found?{...props.email,...found}:props.email;
  }catch{return props.email;}
 });
 const email=()=>resourceValue(detail)??props.email;
 const [location]=createResource(()=>({account:accountId(),id:props.email.id}),async key=>{
  if(!key.account)return null;
  try{
   const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/location',{accountId:key.account,ids:[key.id]},'l']]});
   return responseFor<{list:Location[]}>(response,'l').list[0]??null;
  }catch{return null;}
 });
 const [verdict]=createResource(()=>({account:accountId(),id:props.email.id,on:sections().includes('scoring')||sections().includes('attachments')}),async key=>{
  if(!key.account||!key.on)return null;
  try{
   const response=await client.jmap({using:[CAP_CORE,CAP_CRYPTO,CAP_SECURITY],methodCalls:[['SecurityVerdict/get',{accountId:key.account,ids:[key.id]},'v']]});
   return responseFor<{list:SecurityVerdict[]}>(response,'v').list[0]??null;
  }catch{return null;}
 });
 const messageUrl=()=>{const template=app.downloadUrl(),account=app.accountId(),blob=email().blobId;return template&&account&&blob?buildDownloadUrl(template,{accountId:account,blobId:blob,name:'message.eml',mime:'message/rfc822'}):null;};
 const [rawHeaders]=createResource(()=>({url:messageUrl(),on:sections().includes('headers')||sections().includes('scoring')||sections().includes('actions')}),async key=>{
  if(!key.url||!key.on)return null;
  const abort=new AbortController(),timer=setTimeout(()=>abort.abort(),10000);
  try{return await fetchMailHeaders(key.url,abort.signal);}catch{return null;}finally{clearTimeout(timer);}
 });
 const [trackers]=createResource(()=>sections().includes('scoring')&&active()==='scoring'?{account:accountId(),id:props.email.id}:undefined,async key=>key.account?trackerCountFor(client,key.account,key.id):null);
 const tabRefs:HTMLButtonElement[]=[];
 function onTabKey(event:KeyboardEvent):void {
  const list=sections(),at=list.indexOf(active());
  const next=event.key==='ArrowRight'?(at+1)%list.length:event.key==='ArrowLeft'?(at-1+list.length)%list.length:event.key==='Home'?0:event.key==='End'?list.length-1:-1;
  if(next<0)return;event.preventDefault();setActive(list[next]!);tabRefs[next]?.focus();
 }
 return <SecurityPopup title={t('props-message-title',{subject:isolate(email().subject??t('mail-no-subject'))})} labelledBy={titleId} onClose={props.onClose}>
  <div class={`${css.body} ${css.phone}`} data-testid="message-properties">
   <div class={css.tabs} role="tablist" aria-label={t('props-sections')} onKeyDown={onTabKey}>
    <For each={sections()}>{(section,index)=><button type="button" role="tab" id={`${titleId}-tab-${section}`} aria-selected={active()===section} aria-controls={`${titleId}-panel`} tabindex={active()===section?0:-1} class={css.tab} ref={el=>{tabRefs[index()]=el;}} onClick={()=>setActive(section)}>{t(`props-mtab-${section}`)}</button>}</For>
   </div>
   <div class={css.panel} role="tabpanel" id={`${titleId}-panel`} aria-labelledby={`${titleId}-tab-${active()}`} tabindex="0">
    <Show when={active()==='summary'}><Summary email={email()} location={resourceValue(location)??null} onClose={props.onClose}/></Show>
    <Show when={active()==='scoring'}><Scoring verdict={resourceValue(verdict)??null} loading={verdict.loading} raw={resourceValue(rawHeaders)??null} email={email()} trackers={resourceValue(trackers)}/></Show>
    <Show when={active()==='headers'}><Headers raw={resourceValue(rawHeaders)??null} loading={rawHeaders.loading} hops={resourceValue(verdict)?.received??[]}/></Show>
    <Show when={active()==='attachments'}><Attachments email={email()} verdict={resourceValue(verdict)??null}/></Show>
    <Show when={active()==='actions'}><MessageActionsPane email={email()} url={messageUrl()} raw={resourceValue(rawHeaders)??null} actions={props.actions} onClose={props.onClose}/></Show>
   </div>
  </div>
 </SecurityPopup>;
}

function Pair(props:{label:string;children:JSX.Element}):JSX.Element {return <><dt>{props.label}</dt><dd>{props.children}</dd></>;}

function Summary(props:{email:Email;location:Location|null;onClose:()=>void}):JSX.Element {
 const app=useApp();
 const e=()=>props.email;
 const folderIds=()=>Object.keys(e().mailboxIds??{});
 const folderName=(id:string)=>app.mailboxes().find(box=>box.id===id)?.mailwomanPath??app.mailboxes().find(box=>box.id===id)?.name??id;
 const labels=()=>Object.entries(e().keywords??{}).filter(([,on])=>on).map(([k])=>k);
 const remote=()=>{const loc=props.location;return loc?remoteForDisplay({path:loc.mailboxPath??'',uidValidity:loc.uidValidity},remoteVisibility()):null;};
 return <dl class={css.grid} data-testid="props-summary">
  <Pair label={t('mail-subject')}>{isolate(e().subject??t('mail-no-subject'))}</Pair>
  <Pair label={t('mail-compose-from')}>{isolate(addresses(e().from))}</Pair>
  <Pair label={t('mail-compose-to')}>{isolate(addresses(e().to))||t('props-none')}</Pair>
  <Show when={(e().cc??[]).length>0}><Pair label={t('mail-compose-cc')}>{isolate(addresses(e().cc))}</Pair></Show>
  <Show when={(e().bcc??[]).length>0}><Pair label={t('mail-compose-bcc')}>{isolate(addresses(e().bcc))}</Pair></Show>
  <Show when={(e().replyTo??[]).length>0}><Pair label={t('props-reply-to')}>{isolate(addresses(e().replyTo))}</Pair></Show>
  <Pair label={t('props-sent')}>{when(e().sentAt)}</Pair>
  <Pair label={t('props-received')}>{when(e().receivedAt)}</Pair>
  <Pair label={t('props-size')}>{formatBytes(e().size??0)}</Pair>
  <Pair label={t('props-priority')}><Show when={priorityOf(e())!=='normal'} fallback={t('priority-normal')}><PriorityMark priority={priorityOf(e())} badge/></Show></Pair>
  <Pair label={t('props-message-id')}><span class={css.mono}>{(e().messageId??[]).join(' ')||t('props-none')}</span></Pair>
  <Pair label={t('props-thread-id')}><span class={css.mono}>{e().threadId??t('props-none')}</span></Pair>
  <Show when={(e().inReplyTo??[]).length>0}><Pair label={t('props-in-reply-to')}><span class={css.mono}>{(e().inReplyTo??[]).join(' ')}</span></Pair></Show>
  <Show when={(e().references??[]).length>0}><Pair label={t('props-references')}><span class={css.mono}>{(e().references??[]).join(' ')}</span></Pair></Show>
  <Pair label={t('props-flags')}>{labels().join(', ')||t('props-none')}</Pair>
  <Pair label={t('props-folder')}><For each={folderIds()}>{id=><span class={css.row}>{isolate(folderName(id))} <button type="button" class="btn btn--ghost" data-testid="props-goto-folder" onClick={()=>{props.onClose();void focusMessageInFolder(app,id,e().id,{open:true});}}>{t('props-goto-folder')}</button></span>}</For></Pair>
  <Pair label={t('props-email-id')}><span class={css.mono}>{e().id}</span></Pair>
  <Show when={e().blobId}><Pair label={t('props-blob-id')}><span class={css.mono}>{e().blobId}</span></Pair></Show>
  <Show when={remote()}>{r=><>
   <Pair label={t('props-remote-path')}><span class={css.mono}>{isolate(r().path)}</span></Pair>
   <Pair label="UID"><span class={css.mono}>{props.location!.uid}</span></Pair>
   <Pair label="UIDVALIDITY"><span class={css.mono}>{props.location!.uidValidity}</span></Pair>
  </>}</Show>
 </dl>;
}

function Scoring(props:{verdict:SecurityVerdict|null;loading:boolean;raw:string|null;email:Email;trackers:number|null|undefined}):JSX.Element {
 const spam=()=>props.raw?spamVerdict(props.raw):null;
 const v=()=>props.verdict;
 return <div data-testid="props-scoring" aria-busy={props.loading}>
  <section aria-labelledby="props-privacy-title"><h4 id="props-privacy-title">{t('props-privacy')}</h4>
   <dl class={css.grid}>
    <Pair label={t('props-trackers')}><span data-testid="props-trackers">{props.trackers===undefined?t('props-loading'):props.trackers===null?t('props-unknown'):String(props.trackers)}</span></Pair>
    <Pair label={t('props-training')}><span data-testid="props-training">{t(`props-training-${trainingState(props.email,spam()?.status??null)}`)}</span></Pair>
   </dl>
   <p class={css.note}>{t('props-dlp-note')}</p></section>
  <Show when={spam()}>{s=><section aria-labelledby="props-spam-title"><h4 id="props-spam-title">{t('props-spam')}</h4>
   <Show when={s().score.length||s().status} fallback={<p class={css.note}>{t('props-spam-none')}</p>}>
    <dl class={css.grid}>
     <For each={s().score}>{score=><Pair label={score.source}>{`${score.score}${score.threshold!==null?` / ${score.threshold}`:''}`} ({t(`spam-score-band-${spamBand(score)}`)})</Pair>}</For>
     <Show when={s().status}><Pair label="X-Spam-Status"><span class={css.mono}>{s().status}</span></Pair></Show>
     <Show when={s().flag!==null}><Pair label="X-Spam-Flag">{t(s().flag?'props-yes':'props-no')}</Pair></Show>
    </dl></Show></section>}</Show>
  <Show when={v()} fallback={<p class={css.note} role="status">{props.loading?t('props-loading'):t('props-scoring-unavailable')}</p>}>{verdict=><>
   <section aria-labelledby="props-auth-title"><h4 id="props-auth-title">{t('props-auth')}</h4>
    <dl class={css.grid}>
     <Pair label="SPF">{AUTH_RESULT_LABEL[verdict().auth.spf.result]}{verdict().auth.spf.domain?` (${verdict().auth.spf.domain})`:''}</Pair>
     <Pair label="DKIM">{AUTH_RESULT_LABEL[verdict().auth.dkim.result]}{verdict().auth.dkim.domain?` (${verdict().auth.dkim.domain})`:''}</Pair>
     <Pair label="DMARC">{AUTH_RESULT_LABEL[verdict().auth.dmarc.result]} - {t(verdict().auth.dmarc.aligned?'security-aligned':'security-not-aligned')}</Pair>
     <Pair label="ARC">{AUTH_RESULT_LABEL[verdict().auth.arc.result]}</Pair>
    </dl></section>
   <section aria-labelledby="props-sec-title"><h4 id="props-sec-title">{t('props-security')}</h4>
    <dl class={css.grid}>
     <Pair label={t('props-encryption')}>{verdict().encryption.isEncrypted?verdict().encryption.kind.toUpperCase():t('props-no')}</Pair>
     <Pair label={t('props-signature')}>{verdict().signature?String(verdict().signature!.status):t('props-none')}</Pair>
     <Pair label={t('props-signals')}>{verdict().anomalies.length?verdict().anomalies.map(a=>ANOMALY_LABEL[a]).join('; '):t('props-none')}</Pair>
     <Pair label={t('props-att-risk')}>{verdict().attachments.filter(a=>a.risk!=='none'||a.mismatch).length?verdict().attachments.filter(a=>a.risk!=='none'||a.mismatch).map(a=>`${a.name}: ${a.risk}${a.mismatch?' (type mismatch)':''}`).join('; '):t('props-none')}</Pair>
    </dl>
    <p class={css.note}>{verdict().plainLanguage}</p></section>
  </>}</Show>
 </div>;
}

function Headers(props:{raw:string|null;loading:boolean;hops:SecurityVerdict['received']}):JSX.Element {
 const [filter,setFilter]=createSignal(''),[showHops,setShowHops]=createSignal(false);
 const list=createMemo(()=>props.raw?headerList(props.raw):[]);
 const shown=createMemo(()=>filterHeaders(list(),filter()));
 const hops=createMemo(()=>list().filter(h=>h.name.toLowerCase()==='received').map(h=>h.value));
 const bars=createMemo(()=>hopBars(props.hops.map(hop=>hop.delayMs)));
 async function copy():Promise<void> {try{await navigator.clipboard.writeText(shown().map(h=>`${h.name}: ${h.value}`).join('\r\n'));toast.success(t('mail-headers-copy'));}catch{toast.error(t('props-act-copy-failed'));}}
 return <div data-testid="props-headers" aria-busy={props.loading}>
  <Show when={props.raw!==null} fallback={<p class={css.note} role="status">{props.loading?t('props-loading'):t('mail-headers-failed')}</p>}>
   <div class={css.row}><label>{t('props-headers-filter')} <input type="search" value={filter()} onInput={event=>setFilter(event.currentTarget.value)}/></label>
    <button type="button" class="btn btn--ghost" onClick={()=>void copy()}><Icon name="clone"/>{t('mail-headers-copy')}</button>
    <Show when={hops().length>0}><button type="button" class="btn btn--ghost" aria-expanded={showHops()} onClick={()=>setShowHops(value=>!value)}>{t('props-received-chain',{count:hops().length})}</button></Show></div>
   <Show when={showHops()}><ol class={css.note} data-testid="props-hops">{/* oldest hop first */}<For each={[...hops()].reverse()}>{(hop,index)=><li class={css.hop}><span>{index()+1}</span><span class={css.mono}>{hop}</span><span data-testid="props-hop-delay"><Show when={hopDelayText(props.hops[index()]?.delayMs??null)}>{text=><><span class={css.bar} style={{width:`${bars()[index()]??0}%`}}/>{text()}</>}</Show></span></li>}</For></ol></Show>
   <pre class={css.headersPre} tabindex="0" aria-label={t('props-tab-headers-label')}>{shown().map(h=>`${h.name}: ${h.value}`).join('\n')}</pre>
  </Show>
 </div>;
}

function Attachments(props:{email:Email;verdict:SecurityVerdict|null}):JSX.Element {
 const app=useApp();
 const files=()=>(props.email as Email&{attachments?:AttachmentPart[]}).attachments??[];
 const risk=(name:string)=>props.verdict?.attachments.find(a=>a.name===name);
 const href=(part:{blobId?:string|null;name?:string|null;type?:string|null})=>{const template=app.downloadUrl(),account=app.accountId();return template&&account&&part.blobId?buildDownloadUrl(template,{accountId:account,blobId:part.blobId,name:part.name??'attachment',mime:part.type??'application/octet-stream'}):undefined;};
 return <div data-testid="props-attachments">
  <Show when={files().length} fallback={<p class={css.note}>{t('props-att-none')}</p>}>
   <ul>
    <For each={files()}>{part=>{const r=()=>risk(part.name??'');return <li class={css.row}>
     <span>{isolate(part.name??t('props-unnamed'))}</span><span class={css.note}>{part.type??''}{part.size?`, ${formatBytes(part.size)}`:''}</span>
     <Show when={r()&&(r()!.risk!=='none'||r()!.mismatch)}><span class={css.danger}><Icon name="alert"/>{r()!.risk}{r()!.mismatch?` (${t('props-att-mismatch')})`:''}</span></Show>
     <Show when={href(part)}><a class="btn btn--ghost" href={href(part)} download={part.name??'attachment'}><Icon name="download"/>{t('props-download')}</a></Show>
    </li>;}}</For>
   </ul></Show>
 </div>;
}

function MessageActionsPane(props:{email:Email;url:string|null;raw:string|null;actions:MessageDialogActions|undefined;onClose:()=>void}):JSX.Element {
 const app=useApp(),client=createConfiguredClient();
 const [step,setStep]=createSignal<0|1|2>(0);
 const policy=()=>permanentDeletePolicy();
 const run=(fn?:()=>void)=>()=>{props.onClose();fn?.();};
 async function copy(text:string):Promise<void> {try{await navigator.clipboard.writeText(text);toast.success(t('props-act-copied'));}catch{toast.error(t('props-act-copy-failed'));}}
 async function downloadMbox():Promise<void> {
  try{
   const response=await apiFetch(withBase(`/api/export/${encodeURIComponent(props.email.id)}?format=mbox`));
   if(!response.ok)throw new Error(String(response.status));
   saveText(await response.blob(),'message.mbox');
  }catch{toast.error(t('props-download-failed'));}
 }
 function saveText(blob:Blob,name:string):void {const url=URL.createObjectURL(blob),a=document.createElement('a');a.href=url;a.download=name;document.body.appendChild(a);a.click();a.remove();setTimeout(()=>URL.revokeObjectURL(url),10_000);}
 async function destroy():Promise<void> {
  const account=app.accountId();if(!account)return;
  const response=await client.jmap(destroyEmails(account,[props.email.id],true));
  const result=responseFor<{destroyed?:string[];notDestroyed?:Record<string,{type?:string;description?:string}>}>(response,'set');
  if(!result.destroyed?.includes(props.email.id))throw new Error(result.notDestroyed?.[props.email.id]?.description??result.notDestroyed?.[props.email.id]?.type??'refused');
  setStep(0);props.onClose();void app.refreshCurrentMailbox();toast.success(t('props-delete-done'));
 }
 const downloads=()=>messageDownloadEnabled();
 const describe=(error:unknown)=>error instanceof Error&&error.message?error.message:t('props-delete-failed');
 return <div data-testid="props-message-actions">
  <div class={css.actions}>
   <Show when={downloads()&&props.url}><a class="btn btn--ghost" data-testid="props-download-eml" href={props.url!} download="message.eml"><Icon name="download"/>{t('props-download-eml')}</a></Show>
   <Show when={downloads()&&props.raw!==null}><button type="button" class="btn btn--ghost" data-testid="props-download-headers" onClick={()=>saveText(new Blob([headersOnly(props.raw!)],{type:'text/plain'}),'headers.txt')}>{t('props-download-headers')}</button></Show>
   <Show when={downloads()}><button type="button" class="btn btn--ghost" data-testid="props-download-mbox" onClick={()=>void downloadMbox()}>{t('props-download-mbox')}</button></Show>
   <button type="button" class="btn btn--ghost" onClick={()=>void copy((props.email.messageId??[]).join(' ')||props.email.id)}>{t('props-copy-message-id')}</button>
   <Show when={props.actions?.onMove}><button type="button" class="btn btn--ghost" onClick={run(props.actions!.onMove)}>{t('props-act-move')}</button></Show>
   <Show when={props.actions?.onSpam}><button type="button" class="btn btn--ghost" onClick={run(props.actions!.onSpam)}>{t('props-act-spam')}</button></Show>
   <Show when={props.actions?.onNotSpam}><button type="button" class="btn btn--ghost" onClick={run(props.actions!.onNotSpam)}>{t('props-act-not-spam')}</button></Show>
   <Show when={props.actions?.onPhishing}><button type="button" class="btn btn--ghost" onClick={run(props.actions!.onPhishing)}>{t('props-act-phishing')}</button></Show>
   <Show when={props.actions?.onPrint}><button type="button" class="btn btn--ghost" onClick={run(props.actions!.onPrint)}><Icon name="print"/>{t('props-act-print')}</button></Show>
   <Show when={props.actions?.onViewSource}><button type="button" class="btn btn--ghost" onClick={run(props.actions!.onViewSource)}>{t('props-act-source')}</button></Show>
   <Show when={props.actions?.onCreateFilter}><button type="button" class="btn btn--ghost" onClick={run(props.actions!.onCreateFilter)}>{t('props-act-filter')}</button></Show>
   <Show when={props.actions?.onBlockSender}><button type="button" class="btn btn--ghost" onClick={run(props.actions!.onBlockSender)}>{t('props-act-block')}</button></Show>
  </div>
  <section aria-labelledby="props-delete-title"><h4 id="props-delete-title">{t('props-delete-title')}</h4>
   <p class={css.note}>{t(policy()==='deny'?'props-delete-denied':'props-delete-warning')}</p>
   <button type="button" class="btn btn--danger" data-testid="props-delete" disabled={policy()==='deny'} title={policy()==='deny'?t('props-delete-denied'):undefined} onClick={()=>setStep(1)}><Icon name="delete"/>{t('props-delete')}</button>
  </section>
  <ConfirmDialog open={step()===1} title={t('props-delete-confirm-title')} body={t('props-delete-confirm-body')} confirmLabel={policy()==='confirm-twice'?t('props-delete-continue'):t('props-delete')} onCancel={()=>setStep(0)} onConfirm={async()=>{if(policy()==='confirm-twice'){setStep(2);return;}await destroy();}} testId="props-delete-confirm" describeError={describe}/>
  <ConfirmDialog open={step()===2} title={t('props-delete-confirm2-title')} body={t('props-delete-confirm2-body')} confirmLabel={t('props-delete')} onCancel={()=>setStep(0)} onConfirm={destroy} testId="props-delete-confirm2" describeError={describe}/>
 </div>;
}
