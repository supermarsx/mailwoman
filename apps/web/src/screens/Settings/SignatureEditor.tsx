import { preferenceAccount, preferenceAccountIsCurrent } from '../../api/accountPreferenceLifecycle.ts';
import { createSignal, lazy, onCleanup, Show, Suspense, type JSX } from 'solid-js';
import type { Signature } from './types.ts';
import type { SettingsService } from './service.ts';
import { t } from '../../i18n';
import { toast } from '../../components/toast/index.ts';
import { useEditorGuard } from './entityEditor.tsx';
import { escapeHtml } from '../../viewers/sandbox.ts';
const plainToHtml=(text:string)=>`<p>${escapeHtml(text).replace(/\n/g,'<br>')}</p>`;
import * as css from './styles.css.ts';
import * as layout from './editor.css.ts';
const RichEditor=lazy(()=>import('../../components/compose/RichTextEditor.tsx'));
export function SignatureEditor(props:{initial?:Signature;service:SettingsService;onSaved:()=>void|Promise<void>;onCancel:()=>void|Promise<void>}):JSX.Element {
 const owner=preferenceAccount();let disposed=false;onCleanup(()=>{disposed=true;});
 const current=()=>!disposed&&(owner===null||preferenceAccountIsCurrent(owner));
 const [draft,setDraft]=createSignal<Signature>(structuredClone(props.initial??{name:'',body:'',isDefault:false}));
 const [rich,setRich]=createSignal(Boolean(props.initial?.html));
 const [source,setSource]=createSignal(false);
 const [baseline,setBaseline]=createSignal(JSON.stringify(draft()));
 const [busy,setBusy]=createSignal(false);const [fieldError,setFieldError]=createSignal('');
 const guard=useEditorGuard(()=>JSON.stringify(draft())!==baseline(),busy);
 const plain=()=>draft().text??draft().body;
 function changeFormat(html:boolean):void {
  setRich(html);setSource(false);
  setDraft(value=>({...value,html:html?(value.html??plainToHtml(plain())):null,text:plain(),body:plain(),textEdited:false}));
 }
 function changeRich(html:string,text:string):void {
  setDraft(value=>({...value,html,...(!value.textEdited?{text,body:text}:{})}));
 }
 async function save():Promise<void>{
  if(busy())return;
  if(!current()){setFieldError(t('settings-sig-account-changed'));return;}
  if(draft().name.trim()===''){setFieldError(t('settings-sig-error-name'));return;}
  setFieldError('');setBusy(true);const submitted=JSON.stringify(draft());
  try {
   await props.service.upsertSignature({...draft(),name:draft().name.trim(),body:plain(),text:plain(),html:rich()?draft().html??'':null});
   if(!current())return;
   setBaseline(submitted);if(JSON.stringify(draft())!==submitted)return;toast.success(t('settings-saved'));await props.onSaved();
  }catch{toast.error(t('settings-sig-error-generic'));}finally{setBusy(false);}
 }
 return <section class={css.section} aria-label={t(props.initial?'settings-sig-edit-title':'settings-sig-new')}>
 <h2 class={css.heading}>{t(props.initial?'settings-sig-edit-title':'settings-sig-new')}</h2>
 <form onSubmit={event=>{event.preventDefault();void save();}} class={css.field} data-testid="signature-editor">
 <fieldset disabled={busy()} class={layout.fields}>
 <label class={css.field}><span class={css.label}>{t('settings-sig-name-label')}</span><input class={css.input} aria-label={t('settings-sig-name-label')} readOnly={props.initial!==undefined} value={draft().name} onInput={e=>setDraft({...draft(),name:e.currentTarget.value})}/></label>
 <label class={css.field}><span class={css.label}>{t('settings-sig-format')}</span><select class={css.input} value={rich()?'html':'plain'} onChange={e=>changeFormat(e.currentTarget.value==='html')}><option value="plain">{t('settings-sig-format-plain')}</option><option value="html">{t('settings-sig-format-html')}</option></select></label>
 <Show when={rich()} fallback={<label class={css.field}><span class={css.label}>{t('settings-sig-body-label')}</span><textarea class={css.textarea} aria-label={t('settings-sig-body-label')} value={plain()} onInput={e=>setDraft({...draft(),body:e.currentTarget.value,text:e.currentTarget.value,textEdited:true})}/></label>}>
 <p class={css.prose}>{t('settings-sig-html-help')}</p>
 <label class={css.check}><input type="checkbox" checked={source()} onChange={e=>setSource(e.currentTarget.checked)}/>{t('settings-sig-html-source')}</label>
 <Show when={source()} fallback={<Suspense fallback={<p role="status">{t('settings-sig-editor-loading')}</p>}><div inert={busy()}><RichEditor initialHtml={draft().html??''} ariaLabel={t('settings-sig-rich-body')} onChange={changeRich}/></div></Suspense>}>
 <label class={css.field}><span class={css.label}>{t('settings-sig-html-source')}</span><textarea class={css.textarea} aria-label={t('settings-sig-html-source')} value={draft().html??''} onInput={e=>setDraft({...draft(),html:e.currentTarget.value})}/></label>
 </Show>
 <label class={css.field}><span class={css.label}>{t('settings-sig-plain-alternative')}</span><textarea class={css.textarea} aria-label={t('settings-sig-plain-alternative')} value={plain()} onInput={e=>setDraft({...draft(),body:e.currentTarget.value,text:e.currentTarget.value,textEdited:true})}/></label>
 <p class={css.prose}>{t('settings-sig-plain-help')}</p>
 </Show>
 <label class={css.check}><input class={css.checkbox} type="checkbox" aria-label={t('settings-sig-default-label')} checked={draft().isDefault} onChange={e=>setDraft({...draft(),isDefault:e.currentTarget.checked})}/>{t('settings-sig-default-label')}</label>
 <div class={css.actions}><button type="submit" class={css.button} disabled={busy()} data-testid="signature-save">{t('settings-save')}</button><button type="button" class={css.ghost} disabled={busy()} onClick={()=>void guard.leave(props.onCancel)}>{t('settings-cancel')}</button></div>
 </fieldset></form><Show when={fieldError()}><p class={css.error} role="alert">{fieldError()}</p></Show>{guard.dialog()}</section>;
}
