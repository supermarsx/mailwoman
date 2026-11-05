import {preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import { createSignal, Show, type JSX, createResource, For, useContext, onMount } from 'solid-js';
import {AppContext} from '../../state/context.ts';
import {isMailbox,invalidRecipients,parseRecipients} from '../../api/jmap.ts';
import type { Identity } from './types.ts';
import { SettingsError, type SettingsService } from './service.ts';
import { isTransientFailure, showFailure } from '../../components/recoverableAction.ts';
import { t } from '../../i18n';
import { toast } from '../../components/toast/index.ts';
import { useEditorGuard } from './entityEditor.tsx';
import * as css from './styles.css.ts';
import * as layout from './editor.css.ts';
import {IconSelect,type IconSelectOption} from '../../components/IconSelect.tsx';
import * as assignments from './signatureAssignments.css.ts';
export function IdentityEditor(props:{initial?:Identity;others?:readonly Identity[];service:SettingsService;onSaved:()=>void|Promise<void>;onCancel:()=>void|Promise<void>}):JSX.Element {
const owner=preferenceAccount();
const app=useContext(AppContext);onMount(()=>{void app?.loadKeys().catch(()=>undefined);});
const ownKeys=()=>app?.keys().filter(key=>key.kind==='pgp'&&key.isOwn&&key.hasPrivate&&key.trust!=='revoked'&&(!key.expiresAt||Date.parse(key.expiresAt)>Date.now()))??[];
const [draft,setDraft]=createSignal<Identity>(structuredClone(props.initial??{id:'',name:'',email:''}));const [baseline,setBaseline]=createSignal(JSON.stringify(draft()));const [busy,setBusy]=createSignal(false);const [fieldError,setFieldError]=createSignal('');const guard=useEditorGuard(()=>JSON.stringify(draft())!==baseline(),busy);const [signatures]=createResource(()=>props.service.listSignatures().catch(()=>[]));
const encodeSignature=(name:string)=>'signature:'+encodeURIComponent(name);
const signatureOptions=(current:string|undefined):IconSelectOption[]=>{const options=(signatures()??[]).map(sig=>({value:encodeSignature(sig.name),label:sig.name,icon:'signature' as const}));if(current&&!options.some(option=>option.value===encodeSignature(current)))options.push({value:encodeSignature(current),label:t('settings-ident-signature-unavailable',{name:current}),icon:'signature'});return options;};
const newSignatureValue=()=>draft().signatureName===undefined?'account':draft().signatureName===''?'none':encodeSignature(draft().signatureName!);
const replySignatureValue=()=>draft().replySignatureName===undefined?'inherit':draft().replySignatureName===''?'none':encodeSignature(draft().replySignatureName!);
function assignSignature(kind:'new'|'reply',value:string):void {if(kind==='new'){const {signatureName:_drop,...rest}=draft();setDraft(value==='account'?rest:{...rest,signatureName:value==='none'?'':decodeURIComponent(value.slice(10))});}else{const {replySignatureName:_drop,...rest}=draft();setDraft(value==='inherit'?rest:{...rest,replySignatureName:value==='none'?'':decodeURIComponent(value.slice(10))});}}
async function save():Promise<void>{if(busy())return;if(owner!==preferenceAccount()){setFieldError(t('settings-sig-account-changed'));return;}if(draft().name.trim()===''||!isMailbox(draft().email.trim())){setFieldError(t('settings-ident-error-fields'));return;}
if((props.others??[]).some(other=>other.id!==draft().id&&other.email.trim().toLowerCase()===draft().email.trim().toLowerCase())){setFieldError(t('settings-ident-email-duplicate'));return;}
const optional=draft(),bcc=optional.autoBcc??'',organization=optional.organization??'',reply=optional.replyTo??'';
if([...organization].length>255||/[\p{Cc}]/u.test(organization)||[...bcc].length>4096||/[\p{Cc}]/u.test(bcc)||invalidRecipients(bcc).length>0||parseRecipients(bcc).length>20||reply&&!isMailbox(reply)||optional.pgpKeyId&&!ownKeys().some(key=>key.id===optional.pgpKeyId)){setFieldError(t('settings-ident-error-optional'));return;}
setFieldError('');setBusy(true);const value={...draft(),name:draft().name.trim(),email:draft().email.trim()},submitted=JSON.stringify(draft());
// t91: a create is keyed (service.ts); a transient failure gets one toast whose Retry sends the same identity again.
const persist=async():Promise<void>=>{if(owner!==preferenceAccount())return;await props.service.upsertIdentity(value);if(owner!==preferenceAccount())return;setBaseline(submitted);toast.success(t('settings-saved'));await props.onSaved();};
try{await persist();}catch(cause){if(isTransientFailure(cause))showFailure({key:'settings-identity-save',error:cause,message:reason=>t('settings-ident-save-failed',{reason}),retry:persist});else toast.error(cause instanceof SettingsError?cause.message:t('settings-ident-error-generic'));}finally{setBusy(false);}}
const [touched,setTouched]=createSignal<Record<string,boolean>>({});const touch=(key:string)=>setTouched(current=>({...current,[key]:true}));
const emailKey=(value:string)=>value.trim().toLowerCase();
const nameError=()=>touched().name&&draft().name.trim()===''?t('settings-ident-name-required'):'';
const emailError=()=>{if(!touched().email)return '';const value=draft().email.trim();if(value===''||!isMailbox(value))return t('settings-ident-email-invalid');if((props.others??[]).some(other=>other.id!==draft().id&&emailKey(other.email)===emailKey(value)))return t('settings-ident-email-duplicate');return '';};
const replyError=()=>{const value=draft().replyTo??'';return touched().replyTo&&value!==''&&!isMailbox(value)?t('settings-ident-replyto-invalid'):'';};
const bccError=()=>{const value=draft().autoBcc??'';return touched().autoBcc&&value!==''&&(invalidRecipients(value).length>0||parseRecipients(value).length>20)?t('settings-ident-bcc-invalid'):'';};
const err=(id:string,message:()=>string)=><Show when={message()}><span class={layout.fieldError} id={id}>{message()}</span></Show>;
const submit=()=>{setTouched({name:true,email:true,replyTo:true,autoBcc:true});void save();};
return <section class={css.section} aria-label={t(props.initial?'settings-ident-edit-title':'settings-ident-new')}><h2 class={css.heading}>{t(props.initial?'settings-ident-edit-title':'settings-ident-new')}</h2>
<form onSubmit={event=>{event.preventDefault();submit();}} class={layout.form} data-testid="identity-editor" noValidate><fieldset disabled={busy()} class={layout.fields}>
          <fieldset class={layout.group}><legend class={layout.legend}>{t('settings-ident-group-basics')}</legend>
          <label class={css.field}>
            <span class={css.label}>{t('settings-ident-name-label')}</span>
            <input class={`${css.input} ${layout.control}`} aria-label={t('settings-ident-name-label')} aria-required="true" aria-invalid={nameError()?'true':undefined} aria-describedby={nameError()?'ident-name-err':undefined} autocomplete="name" value={draft().name} onBlur={()=>touch('name')} onInput={(e) => setDraft({ ...draft(), name: e.currentTarget.value })}/>
            {err('ident-name-err',nameError)}
          </label>
          <label class={css.field}>
            <span class={css.label}>{t('settings-ident-email-label')}</span>
            <input class={`${css.input} ${layout.control}`} type="text" inputMode="email" autocapitalize="off" spellcheck={false} autocomplete="email" aria-label={t('settings-ident-email-label')} aria-required="true" aria-invalid={emailError()?'true':undefined} aria-describedby={emailError()?'ident-email-err':undefined} value={draft().email} onBlur={()=>touch('email')} onInput={(e) => setDraft({ ...draft(), email: e.currentTarget.value })}/>
            {err('ident-email-err',emailError)}
          </label>
          <label class={css.field}><span class={css.label}>{t('settings-ident-organization')}</span><input class={`${css.input} ${layout.control}`} maxLength={255} value={draft().organization??''} onInput={e=>setDraft({...draft(),organization:e.currentTarget.value})}/></label>
          </fieldset>
          <fieldset class={layout.group}><legend class={layout.legend}>{t('settings-ident-group-replies')}</legend>
          <label class={css.field}>
            <span class={css.label}>{t('settings-ident-replyto-label')}</span>
            <input class={`${css.input} ${layout.control}`} type="text" inputMode="email" autocapitalize="off" spellcheck={false} aria-label={t('settings-ident-replyto-label')} aria-invalid={replyError()?'true':undefined} aria-describedby={replyError()?'ident-reply-err':'ident-reply-help'} value={draft().replyTo ?? ''} onBlur={()=>touch('replyTo')} onInput={(e) => setDraft({ ...draft(), replyTo: e.currentTarget.value })}/>
            {err('ident-reply-err',replyError)}<span class={css.meta} id="ident-reply-help">{t('settings-ident-replyto-help')}</span>
          </label>
          <label class={css.field}><span class={css.label}>{t('settings-ident-auto-bcc')}</span><input class={`${css.input} ${layout.control}`} aria-label={t('settings-ident-auto-bcc')} aria-invalid={bccError()?'true':undefined} aria-describedby={bccError()?'ident-bcc-err':undefined} maxLength={4096} value={draft().autoBcc??''} onBlur={()=>touch('autoBcc')} onInput={e=>setDraft({...draft(),autoBcc:e.currentTarget.value})}/>{err('ident-bcc-err',bccError)}<span class={css.meta}>{t('settings-ident-auto-bcc-help')}</span></label>
          </fieldset>
          <fieldset class={layout.group}><legend class={layout.legend}>{t('settings-ident-group-signature')}</legend>
          <div class={`${css.field} ${assignments.field} ${layout.control}`}><span class={css.label}>{t('settings-ident-signature-new')}</span><IconSelect showLabel label={t('settings-ident-signature-new')} value={newSignatureValue()} disabled={busy()} options={[{value:'account',label:t('settings-ident-signature-account'),icon:'person'},{value:'none',label:t('settings-ident-signature-none'),icon:'minus'},...signatureOptions(draft().signatureName)]} onChange={value=>assignSignature('new',value)} testId="identity-new-signature"/></div>
          <div class={`${css.field} ${assignments.field} ${layout.control}`}><span class={css.label}>{t('settings-ident-signature-replies')}</span><IconSelect showLabel label={t('settings-ident-signature-replies')} value={replySignatureValue()} disabled={busy()} options={[{value:'inherit',label:t('settings-ident-signature-inherit'),icon:'link'},{value:'none',label:t('settings-ident-signature-none'),icon:'minus'},...signatureOptions(draft().replySignatureName)]} onChange={value=>assignSignature('reply',value)} testId="identity-reply-signature"/><span class={css.meta}>{t('settings-ident-signature-help')}</span></div>
          </fieldset>
          <fieldset class={layout.group}><legend class={layout.legend}>{t('settings-ident-group-sending')}</legend>
          <label class={css.field}><span class={css.label}>{t('settings-ident-pgp-key')}</span><select class={`${css.select} ${layout.control}`} aria-label={t('settings-ident-pgp-key')} disabled={!app} value={draft().pgpKeyId??''} onChange={e=>setDraft({...draft(),pgpKeyId:e.currentTarget.value})}><option value="" selected={!draft().pgpKeyId}>{t('settings-ident-pgp-none')}</option><Show when={draft().pgpKeyId&&!ownKeys().some(key=>key.id===draft().pgpKeyId)}><option value={draft().pgpKeyId} selected>{t('settings-ident-pgp-unavailable')}</option></Show><For each={ownKeys()}>{key=><option value={key.id} selected={draft().pgpKeyId===key.id}>{key.addresses.join(', ')} · {key.keyId}</option>}</For></select><span class={css.meta}>{t('settings-ident-pgp-help')}</span></label>
          </fieldset>
          <div class={css.actions}>
            <button type="submit" class={css.button} disabled={busy()} data-testid="identity-save">{t('settings-save')}</button>
            <button type="button" class={css.ghost} disabled={busy()} onClick={()=>void guard.leave(props.onCancel)}>{t('settings-cancel')}</button>
          </div>
        </fieldset></form>
<Show when={fieldError()}><p class={css.error} role="alert">{fieldError()}</p></Show>{guard.dialog()}</section>;
}
