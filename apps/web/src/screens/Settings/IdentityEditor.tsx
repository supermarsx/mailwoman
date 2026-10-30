import {preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import { createSignal, Show, type JSX, createResource, For, useContext, onMount } from 'solid-js';
import {AppContext} from '../../state/context.ts';
import {invalidRecipients,parseRecipients} from '../../api/jmap.ts';
import type { Identity } from './types.ts';
import { SettingsError, type SettingsService } from './service.ts';
import { isTransientFailure, showFailure } from '../../components/recoverableAction.ts';
import { t } from '../../i18n';
import { toast } from '../../components/toast/index.ts';
import { useEditorGuard } from './entityEditor.tsx';
import * as css from './styles.css.ts';
import * as layout from './editor.css.ts';
export function IdentityEditor(props:{initial?:Identity;service:SettingsService;onSaved:()=>void|Promise<void>;onCancel:()=>void|Promise<void>}):JSX.Element {
const owner=preferenceAccount();
const app=useContext(AppContext);onMount(()=>{void app?.loadKeys().catch(()=>undefined);});
const ownKeys=()=>app?.keys().filter(key=>key.kind==='pgp'&&key.isOwn&&key.hasPrivate&&key.trust!=='revoked'&&(!key.expiresAt||Date.parse(key.expiresAt)>Date.now()))??[];
const [draft,setDraft]=createSignal<Identity>(structuredClone(props.initial??{id:'',name:'',email:''}));const [baseline,setBaseline]=createSignal(JSON.stringify(draft()));const [busy,setBusy]=createSignal(false);const [fieldError,setFieldError]=createSignal('');const guard=useEditorGuard(()=>JSON.stringify(draft())!==baseline(),busy);const [signatures]=createResource(()=>props.service.listSignatures().catch(()=>[]));
async function save():Promise<void>{if(busy())return;if(owner!==preferenceAccount()){setFieldError(t('settings-sig-account-changed'));return;}if(draft().name.trim()===''||! /^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(draft().email.trim())){setFieldError(t('settings-ident-error-fields'));return;}
const optional=draft(),bcc=optional.autoBcc??'',organization=optional.organization??'',reply=optional.replyTo??'';
if([...organization].length>255||/[\p{Cc}]/u.test(organization)||[...bcc].length>4096||/[\p{Cc}]/u.test(bcc)||invalidRecipients(bcc).length>0||parseRecipients(bcc).length>20||reply&&(invalidRecipients(reply).length>0||parseRecipients(reply).length!==1)||optional.pgpKeyId&&!ownKeys().some(key=>key.id===optional.pgpKeyId)){setFieldError(t('settings-ident-error-optional'));return;}
setFieldError('');setBusy(true);const value={...draft(),name:draft().name.trim(),email:draft().email.trim()},submitted=JSON.stringify(draft());
// t91: a create is keyed (service.ts); a transient failure gets one toast whose Retry sends the same identity again.
const persist=async():Promise<void>=>{if(owner!==preferenceAccount())return;await props.service.upsertIdentity(value);if(owner!==preferenceAccount())return;setBaseline(submitted);toast.success(t('settings-saved'));await props.onSaved();};
try{await persist();}catch(cause){if(isTransientFailure(cause))showFailure({key:'settings-identity-save',error:cause,message:reason=>t('settings-ident-save-failed',{reason}),retry:persist});else toast.error(cause instanceof SettingsError?cause.message:t('settings-ident-error-generic'));}finally{setBusy(false);}}
return <section class={css.section} aria-label={t(props.initial?'settings-ident-edit-title':'settings-ident-new')}><h2 class={css.heading}>{t(props.initial?'settings-ident-edit-title':'settings-ident-new')}</h2>
<form onSubmit={event=>{event.preventDefault();void save();}} class={css.field} data-testid="identity-editor"><fieldset disabled={busy()} class={layout.fields}>
          <label class={css.field}>
            <span class={css.label}>{t('settings-ident-name-label')}</span>
            <input
              class={css.input}
              aria-label={t('settings-ident-name-label')}
              value={draft().name}
              onInput={(e) => setDraft({ ...draft(), name: e.currentTarget.value })}
            />
          </label>
          <label class={css.field}>
            <span class={css.label}>{t('settings-ident-email-label')}</span>
            <input
              class={css.input}
              type="email"
              autocomplete="email"
              aria-label={t('settings-ident-email-label')}
              value={draft().email}
              onInput={(e) => setDraft({ ...draft(), email: e.currentTarget.value })}
            />
          </label>
          <label class={css.field}>
            <span class={css.label}>{t('settings-ident-replyto-label')}</span>
            <input
              class={css.input}
              type="email"
              aria-label={t('settings-ident-replyto-label')}
              value={draft().replyTo ?? ''}
              onInput={(e) => setDraft({ ...draft(), replyTo: e.currentTarget.value })}
            />
          </label>
          <label class={css.field}>
            <span class={css.label}>{t('settings-ident-signature-label')}</span>
            <select
              class={css.select}
              aria-label={t('settings-ident-signature-label')}
              value={draft().signatureName ?? ''}
              onChange={(e) => {
                const value = e.currentTarget.value;
                const { signatureName: _drop, ...rest } = draft();
                setDraft(value === '' ? rest : { ...rest, signatureName: value });
              }}
            >
              <option value="">{t('settings-ident-signature-none')}</option>
              <For each={signatures() ?? []}>{(sig) => <option value={sig.name}>{sig.name}</option>}</For>
            </select>
          </label>
          <label class={css.field}><span class={css.label}>{t('settings-ident-organization')}</span><input class={css.input} maxLength={255} value={draft().organization??''} onInput={e=>setDraft({...draft(),organization:e.currentTarget.value})}/></label>
          <label class={css.field}><span class={css.label}>{t('settings-ident-auto-bcc')}</span><input class={css.input} aria-label={t('settings-ident-auto-bcc')} maxLength={4096} value={draft().autoBcc??''} onInput={e=>setDraft({...draft(),autoBcc:e.currentTarget.value})}/><span class={css.meta}>{t('settings-ident-auto-bcc-help')}</span></label>
          <label class={css.field}><span class={css.label}>{t('settings-ident-pgp-key')}</span><select class={css.select} aria-label={t('settings-ident-pgp-key')} disabled={!app} value={draft().pgpKeyId??''} onChange={e=>setDraft({...draft(),pgpKeyId:e.currentTarget.value})}><option value="">{t('settings-ident-pgp-none')}</option><Show when={draft().pgpKeyId&&!ownKeys().some(key=>key.id===draft().pgpKeyId)}><option value={draft().pgpKeyId}>{t('settings-ident-pgp-unavailable')}</option></Show><For each={ownKeys()}>{key=><option value={key.id}>{key.addresses.join(', ')} · {key.keyId}</option>}</For></select><span class={css.meta}>{t('settings-ident-pgp-help')}</span></label>
          <div class={css.actions}>
            <button type="button" class={css.button} disabled={busy()} onClick={() => void save()} data-testid="identity-save">
              {t('settings-save')}
            </button>
            <button type="button" class={css.ghost} disabled={busy()} onClick={()=>void guard.leave(props.onCancel)}>
              {t('settings-cancel')}
            </button>
          </div>
        </fieldset></form>
<Show when={fieldError()}><p class={css.error} role="alert">{fieldError()}</p></Show>{guard.dialog()}</section>;
}
