import {preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import { createSignal, Show, type JSX, createResource, For } from 'solid-js';
import type { Identity } from './types.ts';
import type { SettingsService } from './service.ts';
import { t } from '../../i18n';
import { toast } from '../../components/toast/index.ts';
import { useEditorGuard } from './entityEditor.tsx';
import * as css from './styles.css.ts';
import * as layout from './editor.css.ts';
export function IdentityEditor(props:{initial?:Identity;service:SettingsService;onSaved:()=>void|Promise<void>;onCancel:()=>void|Promise<void>}):JSX.Element {
const owner=preferenceAccount();
const [draft,setDraft]=createSignal<Identity>(structuredClone(props.initial??{id:'',name:'',email:''}));const [baseline,setBaseline]=createSignal(JSON.stringify(draft()));const [busy,setBusy]=createSignal(false);const [fieldError,setFieldError]=createSignal('');const guard=useEditorGuard(()=>JSON.stringify(draft())!==baseline(),busy);const [signatures]=createResource(()=>props.service.listSignatures().catch(()=>[]));
async function save():Promise<void>{if(busy())return;if(owner!==preferenceAccount()){setFieldError(t('settings-sig-account-changed'));return;}if(draft().name.trim()===''||! /^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(draft().email.trim())){setFieldError(t('settings-ident-error-fields'));return;}setFieldError('');setBusy(true);try{await props.service.upsertIdentity({...draft(),name:draft().name.trim(),email:draft().email.trim()});if(owner!==preferenceAccount())return;setBaseline(JSON.stringify(draft()));toast.success(t('settings-saved'));await props.onSaved();}catch{toast.error(t('settings-ident-error-generic'));}finally{setBusy(false);}}
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
