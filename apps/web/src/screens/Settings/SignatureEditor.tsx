import { createSignal, Show, type JSX } from 'solid-js';
import type { Signature } from './types.ts';
import type { SettingsService } from './service.ts';
import { t } from '../../i18n';
import { toast } from '../../components/toast/index.ts';
import { useEditorGuard } from './entityEditor.tsx';
import * as css from './styles.css.ts';
import * as layout from './editor.css.ts';
export function SignatureEditor(props:{initial?:Signature;service:SettingsService;onSaved:()=>void|Promise<void>;onCancel:()=>void|Promise<void>}):JSX.Element {
const [draft,setDraft]=createSignal<Signature>(structuredClone(props.initial??{name:'',body:'',isDefault:false}));const [baseline,setBaseline]=createSignal(JSON.stringify(draft()));const [busy,setBusy]=createSignal(false);const [fieldError,setFieldError]=createSignal('');const guard=useEditorGuard(()=>JSON.stringify(draft())!==baseline(),busy);
async function save():Promise<void>{if(busy())return;if(draft().name.trim()===''){setFieldError(t('settings-sig-error-name'));return;}setFieldError('');setBusy(true);try{await props.service.upsertSignature({...draft(),name:draft().name.trim()});setBaseline(JSON.stringify(draft()));toast.success(t('settings-saved'));await props.onSaved();}catch{toast.error(t('settings-sig-error-generic'));}finally{setBusy(false);}}
return <section class={css.section} aria-label={t(props.initial?'settings-sig-edit-title':'settings-sig-new')}><h2 class={css.heading}>{t(props.initial?'settings-sig-edit-title':'settings-sig-new')}</h2>
<form onSubmit={event=>{event.preventDefault();void save();}} class={css.field} data-testid="signature-editor"><fieldset disabled={busy()} class={layout.fields}>
          <label class={css.field}>
            <span class={css.label}>{t('settings-sig-name-label')}</span>
            <input
              class={css.input}
              aria-label={t('settings-sig-name-label')}
              readOnly={props.initial!==undefined}
              value={draft().name}
              onInput={(e) => setDraft({ ...draft(), name: e.currentTarget.value })}
            />
          </label>
          <label class={css.field}>
            <span class={css.label}>{t('settings-sig-body-label')}</span>
            <textarea
              class={css.textarea}
              aria-label={t('settings-sig-body-label')}
              value={draft().body}
              onInput={(e) => setDraft({ ...draft(), body: e.currentTarget.value })}
            />
          </label>
          <label class={css.check}>
            <input
              class={css.checkbox}
              type="checkbox"
              aria-label={t('settings-sig-default-label')}
              checked={draft().isDefault}
              onChange={(e) => setDraft({ ...draft(), isDefault: e.currentTarget.checked })}
            />
            <span>{t('settings-sig-default-label')}</span>
          </label>
          <div class={css.actions}>
            <button type="button" class={css.button} disabled={busy()} onClick={() => void save()} data-testid="signature-save">
              {t('settings-save')}
            </button>
            <button type="button" class={css.ghost} disabled={busy()} onClick={()=>void guard.leave(props.onCancel)}>
              {t('settings-cancel')}
            </button>
          </div>
        </fieldset></form>
<Show when={fieldError()}><p class={css.error} role="alert">{fieldError()}</p></Show>{guard.dialog()}</section>;
}
